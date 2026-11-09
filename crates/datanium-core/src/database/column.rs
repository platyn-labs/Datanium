use crate::database::value::DataType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A stable identifier scoped to its owning table.
pub struct ColumnId(u64);

impl ColumnId {
    pub(super) fn new(id: u64) -> Self {
        Self(id)
    }
}

/// The name and fixed data type of a table column.
pub struct Column {
    id: ColumnId,
    name: String,
    data_type: DataType,
}

impl Column {
    pub(super) fn new(id: ColumnId, name: &str, data_type: DataType) -> Self {
        Self {
            id,
            name: name.to_string(),
            data_type,
        }
    }

    pub fn id(&self) -> &ColumnId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn set_name(&mut self, new_name: &str) {
        self.name = new_name.to_string()
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }
}
