use datanium_core::database::Database;

const MAGIC: &[u8; 4] = b"DATN";
const VERSION: u16 = 1;
const HEADER_LEN: usize = 7;

pub(super) const MAX_FILE_LEN: usize = HEADER_LEN + 64;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum FormatError {
    InvalidLength,
    InvalidName,
    InvalidMagic,
    UnsupportedVersion(u16),
}

pub(super) fn encode_empty_header(database: &Database) -> Vec<u8> {
    let name = database.name().as_bytes();
    // Database::new limits the name to a maximum of 64 bytes
    let name_len = name.len() as u8;

    let mut bytes = Vec::with_capacity(HEADER_LEN + name.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.push(name_len);
    bytes.extend_from_slice(name);
    bytes
}

pub(super) fn decode_empty_database(bytes: &[u8]) -> Result<Database, FormatError> {
    let prefix = bytes.get(..6).ok_or(FormatError::InvalidLength)?;

    if &prefix[..4] != MAGIC {
        return Err(FormatError::InvalidMagic);
    }

    let version = u16::from_le_bytes([prefix[4], prefix[5]]);
    if version != VERSION {
        return Err(FormatError::UnsupportedVersion(version));
    }

    let name_len = usize::from(*bytes.get(6).ok_or(FormatError::InvalidLength)?);
    if !(1..=64).contains(&name_len) || bytes.len() != HEADER_LEN + name_len {
        return Err(FormatError::InvalidLength);
    }

    let name = std::str::from_utf8(&bytes[HEADER_LEN..]).map_err(|_| FormatError::InvalidName)?;

    Database::new(name).map_err(|_| FormatError::InvalidName)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_matches_the_v1_format() {
        let expected = b"DATN\x01\x00\x07example";
        let database = Database::new("example").unwrap();

        assert_eq!(encode_empty_header(&database).as_slice(), expected);
        assert_eq!(decode_empty_database(expected).unwrap().name(), "example");
    }
}
