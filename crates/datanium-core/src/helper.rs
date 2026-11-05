pub fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();

    let valid_start = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_');

    let valid_rest = bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');

    name.len() <= 64 && valid_start && valid_rest
}
