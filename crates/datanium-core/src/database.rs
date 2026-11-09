//! In-memory relational database model.
//!
//! Database, table and column names contain 1 to 64 ASCII characters and match
//! `[A-Za-z_][A-Za-z0-9_]*`. Names are preserved as supplied. Table lookup and
//! uniqueness checks are case-sensitive, so `Customers` and `customers` are
//! distinct names.

mod cell;
mod column;
mod constraint;
mod error;
mod name;
mod row;
mod table;
#[cfg(test)]
mod tests;
mod value;

use std::collections::BTreeMap;

pub use self::cell::{Cell, CellError, CellWarning};
pub use self::column::{Column, ColumnId};
pub use self::constraint::{Constraint, ForeignKeyColumn, ReferentialAction};
pub use self::error::{DatabaseError, TableError};
pub use self::row::{Row, RowId};
pub use self::table::{Table, TableId};
pub use self::value::{DataType, Value};

/// An in-memory database that owns its tables and allocates their IDs.
///
/// Names follow the format documented in this module. Table names are unique
/// within each database using case-sensitive comparison.
///
/// # Example
///
/// ```
/// use datanium_core::database::Database;
///
/// let mut database = Database::new("Shop")?;
/// let id = database.create_table("Customers")?;
/// database.rename_table(&id, "Clients")?;
///
/// assert_eq!(database.name(), "Shop");
/// assert_eq!(database.get_table_by_name("Clients")?.id(), id);
/// # Ok::<(), datanium_core::database::DatabaseError>(())
/// ```
pub struct Database {
    name: String,
    tables: BTreeMap<TableId, Table>,
    next_table_id: u64,
}

impl Database {
    /// Creates an empty database with the supplied name.
    ///
    /// The name must contain 1 to 64 ASCII characters and match
    /// `[A-Za-z_][A-Za-z0-9_]*`. Its case is preserved.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::InvalidDatabaseName`] for an invalid name.
    pub fn new(name: &str) -> Result<Self, DatabaseError> {
        if !name::is_valid_name(name) {
            return Err(DatabaseError::InvalidDatabaseName);
        }

        Ok(Self {
            name: name.to_string(),
            tables: BTreeMap::new(),
            next_table_id: 1,
        })
    }

    /// Returns the database name with its original case.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Creates an empty table and returns its ID.
    ///
    /// The name must contain 1 to 64 ASCII characters and match
    /// `[A-Za-z_][A-Za-z0-9_]*`. Names are preserved and compared case-sensitively.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::TableAlreadyExists`] if the exact name is taken,
    /// [`DatabaseError::InvalidTableName`] if the name is invalid, or
    /// [`DatabaseError::TableIdExhausted`] if the ID counter cannot be advanced.
    /// These checks run in the order listed. On error, the table collection and
    /// ID counter remain unchanged.
    pub fn create_table(&mut self, name: &str) -> Result<TableId, DatabaseError> {
        if self.table_name_exists(name) {
            return Err(DatabaseError::TableAlreadyExists);
        }

        if !name::is_valid_name(name) {
            return Err(DatabaseError::InvalidTableName);
        }

        let next_table_id = self
            .next_table_id
            .checked_add(1)
            .ok_or(DatabaseError::TableIdExhausted)?;

        let id = TableId::new(self.next_table_id);
        let table = Table::new(id, name);

        self.tables.insert(id, table);
        self.next_table_id = next_table_id;

        Ok(id)
    }

    /// Returns all tables in ascending table ID order.
    pub fn tables(&self) -> impl Iterator<Item = &Table> {
        self.tables.values()
    }

    /// Borrows the table with the exact, case-sensitive name.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::TableNotFound`] if no name matches. Lookup
    /// performs exact matching, including for names outside the allowed format.
    pub fn get_table_by_name(&self, name: &str) -> Result<&Table, DatabaseError> {
        self.tables
            .values()
            .find(|table| table.name() == name)
            .ok_or(DatabaseError::TableNotFound)
    }

    /// Borrows the table with the supplied ID.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::TableNotFound`] if the ID is absent.
    pub fn get_table(&self, id: &TableId) -> Result<&Table, DatabaseError> {
        self.tables.get(id).ok_or(DatabaseError::TableNotFound)
    }

    /// Mutably borrows a table by its exact, case-sensitive name.
    /// Returns `TableNotFound` when no table matches.
    pub fn get_table_mut_by_name(&mut self, name: &str) -> Result<&mut Table, DatabaseError> {
        self.tables
            .values_mut()
            .find(|table| table.name() == name)
            .ok_or(DatabaseError::TableNotFound)
    }

    /// Mutably borrows a table by ID, or returns `TableNotFound`.
    pub fn get_table_mut(&mut self, id: &TableId) -> Result<&mut Table, DatabaseError> {
        self.tables.get_mut(id).ok_or(DatabaseError::TableNotFound)
    }

    /// Renames a table while preserving its ID and contents.
    ///
    /// The new name must contain 1 to 64 ASCII characters and match
    /// `[A-Za-z_][A-Za-z0-9_]*`. Its case is preserved and uniqueness is checked
    /// case-sensitively. Renaming to the table's current name succeeds without
    /// changing its state. The table remains accessible by its ID. After a name
    /// change, name lookup uses the new name.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::TableNotFound`] if the ID is absent,
    /// [`DatabaseError::InvalidTableName`] if the new name is invalid, or
    /// [`DatabaseError::TableAlreadyExists`] if another table has that exact name.
    /// These checks run in the order listed. On error, the table collection and
    /// ID counter remain unchanged.
    pub fn rename_table(&mut self, id: &TableId, new_name: &str) -> Result<(), DatabaseError> {
        if !self.table_exists(id) {
            return Err(DatabaseError::TableNotFound);
        }

        if !name::is_valid_name(new_name) {
            return Err(DatabaseError::InvalidTableName);
        }

        let name_taken = self
            .tables
            .iter()
            .any(|(other_id, table)| other_id != id && table.name() == new_name);

        if name_taken {
            return Err(DatabaseError::TableAlreadyExists);
        }

        let table = self
            .tables
            .get_mut(id)
            .ok_or(DatabaseError::TableNotFound)?;

        table.set_name(new_name);

        Ok(())
    }

    /// Finds a table by its exact name and renames it.
    ///
    /// Both lookup and uniqueness checks are case-sensitive. The new name must
    /// contain 1 to 64 ASCII characters and match `[A-Za-z_][A-Za-z0-9_]*`.
    /// The table's ID and contents are preserved. Using the current name as the
    /// new name succeeds without changing its state.
    ///
    /// # Errors
    ///
    /// Returns [`DatabaseError::TableNotFound`] if the current name is absent.
    /// Otherwise, returns the errors documented on [`Self::rename_table`].
    /// On error, the table collection and ID counter remain unchanged.
    pub fn rename_table_by_name(
        &mut self,
        current_name: &str,
        new_name: &str,
    ) -> Result<(), DatabaseError> {
        let id = self.get_table_by_name(current_name)?.id();
        self.rename_table(&id, new_name)
    }

    /// Removes a table by its database-local ID, including its columns and rows.
    ///
    /// The name becomes available for a new table. The ID counter is unchanged,
    /// so a new table with that name receives a fresh ID.
    ///
    /// # Errors
    /// Returns [`DatabaseError::TableNotFound`] if the ID is absent and leaves
    /// the database unchanged.
    pub fn delete_table(&mut self, id: &TableId) -> Result<(), DatabaseError> {
        self.tables
            .remove(id)
            .map(|_| ())
            .ok_or(DatabaseError::TableNotFound)
    }

    /// Adds a constraint to a table and updates affected stored cell warnings.
    ///
    /// `NotNull` keeps null values and marks them with [`CellWarning::NotNull`].
    /// Adding the same `NotNull` rule again succeeds without changing the table.
    ///
    /// # Errors
    /// Returns [`DatabaseError::TableNotFound`] for an unknown table,
    /// [`DatabaseError::Constraint`] for an invalid definition, or
    /// [`DatabaseError::UnsupportedConstraint`] for a rule other than `NotNull`.
    /// On error, the database is unchanged.
    pub fn add_constraint(
        &mut self,
        table_id: &TableId,
        constraint: Constraint,
    ) -> Result<(), DatabaseError> {
        let table = self.get_table_mut(table_id)?;

        match constraint {
            Constraint::NotNull { column } => table
                .add_not_null_constraint(column)
                .map_err(DatabaseError::Constraint),
            _ => Err(DatabaseError::UnsupportedConstraint),
        }
    }

    fn table_exists(&self, id: &TableId) -> bool {
        self.tables.contains_key(id)
    }

    fn table_name_exists(&self, name: &str) -> bool {
        self.tables.values().any(|table| table.name() == name)
    }
}
