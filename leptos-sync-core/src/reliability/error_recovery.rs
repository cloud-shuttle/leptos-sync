//! Error Recovery System
//!
//! This module provides advanced error recovery capabilities including:
//! - Configurable retry policies (exponential backoff, linear, fixed)
//! - Circuit breaker pattern for fault tolerance
//! - Recovery strategies for different error types
//! - Automatic error classification and handling

use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

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
        // Reset circuit breaker
        let mut breaker = self.circuit_breaker.write().await;
        breaker.reset();
        
        // Reset statistics
        let mut stats = self.stats.write().await;
        stats.reset();
        
        self.initialized = true;
        Ok(())
    }
    
    /// Shutdown the error recovery system
    pub async fn shutdown(&mut self) -> Result<(), RecoveryError> {
        self.initialized = false;
        Ok(())
    }
    
    /// Check if the system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Execute an operation with error recovery
    pub async fn execute_with_recovery<F, T, E>(
        &self,
        operation: F,
    ) -> Result<T, RecoveryError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        if !self.initialized {
            return Err(RecoveryError::NotInitialized);
        }
        
        // Check circuit breaker
        let mut breaker = self.circuit_breaker.write().await;
        if !breaker.can_execute() {
            return Err(RecoveryError::CircuitBreakerOpen);
        }
        
        let mut attempt = 0;
        let start_time = Instant::now();
        
        loop {
            attempt += 1;
            
            // Execute the operation
            let result = operation().await;
            
            match result {
                Ok(value) => {
                    // Success - reset circuit breaker and update stats
                    breaker.record_success();
                    let mut stats = self.stats.write().await;
                    stats.record_success(attempt, start_time.elapsed());
                    return Ok(value);
                }
                Err(error) => {
                    // Failure - check if we should retry
                    let error_type = self.classify_error(&error);
                    
                    if attempt > self.policy.max_retries {
                        // Max retries exceeded
                        breaker.record_failure();
                        let mut stats = self.stats.write().await;
                        stats.record_failure(attempt - 1, start_time.elapsed(), error_type.clone());
                        return Err(RecoveryError::MaxRetriesExceeded);
                    }
                    
                    if !self.should_retry(&error_type, attempt) {
                        // Don't retry this error type
                        breaker.record_failure();
                        let mut stats = self.stats.write().await;
                        stats.record_failure(attempt - 1, start_time.elapsed(), error_type.clone());
                        return Err(RecoveryError::NonRetryableError(error_type));
                    }
                    
                    // Calculate delay and wait
                    let delay = self.calculate_delay(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    /// Classify an error to determine recovery strategy
    fn classify_error<E: std::error::Error>(&self, error: &E) -> ErrorType {
        let error_string = error.to_string().to_lowercase();
        
        if error_string.contains("timeout") || error_string.contains("connection") {
            ErrorType::Network
        } else if error_string.contains("permission") || error_string.contains("unauthorized") {
            ErrorType::Permission
        } else if error_string.contains("not found") || error_string.contains("missing") {
            ErrorType::NotFound
        } else if error_string.contains("conflict") || error_string.contains("duplicate") {
            ErrorType::Conflict
        } else if error_string.contains("rate limit") || error_string.contains("throttle") {
            ErrorType::RateLimit
        } else {
            ErrorType::Unknown
        }
    }
    
    /// Determine if an error should be retried
    fn should_retry(&self, error_type: &ErrorType, attempt: usize) -> bool {
        match error_type {
            ErrorType::Network => true,
            ErrorType::RateLimit => true,
            ErrorType::Permission => false,
            ErrorType::NotFound => false,
            ErrorType::Conflict => false,
            ErrorType::Unknown => attempt < 3, // Retry unknown errors up to 3 times
        }
    }
    
    /// Calculate delay for next retry attempt
    fn calculate_delay(&self, attempt: usize) -> Duration {
        match &self.policy.strategy {
            RetryStrategy::ExponentialBackoff(config) => {
                let base_delay = config.base_delay;
                let max_delay = config.max_delay;
                let multiplier = config.multiplier;
                
                let delay = base_delay * multiplier.pow(attempt as u32 - 1);
                delay.min(max_delay)
            }
            RetryStrategy::Linear(config) => {
                let base_delay = config.base_delay;
                let increment = config.increment;
                let max_delay = config.max_delay;
                
                let delay = base_delay + (increment * (attempt - 1) as u32);
                delay.min(max_delay)
            }
            RetryStrategy::Fixed(config) => config.delay,
        }
    }
    
    /// Get recovery statistics
    pub async fn get_stats(&self) -> RecoveryStats {
        self.stats.read().await.clone()
    }
    
    /// Get circuit breaker status
    pub async fn get_circuit_breaker_status(&self) -> CircuitBreakerStatus {
        let breaker = self.circuit_breaker.read().await;
        breaker.get_status()
    }
    
    /// Reset circuit breaker
    pub async fn reset_circuit_breaker(&self) {
        let mut breaker = self.circuit_breaker.write().await;
        breaker.reset();
    }
}

/// Retry policy configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Retry strategy
    pub strategy: RetryStrategy,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            strategy: RetryStrategy::ExponentialBackoff(ExponentialBackoffConfig::default()),
        }
    }
}

/// Retry strategy types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// Exponential backoff strategy
    ExponentialBackoff(ExponentialBackoffConfig),
    /// Linear backoff strategy
    Linear(LinearBackoffConfig),
    /// Fixed delay strategy
    Fixed(FixedDelayConfig),
}

/// Exponential backoff configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExponentialBackoffConfig {
    /// Base delay for first retry
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for each retry
    pub multiplier: u32,
}

impl Default for ExponentialBackoffConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2,
        }
    }
}

/// Linear backoff configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearBackoffConfig {
    /// Base delay for first retry
    pub base_delay: Duration,
    /// Increment for each retry
    pub increment: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
}

impl Default for LinearBackoffConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_millis(100),
            increment: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
        }
    }
}

/// Fixed delay configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixedDelayConfig {
    /// Fixed delay between retries
    pub delay: Duration,
}

impl Default for FixedDelayConfig {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(1000),
        }
    }
}

/// Circuit breaker policy for fault tolerance
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreakerPolicy {
    /// Current state of the circuit breaker
    state: CircuitBreakerState,
    /// Number of consecutive failures
    failure_count: usize,
    /// Number of consecutive successes
    success_count: usize,
    /// Failure threshold to open circuit
    failure_threshold: usize,
    /// Success threshold to close circuit
    success_threshold: usize,
    /// Timeout for half-open state
    timeout: Duration,
    /// Last failure time (not serialized)
    #[serde(skip_serializing, skip_deserializing)]
    last_failure_time: Option<Instant>,
}

impl CircuitBreakerPolicy {
    /// Create a new circuit breaker policy
    pub fn new() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            last_failure_time: None,
        }
    }
    
    /// Check if the circuit breaker allows execution
    pub fn can_execute(&self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    last_failure.elapsed() >= self.timeout
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
    
    /// Record a successful operation
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.failure_count = 0;
        
        if self.state == CircuitBreakerState::HalfOpen && self.success_count >= self.success_threshold {
            self.state = CircuitBreakerState::Closed;
            self.success_count = 0;
        }
    }
    
    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.success_count = 0;
        self.last_failure_time = Some(Instant::now());
        
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        } else if self.state == CircuitBreakerState::HalfOpen {
            self.state = CircuitBreakerState::Open;
        }
    }
    
    /// Reset the circuit breaker
    pub fn reset(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
    }
    
    /// Get the current status of the circuit breaker
    pub fn get_status(&self) -> CircuitBreakerStatus {
        CircuitBreakerStatus {
            state: self.state.clone(),
            failure_count: self.failure_count,
            success_count: self.success_count,
            last_failure_time: self.last_failure_time,
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit is closed - operations are allowed
    Closed,
    /// Circuit is open - operations are blocked
    Open,
    /// Circuit is half-open - testing if service is recovered
    HalfOpen,
}

/// Circuit breaker status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    /// Current state
    pub state: CircuitBreakerState,
    /// Number of consecutive failures
    pub failure_count: usize,
    /// Number of consecutive successes
    pub success_count: usize,
    /// Last failure time (not serialized)
    #[serde(skip_serializing, skip_deserializing)]
    pub last_failure_time: Option<Instant>,
}

/// Error types for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    /// Network-related errors
    Network,
    /// Permission/authorization errors
    Permission,
    /// Resource not found errors
    NotFound,
    /// Conflict/duplicate errors
    Conflict,
    /// Rate limiting errors
    RateLimit,
    /// Unknown error types
    Unknown,
}

/// Recovery statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryStats {
    /// Total number of operations attempted
    pub total_operations: usize,
    /// Number of successful operations
    pub successful_operations: usize,
    /// Number of failed operations
    pub failed_operations: usize,
    /// Total retry attempts
    pub total_retries: usize,
    /// Average retry attempts per operation
    pub avg_retries_per_operation: f64,
    /// Average operation duration
    pub avg_operation_duration: Duration,
    /// Error type distribution
    pub error_type_distribution: std::collections::HashMap<ErrorType, usize>,
}

impl RecoveryStats {
    /// Create new recovery statistics
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            total_retries: 0,
            avg_retries_per_operation: 0.0,
            avg_operation_duration: Duration::ZERO,
            error_type_distribution: std::collections::HashMap::new(),
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        self.total_operations = 0;
        self.successful_operations = 0;
        self.failed_operations = 0;
        self.total_retries = 0;
        self.avg_retries_per_operation = 0.0;
        self.avg_operation_duration = Duration::ZERO;
        self.error_type_distribution.clear();
    }
    
    /// Record a successful operation
    pub fn record_success(&mut self, retry_count: usize, duration: Duration) {
        self.total_operations += 1;
        self.successful_operations += 1;
        self.total_retries += retry_count;
        
        self.update_averages();
    }
    
    /// Record a failed operation
    pub fn record_failure(&mut self, retry_count: usize, duration: Duration, error_type: ErrorType) {
        self.total_operations += 1;
        self.failed_operations += 1;
        self.total_retries += retry_count;
        
        *self.error_type_distribution.entry(error_type).or_insert(0) += 1;
        
        self.update_averages();
    }
    
    /// Update average calculations
    fn update_averages(&mut self) {
        if self.total_operations > 0 {
            self.avg_retries_per_operation = self.total_retries as f64 / self.total_operations as f64;
        }
    }
}

/// Recovery configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Retry policy
    pub retry_policy: RetryPolicy,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerPolicy,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            retry_policy: RetryPolicy::default(),
            circuit_breaker: CircuitBreakerPolicy::new(),
        }
    }
}

/// Recovery result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryResult<T> {
    /// The result value
    pub value: T,
    /// Number of retry attempts
    pub retry_count: usize,
    /// Total operation duration
    pub duration: Duration,
    /// Whether the operation succeeded on first try
    pub first_try_success: bool,
}

/// Recovery errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryError {
    /// System not initialized
    NotInitialized,
    /// Maximum retries exceeded
    MaxRetriesExceeded,
    /// Circuit breaker is open
    CircuitBreakerOpen,
    /// Non-retryable error type
    NonRetryableError(ErrorType),
    /// Operation timeout
    Timeout,
    /// Configuration error
    ConfigurationError(String),
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::NotInitialized => write!(f, "Error recovery system not initialized"),
            RecoveryError::MaxRetriesExceeded => write!(f, "Maximum retry attempts exceeded"),
            RecoveryError::CircuitBreakerOpen => write!(f, "Circuit breaker is open"),
            RecoveryError::NonRetryableError(error_type) => {
                write!(f, "Non-retryable error type: {:?}", error_type)
            }
            RecoveryError::Timeout => write!(f, "Operation timeout"),
            RecoveryError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for RecoveryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[tokio::test]
    async fn test_error_recovery_creation() {
        let recovery = ErrorRecovery::new();
        assert!(!recovery.is_initialized());
    }
    
    #[tokio::test]
    async fn test_error_recovery_initialization() {
        let mut recovery = ErrorRecovery::new();
        let result = recovery.initialize().await;
        assert!(result.is_ok());
        assert!(recovery.is_initialized());
    }
    
    #[tokio::test]
    async fn test_error_recovery_shutdown() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        let result = recovery.shutdown().await;
        assert!(result.is_ok());
        assert!(!recovery.is_initialized());
    }
    
    #[tokio::test]
    async fn test_successful_operation() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let result = recovery.execute_with_recovery(|| {
            Box::pin(async { Ok::<i32, std::io::Error>(42) })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        let stats = recovery.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.failed_operations, 0);
    }
    
    #[tokio::test]
    async fn test_retry_on_network_error() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let attempt_count = std::sync::Arc::new(AtomicUsize::new(0));
        
        let result = recovery.execute_with_recovery(|| {
            let attempt_count = std::sync::Arc::clone(&attempt_count);
            Box::pin(async move {
                let attempt = attempt_count.fetch_add(1, Ordering::SeqCst) + 1;
                if attempt < 3 {
                    Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::TimedOut, "network timeout"))
                } else {
                    Ok(42)
                }
            })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
        
        let stats = recovery.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.total_retries, 2);
    }
    
    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let result = recovery.execute_with_recovery(|| {
            Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::TimedOut, "network timeout")) })
        }).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RecoveryError::MaxRetriesExceeded));
        
        let stats = recovery.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 0);
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.total_retries, 3);
    }
    
    #[tokio::test]
    async fn test_non_retryable_error() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        let result = recovery.execute_with_recovery(|| {
            Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied")) })
        }).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RecoveryError::NonRetryableError(ErrorType::Permission)));
        
        let stats = recovery.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 0);
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.total_retries, 0);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        // Fail enough times to open the circuit breaker
        for _ in 0..6 {
            let _ = recovery.execute_with_recovery(|| {
                Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::TimedOut, "network timeout")) })
            }).await;
        }
        
        // Next operation should fail due to circuit breaker
        let result = recovery.execute_with_recovery(|| {
            Box::pin(async { Ok::<i32, std::io::Error>(42) })
        }).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RecoveryError::CircuitBreakerOpen));
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        // Open the circuit breaker
        for _ in 0..6 {
            let _ = recovery.execute_with_recovery(|| {
                Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::TimedOut, "network timeout")) })
            }).await;
        }
        
        // Reset the circuit breaker
        recovery.reset_circuit_breaker().await;
        
        // Operation should succeed now
        let result = recovery.execute_with_recovery(|| {
            Box::pin(async { Ok::<i32, std::io::Error>(42) })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    
    #[tokio::test]
    async fn test_error_classification() {
        let recovery = ErrorRecovery::new();
        
        // Test network error classification
        let network_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "connection timeout");
        let error_type = recovery.classify_error(&network_error);
        assert_eq!(error_type, ErrorType::Network);
        
        // Test permission error classification
        let permission_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let error_type = recovery.classify_error(&permission_error);
        assert_eq!(error_type, ErrorType::Permission);
        
        // Test rate limit error classification
        let rate_limit_error = std::io::Error::new(std::io::ErrorKind::Other, "rate limit exceeded");
        let error_type = recovery.classify_error(&rate_limit_error);
        assert_eq!(error_type, ErrorType::RateLimit);
    }
    
    #[tokio::test]
    async fn test_retry_policy_configuration() {
        let config = RecoveryConfig {
            retry_policy: RetryPolicy {
                max_retries: 5,
                strategy: RetryStrategy::Fixed(FixedDelayConfig {
                    delay: Duration::from_millis(500),
                }),
            },
            circuit_breaker: CircuitBreakerPolicy::new(),
        };
        
        let mut recovery = ErrorRecovery::with_config(config);
        recovery.initialize().await.unwrap();
        
        let start_time = Instant::now();
        let result = recovery.execute_with_recovery(|| {
            Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::TimedOut, "network timeout")) })
        }).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RecoveryError::MaxRetriesExceeded));
        
        // Should have taken at least 5 * 500ms = 2.5 seconds (with some tolerance)
        assert!(start_time.elapsed() >= Duration::from_millis(2400));
    }
    
    #[tokio::test]
    async fn test_recovery_stats() {
        let mut recovery = ErrorRecovery::new();
        recovery.initialize().await.unwrap();
        
        // Execute some operations
        let _ = recovery.execute_with_recovery(|| {
            Box::pin(async { Ok::<i32, std::io::Error>(42) })
        }).await;
        
        let _ = recovery.execute_with_recovery(|| {
            Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::TimedOut, "network timeout")) })
        }).await;
        
        let stats = recovery.get_stats().await;
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.failed_operations, 1);
        assert!(stats.error_type_distribution.contains_key(&ErrorType::Network));
    }
    
    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert!(matches!(policy.strategy, RetryStrategy::ExponentialBackoff(_)));
    }
    
    #[test]
    fn test_circuit_breaker_policy() {
        let mut policy = CircuitBreakerPolicy::new();
        assert!(policy.can_execute());
        
        // Record some failures
        for _ in 0..5 {
            policy.record_failure();
        }
        
        assert!(!policy.can_execute());
        
        // Manually transition to HalfOpen state (simulating timeout)
        policy.state = CircuitBreakerState::HalfOpen;
        policy.success_count = 0;
        
        // Record some successes
        for _ in 0..3 {
            policy.record_success();
        }
        
        assert!(policy.can_execute());
    }
    
    #[test]
    fn test_recovery_stats_creation() {
        let stats = RecoveryStats::new();
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.successful_operations, 0);
        assert_eq!(stats.failed_operations, 0);
        assert_eq!(stats.total_retries, 0);
    }
    
    #[test]
    fn test_recovery_error_display() {
        let error = RecoveryError::MaxRetriesExceeded;
        let error_string = format!("{}", error);
        assert!(error_string.contains("Maximum retry attempts exceeded"));
    }
}
