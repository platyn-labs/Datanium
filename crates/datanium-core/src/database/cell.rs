use super::value::{DataType, Value};

#[derive(Debug, PartialEq, Eq)]
/// Describes why an original input value does not match its column.
pub enum CellError {
    TypeMismatch {
        expected: DataType,
        actual: DataType,
    },
}

#[derive(Debug, PartialEq, Eq)]
/// A stored value and its validation status.
/// Warning is reserved; current insertion creates only valid or error cells.
pub enum Cell {
    Valid(Value),
    Warning { value: Value, message: String },
    Error { value: Value, error: CellError },
}

impl Cell {
    /// Preserves mismatched values as errors; null is valid for every type.
    pub(super) fn from_value(value: Value, expected: DataType) -> Self {
        match value.data_type() {
            Some(actual) if actual != expected => Self::Error {
                value,
                error: CellError::TypeMismatch { expected, actual },
            },
            _ => Self::Valid(value),
        }
    }
}
