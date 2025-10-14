// Cryptography module for the Bulu programming language
// Provides hashing functions and cryptographic operations

use md5;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashMap;

/// Cryptographic hash algorithms supported by the language
#[derive(Debug, Clone, PartialEq)]
pub enum HashAlgorithm {
    MD5,
    SHA1,
    SHA256,
    SHA512,
}

/// Result of a hash operation
#[derive(Debug, Clone)]
pub struct HashResult {
    pub algorithm: HashAlgorithm,
    pub digest: Vec<u8>,
    pub hex_string: String,
}

impl HashResult {
    /// Create a new hash result
    pub fn new(algorithm: HashAlgorithm, digest: Vec<u8>) -> Self {
        let hex_string = hex::encode(&digest);
        Self {
            algorithm,
            digest,
            hex_string,
        }
    }

    /// Get the hash as a hexadecimal string
    pub fn to_hex(&self) -> String {
        self.hex_string.clone()
    }

    /// Get the raw bytes of the hash
    pub fn to_bytes(&self) -> Vec<u8> {
        self.digest.clone()
    }
}

/// Cryptography context for managing hash operations
pub struct CryptoContext {
    supported_algorithms: HashMap<String, HashAlgorithm>,
}

impl CryptoContext {
    /// Create a new cryptography context
    pub fn new() -> Self {
        let mut supported_algorithms = HashMap::new();
        supported_algorithms.insert("md5".to_string(), HashAlgorithm::MD5);
        supported_algorithms.insert("sha1".to_string(), HashAlgorithm::SHA1);
        supported_algorithms.insert("sha256".to_string(), HashAlgorithm::SHA256);
        supported_algorithms.insert("sha512".to_string(), HashAlgorithm::SHA512);

        Self {
            supported_algorithms,
        }
    }

    /// Hash data using MD5 algorithm
    pub fn md5(&self, data: &[u8]) -> HashResult {
        let digest = md5::compute(data);
        HashResult::new(HashAlgorithm::MD5, digest.0.to_vec())
    }

    /// Hash data using SHA-1 algorithm
    pub fn sha1(&self, data: &[u8]) -> HashResult {
        let mut hasher = Sha1::new();
        Sha1Digest::update(&mut hasher, data);
        let digest = hasher.finalize().to_vec();
        HashResult::new(HashAlgorithm::SHA1, digest)
    }

    /// Hash data using SHA-256 algorithm
    pub fn sha256(&self, data: &[u8]) -> HashResult {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize().to_vec();
        HashResult::new(HashAlgorithm::SHA256, digest)
    }

    /// Hash data using SHA-512 algorithm
    pub fn sha512(&self, data: &[u8]) -> HashResult {
        let mut hasher = Sha512::new();
        hasher.update(data);
        let digest = hasher.finalize().to_vec();
        HashResult::new(HashAlgorithm::SHA512, digest)
    }

    /// Hash data using the specified algorithm
    pub fn hash(&self, algorithm: &str, data: &[u8]) -> Result<HashResult, String> {
        match algorithm.to_lowercase().as_str() {
            "md5" => Ok(self.md5(data)),
            "sha1" => Ok(self.sha1(data)),
            "sha256" => Ok(self.sha256(data)),
            "sha512" => Ok(self.sha512(data)),
            _ => Err(format!("Unsupported hash algorithm: {}", algorithm)),
        }
    }

    /// Get list of supported hash algorithms
    pub fn supported_algorithms(&self) -> Vec<String> {
        self.supported_algorithms.keys().cloned().collect()
    }

    /// Hash a string using the specified algorithm
    pub fn hash_string(&self, algorithm: &str, input: &str) -> Result<HashResult, String> {
        self.hash(algorithm, input.as_bytes())
    }

    /// Verify a hash against expected value
    pub fn verify_hash(
        &self,
        algorithm: &str,
        data: &[u8],
        expected_hex: &str,
    ) -> Result<bool, String> {
        let result = self.hash(algorithm, data)?;
        Ok(result.to_hex().to_lowercase() == expected_hex.to_lowercase())
    }
}

/// Built-in functions for cryptographic operations
pub mod builtins {
    use super::*;

    /// Global crypto context
    static mut CRYPTO_CONTEXT: Option<CryptoContext> = None;

    /// Initialize the crypto context
    pub fn init_crypto() {
        unsafe {
            CRYPTO_CONTEXT = Some(CryptoContext::new());
        }
    }

    /// Get the global crypto context
    fn get_crypto_context() -> &'static CryptoContext {
        unsafe {
            CRYPTO_CONTEXT
                .as_ref()
                .expect("Crypto context not initialized")
        }
    }

    /// Hash data using MD5
    pub fn crypto_md5(data: &[u8]) -> HashResult {
        get_crypto_context().md5(data)
    }

    /// Hash data using SHA-1
    pub fn crypto_sha1(data: &[u8]) -> HashResult {
        get_crypto_context().sha1(data)
    }

    /// Hash data using SHA-256
    pub fn crypto_sha256(data: &[u8]) -> HashResult {
        get_crypto_context().sha256(data)
    }

    /// Hash data using SHA-512
    pub fn crypto_sha512(data: &[u8]) -> HashResult {
        get_crypto_context().sha512(data)
    }

    /// Hash string using specified algorithm
    pub fn crypto_hash_string(algorithm: &str, input: &str) -> Result<String, String> {
        let result = get_crypto_context().hash_string(algorithm, input)?;
        Ok(result.to_hex())
    }

    /// Verify hash against expected value
    pub fn crypto_verify(algorithm: &str, data: &[u8], expected: &str) -> Result<bool, String> {
        get_crypto_context().verify_hash(algorithm, data, expected)
    }

    /// Get supported hash algorithms
    pub fn crypto_algorithms() -> Vec<String> {
        get_crypto_context().supported_algorithms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_hash() {
        let crypto = CryptoContext::new();
        let result = crypto.md5(b"hello world");
        assert_eq!(result.algorithm, HashAlgorithm::MD5);
        assert_eq!(result.to_hex(), "5eb63bbbe01eeed093cb22bb8f5acdc3");
    }

    #[test]
    fn test_sha1_hash() {
        let crypto = CryptoContext::new();
        let result = crypto.sha1(b"hello world");
        assert_eq!(result.algorithm, HashAlgorithm::SHA1);
        assert_eq!(result.to_hex(), "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");
    }

    #[test]
    fn test_sha256_hash() {
        let crypto = CryptoContext::new();
        let result = crypto.sha256(b"hello world");
        assert_eq!(result.algorithm, HashAlgorithm::SHA256);
        assert_eq!(
            result.to_hex(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sha512_hash() {
        let crypto = CryptoContext::new();
        let result = crypto.sha512(b"hello world");
        assert_eq!(result.algorithm, HashAlgorithm::SHA512);
        assert_eq!(result.to_hex(), "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f");
    }

    #[test]
    fn test_hash_string() {
        let crypto = CryptoContext::new();
        let result = crypto.hash_string("md5", "test").unwrap();
        assert_eq!(result.to_hex(), "098f6bcd4621d373cade4e832627b4f6");
    }

    #[test]
    fn test_verify_hash() {
        let crypto = CryptoContext::new();
        let is_valid = crypto
            .verify_hash("md5", b"test", "098f6bcd4621d373cade4e832627b4f6")
            .unwrap();
        assert!(is_valid);

        let is_invalid = crypto.verify_hash("md5", b"test", "invalid_hash").unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_unsupported_algorithm() {
        let crypto = CryptoContext::new();
        let result = crypto.hash("unsupported", b"test");
        assert!(result.is_err());
    }

    #[test]
    fn test_supported_algorithms() {
        let crypto = CryptoContext::new();
        let algorithms = crypto.supported_algorithms();
        assert!(algorithms.contains(&"md5".to_string()));
        assert!(algorithms.contains(&"sha1".to_string()));
        assert!(algorithms.contains(&"sha256".to_string()));
        assert!(algorithms.contains(&"sha512".to_string()));
    }
}
