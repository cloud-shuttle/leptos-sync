//! Error Recovery Module
//!
//! This module provides advanced error recovery capabilities including:
//! - Configurable retry policies (exponential backoff, linear, fixed)
//! - Circuit breaker pattern for fault tolerance
//! - Recovery strategies for different error types
//! - Automatic error classification and handling

pub mod retry_policy;
pub mod circuit_breaker;
pub mod types;

// Re-export main types for convenience
pub use retry_policy::{
    RetryPolicy, RetryStrategy, ExponentialBackoffConfig, LinearBackoffConfig, FixedDelayConfig
};
pub use circuit_breaker::{
    CircuitBreakerPolicy, CircuitBreakerState, CircuitBreakerStatus
};
pub use types::{
    ErrorType, RecoveryStats, RecoveryResult, RecoveryError
};

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Configuration for the error recovery system
#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryConfig {
    /// Retry policy configuration
    pub retry_policy: RetryPolicy,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerPolicy,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            retry_policy: RetryPolicy::default(),
            circuit_breaker: CircuitBreakerPolicy::default(),
        }
    }
}

/// Error recovery system with configurable retry policies
#[derive(Debug, Clone)]
pub struct ErrorRecovery {
    /// Retry policy configuration
    policy: RetryPolicy,
    /// Circuit breaker for fault tolerance
    circuit_breaker: Arc<RwLock<CircuitBreakerPolicy>>,
    /// Recovery statistics
    stats: Arc<RwLock<RecoveryStats>>,
    /// Whether the system is initialized
    initialized: bool,
}

impl ErrorRecovery {
    /// Create a new error recovery system with default policy
    pub fn new() -> Self {
        Self {
            policy: RetryPolicy::default(),
            circuit_breaker: Arc::new(RwLock::new(CircuitBreakerPolicy::new())),
            stats: Arc::new(RwLock::new(RecoveryStats::new())),
            initialized: false,
        }
    }
    
    /// Create a new error recovery system with custom configuration
    pub fn with_config(config: RecoveryConfig) -> Self {
        Self {
            policy: config.retry_policy,
            circuit_breaker: Arc::new(RwLock::new(config.circuit_breaker)),
            stats: Arc::new(RwLock::new(RecoveryStats::new())),
            initialized: false,
        }
    }
    
    /// Initialize the error recovery system
    pub async fn initialize(&mut self) -> Result<(), RecoveryError> {
        if self.initialized {
            return Err(RecoveryError::NotInitialized);
        }
        
        // Reset circuit breaker
        {
            let mut circuit_breaker = self.circuit_breaker.write().await;
            let mut status = circuit_breaker.get_status();
            circuit_breaker.reset(&mut status);
        }
        
        // Reset statistics
        {
            let mut stats = self.stats.write().await;
            stats.reset();
        }
        
        self.initialized = true;
        Ok(())
    }
    
    /// Shutdown the error recovery system
    pub async fn shutdown(&mut self) -> Result<(), RecoveryError> {
        if !self.initialized {
            return Err(RecoveryError::NotInitialized);
        }
        
        self.initialized = false;
        Ok(())
    }
    
    /// Execute an operation with retry logic and circuit breaker
    pub async fn execute_with_recovery<F, T, E>(
        &self,
        operation: F,
    ) -> Result<RecoveryResult<T>, RecoveryError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'static>> + Send + Sync,
        E: std::fmt::Display + Send + 'static,
    {
        if !self.initialized {
            return Err(RecoveryError::NotInitialized);
        }
        
        let start_time = Instant::now();
        let mut retry_count = 0;
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.record_attempt();
        }
        
        loop {
            // Check circuit breaker
            {
                let circuit_breaker = self.circuit_breaker.read().await;
                let status = circuit_breaker.get_status();
                
                if !circuit_breaker.should_allow_request(&status) {
                    let mut stats = self.stats.write().await;
                    stats.record_circuit_breaker_trip();
                    
                    return Ok(RecoveryResult::failure(
                        "Circuit breaker is open".to_string(),
                        retry_count,
                        start_time.elapsed(),
                    ));
                }
            }
            
            // Execute operation
            let result = operation().await;
            
            match result {
                Ok(value) => {
                    // Success - update circuit breaker and statistics
                    {
                        let mut circuit_breaker = self.circuit_breaker.write().await;
                        let mut status = circuit_breaker.get_status();
                        circuit_breaker.update_state(&mut status, true);
                    }
                    
                    {
                        let mut stats = self.stats.write().await;
                        stats.record_success(retry_count, start_time.elapsed());
                    }
                    
                    return Ok(RecoveryResult::success(value, retry_count, start_time.elapsed()));
                }
                Err(error) => {
                    // Failure - update circuit breaker
                    {
                        let mut circuit_breaker = self.circuit_breaker.write().await;
                        let mut status = circuit_breaker.get_status();
                        circuit_breaker.update_state(&mut status, false);
                    }
                    
                    // Check if we should retry
                    if !self.policy.should_retry(retry_count) {
                        let mut stats = self.stats.write().await;
                        stats.record_failure(retry_count, start_time.elapsed(), ErrorType::Unknown);
                        
                        return Ok(RecoveryResult::failure(
                            format!("Max retries exceeded: {}", error),
                            retry_count,
                            start_time.elapsed(),
                        ));
                    }
                    
                    // Calculate delay and wait
                    let delay = self.policy.calculate_delay(retry_count + 1);
                    if delay > Duration::from_millis(0) {
                        tokio::time::sleep(delay).await;
                    }
                    
                    retry_count += 1;
                }
            }
        }
    }
    
    /// Get recovery statistics
    pub async fn get_stats(&self) -> RecoveryStats {
        self.stats.read().await.clone()
    }
    
    /// Get circuit breaker status
    pub async fn get_circuit_breaker_status(&self) -> CircuitBreakerStatus {
        let circuit_breaker = self.circuit_breaker.read().await;
        circuit_breaker.get_status()
    }
    
    /// Reset circuit breaker
    pub async fn reset_circuit_breaker(&self) -> Result<(), RecoveryError> {
        if !self.initialized {
            return Err(RecoveryError::NotInitialized);
        }
        
        let mut circuit_breaker = self.circuit_breaker.write().await;
        let mut status = circuit_breaker.get_status();
        circuit_breaker.reset(&mut status);
        
        Ok(())
    }
    
    /// Check if the system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get retry policy
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.policy
    }
    
    /// Get circuit breaker policy
    pub async fn circuit_breaker_policy(&self) -> CircuitBreakerPolicy {
        self.circuit_breaker.read().await.clone()
    }
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_error_recovery_creation() {
        let mut recovery = ErrorRecovery::new();
        assert!(!recovery.is_initialized());
        
        recovery.initialize().await.unwrap();
        assert!(recovery.is_initialized());
    }
    
    #[tokio::test]
    async fn test_error_recovery_with_config() {
        let config = RecoveryConfig::default();
        let mut recovery = ErrorRecovery::with_config(config);
        
        recovery.initialize().await.unwrap();
        assert!(recovery.is_initialized());
    }
    
    #[tokio::test]
    async fn test_error_recovery_initialization() {
        let mut recovery = ErrorRecovery::new();
        
        // Initialize
        let result = recovery.initialize().await;
        assert!(result.is_ok());
        assert!(recovery.is_initialized());
        
        // Try to initialize again (should fail)
        let result = recovery.initialize().await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_error_recovery_shutdown() {
        let mut recovery = ErrorRecovery::new();
        
        // Initialize first
        recovery.initialize().await.unwrap();
        assert!(recovery.is_initialized());
        
        // Then shutdown
        let result = recovery.shutdown().await;
        assert!(result.is_ok());
        assert!(!recovery.is_initialized());
    }
    
    #[tokio::test]
    async fn test_execute_with_recovery_success() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let operation = || -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'static>> {
            Box::pin(async {
                Ok::<String, String>("success".to_string())
            })
        };
        
        let result = recovery.execute_with_recovery(operation).await.unwrap();
        assert!(result.is_success());
        assert_eq!(result.clone().unwrap(), "success");
        assert_eq!(result.retry_count, 0);
    }
    
    #[tokio::test]
    async fn test_execute_with_recovery_failure() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let operation = || -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'static>> {
            Box::pin(async {
                Err::<String, String>("operation failed".to_string())
            })
        };
        
        let result = recovery.execute_with_recovery(operation).await.unwrap();
        assert!(result.is_failure());
        assert!(result.error_message.unwrap().contains("Max retries exceeded"));
        assert_eq!(result.retry_count, 3); // Default max retries
    }
    
    #[tokio::test]
    async fn test_execute_with_recovery_before_initialization() {
        let recovery = ErrorRecovery::new();
        
        let operation = || -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'static>> {
            Box::pin(async {
                Ok::<String, String>("success".to_string())
            })
        };
        
        let result = recovery.execute_with_recovery(operation).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_get_stats() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let operation = || -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'static>> {
            Box::pin(async {
                Ok::<String, String>("success".to_string())
            })
        };
        
        recovery.execute_with_recovery(operation).await.unwrap();
        
        let stats = recovery.get_stats().await;
        assert_eq!(stats.total_attempts, 1);
        assert_eq!(stats.successful_recoveries, 1);
        assert_eq!(stats.failed_recoveries, 0);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_status() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let status = recovery.get_circuit_breaker_status().await;
        assert_eq!(status.state, CircuitBreakerState::Closed);
        assert_eq!(status.failure_count, 0);
        assert_eq!(status.success_count, 0);
    }
    
    #[tokio::test]
    async fn test_reset_circuit_breaker() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let result = recovery.reset_circuit_breaker().await;
        assert!(result.is_ok());
        
        let status = recovery.get_circuit_breaker_status().await;
        assert_eq!(status.state, CircuitBreakerState::Closed);
    }
    
    #[tokio::test]
    async fn test_reset_circuit_breaker_before_initialization() {
        let recovery = ErrorRecovery::new();
        
        let result = recovery.reset_circuit_breaker().await;
        assert!(result.is_err());
    }
}
