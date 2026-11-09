use std::collections::BTreeMap;

use datanium_core::database::Database;
use datanium_storage::{DatabaseStore, StorageError};

#[derive(Debug)]
pub(crate) enum CreateDatabaseError {
    AlreadyExists,
    Storage(StorageError),
}

pub(crate) struct DatabaseCatalog {
    store: DatabaseStore,
    databases: BTreeMap<String, Database>,
}

impl DatabaseCatalog {
    pub(crate) fn open(store: DatabaseStore) -> Result<Self, StorageError> {
        let databases = store
            .load_databases()?
            .into_iter()
            .map(|database| (database.name().to_owned(), database))
            .collect();

        Ok(Self { store, databases })
    }

    pub(crate) fn create_database(&mut self, name: &str) -> Result<(), CreateDatabaseError> {
        if self.databases.contains_key(name) {
            return Err(CreateDatabaseError::AlreadyExists);
        }

        let database = self
            .store
            .create_empty_database(name)
            .map_err(|error| match error {
                StorageError::AreadyExists(_) => CreateDatabaseError::AlreadyExists,
                error => CreateDatabaseError::Storage(error),
            })?;

        self.databases.insert(database.name().to_owned(), database);
        Ok(())
    }

    pub(crate) fn list_databases(&self) -> impl Iterator<Item = &str> {
        self.databases.keys().map(String::as_str)
    }
}
