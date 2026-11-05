use std::collections::BTreeMap;

use super::constraint::Constraint;
use super::value::{Cell, DataType};

/// A table identifier allocated by its owning database.
///
/// IDs are scoped to a database and remain unchanged when a table is renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ColumnId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RowId(u64);

/// A table owned by a [`super::Database`].
///
/// Its ID is stable and its name can be changed through the owning database.
pub struct Table {
    id: TableId,
    name: String,
    columns: BTreeMap<ColumnId, Column>,
    rows: BTreeMap<RowId, Row>,
    constraints: Vec<Constraint>,
}

struct Column {
    name: String,
    data_type: DataType,
}

struct Row {
    cells: BTreeMap<ColumnId, Cell>,
}

#[derive(Debug, PartialEq, Eq)]
enum TableError {
    UnknownColumn(ColumnId),
    UnknownRow(RowId),
}

impl Table {
    pub(super) fn new(id: TableId, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            columns: BTreeMap::new(),
            rows: BTreeMap::new(),
            constraints: Vec::new(),
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
}

impl TableId {
    pub(super) fn new(id: u64) -> Self {
        Self(id)
    }
}
