use std::collections::BTreeMap;

use super::error::TableError;
use super::name;
use super::row::{Row, RowId};
use crate::database::column::{Column, ColumnId};
use crate::database::value::Value;

use super::cell::Cell;
use super::constraint::Constraint;
use super::value::DataType;

/// A table identifier allocated by its owning database.
///
/// IDs are scoped to a database and remain unchanged when a table is renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableId(u64);

/// A table owned by a [`super::Database`].
///
/// Its ID is stable and its name can be changed through the owning database.
pub struct Table {
    id: TableId,
    name: String,
    columns: BTreeMap<ColumnId, Column>,
    rows: BTreeMap<RowId, Row>,
    constraints: Vec<Constraint>,
    next_column_id: u64,
    next_row_id: u64,
}

impl Table {
    pub(super) fn new(id: TableId, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            columns: BTreeMap::new(),
            rows: BTreeMap::new(),
            constraints: Vec::new(),
            next_column_id: 1,
            next_row_id: 1,
        }
    }

    /// Returns the table's ID within its owning database.
    pub fn id(&self) -> TableId {
        self.id
    }

    /// Returns the current table name with its stored case.
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn set_name(&mut self, new_name: &str) {
        self.name = new_name.to_string()
    }

    /// Creates a column with a stable, table-local ID and the supplied type.
    ///
    /// Names follow the database naming rules and are case-sensitive.
    /// Existing rows receive a valid null cell for the new column.
    ///
    /// # Errors
    /// Checks for a duplicate name, an invalid name, then ID exhaustion.
    /// On error, columns, rows and the ID counter remain unchanged.
    pub fn create_column(
        &mut self,
        name: &str,
        data_type: DataType,
    ) -> Result<ColumnId, TableError> {
        if self.column_name_exists(name) {
            return Err(TableError::ColumnAlreadyExists);
        }

        if !name::is_valid_name(name) {
            return Err(TableError::InvalidColumnName);
        }

        let next_column_id = self
            .next_column_id
            .checked_add(1)
            .ok_or(TableError::ColumnIdExhausted)?;

        let id = ColumnId::new(self.next_column_id);
        let column = Column::new(id, name, data_type);

        for row in self.rows.values_mut() {
            row.add_null_cell(id);
        }
        self.columns.insert(id, column);
        self.next_column_id = next_column_id;

        Ok(id)
    }

    /// Returns all columns in ascending column ID order.
    pub fn columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.values()
    }

    /// Borrows a column by ID, or returns `ColumnNotFound`.
    pub fn get_column(&self, id: &ColumnId) -> Result<&Column, TableError> {
        self.columns.get(id).ok_or(TableError::ColumnNotFound)
    }

    /// Borrows a column by its exact, case-sensitive name.
    /// Accepts owned or borrowed strings without allocating or taking a copy.
    /// Returns `ColumnNotFound` when absent, including for invalid name formats.
    pub fn get_column_by_name<S: AsRef<str>>(&self, name: S) -> Result<&Column, TableError> {
        let name = name.as_ref();
        self.columns
            .values()
            .find(|column| column.name() == name)
            .ok_or(TableError::ColumnNotFound)
    }

    /// Renames a column by its table-local ID.
    ///
    /// The new name must contain 1 to 64 ASCII characters and match
    /// `[A-Za-z_][A-Za-z0-9_]*`. Its case is preserved and uniqueness within
    /// the table is checked case-sensitively. Using the current name succeeds
    /// without changing the column.
    ///
    /// The column keeps its ID and data type. Existing cell values and their
    /// validation states are preserved. After a name change,
    /// [`Self::get_column_by_name`] resolves the column by its new name.
    ///
    /// # Errors
    /// Returns [`TableError::ColumnNotFound`] if the ID is absent,
    /// [`TableError::InvalidColumnName`] if the new name is invalid, or
    /// [`TableError::ColumnAlreadyExists`] if another column has that exact name.
    /// These checks run in the order listed. On error, the table is unchanged.
    pub fn rename_column(&mut self, id: &ColumnId, new_name: &str) -> Result<(), TableError> {
        let column = self.get_column(id)?;

        if !name::is_valid_name(new_name) {
            return Err(TableError::InvalidColumnName);
        }

        if column.name() == new_name {
            return Ok(());
        }

        if self.column_name_exists(new_name) {
            return Err(TableError::ColumnAlreadyExists);
        }

        let column = self.columns.get_mut(id).ok_or(TableError::ColumnNotFound)?;
        column.set_name(new_name);

        Ok(())
    }

    /// Removes a column by its table-local ID, its cells, and its `NotNull` rule.
    ///
    /// Other columns and cells are preserved. Deleting the last column leaves
    /// the rows in place with no cells. The name becomes available again, and
    /// the column ID counter is unchanged.
    ///
    /// # Errors
    /// Returns [`TableError::ColumnNotFound`] if the ID is absent and leaves the
    /// table unchanged.
    pub fn delete_column(&mut self, id: &ColumnId) -> Result<(), TableError> {
        self.columns.remove(id).ok_or(TableError::ColumnNotFound)?;
        self.constraints.retain(
            |constraint| !matches!(constraint, Constraint::NotNull { column } if column == id),
        );

        for row in self.rows.values_mut() {
            row.remove_cell(id);
        }

        Ok(())
    }

    /// Inserts values by column ID and returns a new table-local row ID.
    ///
    /// Omitted columns receive null cells, just like explicit null inputs.
    /// Nulls in `NotNull` columns and type mismatches are stored with a warning
    /// that preserves the original value. No implicit conversions are performed.
    /// Empty input creates an all-null row, or an empty row in a columnless table.
    ///
    /// # Errors
    /// Checks ID exhaustion first, then processes input in order, rejecting
    /// unknown columns or duplicate column IDs. Rows and the counter remain
    /// unchanged on error.
    pub fn add_row(
        &mut self,
        values: impl IntoIterator<Item = (ColumnId, Value)>,
    ) -> Result<RowId, TableError> {
        let next_row_id = self
            .next_row_id
            .checked_add(1)
            .ok_or(TableError::RowIdExhausted)?;

        let mut cells = BTreeMap::new();

        for (column_id, value) in values {
            let column = self.get_column(&column_id)?;

            if cells.contains_key(&column_id) {
                return Err(TableError::DuplicateColumn(column_id));
            }

            let cell = self.cell_for_value(column, value);
            cells.insert(column_id, cell);
        }

        for column in self.columns.values() {
            cells
                .entry(*column.id())
                .or_insert_with(|| self.cell_for_value(column, Value::Null));
        }

        let id = RowId::new(self.next_row_id);
        let row = Row::new(cells);

        self.rows.insert(id, row);
        self.next_row_id = next_row_id;

        Ok(id)
    }

    /// Resolves exact, case-sensitive column names and delegates to `add_row`.
    /// Accepts owned or borrowed names. Omitted values and type mismatches follow
    /// the same rules as `add_row`.
    ///
    /// # Errors
    /// Resolves names in input order, failing on the first unknown name before
    /// calling `add_row`. Otherwise returns its errors, including duplicate
    /// columns. On error, rows and the ID counter remain unchanged.
    pub fn add_row_by_column_names<S: AsRef<str>>(
        &mut self,
        values: impl IntoIterator<Item = (S, Value)>,
    ) -> Result<RowId, TableError> {
        let values = values
            .into_iter()
            .map(|(name, value)| {
                let id = self.column_name_to_id(name.as_ref())?;
                Ok((id, value))
            })
            .collect::<Result<Vec<_>, TableError>>()?;

        self.add_row(values)
    }

    /// Returns row IDs and rows in ascending row ID order.
    pub fn rows(&self) -> impl Iterator<Item = (RowId, &Row)> {
        self.rows.iter().map(|(id, row)| (*id, row))
    }

    /// Borrows the row with this table-local ID, or returns `RowNotFound`.
    pub fn get_row(&self, id: &RowId) -> Result<&Row, TableError> {
        self.rows.get(id).ok_or(TableError::RowNotFound)
    }

    /// Updates supplied cells while preserving omitted cells and the row ID.
    ///
    /// Updated cells follow the same validation rules as [`Self::add_row`].
    /// Empty input succeeds for an existing row. Failed operations leave all
    /// rows and ID counters unchanged.
    ///
    /// # Errors
    /// Checks row existence first, then rejects unknown or duplicate columns
    /// in input order.
    pub fn update_row(
        &mut self,
        id: &RowId,
        values: impl IntoIterator<Item = (ColumnId, Value)>,
    ) -> Result<(), TableError> {
        self.get_row(id)?;

        let mut changes = BTreeMap::new();

        for (column_id, value) in values {
            let column = self.get_column(&column_id)?;

            if changes.contains_key(&column_id) {
                return Err(TableError::DuplicateColumn(column_id));
            }

            changes.insert(column_id, self.cell_for_value(column, value));
        }

        let row = self.rows.get_mut(id).ok_or(TableError::RowNotFound)?;

        row.apply_changes(changes);

        Ok(())
    }

    /// Removes a row by its table-local ID, including all of its cells.
    ///
    /// Other rows and the row ID counter are unchanged.
    ///
    /// # Errors
    /// Returns [`TableError::RowNotFound`] if the ID is absent and leaves the
    /// table unchanged.
    pub fn delete_row(&mut self, id: &RowId) -> Result<(), TableError> {
        self.rows
            .remove(id)
            .map(|_| ())
            .ok_or(TableError::RowNotFound)
    }

    pub(super) fn add_not_null_constraint(&mut self, column: ColumnId) -> Result<(), TableError> {
        self.get_column(&column)?;

        if self.requires_non_null(&column) {
            return Ok(());
        }

        self.constraints.push(Constraint::NotNull { column });

        for row in self.rows.values_mut() {
            if let Some(cell @ Cell::Valid(Value::Null)) = row.get_cell_mut(&column) {
                *cell = Cell::not_null_warning();
            }
        }

        Ok(())
    }

    fn cell_for_value(&self, column: &Column, value: Value) -> Cell {
        if matches!(value, Value::Null) && self.requires_non_null(column.id()) {
            return Cell::not_null_warning();
        }

        Cell::from_value(value, *column.data_type())
    }

    fn requires_non_null(&self, id: &ColumnId) -> bool {
        self.constraints
            .iter()
            .any(|constraint| matches!(constraint, Constraint::NotNull { column } if column == id))
    }

    fn column_name_exists(&self, name: &str) -> bool {
        self.columns.values().any(|column| column.name() == name)
    }

    fn column_name_to_id(&self, name: &str) -> Result<ColumnId, TableError> {
        Ok(*self.get_column_by_name(name)?.id())
    }
}

impl TableId {
    pub(super) fn new(id: u64) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests;
