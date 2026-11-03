use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct DatabaseId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TableId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ColumnId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RowId(u64);

struct Database {
    id: DatabaseId,
    name: String,
    tables: BTreeMap<TableId, Table>,
}

struct Table {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataType {
    Int,
    Text,
    Bool,
}

struct Row {
    cells: BTreeMap<ColumnId, Cell>,
}

#[derive(Debug, PartialEq, Eq)]
enum Cell {
    Valid(Value),
    Warning { value: Value, message: String },
    Error { value: Value, error: CellError },
}

#[derive(Debug, PartialEq, Eq)]
enum TableError {
    UnknownColumn(ColumnId),
    UnknownRow(RowId),
}

#[derive(Debug, PartialEq, Eq)]
enum CellError {
    TypeMismatch {
        expected: DataType,
        actual: DataType,
    },
}

#[derive(Debug, PartialEq, Eq)]
enum Value {
    Null,
    Int(i64),
    Text(String),
    Bool(bool),
}

enum Constraint {
    NotNull {
        column: ColumnId,
    },
    Unique {
        columns: Vec<ColumnId>,
    },
    PrimaryKey {
        columns: Vec<ColumnId>,
    },
    ForeignKey {
        target_table: TableId,
        columns: Vec<ForeignKeyColumn>,
        on_delete: ReferentialAction,
        on_update: ReferentialAction,
    },
}

struct ForeignKeyColumn {
    source: ColumnId,
    target: ColumnId,
}

enum ReferentialAction {
    Restrict,
    Cascade,
    SetNull,
}
