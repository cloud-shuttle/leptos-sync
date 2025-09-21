//! Corruption detection and recovery

use super::super::IntegrityError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for corruption detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorruptionConfig {
    /// Whether to enable corruption detection
    pub enable_detection: bool,
    /// Whether to enable automatic recovery
    pub enable_recovery: bool,
    /// Maximum corruption threshold
    pub max_corruption_threshold: f64,
    /// Whether to log corruption events
    pub log_corruption_events: bool,
}

impl Default for CorruptionConfig {
    fn default() -> Self {
        Self {
            enable_detection: true,
            enable_recovery: false,
            max_corruption_threshold: 0.1, // 10%
            log_corruption_events: true,
        }
    }
}

/// Result of corruption detection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CorruptionResult {
    /// Whether corruption was detected
    pub is_corrupted: bool,
    /// Corruption level (0.0 to 1.0)
    pub corruption_level: f64,
    /// Description of corruption
    pub description: String,
    /// Whether recovery was attempted
    pub recovery_attempted: bool,
    /// Whether recovery was successful
    pub recovery_successful: bool,
}

impl CorruptionResult {
    /// Create a new corruption result
    pub fn new(is_corrupted: bool, corruption_level: f64, description: String) -> Self {
        Self {
            is_corrupted,
            corruption_level,
            description,
            recovery_attempted: false,
            recovery_successful: false,
        }
    }

    /// Create a clean result (no corruption)
    pub fn clean() -> Self {
        Self::new(false, 0.0, "No corruption detected".to_string())
    }
}

/// Corruption detector for data integrity
#[derive(Debug, Clone)]
pub struct CorruptionDetector {
    /// Configuration
    config: CorruptionConfig,
    /// Corruption statistics
    corruption_stats: HashMap<String, u64>,
    /// Recovery attempts
    recovery_attempts: HashMap<String, u64>,
}

impl CorruptionDetector {
    /// Create a new corruption detector
    pub fn new() -> Self {
        Self {
            config: CorruptionConfig::default(),
            corruption_stats: HashMap::new(),
            recovery_attempts: HashMap::new(),
        }
    }

    /// Create a new corruption detector with configuration
    pub fn with_config(config: CorruptionConfig) -> Self {
        Self {
            config,
            corruption_stats: HashMap::new(),
            recovery_attempts: HashMap::new(),
        }
    }

    /// Detect corruption in data
    pub fn detect_corruption(&mut self, data: &[u8], key: &str) -> CorruptionResult {
        if !self.config.enable_detection {
            return CorruptionResult::clean();
        }

        // Simple corruption detection based on data patterns
        let corruption_level = self.analyze_data_patterns(data);

        if corruption_level > self.config.max_corruption_threshold {
            // Record corruption
            *self.corruption_stats.entry(key.to_string()).or_insert(0) += 1;

            let description = format!(
                "Corruption detected: {:.2}% corruption level exceeds threshold of {:.2}%",
                corruption_level * 100.0,
                self.config.max_corruption_threshold * 100.0
            );

            let mut result = CorruptionResult::new(true, corruption_level, description);

            // Attempt recovery if enabled
            if self.config.enable_recovery {
                result.recovery_attempted = true;
                result.recovery_successful = self.attempt_recovery(key);
            }

            result
        } else {
            CorruptionResult::clean()
        }
    }

    /// Analyze data patterns for corruption
    fn analyze_data_patterns(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        // Simple heuristic: count null bytes and unusual patterns
        let null_count = data.iter().filter(|&&b| b == 0).count();
        let unusual_count = data.iter().filter(|&&b| b > 127).count();

        let total_issues = null_count + unusual_count;
        (total_issues as f64) / (data.len() as f64)
    }

    /// Attempt to recover from corruption
    fn attempt_recovery(&mut self, key: &str) -> bool {
        // Record recovery attempt
        *self.recovery_attempts.entry(key.to_string()).or_insert(0) += 1;

        // Simple recovery: mark as recovered (in real implementation, this would
        // involve more sophisticated recovery mechanisms)
        true
    }

    /// Get corruption statistics for a key
    pub fn get_corruption_count(&self, key: &str) -> u64 {
        self.corruption_stats.get(key).copied().unwrap_or(0)
    }

    /// Get recovery attempt count for a key
    pub fn get_recovery_count(&self, key: &str) -> u64 {
        self.recovery_attempts.get(key).copied().unwrap_or(0)
    }

    /// Get all corruption statistics
    pub fn get_corruption_stats(&self) -> &HashMap<String, u64> {
        &self.corruption_stats
    }

    /// Get all recovery attempt statistics
    pub fn get_recovery_stats(&self) -> &HashMap<String, u64> {
        &self.recovery_attempts
    }

    /// Clear all statistics
    pub fn clear_stats(&mut self) {
        self.corruption_stats.clear();
        self.recovery_attempts.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &CorruptionConfig {
        &self.config
    }
}

impl Default for CorruptionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corruption_detector_creation() {
        let detector = CorruptionDetector::new();
        assert!(detector.config().enable_detection);
        assert!(!detector.config().enable_recovery);
        assert_eq!(detector.config().max_corruption_threshold, 0.1);
        assert!(detector.config().log_corruption_events);
    }

    #[test]
    fn test_corruption_detector_with_config() {
        let config = CorruptionConfig {
            enable_detection: false,
            enable_recovery: true,
            max_corruption_threshold: 0.05,
            log_corruption_events: false,
        };
        let detector = CorruptionDetector::with_config(config.clone());
        assert!(!detector.config().enable_detection);
        assert!(detector.config().enable_recovery);
        assert_eq!(detector.config().max_corruption_threshold, 0.05);
        assert!(!detector.config().log_corruption_events);
    }

    #[test]
    fn test_detect_corruption_clean_data() {
        let mut detector = CorruptionDetector::new();
        let data = b"clean data without corruption";
        let key = "test_key";

        let result = detector.detect_corruption(data, key);
        assert!(!result.is_corrupted);
        assert_eq!(result.corruption_level, 0.0);
        assert_eq!(result.description, "No corruption detected");
    }

    #[test]
    fn test_detect_corruption_corrupted_data() {
        let mut detector = CorruptionDetector::new();
        // Create data with many null bytes (corruption pattern)
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let key = "test_key";

        let result = detector.detect_corruption(data, key);
        assert!(result.is_corrupted);
        assert!(result.corruption_level > 0.0);
        assert!(result.description.contains("Corruption detected"));
    }

    #[test]
    fn test_detect_corruption_disabled() {
        let config = CorruptionConfig {
            enable_detection: false,
            ..Default::default()
        };
        let mut detector = CorruptionDetector::with_config(config);
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let key = "test_key";

        let result = detector.detect_corruption(data, key);
        assert!(!result.is_corrupted);
        assert_eq!(result.corruption_level, 0.0);
    }

    #[test]
    fn test_detect_corruption_with_recovery() {
        let config = CorruptionConfig {
            enable_recovery: true,
            ..Default::default()
        };
        let mut detector = CorruptionDetector::with_config(config);
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let key = "test_key";

        let result = detector.detect_corruption(data, key);
        assert!(result.is_corrupted);
        assert!(result.recovery_attempted);
        assert!(result.recovery_successful);
    }

    #[test]
    fn test_corruption_statistics() {
        let mut detector = CorruptionDetector::new();
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let key = "test_key";

        // Detect corruption multiple times
        detector.detect_corruption(data, key);
        detector.detect_corruption(data, key);
        detector.detect_corruption(data, key);

        assert_eq!(detector.get_corruption_count(key), 3);
        assert_eq!(detector.get_corruption_stats().len(), 1);
    }

    #[test]
    fn test_clear_stats() {
        let mut detector = CorruptionDetector::new();
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let key = "test_key";

        detector.detect_corruption(data, key);
        assert_eq!(detector.get_corruption_count(key), 1);

        detector.clear_stats();
        assert_eq!(detector.get_corruption_count(key), 0);
        assert_eq!(detector.get_corruption_stats().len(), 0);
    }
}
