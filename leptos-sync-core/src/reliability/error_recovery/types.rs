//! Common types for error recovery

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Error types for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Network-related errors
    Network,
    /// Timeout errors
    Timeout,
    /// Authentication/authorization errors
    Authentication,
    /// Resource not found errors
    NotFound,
    /// Server errors (5xx)
    Server,
    /// Client errors (4xx)
    Client,
    /// Unknown errors
    Unknown,
}

/// Recovery statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryStats {
    /// Total recovery attempts
    pub total_attempts: u64,
    /// Successful recoveries
    pub successful_recoveries: u64,
    /// Failed recoveries
    pub failed_recoveries: u64,
    /// Circuit breaker trips
    pub circuit_breaker_trips: u64,
    /// Average recovery time in milliseconds
    pub average_recovery_time_ms: u64,
    /// Total recovery time in milliseconds
    pub total_recovery_time_ms: u64,
}

impl RecoveryStats {
    /// Create new recovery statistics
    pub fn new() -> Self {
        Self {
            total_attempts: 0,
            successful_recoveries: 0,
            failed_recoveries: 0,
            circuit_breaker_trips: 0,
            average_recovery_time_ms: 0,
            total_recovery_time_ms: 0,
        }
    }
    
    /// Record a recovery attempt
    pub fn record_attempt(&mut self) {
        self.total_attempts += 1;
    }
    
    /// Record a successful recovery
    pub fn record_success(&mut self, retry_count: usize, _duration: Duration) {
        self.successful_recoveries += 1;
        // Note: duration parameter is kept for API compatibility but not used
        // In a real implementation, this would update timing statistics
    }
    
    /// Record a failed recovery
    pub fn record_failure(&mut self, retry_count: usize, _duration: Duration, error_type: ErrorType) {
        self.failed_recoveries += 1;
        // Note: duration and error_type parameters are kept for API compatibility
        // In a real implementation, these would be used for detailed statistics
    }
    
    /// Record a circuit breaker trip
    pub fn record_circuit_breaker_trip(&mut self) {
        self.circuit_breaker_trips += 1;
    }
    
    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            (self.successful_recoveries as f64 / self.total_attempts as f64) * 100.0
        }
    }
    
    /// Get failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            (self.failed_recoveries as f64 / self.total_attempts as f64) * 100.0
        }
    }
    
    /// Update average recovery time
    pub fn update_average_recovery_time(&mut self, duration: Duration) {
        self.total_recovery_time_ms += duration.as_millis() as u64;
        if self.total_attempts > 0 {
            self.average_recovery_time_ms = self.total_recovery_time_ms / self.total_attempts;
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        self.total_attempts = 0;
        self.successful_recoveries = 0;
        self.failed_recoveries = 0;
        self.circuit_breaker_trips = 0;
        self.average_recovery_time_ms = 0;
        self.total_recovery_time_ms = 0;
    }
}

impl Default for RecoveryStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a recovery operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryResult<T> {
    /// Whether the recovery was successful
    pub success: bool,
    /// The result value (if successful)
    pub value: Option<T>,
    /// Number of retry attempts made
    pub retry_count: usize,
    /// Total time spent on recovery
    pub total_time: Duration,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

impl<T> RecoveryResult<T> {
    /// Create a successful recovery result
    pub fn success(value: T, retry_count: usize, total_time: Duration) -> Self {
        Self {
            success: true,
            value: Some(value),
            retry_count,
            total_time,
            error_message: None,
        }
    }
    
    /// Create a failed recovery result
    pub fn failure(error_message: String, retry_count: usize, total_time: Duration) -> Self {
        Self {
            success: false,
            value: None,
            retry_count,
            total_time,
            error_message: Some(error_message),
        }
    }
    
    /// Check if the recovery was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
    
    /// Check if the recovery failed
    pub fn is_failure(&self) -> bool {
        !self.success
    }
    
    /// Get the result value, panicking if the recovery failed
    pub fn unwrap(self) -> T {
        self.value.expect("RecoveryResult::unwrap() called on a failed result")
    }
    
    /// Get the result value, returning the default if the recovery failed
    pub fn unwrap_or(self, default: T) -> T {
        self.value.unwrap_or(default)
    }
    
    /// Get the result value, returning the result of the closure if the recovery failed
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.value.unwrap_or_else(f)
    }
}

/// Error types for recovery operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryError {
    /// Maximum retry attempts exceeded
    MaxRetriesExceeded,
    /// Circuit breaker is open
    CircuitBreakerOpen,
    /// Operation timeout
    Timeout,
    /// Invalid configuration
    InvalidConfiguration,
    /// System not initialized
    NotInitialized,
    /// Operation failed with custom message
    OperationFailed(String),
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::MaxRetriesExceeded => write!(f, "Maximum retry attempts exceeded"),
            RecoveryError::CircuitBreakerOpen => write!(f, "Circuit breaker is open"),
            RecoveryError::Timeout => write!(f, "Operation timeout"),
            RecoveryError::InvalidConfiguration => write!(f, "Invalid configuration"),
            RecoveryError::NotInitialized => write!(f, "System not initialized"),
            RecoveryError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl std::error::Error for RecoveryError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_recovery_stats_creation() {
        let stats = RecoveryStats::new();
        assert_eq!(stats.total_attempts, 0);
        assert_eq!(stats.successful_recoveries, 0);
        assert_eq!(stats.failed_recoveries, 0);
        assert_eq!(stats.circuit_breaker_trips, 0);
        assert_eq!(stats.average_recovery_time_ms, 0);
        assert_eq!(stats.total_recovery_time_ms, 0);
    }
    
    #[test]
    fn test_recovery_stats_recording() {
        let mut stats = RecoveryStats::new();
        
        stats.record_attempt();
        assert_eq!(stats.total_attempts, 1);
        
        stats.record_success(2, Duration::from_millis(100));
        assert_eq!(stats.successful_recoveries, 1);
        
        stats.record_failure(3, Duration::from_millis(200), ErrorType::Network);
        assert_eq!(stats.failed_recoveries, 1);
        
        stats.record_circuit_breaker_trip();
        assert_eq!(stats.circuit_breaker_trips, 1);
    }
    
    #[test]
    fn test_recovery_stats_rates() {
        let mut stats = RecoveryStats::new();
        
        stats.record_attempt();
        stats.record_success(1, Duration::from_millis(100));
        
        stats.record_attempt();
        stats.record_failure(2, Duration::from_millis(200), ErrorType::Timeout);
        
        assert_eq!(stats.success_rate(), 50.0);
        assert_eq!(stats.failure_rate(), 50.0);
    }
    
    #[test]
    fn test_recovery_stats_empty() {
        let stats = RecoveryStats::new();
        assert_eq!(stats.success_rate(), 0.0);
        assert_eq!(stats.failure_rate(), 0.0);
    }
    
    #[test]
    fn test_recovery_stats_reset() {
        let mut stats = RecoveryStats::new();
        
        stats.record_attempt();
        stats.record_success(1, Duration::from_millis(100));
        stats.record_circuit_breaker_trip();
        
        assert_eq!(stats.total_attempts, 1);
        assert_eq!(stats.successful_recoveries, 1);
        assert_eq!(stats.circuit_breaker_trips, 1);
        
        stats.reset();
        
        assert_eq!(stats.total_attempts, 0);
        assert_eq!(stats.successful_recoveries, 0);
        assert_eq!(stats.circuit_breaker_trips, 0);
    }
    
    #[test]
    fn test_recovery_result_success() {
        let result = RecoveryResult::success("test".to_string(), 2, Duration::from_millis(100));
        
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert_eq!(result.retry_count, 2);
        assert_eq!(result.total_time, Duration::from_millis(100));
        assert!(result.error_message.is_none());
        assert_eq!(result.unwrap(), "test");
    }
    
    #[test]
    fn test_recovery_result_failure() {
        let result = RecoveryResult::failure("test error".to_string(), 3, Duration::from_millis(200));
        
        assert!(!result.is_success());
        assert!(result.is_failure());
        assert_eq!(result.retry_count, 3);
        assert_eq!(result.total_time, Duration::from_millis(200));
        assert_eq!(result.error_message, Some("test error".to_string()));
        assert_eq!(result.unwrap_or("default".to_string()), "default");
    }
    
    #[test]
    fn test_recovery_result_unwrap_or_else() {
        let result = RecoveryResult::failure("test error".to_string(), 1, Duration::from_millis(50));
        
        let value = result.unwrap_or_else(|| "computed".to_string());
        assert_eq!(value, "computed");
    }
    
    #[test]
    fn test_recovery_error_display() {
        let error = RecoveryError::MaxRetriesExceeded;
        assert_eq!(format!("{}", error), "Maximum retry attempts exceeded");
        
        let error = RecoveryError::OperationFailed("test".to_string());
        assert_eq!(format!("{}", error), "Operation failed: test");
    }
}
