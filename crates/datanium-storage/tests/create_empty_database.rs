use std::{fs, io};

use datanium_storage::{DatabaseStore, StorageError};
use tempfile::tempdir;

#[test]
fn creates_an_empty_database_and_writes_the_v1_file() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());

    let database = store.create_empty_database("example").unwrap();

    assert_eq!(database.name(), "example");
    assert_eq!(database.tables().count(), 0);
    let bytes = fs::read(dir.path().join("example.dtn")).unwrap();
    assert_eq!(bytes.as_slice(), b"DATN\x01\x00\x07example");
}

#[test]
fn rejects_a_duplicate_without_changing_the_existing_file() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let file_path = dir.path().join("example.dtn");
    store.create_empty_database("example").unwrap();
    let original = fs::read(&file_path).unwrap();

    let result = store.create_empty_database("example");

    assert!(matches!(result, Err(StorageError::AreadyExists(path)) if path == file_path));
    assert_eq!(fs::read(&file_path).unwrap(), original);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn rejects_an_invalid_name_without_creating_a_file() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());

    let result = store.create_empty_database("123example");

    assert!(matches!(result, Err(StorageError::InvalidDatabaseName)));
    assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
}

#[test]
fn reports_a_missing_data_directory() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("missing");
    let store = DatabaseStore::new(data_dir.clone());

    let result = store.create_empty_database("example");

    assert!(matches!(
        result,
        Err(StorageError::Io { path, source })
            if path == data_dir && source.kind() == io::ErrorKind::NotFound
    ));
}
