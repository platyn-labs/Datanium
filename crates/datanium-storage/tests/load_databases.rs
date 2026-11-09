use std::{fs, io};

use datanium_storage::{DatabaseStore, StorageError};
use tempfile::tempdir;

#[test]
fn loads_databases_written_by_another_store() {
    let dir = tempdir().unwrap();
    let writer = DatabaseStore::new(dir.path().to_path_buf());
    writer.create_empty_database("example").unwrap();
    writer.create_empty_database("second").unwrap();
    let reader = DatabaseStore::new(dir.path().to_path_buf());

    let databases = reader.load_databases().unwrap();

    let mut names: Vec<_> = databases.iter().map(|database| database.name()).collect();
    names.sort_unstable();
    assert_eq!(names, ["example", "second"]);
    assert!(
        databases
            .iter()
            .all(|database| database.tables().count() == 0)
    );
}

#[test]
fn loads_no_databases_from_an_empty_directory() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());

    assert!(store.load_databases().unwrap().is_empty());
}

#[test]
fn skips_other_files_and_subdirectories() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    store.create_empty_database("example").unwrap();
    fs::write(dir.path().join("notes.txt"), b"notes").unwrap();
    let nested = dir.path().join("nested.dtn");
    fs::create_dir(&nested).unwrap();
    DatabaseStore::new(nested)
        .create_empty_database("nested")
        .unwrap();

    let databases = store.load_databases().unwrap();

    assert_eq!(databases.len(), 1);
    assert_eq!(databases[0].name(), "example");
}

#[test]
fn reports_a_missing_data_directory() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("missing");
    let store = DatabaseStore::new(data_dir.clone());

    assert!(matches!(
        store.load_databases(),
        Err(StorageError::Io { path, source })
            if path == data_dir && source.kind() == io::ErrorKind::NotFound
    ));
}

#[test]
fn rejects_a_truncated_database_file() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let path = dir.path().join("example.dtn");
    fs::write(&path, b"DATN\x01\x00\x07exampl").unwrap();

    assert!(
        matches!(store.load_databases(), Err(StorageError::InvalidFormat(found)) if found == path)
    );
}

#[test]
fn rejects_an_unsupported_format_version() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let path = dir.path().join("example.dtn");
    fs::write(&path, b"DATN\x02\x00\x07example").unwrap();

    assert!(matches!(
        store.load_databases(),
        Err(StorageError::UnsupportedVersion { path: found, version: 2 }) if found == path
    ));
}

#[test]
fn rejects_a_filename_that_differs_from_the_database_name() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let path = dir.path().join("other.dtn");
    fs::write(&path, b"DATN\x01\x00\x07example").unwrap();

    assert!(
        matches!(store.load_databases(), Err(StorageError::InvalidFormat(found)) if found == path)
    );
}
