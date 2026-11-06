use std::collections::BTreeMap;
use std::fmt;

use crate::database::column::{Column, ColumnId};
use crate::database::value::Value;
use crate::helper;

use super::constraint::Constraint;
use super::value::{Cell, DataType};

/// A table identifier allocated by its owning database.
///
/// IDs are scoped to a database and remain unchanged when a table is renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A stable identifier scoped to its owning table.
pub struct RowId(u64);

impl RowId {
    /// Constructs an ID for lookup; this does not insert a row or reserve an ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// A table owned by a [`super::Database`].
///
/// Its ID is stable and its name can be changed through the owning database.
pub struct Table {
    id: TableId,
    name: String,
    columns: BTreeMap<ColumnId, Column>,
    rows: BTreeMap<RowId, Row>,
    #[expect(
        dead_code,
        reason = "Constraint enforcement will be implemented separately"
    )]
    constraints: Vec<Constraint>,
    next_column_id: u64,
    next_row_id: u64,
}

/// A stored row with one cell for every column in its table.
pub struct Row {
    cells: BTreeMap<ColumnId, Cell>,
}

impl Row {
    /// Borrows a cell. Returns `None` for a column absent from this row.
    /// A stored null is returned as `Some(Cell::Valid(Value::Null))`.
    pub fn get_cell(&self, column: &ColumnId) -> Option<&Cell> {
        self.cells.get(column)
    }
}

#[derive(Debug, PartialEq, Eq)]
/// A schema, row lookup, or insertion failure.
pub enum TableError {
    ColumnIdExhausted,
    RowIdExhausted,
    ColumnAlreadyExists,
    InvalidColumnName,
    ColumnNotFound,
    DuplicateColumn(ColumnId),
    RowNotFound,
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

        if !helper::is_valid_name(name) {
            return Err(TableError::InvalidColumnName);
        }

        let next_column_id = self
            .next_column_id
            .checked_add(1)
            .ok_or(TableError::ColumnIdExhausted)?;

        let id = ColumnId::new(self.next_column_id);
        let column = Column::new(id, name, data_type);

        for row in self.rows.values_mut() {
            row.cells.insert(id, Cell::Valid(super::value::Value::Null));
        }
        self.columns.insert(id, column);
        self.next_column_id = next_column_id;

        Ok(id)
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

    /// Inserts values by column ID and returns a new table-local row ID.
    ///
    /// Omitted columns become valid null cells. Explicit null is valid for every
    /// type. Type mismatches are stored as error cells retaining the original
    /// value; successful insertion can therefore contain error cells. No implicit
    /// conversions or constraint enforcement are performed.
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

            let cell = Cell::from_value(value, *column.data_type());
            cells.insert(column_id, cell);
        }

        for column_id in self.columns.keys() {
            cells.entry(*column_id).or_insert(Cell::Valid(Value::Null));
        }

        let id = RowId::new(self.next_row_id);
        let row = Row { cells };

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

    /// Borrows the row with this table-local ID, or returns `RowNotFound`.
    pub fn get_row(&self, id: &RowId) -> Result<&Row, TableError> {
        self.rows.get(id).ok_or(TableError::RowNotFound)
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

impl fmt::Display for TableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ColumnIdExhausted => "column ID counter is exhausted",
            Self::RowIdExhausted => "row ID counter is exhausted",
            Self::ColumnAlreadyExists => "a column with this name already exists",
            Self::InvalidColumnName => "invalid column name",
            Self::ColumnNotFound => "column not found",
            Self::DuplicateColumn(_) => "a column was supplied more than once",
            Self::RowNotFound => "row not found",
        })
    }
}

impl std::error::Error for TableError {}

#[cfg(test)]
mod tests;
