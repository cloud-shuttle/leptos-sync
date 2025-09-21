//! Version verification for data consistency

use super::super::IntegrityError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for version verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionConfig {
    /// Whether to verify versions on read
    pub verify_on_read: bool,
    /// Whether to increment versions on write
    pub increment_on_write: bool,
    /// Maximum version difference allowed
    pub max_version_diff: u64,
}

impl Default for VersionConfig {
    fn default() -> Self {
        Self {
            verify_on_read: true,
            increment_on_write: true,
            max_version_diff: 1,
        }
    }
}

/// Version verifier for data consistency
#[derive(Debug, Clone)]
pub struct VersionVerifier {
    /// Configuration
    config: VersionConfig,
    /// Version tracking for data items
    versions: HashMap<String, u64>,
}

impl VersionVerifier {
    /// Create a new version verifier
    pub fn new() -> Self {
        Self {
            config: VersionConfig::default(),
            versions: HashMap::new(),
        }
    }
    
    /// Create a new version verifier with configuration
    pub fn with_config(config: VersionConfig) -> Self {
        Self {
            config,
            versions: HashMap::new(),
        }
    }
    
    /// Get current version for a data item
    pub fn get_version(&self, key: &str) -> u64 {
        self.versions.get(key).copied().unwrap_or(0)
    }
    
    /// Increment version for a data item
    pub fn increment_version(&mut self, key: &str) -> u64 {
        let current_version = self.get_version(key);
        let new_version = current_version + 1;
        self.versions.insert(key.to_string(), new_version);
        new_version
    }
    
    /// Verify version consistency
    pub fn verify_version(&self, key: &str, expected_version: u64) -> Result<bool, IntegrityError> {
        let current_version = self.get_version(key);
        let version_diff = if current_version > expected_version {
            current_version - expected_version
        } else {
            expected_version - current_version
        };
        
        if version_diff > self.config.max_version_diff {
            return Err(IntegrityError::VersionMismatch {
                key: key.to_string(),
                expected: expected_version,
                actual: current_version,
            });
        }
        
        Ok(true)
    }
    
    /// Set version for a data item
    pub fn set_version(&mut self, key: &str, version: u64) {
        self.versions.insert(key.to_string(), version);
    }
    
    /// Remove version tracking for a data item
    pub fn remove_version(&mut self, key: &str) {
        self.versions.remove(key);
    }
    
    /// Get all tracked versions
    pub fn get_all_versions(&self) -> &HashMap<String, u64> {
        &self.versions
    }
    
    /// Clear all version tracking
    pub fn clear_versions(&mut self) {
        self.versions.clear();
    }
    
    /// Get configuration
    pub fn config(&self) -> &VersionConfig {
        &self.config
    }
}

impl Default for VersionVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_verifier_creation() {
        let verifier = VersionVerifier::new();
        assert!(verifier.config().verify_on_read);
        assert!(verifier.config().increment_on_write);
        assert_eq!(verifier.config().max_version_diff, 1);
    }
    
    #[test]
    fn test_version_verifier_with_config() {
        let config = VersionConfig {
            verify_on_read: false,
            increment_on_write: true,
            max_version_diff: 5,
        };
        let verifier = VersionVerifier::with_config(config.clone());
        assert!(!verifier.config().verify_on_read);
        assert!(verifier.config().increment_on_write);
        assert_eq!(verifier.config().max_version_diff, 5);
    }
    
    #[test]
    fn test_get_version() {
        let verifier = VersionVerifier::new();
        assert_eq!(verifier.get_version("test_key"), 0);
    }
    
    #[test]
    fn test_increment_version() {
        let mut verifier = VersionVerifier::new();
        let key = "test_key";
        
        assert_eq!(verifier.get_version(key), 0);
        assert_eq!(verifier.increment_version(key), 1);
        assert_eq!(verifier.get_version(key), 1);
        assert_eq!(verifier.increment_version(key), 2);
        assert_eq!(verifier.get_version(key), 2);
    }
    
    #[test]
    fn test_verify_version() {
        let mut verifier = VersionVerifier::new();
        let key = "test_key";
        
        // Set version to 5
        verifier.set_version(key, 5);
        
        // Verify exact match
        assert!(verifier.verify_version(key, 5).unwrap());
        
        // Verify within tolerance
        assert!(verifier.verify_version(key, 4).unwrap());
        assert!(verifier.verify_version(key, 6).unwrap());
        
        // Verify outside tolerance
        assert!(verifier.verify_version(key, 3).is_err());
        assert!(verifier.verify_version(key, 7).is_err());
    }
    
    #[test]
    fn test_set_version() {
        let mut verifier = VersionVerifier::new();
        let key = "test_key";
        
        verifier.set_version(key, 42);
        assert_eq!(verifier.get_version(key), 42);
    }
    
    #[test]
    fn test_remove_version() {
        let mut verifier = VersionVerifier::new();
        let key = "test_key";
        
        verifier.set_version(key, 5);
        assert_eq!(verifier.get_version(key), 5);
        
        verifier.remove_version(key);
        assert_eq!(verifier.get_version(key), 0);
    }
    
    #[test]
    fn test_clear_versions() {
        let mut verifier = VersionVerifier::new();
        
        verifier.set_version("key1", 1);
        verifier.set_version("key2", 2);
        assert_eq!(verifier.get_all_versions().len(), 2);
        
        verifier.clear_versions();
        assert_eq!(verifier.get_all_versions().len(), 0);
    }
}
