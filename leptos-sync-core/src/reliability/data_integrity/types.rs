//! Common types for data integrity

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data format types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataFormat {
    /// JSON format
    Json,
    /// Binary format
    Binary,
    /// Text format
    Text,
    /// Custom format
    Custom(String),
}

/// Metadata for data items
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataMetadata {
    /// Data format
    pub format: DataFormat,
    /// Size in bytes
    pub size: usize,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Custom metadata
    pub custom: HashMap<String, String>,
}

impl DataMetadata {
    /// Create new data metadata
    pub fn new(format: DataFormat, size: usize) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            format,
            size,
            created_at: now,
            modified_at: now,
            custom: HashMap::new(),
        }
    }
    
    /// Update modified timestamp
    pub fn touch(&mut self) {
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// Add custom metadata
    pub fn add_custom(&mut self, key: String, value: String) {
        self.custom.insert(key, value);
    }
    
    /// Get custom metadata
    pub fn get_custom(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }
}

/// Result of integrity verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityResult {
    /// Whether the data is valid
    pub is_valid: bool,
    /// Checksum verification result
    pub checksum_valid: bool,
    /// Version verification result
    pub version_valid: bool,
    /// Corruption detection result
    pub corruption_detected: bool,
    /// Error message if validation failed
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: DataMetadata,
}

impl IntegrityResult {
    /// Create a new integrity result
    pub fn new(is_valid: bool, metadata: DataMetadata) -> Self {
        Self {
            is_valid,
            checksum_valid: is_valid,
            version_valid: is_valid,
            corruption_detected: !is_valid,
            error_message: if is_valid { None } else { Some("Validation failed".to_string()) },
            metadata,
        }
    }
    
    /// Create a valid result
    pub fn valid(metadata: DataMetadata) -> Self {
        Self::new(true, metadata)
    }
    
    /// Create an invalid result
    pub fn invalid(error_message: String, metadata: DataMetadata) -> Self {
        Self {
            is_valid: false,
            checksum_valid: false,
            version_valid: false,
            corruption_detected: true,
            error_message: Some(error_message),
            metadata,
        }
    }
}

/// Statistics for integrity operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntegrityStats {
    /// Total verifications performed
    pub total_verifications: u64,
    /// Successful verifications
    pub successful_verifications: u64,
    /// Failed verifications
    pub failed_verifications: u64,
    /// Corruption detections
    pub corruption_detections: u64,
    /// Recovery attempts
    pub recovery_attempts: u64,
    /// Successful recoveries
    pub successful_recoveries: u64,
}

impl IntegrityStats {
    /// Create new integrity statistics
    pub fn new() -> Self {
        Self {
            total_verifications: 0,
            successful_verifications: 0,
            failed_verifications: 0,
            corruption_detections: 0,
            recovery_attempts: 0,
            successful_recoveries: 0,
        }
    }
    
    /// Record a verification
    pub fn record_verification(&mut self, success: bool) {
        self.total_verifications += 1;
        if success {
            self.successful_verifications += 1;
        } else {
            self.failed_verifications += 1;
        }
    }
    
    /// Record corruption detection
    pub fn record_corruption_detection(&mut self) {
        self.corruption_detections += 1;
    }
    
    /// Record recovery attempt
    pub fn record_recovery_attempt(&mut self, success: bool) {
        self.recovery_attempts += 1;
        if success {
            self.successful_recoveries += 1;
        }
    }
    
    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_verifications == 0 {
            0.0
        } else {
            (self.successful_verifications as f64) / (self.total_verifications as f64)
        }
    }
    
    /// Get recovery success rate
    pub fn recovery_success_rate(&self) -> f64 {
        if self.recovery_attempts == 0 {
            0.0
        } else {
            (self.successful_recoveries as f64) / (self.recovery_attempts as f64)
        }
    }
}

impl Default for IntegrityStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_metadata_creation() {
        let metadata = DataMetadata::new(DataFormat::Json, 1024);
        assert_eq!(metadata.format, DataFormat::Json);
        assert_eq!(metadata.size, 1024);
        assert!(metadata.created_at > 0);
        assert_eq!(metadata.created_at, metadata.modified_at);
        assert!(metadata.custom.is_empty());
    }
    
    #[test]
    fn test_data_metadata_touch() {
        let mut metadata = DataMetadata::new(DataFormat::Binary, 512);
        let original_modified = metadata.modified_at;
        
        // Wait a small amount to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        metadata.touch();
        
        assert!(metadata.modified_at > original_modified);
    }
    
    #[test]
    fn test_data_metadata_custom() {
        let mut metadata = DataMetadata::new(DataFormat::Text, 256);
        
        metadata.add_custom("key1".to_string(), "value1".to_string());
        metadata.add_custom("key2".to_string(), "value2".to_string());
        
        assert_eq!(metadata.get_custom("key1"), Some(&"value1".to_string()));
        assert_eq!(metadata.get_custom("key2"), Some(&"value2".to_string()));
        assert_eq!(metadata.get_custom("key3"), None);
    }
    
    #[test]
    fn test_integrity_result_creation() {
        let metadata = DataMetadata::new(DataFormat::Json, 1024);
        let result = IntegrityResult::valid(metadata.clone());
        
        assert!(result.is_valid);
        assert!(result.checksum_valid);
        assert!(result.version_valid);
        assert!(!result.corruption_detected);
        assert!(result.error_message.is_none());
        assert_eq!(result.metadata, metadata);
    }
    
    #[test]
    fn test_integrity_result_invalid() {
        let metadata = DataMetadata::new(DataFormat::Binary, 512);
        let error_msg = "Test error".to_string();
        let result = IntegrityResult::invalid(error_msg.clone(), metadata.clone());
        
        assert!(!result.is_valid);
        assert!(!result.checksum_valid);
        assert!(!result.version_valid);
        assert!(result.corruption_detected);
        assert_eq!(result.error_message, Some(error_msg));
        assert_eq!(result.metadata, metadata);
    }
    
    #[test]
    fn test_integrity_stats_creation() {
        let stats = IntegrityStats::new();
        assert_eq!(stats.total_verifications, 0);
        assert_eq!(stats.successful_verifications, 0);
        assert_eq!(stats.failed_verifications, 0);
        assert_eq!(stats.corruption_detections, 0);
        assert_eq!(stats.recovery_attempts, 0);
        assert_eq!(stats.successful_recoveries, 0);
    }
    
    #[test]
    fn test_integrity_stats_recording() {
        let mut stats = IntegrityStats::new();
        
        // Record verifications
        stats.record_verification(true);
        stats.record_verification(false);
        stats.record_verification(true);
        
        assert_eq!(stats.total_verifications, 3);
        assert_eq!(stats.successful_verifications, 2);
        assert_eq!(stats.failed_verifications, 1);
        
        // Record corruption and recovery
        stats.record_corruption_detection();
        stats.record_recovery_attempt(true);
        stats.record_recovery_attempt(false);
        
        assert_eq!(stats.corruption_detections, 1);
        assert_eq!(stats.recovery_attempts, 2);
        assert_eq!(stats.successful_recoveries, 1);
    }
    
    #[test]
    fn test_integrity_stats_rates() {
        let mut stats = IntegrityStats::new();
        
        // Test empty stats
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.recovery_success_rate(), 0.0);
        
        // Test with data
        stats.record_verification(true);
        stats.record_verification(false);
        assert_eq!(stats.success_rate(), 0.5);
        
        stats.record_recovery_attempt(true);
        stats.record_recovery_attempt(false);
        assert_eq!(stats.recovery_success_rate(), 0.5);
    }
}
