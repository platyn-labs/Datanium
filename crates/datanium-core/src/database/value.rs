#[derive(Debug, PartialEq, Eq)]
pub(super) enum CellError {
    TypeMismatch {
        expected: DataType,
        actual: DataType,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Value {
    Null,
    Int(i64),
    Text(String),
    Bool(bool),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Cell {
    Valid(Value),
    Warning { value: Value, message: String },
    Error { value: Value, error: CellError },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DataType {
    Int,
    Text,
    Bool,
}
