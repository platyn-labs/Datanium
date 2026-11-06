use crate::database::{Database, DatabaseError, TableId};

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
