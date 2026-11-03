mod contract;
mod fault_injection;

mod in_memory {
    use datanium_storage::in_memory::InMemoryStorage;

    fn with_storage(run: impl FnOnce(&mut InMemoryStorage)) {
        let mut storage = InMemoryStorage::new();
        run(&mut storage);
    }

    crate::contract::storage_contract_tests!(with_storage);
}
