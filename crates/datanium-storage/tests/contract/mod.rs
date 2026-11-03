use datanium_core::storage::Storage;

pub fn reads_last_write(storage: &mut impl Storage) {
    storage.put(b"key", b"first").unwrap();
    assert_eq!(storage.get(b"key").unwrap(), Some(b"first".to_vec()));

    storage.put(b"key", b"x").unwrap();
    assert_eq!(storage.get(b"key").unwrap(), Some(b"x".to_vec()));
}

pub fn keys_are_independent(storage: &mut impl Storage) {
    storage.put(b"a", b"one").unwrap();
    storage.put(b"b", b"two").unwrap();

    storage.put(b"a", b"updated").unwrap();
    assert_eq!(storage.get(b"b").unwrap(), Some(b"two".to_vec()));

    storage.delete(b"a").unwrap();
    assert_eq!(storage.get(b"b").unwrap(), Some(b"two".to_vec()));
}

pub fn empty_is_distinct_from_missing(storage: &mut impl Storage) {
    assert_eq!(storage.get(b"key").unwrap(), None);

    storage.put(b"key", b"").unwrap();
    assert_eq!(storage.get(b"key").unwrap(), Some(Vec::new()));

    storage.put(b"", b"value").unwrap();
    assert_eq!(storage.get(b"").unwrap(), Some(b"value".to_vec()));
}

pub fn delete_is_idempotent(storage: &mut impl Storage) {
    storage.delete(b"key").unwrap();

    storage.put(b"key", b"value").unwrap();
    storage.delete(b"key").unwrap();
    assert_eq!(storage.get(b"key").unwrap(), None);

    storage.delete(b"key").unwrap();
    assert_eq!(storage.get(b"key").unwrap(), None);
}

pub fn binary_data_is_owned(storage: &mut impl Storage) {
    let original_key = [0xff, 0];
    let mut key = original_key;
    let mut value = [0, 0xfe];

    storage.put(&key, &value).unwrap();
    key.fill(0);
    value.fill(0);

    let mut returned = storage.get(&original_key).unwrap().unwrap();
    assert_eq!(returned, [0, 0xfe]);

    returned.fill(0);
    assert_eq!(storage.get(&original_key).unwrap(), Some(vec![0, 0xfe]));

    storage.delete(&original_key).unwrap();
    assert_eq!(storage.get(&original_key).unwrap(), None);
}

pub fn failed_insert_preserves_state<S: Storage>(
    storage: &mut S,
    arm_failure: impl FnOnce(&mut S),
) {
    storage.put(b"other", b"keep").unwrap();
    assert_eq!(storage.get(b"new").unwrap(), None);

    arm_failure(storage);
    assert!(storage.put(b"new", b"value").is_err());

    assert_eq!(storage.get(b"new").unwrap(), None);
    assert_eq!(storage.get(b"other").unwrap(), Some(b"keep".to_vec()));
}

pub fn failed_put_preserves_state<S: Storage>(storage: &mut S, arm_failure: impl FnOnce(&mut S)) {
    storage.put(b"key", b"old").unwrap();
    storage.put(b"other", b"keep").unwrap();

    arm_failure(storage);
    assert!(storage.put(b"key", b"new").is_err());

    assert_previous_state(storage);
}

pub fn failed_delete_preserves_state<S: Storage>(
    storage: &mut S,
    arm_failure: impl FnOnce(&mut S),
) {
    storage.put(b"key", b"old").unwrap();
    storage.put(b"other", b"keep").unwrap();

    arm_failure(storage);
    assert!(storage.delete(b"key").is_err());

    assert_previous_state(storage);
}

fn assert_previous_state(storage: &impl Storage) {
    assert_eq!(storage.get(b"key").unwrap(), Some(b"old".to_vec()));
    assert_eq!(storage.get(b"other").unwrap(), Some(b"keep".to_vec()));
}

macro_rules! register_cases {
    ($fixture:path; $($case:ident),+ $(,)?) => {
        $(
            #[test]
            fn $case() {
                $fixture($crate::contract::$case);
            }
        )+
    };
}
pub(crate) use register_cases;

macro_rules! storage_contract_tests {
    ($fixture:path $(, failures = $failure_fixture:path)? $(,)?) => {
        $crate::contract::register_cases!($fixture;
            reads_last_write,
            keys_are_independent,
            empty_is_distinct_from_missing,
            delete_is_idempotent,
            binary_data_is_owned,
        );

        $(
            $crate::contract::register_cases!($failure_fixture;
                failed_insert_preserves_state,
                failed_put_preserves_state,
                failed_delete_preserves_state,
            );
        )?
    };
}
pub(crate) use storage_contract_tests;
