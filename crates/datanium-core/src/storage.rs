//! Backend-independent key-value storage contract.
//!
//! Storage backends implement this interface and depend on datanium-core.
//! Keys and values are arbitrary byte sequences, including empty sequences.
//!
//! Each backend documents its durability guarantees, including whether
//! successful writes survive a restart.

/// A synchronous key-value storage backend.
///
/// Reads return owned values so callers can retain them independently
/// of subsequent storage operations.
///
/// Failed writes and deletions must preserve the previously visible
/// key-value state.
///
/// # Example
///
/// ```
/// use datanium_core::storage::Storage;
///
/// fn example<S: Storage>(storage: &mut S) -> Result<(), S::Error> {
///     storage.put(b"name", b"Datanium")?;
///
///     assert_eq!(
///         storage.get(b"name")?,
///         Some(b"Datanium".to_vec()),
///     );
///
///     storage.put(b"name", b"Updated")?;
///
///     assert_eq!(
///         storage.get(b"name")?,
///         Some(b"Updated".to_vec()),
///     );
///
///     storage.delete(b"name")?;
///     assert_eq!(storage.get(b"name")?, None);
///
///     // Deleting an absent key succeeds.
///     storage.delete(b"name")?;
///
///     Ok(())
/// }
/// ```
pub trait Storage {
    /// The backend-specific error type.
    ///
    /// Implementations should expose distinguishable failure kinds
    /// when callers can handle them differently.
    type Error: std::error::Error;

    /// Returns the value associated with `key`.
    ///
    /// Returns `Ok(None)` when the key is absent.
    /// An existing key with an empty value returns `Ok(Some(Vec::new()))`.
    ///
    /// # Errors
    ///
    /// Returns an error when the backend cannot read the value.
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Associates `value` with `key`.
    ///
    /// Creates a new entry or completely replaces the existing value.
    /// The caller retains ownership of the supplied key and value.
    ///
    /// On success, subsequent reads return the supplied value until
    /// the entry is overwritten or deleted.
    ///
    /// # Errors
    ///
    /// Returns an error when the backend cannot complete the write.
    /// On error, the previously visible key-value state is preserved.
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    /// Removes the entry associated with `key`.
    ///
    /// Deleting an absent key succeeds.
    /// On success, subsequent reads of the key return `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error when the backend cannot complete the deletion.
    /// On error, the previously visible key-value state is preserved.
    fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error>;
}
