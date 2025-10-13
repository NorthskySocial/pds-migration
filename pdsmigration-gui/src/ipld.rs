use cid::multihash::Multihash;
use cid::Cid;
use serde::Serialize;
use sha2::{Digest, Sha256};

const SHA2_256: u64 = 0x12;
const DAGCBORCODEC: u64 = 0x71;
// https://docs.rs/libipld-core/0.16.0/src/libipld_core/raw.rs.html#19
const RAWCODEC: u64 = 0x77;

pub fn cid_for_cbor<T: Serialize>(data: &T) -> Cid {
    let bytes = struct_to_cbor(data);
    let mut sha = Sha256::new();
    sha.update(&bytes);
    let hash = sha.finalize();
    let cid = Cid::new_v1(
        DAGCBORCODEC,
        #[allow(deprecated)]
        Multihash::<64>::wrap(SHA2_256, hash.as_slice()).unwrap(),
    );
    cid
}

pub fn sha256_to_cid(hash: Vec<u8>) -> Cid {
    let cid = Cid::new_v1(
        RAWCODEC,
        #[allow(deprecated)]
        Multihash::<64>::wrap(SHA2_256, hash.as_slice()).unwrap(),
    );
    cid
}

pub fn struct_to_cbor<T: Serialize>(obj: &T) -> Vec<u8> {
    serde_ipld_dagcbor::to_vec(obj).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        name: String,
        value: u32,
    }

    #[test]
    fn test_constants() {
        // Test that constants are correctly defined
        assert_eq!(SHA2_256, 0x12);
        assert_eq!(DAGCBORCODEC, 0x71);
        assert_eq!(RAWCODEC, 0x77);
    }

    #[test]
    fn test_struct_to_cbor() {
        let test_data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        let cbor_bytes = struct_to_cbor(&test_data);

        // Should produce non-empty bytes
        assert!(!cbor_bytes.is_empty());

        // Should be deterministic - same input produces same output
        let cbor_bytes2 = struct_to_cbor(&test_data);
        assert_eq!(cbor_bytes, cbor_bytes2);

        // Different data should produce different CBOR
        let different_data = TestStruct {
            name: "different".to_string(),
            value: 100,
        };
        let different_cbor = struct_to_cbor(&different_data);
        assert_ne!(cbor_bytes, different_cbor);
    }

    #[test]
    fn test_cid_for_cbor() {
        let test_data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        let cid = cid_for_cbor(&test_data);

        // Should be version 1
        assert_eq!(cid.version(), cid::Version::V1);

        // Should use DAG-CBOR codec
        assert_eq!(cid.codec(), DAGCBORCODEC);

        // Should be deterministic
        let cid2 = cid_for_cbor(&test_data);
        assert_eq!(cid, cid2);

        // Different data should produce different CID
        let different_data = TestStruct {
            name: "different".to_string(),
            value: 100,
        };
        let different_cid = cid_for_cbor(&different_data);
        assert_ne!(cid, different_cid);
    }

    #[test]
    fn test_sha256_to_cid() {
        let hash = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        let cid = sha256_to_cid(hash.clone());

        // Should be version 1
        assert_eq!(cid.version(), cid::Version::V1);

        // Should use RAW codec
        assert_eq!(cid.codec(), RAWCODEC);

        // Should be deterministic
        let cid2 = sha256_to_cid(hash.clone());
        assert_eq!(cid, cid2);

        // Different hash should produce different CID
        let different_hash = vec![
            0xff; 32  // All 0xff bytes
        ];
        let different_cid = sha256_to_cid(different_hash);
        assert_ne!(cid, different_cid);
    }

    #[test]
    fn test_cid_string_representation() {
        let test_data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        let cid = cid_for_cbor(&test_data);
        let cid_string = cid.to_string();

        // Should be a valid CID string (starts with 'b' for base32)
        assert!(cid_string.starts_with('b'));
        assert!(cid_string.len() > 10); // Should be reasonably long

        // Should be parseable back to CID
        let parsed_cid: Cid = cid_string.parse().unwrap();
        assert_eq!(cid, parsed_cid);
    }

    #[test]
    fn test_empty_struct_serialization() {
        #[derive(Serialize)]
        struct EmptyStruct {}

        let empty = EmptyStruct {};
        let cbor_bytes = struct_to_cbor(&empty);

        // Should handle empty structs without panicking
        assert!(!cbor_bytes.is_empty());

        let cid = cid_for_cbor(&empty);
        assert_eq!(cid.version(), cid::Version::V1);
    }

    #[test]
    fn test_complex_struct_serialization() {
        #[derive(Serialize)]
        struct ComplexStruct {
            text: String,
            number: i64,
            optional: Option<String>,
            list: Vec<u32>,
            nested: NestedStruct,
        }

        #[derive(Serialize)]
        struct NestedStruct {
            inner_value: bool,
        }

        let complex = ComplexStruct {
            text: "complex test".to_string(),
            number: -123,
            optional: Some("optional value".to_string()),
            list: vec![1, 2, 3, 4],
            nested: NestedStruct { inner_value: true },
        };

        let cbor_bytes = struct_to_cbor(&complex);
        assert!(!cbor_bytes.is_empty());

        let cid = cid_for_cbor(&complex);
        assert_eq!(cid.version(), cid::Version::V1);
        assert_eq!(cid.codec(), DAGCBORCODEC);
    }

    #[test]
    fn test_sha256_hash_sizes() {
        // Test with correct SHA256 hash size (32 bytes)
        let correct_hash = vec![0x00; 32];
        let cid = sha256_to_cid(correct_hash);
        assert_eq!(cid.version(), cid::Version::V1);

        // Test with different sizes - should still work due to Multihash flexibility
        let short_hash = vec![0x00; 16];
        let cid_short = sha256_to_cid(short_hash);
        assert_eq!(cid_short.version(), cid::Version::V1);

        let long_hash = vec![0x00; 64];
        let cid_long = sha256_to_cid(long_hash);
        assert_eq!(cid_long.version(), cid::Version::V1);
    }
}
