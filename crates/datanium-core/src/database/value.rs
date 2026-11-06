#[derive(Debug, PartialEq, Eq)]
/// Describes why an original input value does not match its column.
pub enum CellError {
    TypeMismatch {
        expected: DataType,
        actual: DataType,
    },
}

#[derive(Debug, PartialEq, Eq)]
/// An input or stored scalar. Null has no concrete data type.
pub enum Value {
    Null,
    Int(i64),
    Text(String),
    Bool(bool),
}

impl Value {
    /// Returns the concrete type, or `None` for null.
    pub(super) fn data_type(&self) -> Option<DataType> {
        match self {
            Self::Null => None,
            Self::Int(_) => Some(DataType::Int),
            Self::Text(_) => Some(DataType::Text),
            Self::Bool(_) => Some(DataType::Bool),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Int,
    Text,
    Bool,
}
