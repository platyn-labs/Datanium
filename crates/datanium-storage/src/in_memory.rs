//! Volatile key-value storage backed by a hash map.

use std::collections::HashMap;

use datanium_core::storage::Storage;

/// An in-memory implementation of the storage contract.
///
/// Each instance owns an independent collection of keys and values. All entries
/// are discarded when the instance is dropped. Empty keys and values are valid.
///
/// # Example
///
/// ```
/// use datanium_core::storage::Storage;
/// use datanium_storage::in_memory::InMemoryStorage;
///
/// let mut storage = InMemoryStorage::new();
/// storage.put(b"name", b"Datanium").expect("write succeeds");
/// assert_eq!(storage.get(b"name"), Ok(Some(b"Datanium".to_vec())));
///
/// storage.delete(b"name").expect("deletion succeeds");
/// assert_eq!(storage.get(b"name"), Ok(None));
/// ```
#[derive(Default, Debug)]
pub struct InMemoryStorage {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl InMemoryStorage {
    /// Creates an empty storage instance, equivalent to [`Self::default`].
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Storage for InMemoryStorage {
    /// Operations return successful results and use an uninhabited error type.
    type Error = std::convert::Infallible;

    /// Returns an owned copy of the value, or `Ok(None)` for an absent key.
    ///
    /// Modifying the returned value leaves the stored value intact.
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        let result = self.data.get(key);
        Ok(result.cloned())
    }

    /// Copies the key and value into this instance, replacing any existing value.
    ///
    /// A successful write is immediately visible to subsequent reads of this
    /// instance. Its lifetime is limited to the lifetime of this instance.
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        let key = key.to_vec();
        let value = value.to_vec();
        self.data.insert(key, value);
        Ok(())
    }

    /// Removes the entry, succeeding even when the key is already absent.
    fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error> {
        self.data.remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_new_data() {
        let mut storage = InMemoryStorage::new();
        let key = b"name";
        let value = b"Datanium";

        assert_eq!(storage.get(key), Ok(None));
        assert_eq!(storage.put(key, value), Ok(()));
        assert_eq!(storage.get(key), Ok(Some(value.to_vec())));
    }
}
