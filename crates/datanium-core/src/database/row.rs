use std::collections::BTreeMap;

use super::cell::Cell;
use super::column::ColumnId;
use super::value::Value;

/// A stable identifier scoped to its owning table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RowId(u64);

impl RowId {
    /// Constructs an ID for lookup; this does not insert a row or reserve an ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// A stored row with one cell for every column in its table.
pub struct Row {
    cells: BTreeMap<ColumnId, Cell>,
}

impl Row {
    pub(super) fn new(cells: BTreeMap<ColumnId, Cell>) -> Self {
        Self { cells }
    }

    pub(super) fn add_null_cell(&mut self, column: ColumnId) {
        self.cells.insert(column, Cell::Valid(Value::Null));
    }

    /// Borrows a cell. Returns `None` for a column absent from this row.
    /// A stored null is returned as `Some(Cell::Valid(Value::Null))`.
    pub fn get_cell(&self, column: &ColumnId) -> Option<&Cell> {
        self.cells.get(column)
    }

    pub(super) fn apply_changes(&mut self, changes: BTreeMap<ColumnId, Cell>) {
        self.cells.extend(changes);
    }
}
