use std::collections::BTreeMap;
use std::fmt;

use crate::database::column::{Column, ColumnId};
use crate::helper;

use super::constraint::Constraint;
use super::value::{Cell, DataType};

/// A table identifier allocated by its owning database.
///
/// IDs are scoped to a database and remain unchanged when a table is renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RowId(u64);

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
    #[expect(dead_code, reason = "Row insertion will be implemented separately")]
    next_row_id: u64,
}

struct Row {
    cells: BTreeMap<ColumnId, Cell>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TableError {
    ColumnIdExhausted,
    ColumnAlreadyExists,
    InvalidColumnName,
    ColumnNotFound,
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

    /// Borrows a column by its exact name, or returns `ColumnNotFound`.
    pub fn get_column_by_name(&self, name: &str) -> Result<&Column, TableError> {
        self.columns
            .values()
            .find(|column| column.name() == name)
            .ok_or(TableError::ColumnNotFound)
    }

    fn column_name_exists(&self, name: &str) -> bool {
        self.columns.values().any(|column| column.name() == name)
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
            Self::ColumnAlreadyExists => "a column with this name already exists",
            Self::InvalidColumnName => "invalid column name",
            Self::ColumnNotFound => "column not found",
        })
    }
}

impl std::error::Error for TableError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_columns_preserve_state_and_counter() {
        let mut table = Table::new(TableId::new(1), "Items");
        let id = table.create_column("Name", DataType::Text).unwrap();
        let long_name = "a".repeat(65);
        for name in ["", "1name", "my name", "my-name", "ä", "name\n", &long_name] {
            assert_eq!(
                table.create_column(name, DataType::Int),
                Err(TableError::InvalidColumnName)
            );
            assert_eq!(table.columns.len(), 1);
            assert_eq!(table.next_column_id, 2);
        }
        assert_eq!(
            table.create_column("Name", DataType::Bool),
            Err(TableError::ColumnAlreadyExists)
        );
        table.next_column_id = u64::MAX;
        assert_eq!(
            table.create_column("Other", DataType::Int),
            Err(TableError::ColumnIdExhausted)
        );
        assert_eq!(table.next_column_id, u64::MAX);
        assert_eq!(table.columns.len(), 1);
        assert_eq!(table.get_column(&id).unwrap().data_type(), &DataType::Text);
    }

    #[test]
    fn new_column_backfills_existing_rows_without_replacing_cells() {
        let mut table = Table::new(TableId::new(1), "Items");
        let first = table.create_column("Name", DataType::Text).unwrap();
        for number in 1..=2 {
            table.rows.insert(
                RowId(number),
                Row {
                    cells: BTreeMap::from([(first, Cell::Valid(super::super::value::Value::Null))]),
                },
            );
        }
        let second = table.create_column("Active", DataType::Bool).unwrap();
        for row in table.rows.values() {
            assert_eq!(row.cells.len(), 2);
            assert_eq!(
                row.cells.get(&first),
                Some(&Cell::Valid(super::super::value::Value::Null))
            );
            assert_eq!(
                row.cells.get(&second),
                Some(&Cell::Valid(super::super::value::Value::Null))
            );
        }
    }
}
