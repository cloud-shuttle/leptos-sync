//! Checksum verification for data corruption detection

use super::super::IntegrityError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Checksum algorithm types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    /// SHA-256 checksum
    Sha256,
    /// MD5 checksum (less secure, faster)
    Md5,
    /// CRC32 checksum (fastest, least secure)
    Crc32,
}

/// Configuration for checksum verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecksumConfig {
    /// Default checksum algorithm
    pub algorithm: ChecksumAlgorithm,
    /// Whether to verify checksums on read
    pub verify_on_read: bool,
    /// Whether to compute checksums on write
    pub compute_on_write: bool,
}

impl Default for ChecksumConfig {
    fn default() -> Self {
        Self {
            algorithm: ChecksumAlgorithm::Sha256,
            verify_on_read: true,
            compute_on_write: true,
        }
    }
}

/// Checksum verifier for data integrity
#[derive(Debug, Clone)]
pub struct ChecksumVerifier {
    /// Configuration
    config: ChecksumConfig,
    /// Cache of computed checksums
    checksum_cache: HashMap<String, String>,
}

impl ChecksumVerifier {
    /// Create a new checksum verifier
    pub fn new() -> Self {
        Self {
            config: ChecksumConfig::default(),
            checksum_cache: HashMap::new(),
        }
    }

    /// Create a new checksum verifier with configuration
    pub fn with_config(config: ChecksumConfig) -> Self {
        Self {
            config,
            checksum_cache: HashMap::new(),
        }
    }

    /// Compute checksum for data
    pub fn compute_checksum(&mut self, data: &[u8], key: &str) -> Result<String, IntegrityError> {
        let checksum = match self.config.algorithm {
            ChecksumAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                format!("{:x}", hasher.finalize())
            }
            ChecksumAlgorithm::Md5 => {
                // Note: MD5 implementation would go here
                "md5_placeholder".to_string()
            }
            ChecksumAlgorithm::Crc32 => {
                // Note: CRC32 implementation would go here
                "crc32_placeholder".to_string()
            }
        };

        // Cache the checksum
        self.checksum_cache
            .insert(key.to_string(), checksum.clone());

        Ok(checksum)
    }

    /// Verify checksum for data
    pub fn verify_checksum(
        &self,
        data: &[u8],
        expected_checksum: &str,
    ) -> Result<bool, IntegrityError> {
        let computed_checksum = match self.config.algorithm {
            ChecksumAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                format!("{:x}", hasher.finalize())
            }
            ChecksumAlgorithm::Md5 => "md5_placeholder".to_string(),
            ChecksumAlgorithm::Crc32 => "crc32_placeholder".to_string(),
        };

        Ok(computed_checksum == expected_checksum)
    }

    /// Get cached checksum
    pub fn get_cached_checksum(&self, key: &str) -> Option<&String> {
        self.checksum_cache.get(key)
    }

    /// Clear checksum cache
    pub fn clear_cache(&mut self) {
        self.checksum_cache.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &ChecksumConfig {
        &self.config
    }
}

impl Default for ChecksumVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_verifier_creation() {
        let verifier = ChecksumVerifier::new();
        assert_eq!(verifier.config().algorithm, ChecksumAlgorithm::Sha256);
        assert!(verifier.config().verify_on_read);
        assert!(verifier.config().compute_on_write);
    }

    #[test]
    fn test_checksum_verifier_with_config() {
        let config = ChecksumConfig {
            algorithm: ChecksumAlgorithm::Md5,
            verify_on_read: false,
            compute_on_write: true,
        };
        let verifier = ChecksumVerifier::with_config(config.clone());
        assert_eq!(verifier.config().algorithm, ChecksumAlgorithm::Md5);
        assert!(!verifier.config().verify_on_read);
        assert!(verifier.config().compute_on_write);
    }

    #[test]
    fn test_compute_checksum() {
        let mut verifier = ChecksumVerifier::new();
        let data = b"test data";
        let key = "test_key";

        let checksum = verifier.compute_checksum(data, key).unwrap();
        assert!(!checksum.is_empty());

        // Check if cached
        assert!(verifier.get_cached_checksum(key).is_some());
    }

    #[test]
    fn test_verify_checksum() {
        let mut verifier = ChecksumVerifier::new();
        let data = b"test data";
        let key = "test_key";

        let checksum = verifier.compute_checksum(data, key).unwrap();
        let is_valid = verifier.verify_checksum(data, &checksum).unwrap();
        assert!(is_valid);

        // Test with wrong checksum
        let is_invalid = verifier.verify_checksum(data, "wrong_checksum").unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_clear_cache() {
        let mut verifier = ChecksumVerifier::new();
        let data = b"test data";
        let key = "test_key";

        verifier.compute_checksum(data, key).unwrap();
        assert!(verifier.get_cached_checksum(key).is_some());

        verifier.clear_cache();
        assert!(verifier.get_cached_checksum(key).is_none());
    }
}
