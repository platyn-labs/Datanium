use std::{fs, io};

use datanium_storage::{DatabaseStore, StorageError};
use tempfile::tempdir;

#[test]
fn opens_the_requested_database_with_a_corrupt_neighbor() {
    let dir = tempdir().unwrap();
    let writer = DatabaseStore::new(dir.path().to_path_buf());
    writer.create_empty_database("example").unwrap();
    writer.create_empty_database("other").unwrap();
    fs::write(dir.path().join("broken.dtn"), b"broken").unwrap();
    let reader = DatabaseStore::new(dir.path().to_path_buf());

    let database = reader.open_database("example").unwrap();

    assert_eq!(database.name(), "example");
    assert_eq!(database.tables().count(), 0);
}

#[test]
fn reports_a_missing_database_file() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let expected = dir.path().join("missing.dtn");

    assert!(matches!(
        store.open_database("missing"),
        Err(StorageError::Io { path, source })
            if path == expected && source.kind() == io::ErrorKind::NotFound
    ));
}

#[test]
fn rejects_invalid_names_and_path_components() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());

    for name in ["123example", "../example"] {
        assert!(matches!(
            store.open_database(name),
            Err(StorageError::InvalidDatabaseName)
        ));
    }
}

#[test]
fn rejects_a_directory_with_a_database_filename() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let path = dir.path().join("example.dtn");
    fs::create_dir(&path).unwrap();

    assert!(matches!(
        store.open_database("example"),
        Err(StorageError::InvalidFormat(found)) if found == path
    ));
}

#[cfg(unix)]
#[test]
fn rejects_a_symlink_to_a_valid_database() {
    let dir = tempdir().unwrap();
    let target = tempdir().unwrap();
    DatabaseStore::new(target.path().to_path_buf())
        .create_empty_database("example")
        .unwrap();
    let path = dir.path().join("example.dtn");
    std::os::unix::fs::symlink(target.path().join("example.dtn"), &path).unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());

    assert!(matches!(
        store.open_database("example"),
        Err(StorageError::InvalidFormat(found)) if found == path
    ));
}

#[test]
fn rejects_a_truncated_database_file() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let path = dir.path().join("example.dtn");
    fs::write(&path, b"DATN\x01\x00\x07exampl").unwrap();

    assert!(matches!(
        store.open_database("example"),
        Err(StorageError::InvalidFormat(found)) if found == path
    ));
}

#[test]
fn rejects_an_unsupported_format_version() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let path = dir.path().join("example.dtn");
    fs::write(&path, b"DATN\x02\x00\x07example").unwrap();

    assert!(matches!(
        store.open_database("example"),
        Err(StorageError::UnsupportedVersion { path: found, version: 2 }) if found == path
    ));
}

#[test]
fn rejects_a_filename_that_differs_from_the_database_name() {
    let dir = tempdir().unwrap();
    let store = DatabaseStore::new(dir.path().to_path_buf());
    let path = dir.path().join("other.dtn");
    fs::write(&path, b"DATN\x01\x00\x07example").unwrap();

    assert!(matches!(
        store.open_database("other"),
        Err(StorageError::InvalidFormat(found)) if found == path
    ));
}
