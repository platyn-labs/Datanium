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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Int,
    Text,
    Bool,
}
