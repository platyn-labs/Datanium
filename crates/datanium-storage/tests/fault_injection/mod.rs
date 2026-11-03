use std::io;

use datanium_core::storage::Storage;
use datanium_storage::in_memory::InMemoryStorage;

#[derive(Default)]
struct FallibleStorage {
    inner: InMemoryStorage,
    fail_next_write: bool,
}

impl FallibleStorage {
    fn arm_failure(&mut self) {
        self.fail_next_write = true;
    }

    fn check_write(&mut self) -> io::Result<()> {
        if std::mem::take(&mut self.fail_next_write) {
            return Err(io::Error::other("injected write failure"));
        }

        Ok(())
    }
}

impl Storage for FallibleStorage {
    type Error = io::Error;

    fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        Ok(self.inner.get(key).unwrap())
    }

    fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        self.check_write()?;
        self.inner.put(key, value).unwrap();
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> io::Result<()> {
        self.check_write()?;
        self.inner.delete(key).unwrap();
        Ok(())
    }
}

fn with_storage(run: impl FnOnce(&mut FallibleStorage)) {
    let mut storage = FallibleStorage::default();
    run(&mut storage);
}

fn with_failure(run: impl FnOnce(&mut FallibleStorage, fn(&mut FallibleStorage))) {
    with_storage(|storage| run(storage, FallibleStorage::arm_failure));
}

crate::contract::storage_contract_tests!(with_storage, failures = with_failure);
