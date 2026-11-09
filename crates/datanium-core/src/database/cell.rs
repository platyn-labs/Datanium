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
/// A type mismatch retains the original value with a [`CellWarning::TypeMismatch`].
pub enum Cell {
    Valid(Value),
    Warning { value: Value, warning: CellWarning },
    Error { value: Value, error: CellError },
}

#[derive(Debug, PartialEq, Eq)]
/// Describes a validation issue reported on a stored value.
pub enum CellWarning {
    TypeMismatch {
        expected: DataType,
        actual: DataType,
    },
    NotNull,
}

impl Cell {
    /// Preserves mismatched values with a warning. Null is compatible with every type.
    pub(super) fn from_value(value: Value, expected: DataType) -> Self {
        match value.data_type() {
            Some(actual) if actual != expected => Self::Warning {
                value,
                warning: CellWarning::TypeMismatch { expected, actual },
            },
            _ => Self::Valid(value),
        }
    }

    pub(super) fn not_null_warning() -> Self {
        Self::Warning {
            value: Value::Null,
            warning: CellWarning::NotNull,
        }
    }
}
