use crate::database::{Database, DatabaseError, TableId};

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
    let mut database = Database::new("Database").expect("valid database name should be accepted");
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
    let mut database = Database::new("Database").expect("valid database name should be accepted");

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
    let mut database = Database::new("Database").expect("valid database name should be accepted");
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
