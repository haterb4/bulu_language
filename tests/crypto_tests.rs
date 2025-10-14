// Unit tests for the cryptography module

use bulu::std::crypto::*;

#[test]
fn test_crypto_context_creation() {
    let crypto = CryptoContext::new();
    let algorithms = crypto.supported_algorithms();
    
    assert!(algorithms.contains(&"md5".to_string()));
    assert!(algorithms.contains(&"sha1".to_string()));
    assert!(algorithms.contains(&"sha256".to_string()));
    assert!(algorithms.contains(&"sha512".to_string()));
    assert_eq!(algorithms.len(), 4);
}

#[test]
fn test_md5_hashing() {
    let crypto = CryptoContext::new();
    
    // Test empty string
    let result = crypto.md5(b"");
    assert_eq!(result.algorithm, HashAlgorithm::MD5);
    assert_eq!(result.to_hex(), "d41d8cd98f00b204e9800998ecf8427e");
    
    // Test "hello world"
    let result = crypto.md5(b"hello world");
    assert_eq!(result.to_hex(), "5eb63bbbe01eeed093cb22bb8f5acdc3");
    
    // Test longer string
    let result = crypto.md5(b"The quick brown fox jumps over the lazy dog");
    assert_eq!(result.to_hex(), "9e107d9d372bb6826bd81d3542a419d6");
}

#[test]
fn test_sha1_hashing() {
    let crypto = CryptoContext::new();
    
    // Test empty string
    let result = crypto.sha1(b"");
    assert_eq!(result.algorithm, HashAlgorithm::SHA1);
    assert_eq!(result.to_hex(), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
    
    // Test "hello world"
    let result = crypto.sha1(b"hello world");
    assert_eq!(result.to_hex(), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    
    // Test longer string
    let result = crypto.sha1(b"The quick brown fox jumps over the lazy dog");
    assert_eq!(result.to_hex(), "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
}

#[test]
fn test_sha256_hashing() {
    let crypto = CryptoContext::new();
    
    // Test empty string
    let result = crypto.sha256(b"");
    assert_eq!(result.algorithm, HashAlgorithm::SHA256);
    assert_eq!(result.to_hex(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    
    // Test "hello world"
    let result = crypto.sha256(b"hello world");
    assert_eq!(result.to_hex(), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    
    // Test longer string
    let result = crypto.sha256(b"The quick brown fox jumps over the lazy dog");
    assert_eq!(result.to_hex(), "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592");
}

#[test]
fn test_sha512_hashing() {
    let crypto = CryptoContext::new();
    
    // Test empty string
    let result = crypto.sha512(b"");
    assert_eq!(result.algorithm, HashAlgorithm::SHA512);
    assert_eq!(result.to_hex(), "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e");
    
    // Test "hello world"
    let result = crypto.sha512(b"hello world");
    assert_eq!(result.to_hex(), "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f");
}

#[test]
fn test_generic_hash_function() {
    let crypto = CryptoContext::new();
    
    // Test all supported algorithms
    let test_data = b"test data";
    
    let md5_result = crypto.hash("md5", test_data).unwrap();
    assert_eq!(md5_result.algorithm, HashAlgorithm::MD5);
    
    let sha1_result = crypto.hash("sha1", test_data).unwrap();
    assert_eq!(sha1_result.algorithm, HashAlgorithm::SHA1);
    
    let sha256_result = crypto.hash("sha256", test_data).unwrap();
    assert_eq!(sha256_result.algorithm, HashAlgorithm::SHA256);
    
    let sha512_result = crypto.hash("sha512", test_data).unwrap();
    assert_eq!(sha512_result.algorithm, HashAlgorithm::SHA512);
    
    // Test case insensitive
    let md5_upper = crypto.hash("MD5", test_data).unwrap();
    assert_eq!(md5_result.to_hex(), md5_upper.to_hex());
    
    // Test unsupported algorithm
    let invalid_result = crypto.hash("invalid", test_data);
    assert!(invalid_result.is_err());
}

#[test]
fn test_hash_string_function() {
    let crypto = CryptoContext::new();
    
    let result = crypto.hash_string("md5", "hello").unwrap();
    assert_eq!(result.to_hex(), "5d41402abc4b2a76b9719d911017c592");
    
    let result = crypto.hash_string("sha256", "world").unwrap();
    assert_eq!(result.to_hex(), "486ea46224d1bb4fb680f34f7c9ad96a8f24ec88be73ea8e5a6c65260e9cb8a7");
}

#[test]
fn test_hash_verification() {
    let crypto = CryptoContext::new();
    
    // Test valid hash verification
    let is_valid = crypto.verify_hash("md5", b"test", "098f6bcd4621d373cade4e832627b4f6").unwrap();
    assert!(is_valid);
    
    // Test invalid hash verification
    let is_invalid = crypto.verify_hash("md5", b"test", "invalid_hash").unwrap();
    assert!(!is_invalid);
    
    // Test case insensitive verification
    let is_valid_upper = crypto.verify_hash("md5", b"test", "098F6BCD4621D373CADE4E832627B4F6").unwrap();
    assert!(is_valid_upper);
    
    // Test with different algorithm
    let is_valid_sha1 = crypto.verify_hash("sha1", b"test", "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3").unwrap();
    assert!(is_valid_sha1);
}

#[test]
fn test_hash_result_methods() {
    let crypto = CryptoContext::new();
    let result = crypto.md5(b"test");
    
    // Test hex representation
    let hex = result.to_hex();
    assert_eq!(hex, "098f6bcd4621d373cade4e832627b4f6");
    assert_eq!(hex.len(), 32); // MD5 is 128 bits = 32 hex chars
    
    // Test bytes representation
    let bytes = result.to_bytes();
    assert_eq!(bytes.len(), 16); // MD5 is 128 bits = 16 bytes
    
    // Verify hex and bytes match
    let hex_from_bytes = hex::encode(&bytes);
    assert_eq!(hex, hex_from_bytes);
}

#[test]
fn test_hash_algorithm_enum() {
    assert_eq!(HashAlgorithm::MD5, HashAlgorithm::MD5);
    assert_ne!(HashAlgorithm::MD5, HashAlgorithm::SHA1);
    
    // Test Debug trait
    let debug_str = format!("{:?}", HashAlgorithm::SHA256);
    assert_eq!(debug_str, "SHA256");
}

#[test]
fn test_builtin_functions() {
    use bulu::std::crypto::builtins::*;
    
    // Initialize crypto system
    init_crypto();
    
    // Test individual hash functions
    let md5_result = crypto_md5(b"test");
    assert_eq!(md5_result.to_hex(), "098f6bcd4621d373cade4e832627b4f6");
    
    let sha1_result = crypto_sha1(b"test");
    assert_eq!(sha1_result.to_hex(), "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3");
    
    let sha256_result = crypto_sha256(b"test");
    assert_eq!(sha256_result.to_hex(), "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08");
    
    let sha512_result = crypto_sha512(b"test");
    assert!(sha512_result.to_hex().len() == 128); // SHA-512 produces 128 hex chars
    
    // Test string hashing
    let hash_result = crypto_hash_string("md5", "hello").unwrap();
    assert_eq!(hash_result, "5d41402abc4b2a76b9719d911017c592");
    
    // Test verification
    let is_valid = crypto_verify("md5", b"test", "098f6bcd4621d373cade4e832627b4f6").unwrap();
    assert!(is_valid);
    
    // Test algorithms list
    let algorithms = crypto_algorithms();
    assert!(algorithms.contains(&"md5".to_string()));
    assert!(algorithms.contains(&"sha256".to_string()));
}

#[test]
fn test_concurrent_hashing() {
    use std::thread;
    use std::sync::Arc;
    
    let crypto = Arc::new(CryptoContext::new());
    let mut handles = vec![];
    
    // Spawn multiple threads to test thread safety
    for i in 0..10 {
        let crypto_clone = Arc::clone(&crypto);
        let handle = thread::spawn(move || {
            let data = format!("test data {}", i);
            let result = crypto_clone.md5(data.as_bytes());
            result.to_hex()
        });
        handles.push(handle);
    }
    
    // Collect results
    let mut results = vec![];
    for handle in handles {
        results.push(handle.join().unwrap());
    }
    
    // Verify all results are different (since input data was different)
    assert_eq!(results.len(), 10);
    for i in 0..results.len() {
        for j in i+1..results.len() {
            assert_ne!(results[i], results[j]);
        }
    }
}

#[test]
fn test_large_data_hashing() {
    let crypto = CryptoContext::new();
    
    // Test with large data (1MB)
    let large_data = vec![0u8; 1024 * 1024];
    let result = crypto.sha256(&large_data);
    
    // Should not panic and should produce valid hash
    assert_eq!(result.to_hex().len(), 64); // SHA-256 produces 64 hex chars
    assert_eq!(result.to_bytes().len(), 32); // SHA-256 produces 32 bytes
}

#[test]
fn test_binary_data_hashing() {
    let crypto = CryptoContext::new();
    
    // Test with binary data containing null bytes
    let binary_data = vec![0, 1, 2, 3, 255, 254, 253, 0, 0, 0];
    let result = crypto.md5(&binary_data);
    
    // Should handle binary data correctly
    assert_eq!(result.to_hex().len(), 32);
    assert_eq!(result.to_bytes().len(), 16);
}