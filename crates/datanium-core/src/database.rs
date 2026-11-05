mod constraint;
mod table;
#[cfg(test)]
mod tests;
mod value;

use std::collections::BTreeMap;

pub use self::table::{Table, TableId};

#[derive(Debug, PartialEq, Eq)]
pub enum DatabaseError {
    TableIdExhausted,
    TableAlreadyExists,
    InvalidDatabaseName,
    InvalidTableName,
    TableNotFound,
}

pub struct Database {
    name: String,
    tables: BTreeMap<TableId, Table>,
    next_table_id: u64,
}

impl Database {
    pub fn new(name: &str) -> Result<Self, DatabaseError> {
        if !Database::is_valid_name(name) {
            return Err(DatabaseError::InvalidDatabaseName);
        }

        Ok(Self {
            name: name.to_string(),
            tables: BTreeMap::new(),
            next_table_id: 1,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn create_table(&mut self, name: &str) -> Result<TableId, DatabaseError> {
        if self.table_name_exists(name) {
            return Err(DatabaseError::TableAlreadyExists);
        }

        if !Database::is_valid_name(name) {
            return Err(DatabaseError::InvalidTableName);
        }

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

    pub fn get_table_by_name(&self, name: &str) -> Result<&Table, DatabaseError> {
        self.tables
            .values()
            .find(|table| table.name() == name)
            .ok_or(DatabaseError::TableNotFound)
    }

    pub fn get_table(&self, id: &TableId) -> Result<&Table, DatabaseError> {
        self.tables.get(id).ok_or(DatabaseError::TableNotFound)
    }

    pub fn rename_table(&mut self, id: &TableId, new_name: &str) -> Result<(), DatabaseError> {
        if !self.table_exists(id) {
            return Err(DatabaseError::TableNotFound);
        }

        if !Database::is_valid_name(new_name) {
            return Err(DatabaseError::InvalidTableName);
        }

        let name_taken = self
            .tables
            .iter()
            .any(|(other_id, table)| other_id != id && table.name() == new_name);

        if name_taken {
            return Err(DatabaseError::TableAlreadyExists);
        }

        let table = self
            .tables
            .get_mut(id)
            .ok_or(DatabaseError::TableNotFound)?;

        table.set_name(new_name);

        Ok(())
    }

    pub fn rename_table_by_name(
        &mut self,
        current_name: &str,
        new_name: &str,
    ) -> Result<(), DatabaseError> {
        let id = self.get_table_by_name(current_name)?.id();
        self.rename_table(&id, new_name)
    }

    fn table_exists(&self, id: &TableId) -> bool {
        self.tables.contains_key(id)
    }

    fn table_name_exists(&self, name: &str) -> bool {
        self.tables.values().any(|table| table.name() == name)
    }

    fn is_valid_name(name: &str) -> bool {
        let mut bytes = name.bytes();

        let valid_start = bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');

        let valid_rest = bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');

        name.len() <= 64 && valid_start && valid_rest
    }
}
