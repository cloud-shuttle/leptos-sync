//! Data Integrity Module
//!
//! This module provides comprehensive data integrity verification including:
//! - Checksum verification for data corruption detection
//! - Version verification for data consistency
//! - Corruption detection and recovery
//! - Data validation and sanitization

pub mod checksum;
pub mod corruption;
pub mod types;
pub mod version;

// Re-export main types for convenience
pub use checksum::{ChecksumAlgorithm, ChecksumConfig, ChecksumVerifier};
pub use corruption::{CorruptionConfig, CorruptionDetector, CorruptionResult};
pub use types::{DataFormat, DataMetadata, IntegrityResult, IntegrityStats};
pub use version::{VersionConfig, VersionVerifier};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for the data integrity system
#[derive(Debug, Clone, PartialEq)]
pub struct IntegrityConfig {
    /// Checksum configuration
    pub checksum_config: ChecksumConfig,
    /// Version configuration
    pub version_config: VersionConfig,
    /// Corruption configuration
    pub corruption_config: CorruptionConfig,
}

impl Default for IntegrityConfig {
    fn default() -> Self {
        Self {
            checksum_config: ChecksumConfig::default(),
            version_config: VersionConfig::default(),
            corruption_config: CorruptionConfig::default(),
        }
    }
}

/// Data integrity system for corruption detection and verification
#[derive(Debug, Clone)]
pub struct DataIntegrity {
    /// Checksum verifier
    checksum_verifier: ChecksumVerifier,
    /// Version verifier
    version_verifier: VersionVerifier,
    /// Corruption detector
    corruption_detector: CorruptionDetector,
    /// Integrity statistics
    stats: Arc<RwLock<IntegrityStats>>,
    /// Whether the system is initialized
    initialized: bool,
}

impl DataIntegrity {
    /// Create a new data integrity system
    pub fn new() -> Self {
        Self {
            checksum_verifier: ChecksumVerifier::new(),
            version_verifier: VersionVerifier::new(),
            corruption_detector: CorruptionDetector::new(),
            stats: Arc::new(RwLock::new(IntegrityStats::new())),
            initialized: false,
        }
    }

    /// Create a new data integrity system with custom configuration
    pub fn with_config(config: IntegrityConfig) -> Self {
        Self {
            checksum_verifier: ChecksumVerifier::with_config(config.checksum_config),
            version_verifier: VersionVerifier::with_config(config.version_config),
            corruption_detector: CorruptionDetector::with_config(config.corruption_config),
            stats: Arc::new(RwLock::new(IntegrityStats::new())),
            initialized: false,
        }
    }

    /// Initialize the data integrity system
    pub async fn initialize(&mut self) -> Result<(), IntegrityError> {
        if self.initialized {
            return Err(IntegrityError::AlreadyInitialized);
        }

        // Initialize components
        self.checksum_verifier = ChecksumVerifier::new();
        self.version_verifier = VersionVerifier::new();
        self.corruption_detector = CorruptionDetector::new();

        self.initialized = true;
        Ok(())
    }

    /// Shutdown the data integrity system
    pub async fn shutdown(&mut self) -> Result<(), IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }

        // Clear all caches and statistics
        self.checksum_verifier.clear_cache();
        self.version_verifier.clear_versions();
        self.corruption_detector.clear_stats();

        self.initialized = false;
        Ok(())
    }

    /// Verify data integrity
    pub async fn verify_integrity(
        &mut self,
        data: &[u8],
        key: &str,
    ) -> Result<IntegrityResult, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }

        let metadata = DataMetadata::new(DataFormat::Binary, data.len());

        // Perform checksum verification
        let checksum_valid = if self.checksum_verifier.config().verify_on_read {
            if let Some(expected_checksum) = self.checksum_verifier.get_cached_checksum(key) {
                self.checksum_verifier
                    .verify_checksum(data, expected_checksum)?
            } else {
                true // No checksum to verify
            }
        } else {
            true
        };

        // Perform version verification
        let version_valid = if self.version_verifier.config().verify_on_read {
            let current_version = self.version_verifier.get_version(key);
            self.version_verifier.verify_version(key, current_version)?
        } else {
            true
        };

        // Perform corruption detection
        let corruption_result = self.corruption_detector.detect_corruption(data, key);
        let corruption_detected = corruption_result.is_corrupted;

        let is_valid = checksum_valid && version_valid && !corruption_detected;

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.record_verification(is_valid);
            if corruption_detected {
                stats.record_corruption_detection();
            }
            if corruption_result.recovery_attempted {
                stats.record_recovery_attempt(corruption_result.recovery_successful);
            }
        }

        let result = if is_valid {
            IntegrityResult::valid(metadata)
        } else {
            let error_msg = format!(
                "Integrity check failed: checksum={}, version={}, corruption={}",
                checksum_valid, version_valid, corruption_detected
            );
            IntegrityResult::invalid(error_msg, metadata)
        };

        Ok(result)
    }

    /// Compute and store checksum for data
    pub async fn compute_checksum(
        &mut self,
        data: &[u8],
        key: &str,
    ) -> Result<String, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }

        if !self.checksum_verifier.config().compute_on_write {
            return Err(IntegrityError::ChecksumComputationDisabled);
        }

        self.checksum_verifier.compute_checksum(data, key)
    }

    /// Increment version for data
    pub async fn increment_version(&mut self, key: &str) -> Result<u64, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }

        if !self.version_verifier.config().increment_on_write {
            return Err(IntegrityError::VersionIncrementDisabled);
        }

        Ok(self.version_verifier.increment_version(key))
    }

    /// Get integrity statistics
    pub async fn get_stats(&self) -> IntegrityStats {
        self.stats.read().await.clone()
    }

    /// Check if the system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Get checksum verifier
    pub fn checksum_verifier(&self) -> &ChecksumVerifier {
        &self.checksum_verifier
    }

    /// Get version verifier
    pub fn version_verifier(&self) -> &VersionVerifier {
        &self.version_verifier
    }

    /// Get corruption detector
    pub fn corruption_detector(&self) -> &CorruptionDetector {
        &self.corruption_detector
    }
}

impl Default for DataIntegrity {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for data integrity operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityError {
    /// System not initialized
    NotInitialized,
    /// System already initialized
    AlreadyInitialized,
    /// Checksum mismatch
    ChecksumMismatch,
    /// Version mismatch
    VersionMismatch {
        key: String,
        expected: u64,
        actual: u64,
    },
    /// Corruption detected
    CorruptionDetected,
    /// Checksum computation disabled
    ChecksumComputationDisabled,
    /// Version increment disabled
    VersionIncrementDisabled,
    /// Invalid data format
    InvalidDataFormat,
    /// Operation failed
    OperationFailed(String),
}

impl std::fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrityError::NotInitialized => write!(f, "Data integrity system not initialized"),
            IntegrityError::AlreadyInitialized => {
                write!(f, "Data integrity system already initialized")
            }
            IntegrityError::ChecksumMismatch => write!(f, "Checksum mismatch detected"),
            IntegrityError::VersionMismatch {
                key,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Version mismatch for key '{}': expected {}, got {}",
                    key, expected, actual
                )
            }
            IntegrityError::CorruptionDetected => write!(f, "Data corruption detected"),
            IntegrityError::ChecksumComputationDisabled => {
                write!(f, "Checksum computation is disabled")
            }
            IntegrityError::VersionIncrementDisabled => write!(f, "Version increment is disabled"),
            IntegrityError::InvalidDataFormat => write!(f, "Invalid data format"),
            IntegrityError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl std::error::Error for IntegrityError {}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_data_integrity_creation() {
        let mut integrity = DataIntegrity::new();
        assert!(!integrity.is_initialized());

        integrity.initialize().await.unwrap();
        assert!(integrity.is_initialized());
    }

    #[tokio::test]
    async fn test_data_integrity_with_config() {
        let config = IntegrityConfig::default();
        let mut integrity = DataIntegrity::with_config(config);

        integrity.initialize().await.unwrap();
        assert!(integrity.is_initialized());
    }

    #[tokio::test]
    async fn test_data_integrity_initialization() {
        let mut integrity = DataIntegrity::new();

        // Initialize
        let result = integrity.initialize().await;
        assert!(result.is_ok());
        assert!(integrity.is_initialized());

        // Try to initialize again (should fail)
        let result = integrity.initialize().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_data_integrity_shutdown() {
        let mut integrity = DataIntegrity::new();

        // Initialize first
        integrity.initialize().await.unwrap();
        assert!(integrity.is_initialized());

        // Then shutdown
        let result = integrity.shutdown().await;
        assert!(result.is_ok());
        assert!(!integrity.is_initialized());
    }

    #[tokio::test]
    async fn test_verify_integrity() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();

        let data = b"test data";
        let key = "test_key";

        let result = integrity.verify_integrity(data, key).await.unwrap();
        assert!(result.is_valid);
        assert!(result.checksum_valid);
        assert!(result.version_valid);
        assert!(!result.corruption_detected);
    }

    #[tokio::test]
    async fn test_compute_checksum() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();

        let data = b"test data";
        let key = "test_key";

        let checksum = integrity.compute_checksum(data, key).await.unwrap();
        assert!(!checksum.is_empty());
    }

    #[tokio::test]
    async fn test_increment_version() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();

        let key = "test_key";

        let version1 = integrity.increment_version(key).await.unwrap();
        let version2 = integrity.increment_version(key).await.unwrap();

        assert_eq!(version1, 1);
        assert_eq!(version2, 2);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();

        let data = b"test data";
        let key = "test_key";

        // Perform some operations
        integrity.verify_integrity(data, key).await.unwrap();
        integrity.compute_checksum(data, key).await.unwrap();
        integrity.increment_version(key).await.unwrap();

        let stats = integrity.get_stats().await;
        assert!(stats.total_verifications > 0);
        assert!(stats.successful_verifications > 0);
    }

    #[tokio::test]
    async fn test_operations_before_initialization() {
        let mut integrity = DataIntegrity::new();

        let data = b"test data";
        let key = "test_key";

        // Operations should fail before initialization
        assert!(integrity.verify_integrity(data, key).await.is_err());
        assert!(integrity.compute_checksum(data, key).await.is_err());
        assert!(integrity.increment_version(key).await.is_err());
    }
}
