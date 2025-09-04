//! Data Integrity System
//!
//! This module provides comprehensive data integrity verification including:
//! - Checksum verification for data corruption detection
//! - Version verification for data consistency
//! - Corruption detection and recovery
//! - Data validation and sanitization

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

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
        self.checksum_verifier.initialize().await?;
        self.version_verifier.initialize().await?;
        self.corruption_detector.initialize().await?;
        
        let mut stats = self.stats.write().await;
        stats.reset();
        
        self.initialized = true;
        Ok(())
    }
    
    /// Shutdown the data integrity system
    pub async fn shutdown(&mut self) -> Result<(), IntegrityError> {
        self.initialized = false;
        Ok(())
    }
    
    /// Check if the system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Verify data integrity
    pub async fn verify_integrity(&self, data: &[u8], metadata: &DataMetadata) -> Result<IntegrityResult, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }
        
        let mut result = IntegrityResult::new();
        let mut stats = self.stats.write().await;
        
        // Verify checksum
        match self.checksum_verifier.verify(data, &metadata.checksum).await {
            Ok(checksum_result) => {
                result.checksum_valid = checksum_result;
                if checksum_result {
                    stats.checksum_verifications_passed += 1;
                } else {
                    stats.checksum_verifications_failed += 1;
                }
            }
            Err(e) => {
                result.checksum_valid = false;
                result.errors.push(IntegrityError::ChecksumVerificationFailed(e.to_string()));
                stats.checksum_verifications_failed += 1;
            }
        }
        
        // Verify version
        match self.version_verifier.verify(&metadata.version, &metadata.expected_version).await {
            Ok(version_result) => {
                result.version_valid = version_result;
                if version_result {
                    stats.version_verifications_passed += 1;
                } else {
                    stats.version_verifications_failed += 1;
                }
            }
            Err(e) => {
                result.version_valid = false;
                result.errors.push(IntegrityError::VersionVerificationFailed(e.to_string()));
                stats.version_verifications_failed += 1;
            }
        }
        
        // Detect corruption
        match self.corruption_detector.detect_corruption(data, &metadata).await {
            Ok(corruption_result) => {
                result.corruption_detected = corruption_result.is_corrupted;
                result.corruption_details = corruption_result.details;
                if corruption_result.is_corrupted {
                    stats.corruption_detections += 1;
                }
            }
            Err(e) => {
                result.corruption_detected = false;
                result.errors.push(IntegrityError::CorruptionDetectionFailed(e.to_string()));
            }
        }
        
        // Determine overall integrity
        result.overall_valid = result.checksum_valid && result.version_valid && !result.corruption_detected;
        
        if result.overall_valid {
            stats.total_verifications_passed += 1;
        } else {
            stats.total_verifications_failed += 1;
        }
        
        stats.total_verifications += 1;
        
        Ok(result)
    }
    
    /// Generate checksum for data
    pub async fn generate_checksum(&self, data: &[u8]) -> Result<String, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }
        
        self.checksum_verifier.generate(data).await
    }
    
    /// Validate data format
    pub async fn validate_data_format(&self, data: &[u8], expected_format: &DataFormat) -> Result<bool, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }
        
        match expected_format {
            DataFormat::Json => {
                match serde_json::from_slice::<serde_json::Value>(data) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            DataFormat::Bincode => {
                match bincode::deserialize::<serde_json::Value>(data) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            DataFormat::Binary => Ok(true), // Binary format is always valid
            DataFormat::Text => {
                match std::str::from_utf8(data) {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
        }
    }
    
    /// Get integrity statistics
    pub async fn get_stats(&self) -> IntegrityStats {
        self.stats.read().await.clone()
    }
    
    /// Reset integrity statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.reset();
    }
}

/// Checksum verifier for data integrity
#[derive(Debug, Clone)]
pub struct ChecksumVerifier {
    /// Algorithm to use for checksum generation
    algorithm: ChecksumAlgorithm,
    /// Whether the verifier is initialized
    initialized: bool,
}

impl ChecksumVerifier {
    /// Create a new checksum verifier
    pub fn new() -> Self {
        Self {
            algorithm: ChecksumAlgorithm::Sha256,
            initialized: false,
        }
    }
    
    /// Create a new checksum verifier with configuration
    pub fn with_config(config: ChecksumConfig) -> Self {
        Self {
            algorithm: config.algorithm,
            initialized: false,
        }
    }
    
    /// Initialize the checksum verifier
    pub async fn initialize(&mut self) -> Result<(), IntegrityError> {
        self.initialized = true;
        Ok(())
    }
    
    /// Generate checksum for data
    pub async fn generate(&self, data: &[u8]) -> Result<String, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }
        
        match self.algorithm {
            ChecksumAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                let result = hasher.finalize();
                Ok(format!("{:x}", result))
            }
            ChecksumAlgorithm::Sha1 => {
                use sha1::{Sha1, Digest};
                let mut hasher = Sha1::new();
                hasher.update(data);
                let result = hasher.finalize();
                Ok(format!("{:x}", result))
            }
            ChecksumAlgorithm::Md5 => {
                let result = md5::compute(data);
                Ok(format!("{:x}", result))
            }
        }
    }
    
    /// Verify checksum for data
    pub async fn verify(&self, data: &[u8], expected_checksum: &str) -> Result<bool, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }
        
        let actual_checksum = self.generate(data).await?;
        Ok(actual_checksum == expected_checksum)
    }
}

/// Version verifier for data consistency
#[derive(Debug, Clone)]
pub struct VersionVerifier {
    /// Whether the verifier is initialized
    initialized: bool,
}

impl VersionVerifier {
    /// Create a new version verifier
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }
    
    /// Create a new version verifier with configuration
    pub fn with_config(_config: VersionConfig) -> Self {
        Self {
            initialized: false,
        }
    }
    
    /// Initialize the version verifier
    pub async fn initialize(&mut self) -> Result<(), IntegrityError> {
        self.initialized = true;
        Ok(())
    }
    
    /// Verify version consistency
    pub async fn verify(&self, actual_version: &str, expected_version: &str) -> Result<bool, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }
        
        Ok(actual_version == expected_version)
    }
}

/// Corruption detector for data validation
#[derive(Debug, Clone)]
pub struct CorruptionDetector {
    /// Whether the detector is initialized
    initialized: bool,
}

impl CorruptionDetector {
    /// Create a new corruption detector
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }
    
    /// Create a new corruption detector with configuration
    pub fn with_config(_config: CorruptionConfig) -> Self {
        Self {
            initialized: false,
        }
    }
    
    /// Initialize the corruption detector
    pub async fn initialize(&mut self) -> Result<(), IntegrityError> {
        self.initialized = true;
        Ok(())
    }
    
    /// Detect corruption in data
    pub async fn detect_corruption(&self, data: &[u8], metadata: &DataMetadata) -> Result<CorruptionResult, IntegrityError> {
        if !self.initialized {
            return Err(IntegrityError::NotInitialized);
        }
        
        let mut result = CorruptionResult::new();
        
        // Check for null bytes in non-binary data
        if metadata.format != DataFormat::Binary && data.contains(&0) {
            result.is_corrupted = true;
            result.details.push("Null bytes detected in non-binary data".to_string());
        }
        
        // Check for unexpected data size
        if let Some(expected_size) = metadata.expected_size {
            if data.len() != expected_size {
                result.is_corrupted = true;
                result.details.push(format!(
                    "Data size mismatch: expected {}, got {}",
                    expected_size,
                    data.len()
                ));
            }
        }
        
        // Check for data format violations
        match metadata.format {
            DataFormat::Json => {
                if let Err(e) = serde_json::from_slice::<serde_json::Value>(data) {
                    result.is_corrupted = true;
                    result.details.push(format!("Invalid JSON: {}", e));
                }
            }
            DataFormat::Text => {
                if let Err(e) = std::str::from_utf8(data) {
                    result.is_corrupted = true;
                    result.details.push(format!("Invalid UTF-8: {}", e));
                }
            }
            _ => {} // Other formats don't have specific validation
        }
        
        Ok(result)
    }
}

/// Data metadata for integrity verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataMetadata {
    /// Data checksum
    pub checksum: String,
    /// Data version
    pub version: String,
    /// Expected version
    pub expected_version: String,
    /// Data format
    pub format: DataFormat,
    /// Expected data size
    pub expected_size: Option<usize>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
}

/// Data format types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataFormat {
    /// JSON format
    Json,
    /// Bincode format
    Bincode,
    /// Binary format
    Binary,
    /// Text format
    Text,
}

/// Checksum algorithms
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    /// SHA-256 algorithm
    Sha256,
    /// SHA-1 algorithm
    Sha1,
    /// MD5 algorithm
    Md5,
}

/// Integrity result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrityResult {
    /// Whether checksum is valid
    pub checksum_valid: bool,
    /// Whether version is valid
    pub version_valid: bool,
    /// Whether corruption was detected
    pub corruption_detected: bool,
    /// Corruption details
    pub corruption_details: Vec<String>,
    /// Whether overall integrity is valid
    pub overall_valid: bool,
    /// Any errors encountered
    pub errors: Vec<IntegrityError>,
}

impl IntegrityResult {
    /// Create a new integrity result
    pub fn new() -> Self {
        Self {
            checksum_valid: false,
            version_valid: false,
            corruption_detected: false,
            corruption_details: Vec::new(),
            overall_valid: false,
            errors: Vec::new(),
        }
    }
}

/// Corruption detection result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorruptionResult {
    /// Whether corruption was detected
    pub is_corrupted: bool,
    /// Details about the corruption
    pub details: Vec<String>,
}

impl CorruptionResult {
    /// Create a new corruption result
    pub fn new() -> Self {
        Self {
            is_corrupted: false,
            details: Vec::new(),
        }
    }
}

/// Integrity statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrityStats {
    /// Total number of verifications
    pub total_verifications: usize,
    /// Number of verifications that passed
    pub total_verifications_passed: usize,
    /// Number of verifications that failed
    pub total_verifications_failed: usize,
    /// Number of checksum verifications that passed
    pub checksum_verifications_passed: usize,
    /// Number of checksum verifications that failed
    pub checksum_verifications_failed: usize,
    /// Number of version verifications that passed
    pub version_verifications_passed: usize,
    /// Number of version verifications that failed
    pub version_verifications_failed: usize,
    /// Number of corruption detections
    pub corruption_detections: usize,
}

impl IntegrityStats {
    /// Create new integrity statistics
    pub fn new() -> Self {
        Self {
            total_verifications: 0,
            total_verifications_passed: 0,
            total_verifications_failed: 0,
            checksum_verifications_passed: 0,
            checksum_verifications_failed: 0,
            version_verifications_passed: 0,
            version_verifications_failed: 0,
            corruption_detections: 0,
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        self.total_verifications = 0;
        self.total_verifications_passed = 0;
        self.total_verifications_failed = 0;
        self.checksum_verifications_passed = 0;
        self.checksum_verifications_failed = 0;
        self.version_verifications_passed = 0;
        self.version_verifications_failed = 0;
        self.corruption_detections = 0;
    }
}

/// Integrity configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrityConfig {
    /// Checksum configuration
    pub checksum_config: ChecksumConfig,
    /// Version configuration
    pub version_config: VersionConfig,
    /// Corruption detection configuration
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

/// Checksum configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChecksumConfig {
    /// Checksum algorithm
    pub algorithm: ChecksumAlgorithm,
}

impl Default for ChecksumConfig {
    fn default() -> Self {
        Self {
            algorithm: ChecksumAlgorithm::Sha256,
        }
    }
}

/// Version configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionConfig {
    /// Enable version verification
    pub enable_version_verification: bool,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            enable_version_verification: true,
        }
    }
}

/// Corruption detection configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorruptionConfig {
    /// Enable corruption detection
    pub enable_corruption_detection: bool,
}

impl Default for CorruptionConfig {
    fn default() -> Self {
        Self {
            enable_corruption_detection: true,
        }
    }
}

/// Integrity errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntegrityError {
    /// System not initialized
    NotInitialized,
    /// Checksum verification failed
    ChecksumVerificationFailed(String),
    /// Version verification failed
    VersionVerificationFailed(String),
    /// Corruption detection failed
    CorruptionDetectionFailed(String),
    /// Checksum mismatch
    ChecksumMismatch,
    /// Version mismatch
    VersionMismatch,
    /// Data corruption detected
    DataCorruption,
    /// Invalid data format
    InvalidDataFormat,
    /// Configuration error
    ConfigurationError(String),
}

impl std::fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrityError::NotInitialized => write!(f, "Data integrity system not initialized"),
            IntegrityError::ChecksumVerificationFailed(msg) => write!(f, "Checksum verification failed: {}", msg),
            IntegrityError::VersionVerificationFailed(msg) => write!(f, "Version verification failed: {}", msg),
            IntegrityError::CorruptionDetectionFailed(msg) => write!(f, "Corruption detection failed: {}", msg),
            IntegrityError::ChecksumMismatch => write!(f, "Checksum mismatch"),
            IntegrityError::VersionMismatch => write!(f, "Version mismatch"),
            IntegrityError::DataCorruption => write!(f, "Data corruption detected"),
            IntegrityError::InvalidDataFormat => write!(f, "Invalid data format"),
            IntegrityError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for IntegrityError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_data_integrity_creation() {
        let integrity = DataIntegrity::new();
        assert!(!integrity.is_initialized());
    }
    
    #[tokio::test]
    async fn test_data_integrity_initialization() {
        let mut integrity = DataIntegrity::new();
        let result = integrity.initialize().await;
        assert!(result.is_ok());
        assert!(integrity.is_initialized());
    }
    
    #[tokio::test]
    async fn test_data_integrity_shutdown() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        let result = integrity.shutdown().await;
        assert!(result.is_ok());
        assert!(!integrity.is_initialized());
    }
    
    #[tokio::test]
    async fn test_checksum_generation() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        
        let data = b"Hello, World!";
        let checksum = integrity.generate_checksum(data).await.unwrap();
        
        // Should generate a valid SHA-256 checksum
        assert_eq!(checksum.len(), 64); // SHA-256 produces 64 hex characters
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
    }
    
    #[tokio::test]
    async fn test_checksum_verification() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        
        let data = b"Hello, World!";
        let checksum = integrity.generate_checksum(data).await.unwrap();
        
        let metadata = DataMetadata {
            checksum: checksum.clone(),
            version: "1.0".to_string(),
            expected_version: "1.0".to_string(),
            format: DataFormat::Text,
            expected_size: Some(data.len()),
            created_at: 1234567890,
            modified_at: 1234567890,
        };
        
        let result = integrity.verify_integrity(data, &metadata).await.unwrap();
        assert!(result.checksum_valid);
        assert!(result.version_valid);
        assert!(!result.corruption_detected);
        assert!(result.overall_valid);
    }
    
    #[tokio::test]
    async fn test_checksum_mismatch() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        
        let data = b"Hello, World!";
        let wrong_checksum = "wrong_checksum";
        
        let metadata = DataMetadata {
            checksum: wrong_checksum.to_string(),
            version: "1.0".to_string(),
            expected_version: "1.0".to_string(),
            format: DataFormat::Text,
            expected_size: Some(data.len()),
            created_at: 1234567890,
            modified_at: 1234567890,
        };
        
        let result = integrity.verify_integrity(data, &metadata).await.unwrap();
        assert!(!result.checksum_valid);
        assert!(result.version_valid);
        assert!(!result.corruption_detected);
        assert!(!result.overall_valid);
    }
    
    #[tokio::test]
    async fn test_version_mismatch() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        
        let data = b"Hello, World!";
        let checksum = integrity.generate_checksum(data).await.unwrap();
        
        let metadata = DataMetadata {
            checksum,
            version: "1.0".to_string(),
            expected_version: "2.0".to_string(),
            format: DataFormat::Text,
            expected_size: Some(data.len()),
            created_at: 1234567890,
            modified_at: 1234567890,
        };
        
        let result = integrity.verify_integrity(data, &metadata).await.unwrap();
        assert!(result.checksum_valid);
        assert!(!result.version_valid);
        assert!(!result.corruption_detected);
        assert!(!result.overall_valid);
    }
    
    #[tokio::test]
    async fn test_corruption_detection() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        
        let data = b"Hello, World!\x00"; // Contains null byte
        let checksum = integrity.generate_checksum(data).await.unwrap();
        
        let metadata = DataMetadata {
            checksum,
            version: "1.0".to_string(),
            expected_version: "1.0".to_string(),
            format: DataFormat::Text, // Text format shouldn't contain null bytes
            expected_size: Some(data.len()),
            created_at: 1234567890,
            modified_at: 1234567890,
        };
        
        let result = integrity.verify_integrity(data, &metadata).await.unwrap();
        assert!(result.checksum_valid);
        assert!(result.version_valid);
        assert!(result.corruption_detected);
        assert!(!result.overall_valid);
        assert!(!result.corruption_details.is_empty());
    }
    
    #[tokio::test]
    async fn test_data_format_validation() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        
        // Test valid JSON
        let json_data = b"{\"key\": \"value\"}";
        let is_valid = integrity.validate_data_format(json_data, &DataFormat::Json).await.unwrap();
        assert!(is_valid);
        
        // Test invalid JSON
        let invalid_json = b"{invalid json}";
        let is_valid = integrity.validate_data_format(invalid_json, &DataFormat::Json).await.unwrap();
        assert!(!is_valid);
        
        // Test valid text
        let text_data = b"Hello, World!";
        let is_valid = integrity.validate_data_format(text_data, &DataFormat::Text).await.unwrap();
        assert!(is_valid);
        
        // Test invalid text (non-UTF-8)
        let invalid_text = b"Hello, World!\xff";
        let is_valid = integrity.validate_data_format(invalid_text, &DataFormat::Text).await.unwrap();
        assert!(!is_valid);
    }
    
    #[tokio::test]
    async fn test_integrity_statistics() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        
        let data = b"Hello, World!";
        let checksum = integrity.generate_checksum(data).await.unwrap();
        
        let metadata = DataMetadata {
            checksum,
            version: "1.0".to_string(),
            expected_version: "1.0".to_string(),
            format: DataFormat::Text,
            expected_size: Some(data.len()),
            created_at: 1234567890,
            modified_at: 1234567890,
        };
        
        // Perform verification
        let _ = integrity.verify_integrity(data, &metadata).await.unwrap();
        
        let stats = integrity.get_stats().await;
        assert_eq!(stats.total_verifications, 1);
        assert_eq!(stats.total_verifications_passed, 1);
        assert_eq!(stats.total_verifications_failed, 0);
        assert_eq!(stats.checksum_verifications_passed, 1);
        assert_eq!(stats.version_verifications_passed, 1);
    }
    
    #[tokio::test]
    async fn test_integrity_statistics_reset() {
        let mut integrity = DataIntegrity::new();
        integrity.initialize().await.unwrap();
        
        let data = b"Hello, World!";
        let checksum = integrity.generate_checksum(data).await.unwrap();
        
        let metadata = DataMetadata {
            checksum,
            version: "1.0".to_string(),
            expected_version: "1.0".to_string(),
            format: DataFormat::Text,
            expected_size: Some(data.len()),
            created_at: 1234567890,
            modified_at: 1234567890,
        };
        
        // Perform verification
        let _ = integrity.verify_integrity(data, &metadata).await.unwrap();
        
        // Reset statistics
        integrity.reset_stats().await;
        
        let stats = integrity.get_stats().await;
        assert_eq!(stats.total_verifications, 0);
        assert_eq!(stats.total_verifications_passed, 0);
        assert_eq!(stats.total_verifications_failed, 0);
    }
    
    #[test]
    fn test_checksum_algorithm_default() {
        let config = ChecksumConfig::default();
        assert_eq!(config.algorithm, ChecksumAlgorithm::Sha256);
    }
    
    #[test]
    fn test_integrity_config_default() {
        let config = IntegrityConfig::default();
        assert_eq!(config.checksum_config.algorithm, ChecksumAlgorithm::Sha256);
        assert!(config.version_config.enable_version_verification);
        assert!(config.corruption_config.enable_corruption_detection);
    }
    
    #[test]
    fn test_integrity_error_display() {
        let error = IntegrityError::ChecksumMismatch;
        let error_string = format!("{}", error);
        assert!(error_string.contains("Checksum mismatch"));
    }
    
    #[test]
    fn test_data_metadata_creation() {
        let metadata = DataMetadata {
            checksum: "test_checksum".to_string(),
            version: "1.0".to_string(),
            expected_version: "1.0".to_string(),
            format: DataFormat::Json,
            expected_size: Some(100),
            created_at: 1234567890,
            modified_at: 1234567890,
        };
        
        assert_eq!(metadata.checksum, "test_checksum");
        assert_eq!(metadata.version, "1.0");
        assert_eq!(metadata.format, DataFormat::Json);
        assert_eq!(metadata.expected_size, Some(100));
    }
}
