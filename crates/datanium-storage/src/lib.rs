//! Persistent storage for Datanium database data.
//!
//! This crate is reserved for loading and saving database state.
//! Persistence is not implemented yet.

use std::{
    ffi::OsStr,
    fmt, fs,
    io::{self, Read, Write},
    path::PathBuf,
};

use datanium_core::database::Database;
use tempfile::NamedTempFile;

mod format;

#[derive(Debug)]
pub enum StorageError {
    InvalidDatabaseName,
    AreadyExists(PathBuf),
    InvalidFormat(PathBuf),
    Io { path: PathBuf, source: io::Error },
    UnsupportedVersion { path: PathBuf, version: u16 },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDatabaseName => write!(f, "Invalid database name."),
            Self::AreadyExists(path) => {
                write!(f, "Database already exists at '{}'.", path.display())
            }
            Self::InvalidFormat(path) => write!(f, "Invalid database file '{}'.", path.display()),
            Self::Io { path, source } => {
                write!(f, "Could not access '{}': {source}", path.display())
            }
            Self::UnsupportedVersion { path, version } => write!(
                f,
                "Unsupported database format version {version} in '{}'.",
                path.display()
            ),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub struct DatabaseStore {
    data_dir: PathBuf,
}

impl DatabaseStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn create_empty_database(&self, name: &str) -> Result<Database, StorageError> {
        let database = Database::new(name).map_err(|_| StorageError::InvalidDatabaseName)?;
        let path = self.data_dir.join(format!("{}.dtn", database.name()));
        let bytes = format::encode_empty_header(&database);

        let mut temp =
            NamedTempFile::new_in(&self.data_dir).map_err(|source| StorageError::Io {
                path: self.data_dir.clone(),
                source,
            })?;

        temp.write_all(&bytes)
            .and_then(|()| temp.as_file().sync_all())
            .map_err(|source| StorageError::Io {
                path: temp.path().to_path_buf(),
                source,
            })?;

        fs::hard_link(temp.path(), &path).map_err(|source| {
            if source.kind() == io::ErrorKind::AlreadyExists {
                StorageError::AreadyExists(path)
            } else {
                StorageError::Io { path, source }
            }
        })?;

        Ok(database)
    }

    pub fn load_databases(&self) -> Result<Vec<Database>, StorageError> {
        let directory_error = |source| StorageError::Io {
            path: self.data_dir.clone(),
            source,
        };

        let mut databases = Vec::new();

        for entry in fs::read_dir(&self.data_dir).map_err(directory_error)? {
            let entry = entry.map_err(directory_error)?;
            let path = entry.path();

            if path.extension() != Some(OsStr::new("dtn")) {
                continue;
            }

            let file_error = |source| StorageError::Io {
                path: path.clone(),
                source,
            };

            if !entry.file_type().map_err(file_error)?.is_file() {
                continue;
            }

            let mut bytes = Vec::new();
            fs::File::open(&path)
                .map_err(file_error)?
                .take((format::MAX_FILE_LEN + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(file_error)?;

            let database = format::decode_empty_database(&bytes).map_err(|error| match error {
                format::FormatError::UnsupportedVersion(version) => {
                    StorageError::UnsupportedVersion {
                        path: path.clone(),
                        version,
                    }
                }
                _ => StorageError::InvalidFormat(path.clone()),
            })?;

            if path.file_stem() != Some(OsStr::new(database.name())) {
                return Err(StorageError::InvalidFormat(path));
            }

            databases.push(database);
        }

        Ok(databases)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn creates_an_empty_database_and_writes_the_v1_file() {
        let dir = tempdir().unwrap();
        let store = DatabaseStore::new(dir.path().to_path_buf());

        let Ok(database) = store.create_empty_database("example") else {
            panic!("creating an empty database should succeed");
        };

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
        assert!(store.create_empty_database("example").is_ok());
        let original = fs::read(&file_path).unwrap();

        let result = store.create_empty_database("example");

        assert!(matches!(result, Err(StorageError::AreadyExists(path)) if path == file_path));
        assert_eq!(fs::read(&file_path).unwrap(), original);
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
    fn loads_a_database_written_by_another_store() {
        let dir = tempdir().unwrap();
        let writer = DatabaseStore::new(dir.path().to_path_buf());
        assert!(writer.create_empty_database("example").is_ok());
        let reader = DatabaseStore::new(dir.path().to_path_buf());

        let Ok(databases) = reader.load_databases() else {
            panic!("loading the database should succeed");
        };

        assert_eq!(databases.len(), 1);
        assert_eq!(databases[0].name(), "example");
        assert_eq!(databases[0].tables().count(), 0);
    }

    #[test]
    fn loads_no_databases_from_an_empty_directory() {
        let dir = tempdir().unwrap();
        let store = DatabaseStore::new(dir.path().to_path_buf());

        assert!(matches!(store.load_databases(), Ok(databases) if databases.is_empty()));
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
}
