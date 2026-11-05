use std::collections::BTreeMap;

use super::constraint::Constraint;
use super::value::{Cell, DataType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TableId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ColumnId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RowId(u64);

pub(super) struct Table {
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

    pub(super) fn name(&self) -> &str {
        &self.name
    }
}

impl TableId {
    pub(super) fn new(id: u64) -> Self {
        Self(id)
    }
}
