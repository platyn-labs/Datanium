mod constraint;
mod table;
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

    pub fn create_table(&mut self, name: &str) -> Result<TableId, DatabaseError> {
        if self.tables.values().any(|table| table.name() == name) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Database creation tests

    #[test]
    fn accepts_valid_database_names() {
        for name in ["Database", "_database", "database_42", "D"] {
            let database = Database::new(name).expect("valid database name should be accepted");

            assert_eq!(database.name, name);
        }
    }

    #[test]
    fn rejects_invalid_database_names() {
        for name in ["", "1database", "my database", "my-db", "ä", "db\n"] {
            assert!(
                matches!(Database::new(name), Err(DatabaseError::InvalidDatabaseName)),
                "expected invalid database name: {name:?}"
            );
        }
    }

    #[test]
    fn accepts_database_name_with_64_characters() {
        let name = "a".repeat(64);

        let database = Database::new(&name).expect("64 characters should be accepted");

        assert_eq!(database.name, name);
    }

    #[test]
    fn rejects_database_name_with_65_characters() {
        let name = "a".repeat(65);

        assert!(matches!(
            Database::new(&name),
            Err(DatabaseError::InvalidDatabaseName)
        ));
    }

    // table creation tests

    #[test]
    fn test_table_ids() {
        let mut database =
            Database::new("Database").expect("valid database name should be accepted");
        assert_eq!(database.next_table_id, 1);

        let table_id = database
            .create_table("Table")
            .expect("creating a table should succeed");

        assert_eq!(table_id, TableId::new(1));
        assert_eq!(database.next_table_id, 2);
        assert_eq!(database.tables.len(), 1);

        let table = database
            .tables
            .get(&table_id)
            .expect("created table should be stpred under its returned ID");

        assert_eq!(table.name(), "Table");
    }

    #[test]
    fn exhausted_table_ids_return_error_without_changing_state() {
        let mut database =
            Database::new("Database").expect("valid database name should be accepted");

        let existing_id = database
            .create_table("Existing")
            .expect("setup should succeed");

        database.next_table_id = u64::MAX;
        let table_count_before = database.tables.len();

        let result = database.create_table("Another");

        assert_eq!(result, Err(DatabaseError::TableIdExhausted));
        assert_eq!(database.next_table_id, u64::MAX);
        assert_eq!(database.tables.len(), table_count_before);
        assert!(database.tables.contains_key(&existing_id));
    }

    #[test]
    fn test_duplicate_table_name() {
        let mut database =
            Database::new("Database").expect("valid database name should be accepted");
        let table_id = database.create_table("Table");
        let same_table_id = database.create_table("Table");
        assert_eq!(table_id.unwrap(), TableId::new(1));
        assert!(matches!(
            same_table_id,
            Err(DatabaseError::TableAlreadyExists)
        ));
    }

    #[test]
    fn create_table_rejects_invalid_names_without_changing_state() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let existing_id = database
            .create_table("Existing")
            .expect("setup should succeed");
        let too_long = "a".repeat(65);

        for name in [
            "", "1table", "my table", "my-table", "ä", "table\n", &too_long,
        ] {
            assert_eq!(
                database.create_table(name),
                Err(DatabaseError::InvalidTableName),
                "expected invalid table name: {name:?}"
            );
            assert_eq!(database.tables.len(), 1);
            assert_eq!(database.next_table_id, 2);
            assert_eq!(database.get_table(&existing_id).unwrap().name(), "Existing");
        }
    }

    #[test]
    fn create_table_accepts_name_with_64_characters() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let name = "a".repeat(64);

        let id = database
            .create_table(&name)
            .expect("64 characters should be accepted");

        assert_eq!(database.get_table(&id).unwrap().name(), name);
    }

    #[test]
    fn get_table_returns_table_with_requested_id() {
        let mut database = Database::new("Database").expect("setup should succeed");
        database
            .create_table("First")
            .expect("setup should succeed");
        let id = database
            .create_table("Second")
            .expect("setup should succeed");

        let table = database.get_table(&id).expect("table should be found");

        assert_eq!(table.id(), id);
        assert_eq!(table.name(), "Second");
    }

    #[test]
    fn get_table_returns_error_for_unknown_id() {
        let mut database = Database::new("Database").expect("setup should succeed");
        database
            .create_table("Existing")
            .expect("setup should succeed");

        assert!(matches!(
            database.get_table(&TableId::new(42)),
            Err(DatabaseError::TableNotFound)
        ));
    }

    #[test]
    fn get_table_by_name_returns_table_with_requested_name() {
        let mut database = Database::new("Database").expect("setup should succeed");
        database
            .create_table("First")
            .expect("setup should succeed");
        let id = database
            .create_table("Second")
            .expect("setup should succeed");

        let table = database
            .get_table_by_name("Second")
            .expect("table should be found");

        assert_eq!(table.id(), id);
        assert_eq!(table.name(), "Second");
    }

    #[test]
    fn get_table_by_name_returns_error_for_unknown_name() {
        let mut database = Database::new("Database").expect("setup should succeed");
        database
            .create_table("Existing")
            .expect("setup should succeed");

        assert!(matches!(
            database.get_table_by_name("Missing"),
            Err(DatabaseError::TableNotFound)
        ));
    }

    #[test]
    fn rename_table_preserves_id_and_updates_name_lookup() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");
        let other_id = database
            .create_table("Other")
            .expect("setup should succeed");

        database
            .rename_table(&id, "Renamed")
            .expect("renaming should succeed");

        assert_eq!(database.get_table(&id).unwrap().name(), "Renamed");
        assert_eq!(database.get_table_by_name("Renamed").unwrap().id(), id);
        assert!(matches!(
            database.get_table_by_name("Original"),
            Err(DatabaseError::TableNotFound)
        ));
        assert_eq!(database.get_table(&other_id).unwrap().name(), "Other");
        assert_eq!(database.tables.len(), 2);
        assert_eq!(database.next_table_id, 3);
    }

    #[test]
    fn rename_table_rejects_unknown_id_without_changing_state() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");

        assert_eq!(
            database.rename_table(&TableId::new(42), "Renamed"),
            Err(DatabaseError::TableNotFound)
        );
        assert_eq!(database.get_table(&id).unwrap().name(), "Original");
        assert!(matches!(
            database.get_table_by_name("Renamed"),
            Err(DatabaseError::TableNotFound)
        ));
        assert_eq!(database.tables.len(), 1);
        assert_eq!(database.next_table_id, 2);
    }

    #[test]
    fn rename_table_rejects_invalid_names_without_changing_state() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");
        let too_long = "a".repeat(65);

        for name in [
            "", "1table", "my table", "my-table", "ä", "table\n", &too_long,
        ] {
            assert_eq!(
                database.rename_table(&id, name),
                Err(DatabaseError::InvalidTableName),
                "expected invalid table name: {name:?}"
            );
            assert_eq!(database.get_table(&id).unwrap().name(), "Original");
            assert_eq!(database.get_table_by_name("Original").unwrap().id(), id);
            assert_eq!(database.tables.len(), 1);
            assert_eq!(database.next_table_id, 2);
        }
    }

    #[test]
    fn rename_table_rejects_duplicate_name_without_changing_state() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");
        let other_id = database
            .create_table("Taken")
            .expect("setup should succeed");

        assert_eq!(
            database.rename_table(&id, "Taken"),
            Err(DatabaseError::TableAlreadyExists)
        );
        assert_eq!(database.get_table(&id).unwrap().name(), "Original");
        assert_eq!(database.get_table_by_name("Original").unwrap().id(), id);
        assert_eq!(database.get_table(&other_id).unwrap().name(), "Taken");
        assert_eq!(database.get_table_by_name("Taken").unwrap().id(), other_id);
        assert_eq!(database.tables.len(), 2);
        assert_eq!(database.next_table_id, 3);
    }

    #[test]
    fn rename_table_accepts_unchanged_name() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");

        assert_eq!(database.rename_table(&id, "Original"), Ok(()));
        assert_eq!(database.get_table_by_name("Original").unwrap().id(), id);
        assert_eq!(database.tables.len(), 1);
        assert_eq!(database.next_table_id, 2);
    }

    #[test]
    fn rename_table_by_name_preserves_id_and_updates_name_lookup() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");
        let other_id = database
            .create_table("Other")
            .expect("setup should succeed");

        database
            .rename_table_by_name("Original", "Renamed")
            .expect("renaming should succeed");

        assert_eq!(database.get_table(&id).unwrap().name(), "Renamed");
        assert_eq!(database.get_table_by_name("Renamed").unwrap().id(), id);
        assert!(matches!(
            database.get_table_by_name("Original"),
            Err(DatabaseError::TableNotFound)
        ));
        assert_eq!(database.get_table(&other_id).unwrap().name(), "Other");
        assert_eq!(database.tables.len(), 2);
        assert_eq!(database.next_table_id, 3);
    }

    #[test]
    fn rename_table_by_name_rejects_unknown_name_without_changing_state() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");

        assert_eq!(
            database.rename_table_by_name("Missing", "Renamed"),
            Err(DatabaseError::TableNotFound)
        );
        assert_eq!(database.get_table(&id).unwrap().name(), "Original");
        assert!(matches!(
            database.get_table_by_name("Renamed"),
            Err(DatabaseError::TableNotFound)
        ));
        assert_eq!(database.tables.len(), 1);
        assert_eq!(database.next_table_id, 2);
    }

    #[test]
    fn rename_table_by_name_rejects_invalid_names_without_changing_state() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");
        let too_long = "a".repeat(65);

        for name in [
            "", "1table", "my table", "my-table", "ä", "table\n", &too_long,
        ] {
            assert_eq!(
                database.rename_table_by_name("Original", name),
                Err(DatabaseError::InvalidTableName),
                "expected invalid table name: {name:?}"
            );
            assert_eq!(database.get_table(&id).unwrap().name(), "Original");
            assert_eq!(database.get_table_by_name("Original").unwrap().id(), id);
            assert_eq!(database.tables.len(), 1);
            assert_eq!(database.next_table_id, 2);
        }
    }

    #[test]
    fn rename_table_by_name_rejects_duplicate_name_without_changing_state() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");
        let other_id = database
            .create_table("Taken")
            .expect("setup should succeed");

        assert_eq!(
            database.rename_table_by_name("Original", "Taken"),
            Err(DatabaseError::TableAlreadyExists)
        );
        assert_eq!(database.get_table(&id).unwrap().name(), "Original");
        assert_eq!(database.get_table_by_name("Original").unwrap().id(), id);
        assert_eq!(database.get_table(&other_id).unwrap().name(), "Taken");
        assert_eq!(database.get_table_by_name("Taken").unwrap().id(), other_id);
        assert_eq!(database.tables.len(), 2);
        assert_eq!(database.next_table_id, 3);
    }

    #[test]
    fn rename_table_by_name_accepts_unchanged_name() {
        let mut database = Database::new("Database").expect("setup should succeed");
        let id = database
            .create_table("Original")
            .expect("setup should succeed");

        assert_eq!(
            database.rename_table_by_name("Original", "Original"),
            Ok(())
        );
        assert_eq!(database.get_table_by_name("Original").unwrap().id(), id);
        assert_eq!(database.tables.len(), 1);
        assert_eq!(database.next_table_id, 2);
    }
}
