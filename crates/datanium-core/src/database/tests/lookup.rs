use crate::database::{Database, DatabaseError, TableId};

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
