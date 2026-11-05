mod constraint;
mod table;
mod value;

use std::collections::BTreeMap;

use self::table::Table;
pub use self::table::TableId;

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseError {
    TableIdExhausted,
}

pub struct Database {
    name: String,
    tables: BTreeMap<TableId, Table>,
    next_table_id: u64,
}

impl Database {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tables: BTreeMap::new(),
            next_table_id: 1,
        }
    }

    pub fn create_table(&mut self, name: &str) -> Result<TableId, DatabaseError> {
        let next_table_id = self
            .next_table_id
            .checked_add(1)
            .ok_or(DatabaseError::TableIdExhausted)?;

        let id = TableId::new(self.next_table_id);
        let table = Table::new(id, name);

        self.tables.insert(id, table);
        self.next_table_id = next_table_id;

        Ok(id)
    }
}
