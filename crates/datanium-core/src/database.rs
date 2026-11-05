//! In-memory relational database model.
//!
//! Database and table names contain 1 to 64 ASCII characters and match
//! `[A-Za-z_][A-Za-z0-9_]*`. Names are preserved as supplied. Table lookup and
//! uniqueness checks are case-sensitive, so `Customers` and `customers` are
//! distinct names.

mod constraint;
mod table;
#[cfg(test)]
mod tests;
mod value;

use std::collections::BTreeMap;
use std::fmt;

pub use self::table::{Table, TableId};

/// An error returned when creating a database or accessing its tables.
#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseError {
    /// The table ID counter cannot be advanced.
    TableIdExhausted,
    /// Another table in this database already has the requested name.
    TableAlreadyExists,
    /// The database name does not follow the documented name format.
    InvalidDatabaseName,
    /// The table name does not follow the documented name format.
    InvalidTableName,
    /// No table matches the requested ID or exact name.
    TableNotFound,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TableIdExhausted => "table ID counter is exhausted",
            Self::TableAlreadyExists => "a table with this name already exists",
            Self::InvalidDatabaseName => "invalid database name",
            Self::InvalidTableName => "invalid table name",
            Self::TableNotFound => "table not found",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for DatabaseError {}

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
        if !Database::is_valid_name(name) {
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

        if !Database::is_valid_name(name) {
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

        if !Database::is_valid_name(new_name) {
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

    fn table_exists(&self, id: &TableId) -> bool {
        self.tables.contains_key(id)
    }

    fn table_name_exists(&self, name: &str) -> bool {
        self.tables.values().any(|table| table.name() == name)
    }

    fn is_valid_name(name: &str) -> bool {
        let mut bytes = name.bytes();

        let valid_start = bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');

        let valid_rest = bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');

        name.len() <= 64 && valid_start && valid_rest
    }
}
