use crate::database::ColumnId;

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
    Warning {
        value: Value,
        warnings: Vec<CellWarning>,
    },
    Error {
        value: Value,
        error: CellError,
    },
}

#[derive(Debug, PartialEq, Eq)]
/// Describes a validation issue reported on a stored value.
pub enum CellWarning {
    TypeMismatch {
        expected: DataType,
        actual: DataType,
    },
    NotNull,
    Unique {
        columns: Vec<ColumnId>,
    },
}

impl Cell {
    /// Preserves mismatched values with a warning. Null is compatible with every type.
    pub(super) fn from_value(value: Value, expected: DataType) -> Self {
        match value.data_type() {
            Some(actual) if actual != expected => Self::Warning {
                value,
                warnings: vec![CellWarning::TypeMismatch { expected, actual }],
            },
            _ => Self::Valid(value),
        }
    }

    pub(super) fn not_null_warning() -> Self {
        Self::Warning {
            value: Value::Null,
            warnings: vec![CellWarning::NotNull],
        }
    }

    pub(super) fn unique_value(&self) -> Option<&Value> {
        let value = match self {
            Self::Valid(value) => value,
            Self::Warning { value, warnings }
                if !warnings
                    .iter()
                    .any(|warning| matches!(warning, CellWarning::TypeMismatch { .. })) =>
            {
                value
            }
            _ => return None,
        };

        if matches!(value, Value::Null) {
            None
        } else {
            Some(value)
        }
    }

    pub(super) fn set_unique_warning(&mut self, columns: &[ColumnId], violated: bool) {
        if let Self::Warning { warnings, .. } = self {
            warnings.retain(|warning| {
                !matches!(
                    warning,
                    CellWarning::Unique { columns: existing }
                        if existing.as_slice() == columns
                )
            });
        }

        if violated {
            let warning = CellWarning::Unique {
                columns: columns.to_vec(),
            };

            match self {
                Self::Valid(value) => {
                    let value = std::mem::replace(value, Value::Null);

                    *self = Self::Warning {
                        value,
                        warnings: vec![warning],
                    };
                }
                Self::Warning { warnings, .. } => {
                    warnings.push(warning);
                }
                Self::Error { .. } => {}
            }
        }

        if let Self::Warning { value, warnings } = self
            && warnings.is_empty()
        {
            let value = std::mem::replace(value, Value::Null);
            *self = Self::Valid(value);
        }
    }
}
