//! Circuit Breaker System
//!
//! This module provides circuit breaker functionality for fault tolerance including:
//! - Circuit breaker pattern implementation
//! - Configurable failure thresholds
//! - Automatic recovery mechanisms
//! - Circuit state monitoring

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Circuit breaker for fault tolerance
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Circuit breaker state
    state: Arc<RwLock<CircuitBreakerState>>,
    /// Circuit breaker configuration
    config: BreakerConfig,
    /// Whether the system is initialized
    initialized: bool,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::new())),
            config: BreakerConfig::default(),
            initialized: false,
        }
    }
    
    /// Create a new circuit breaker with configuration
    pub fn with_config(config: BreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::new())),
            config,
            initialized: false,
        }
    }
    
    /// Initialize the circuit breaker
    pub async fn initialize(&mut self) -> Result<(), BreakerError> {
        let mut state = self.state.write().await;
        state.reset();
        self.initialized = true;
        Ok(())
    }
    
    /// Shutdown the circuit breaker
    pub async fn shutdown(&mut self) -> Result<(), BreakerError> {
        self.initialized = false;
        Ok(())
    }
    
    /// Check if the system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Check if the circuit breaker allows execution
    pub async fn can_execute(&self) -> Result<bool, BreakerError> {
        if !self.initialized {
            return Err(BreakerError::NotInitialized);
        }
        
        let state = self.state.read().await;
        Ok(state.can_execute())
    }
    
    /// Execute an operation through the circuit breaker
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, BreakerError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        if !self.initialized {
            return Err(BreakerError::NotInitialized);
        }
        
        // Check if we can execute
        if !self.can_execute().await? {
            return Err(BreakerError::CircuitOpen);
        }
        
        // Execute the operation
        let result = operation().await;
        
        // Update circuit breaker state based on result
        let mut state = self.state.write().await;
        match result {
            Ok(_) => {
                state.record_success();
                Ok(result.unwrap())
            }
            Err(_) => {
                state.record_failure(self.config.failure_threshold);
                Err(BreakerError::OperationFailed)
            }
        }
    }
    
    /// Record a successful operation
    pub async fn record_success(&self) -> Result<(), BreakerError> {
        if !self.initialized {
            return Err(BreakerError::NotInitialized);
        }
        
        let mut state = self.state.write().await;
        state.record_success();
        Ok(())
    }
    
    /// Record a failed operation
    pub async fn record_failure(&self) -> Result<(), BreakerError> {
        if !self.initialized {
            return Err(BreakerError::NotInitialized);
        }
        
        let mut state = self.state.write().await;
        state.record_failure(self.config.failure_threshold);
        Ok(())
    }
    
    /// Get the current circuit breaker state
    pub async fn get_state(&self) -> CircuitState {
        let state = self.state.read().await;
        state.get_state()
    }
    
    /// Get circuit breaker status
    pub async fn get_status(&self) -> Result<CircuitBreakerStatus, BreakerError> {
        if !self.initialized {
            return Err(BreakerError::NotInitialized);
        }
        
        let state = self.state.read().await;
        Ok(state.get_status())
    }
    
    /// Reset the circuit breaker
    pub async fn reset(&self) -> Result<(), BreakerError> {
        if !self.initialized {
            return Err(BreakerError::NotInitialized);
        }
        
        let mut state = self.state.write().await;
        state.reset();
        Ok(())
    }
}

/// Circuit breaker state
#[derive(Debug, Clone)]
struct CircuitBreakerState {
    /// Current state
    state: CircuitState,
    /// Number of consecutive failures
    failure_count: usize,
    /// Number of consecutive successes
    success_count: usize,
    /// Last failure time
    last_failure_time: Option<Instant>,
    /// Last success time
    last_success_time: Option<Instant>,
}

impl CircuitBreakerState {
    /// Create a new circuit breaker state
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            last_success_time: None,
        }
    }
    
    /// Check if the circuit breaker allows execution
    fn can_execute(&self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    last_failure.elapsed() >= Duration::from_secs(60) // 1 minute timeout
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
    
    /// Record a successful operation
    fn record_success(&mut self) {
        self.success_count += 1;
        self.failure_count = 0;
        self.last_success_time = Some(Instant::now());
        
        if self.state == CircuitState::HalfOpen && self.success_count >= 3 {
            self.state = CircuitState::Closed;
            self.success_count = 0;
        }
    }
    
    /// Record a failed operation
    fn record_failure(&mut self, failure_threshold: usize) {
        self.failure_count += 1;
        self.success_count = 0;
        self.last_failure_time = Some(Instant::now());
        
        if self.failure_count >= failure_threshold {
            self.state = CircuitState::Open;
        } else if self.state == CircuitState::HalfOpen {
            self.state = CircuitState::Open;
        }
    }
    
    /// Get the current state
    fn get_state(&self) -> CircuitState {
        self.state.clone()
    }
    
    /// Get circuit breaker status
    fn get_status(&self) -> CircuitBreakerStatus {
        CircuitBreakerStatus {
            state: self.state.clone(),
            failure_count: self.failure_count,
            success_count: self.success_count,
            last_failure_time: self.last_failure_time,
            last_success_time: self.last_success_time,
        }
    }
    
    /// Reset the circuit breaker state
    fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
        self.last_success_time = None;
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitState {
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
    pub state: CircuitState,
    /// Number of consecutive failures
    pub failure_count: usize,
    /// Number of consecutive successes
    pub success_count: usize,
    /// Last failure time (serialized as timestamp)
    #[serde(skip_serializing, skip_deserializing)]
    pub last_failure_time: Option<Instant>,
    /// Last success time (serialized as timestamp)
    #[serde(skip_serializing, skip_deserializing)]
    pub last_success_time: Option<Instant>,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: usize,
    /// Success threshold to close circuit
    pub success_threshold: usize,
    /// Timeout for half-open state
    pub timeout: Duration,
    /// Enable circuit breaker
    pub enabled: bool,
}

impl Default for BreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            enabled: true,
        }
    }
}

/// Circuit breaker errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BreakerError {
    /// System not initialized
    NotInitialized,
    /// Circuit breaker is open
    CircuitOpen,
    /// Operation failed
    OperationFailed,
    /// Configuration error
    ConfigurationError(String),
}

impl std::fmt::Display for BreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakerError::NotInitialized => write!(f, "Circuit breaker not initialized"),
            BreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            BreakerError::OperationFailed => write!(f, "Operation failed"),
            BreakerError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for BreakerError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_circuit_breaker_creation() {
        let breaker = CircuitBreaker::new();
        assert!(!breaker.is_initialized());
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_initialization() {
        let mut breaker = CircuitBreaker::new();
        let result = breaker.initialize().await;
        assert!(result.is_ok());
        assert!(breaker.is_initialized());
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_shutdown() {
        let mut breaker = CircuitBreaker::new();
        breaker.initialize().await.unwrap();
        let result = breaker.shutdown().await;
        assert!(result.is_ok());
        assert!(!breaker.is_initialized());
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_can_execute() {
        let mut breaker = CircuitBreaker::new();
        breaker.initialize().await.unwrap();
        
        let can_execute = breaker.can_execute().await.unwrap();
        assert!(can_execute);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let mut breaker = CircuitBreaker::new();
        breaker.initialize().await.unwrap();
        
        let result = breaker.execute(|| {
            Box::pin(async { Ok::<i32, std::io::Error>(42) })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        
        let status = breaker.get_status().await.unwrap();
        assert_eq!(status.state, CircuitState::Closed);
        assert_eq!(status.failure_count, 0);
        assert_eq!(status.success_count, 1);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let mut breaker = CircuitBreaker::new();
        breaker.initialize().await.unwrap();
        
        let result = breaker.execute(|| {
            Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::Other, "Operation failed")) })
        }).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BreakerError::OperationFailed));
        
        let status = breaker.get_status().await.unwrap();
        assert_eq!(status.state, CircuitState::Closed);
        assert_eq!(status.failure_count, 1);
        assert_eq!(status.success_count, 0);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let mut breaker = CircuitBreaker::new();
        breaker.initialize().await.unwrap();
        
        // Record multiple failures to open the circuit
        for _ in 0..5 {
            let _ = breaker.execute(|| {
                Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::Other, "Operation failed")) })
            }).await;
        }
        
        let status = breaker.get_status().await.unwrap();
        assert_eq!(status.state, CircuitState::Open);
        assert_eq!(status.failure_count, 5);
        
        // Next operation should fail due to circuit being open
        let result = breaker.execute(|| {
            Box::pin(async { Ok::<i32, std::io::Error>(42) })
        }).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BreakerError::CircuitOpen));
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let mut breaker = CircuitBreaker::new();
        breaker.initialize().await.unwrap();
        
        // Open the circuit
        for _ in 0..5 {
            let _ = breaker.execute(|| {
                Box::pin(async { Err::<i32, std::io::Error>(std::io::Error::new(std::io::ErrorKind::Other, "Operation failed")) })
            }).await;
        }
        
        let status = breaker.get_status().await.unwrap();
        assert_eq!(status.state, CircuitState::Open);
        
        // Reset the circuit
        breaker.reset().await.unwrap();
        
        let status = breaker.get_status().await.unwrap();
        assert_eq!(status.state, CircuitState::Closed);
        assert_eq!(status.failure_count, 0);
        assert_eq!(status.success_count, 0);
        
        // Should be able to execute again
        let can_execute = breaker.can_execute().await.unwrap();
        assert!(can_execute);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_record_success() {
        let mut breaker = CircuitBreaker::new();
        breaker.initialize().await.unwrap();
        
        breaker.record_success().await.unwrap();
        
        let status = breaker.get_status().await.unwrap();
        assert_eq!(status.success_count, 1);
        assert_eq!(status.failure_count, 0);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_record_failure() {
        let mut breaker = CircuitBreaker::new();
        breaker.initialize().await.unwrap();
        
        breaker.record_failure().await.unwrap();
        
        let status = breaker.get_status().await.unwrap();
        assert_eq!(status.failure_count, 1);
        assert_eq!(status.success_count, 0);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_with_config() {
        let config = BreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            enabled: true,
        };
        
        let mut breaker = CircuitBreaker::with_config(config);
        breaker.initialize().await.unwrap();
        
        // Should open after 3 failures
        for _ in 0..3 {
            breaker.record_failure().await.unwrap();
        }
        
        let status = breaker.get_status().await.unwrap();
        assert_eq!(status.state, CircuitState::Open);
        assert_eq!(status.failure_count, 3);
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_not_initialized() {
        let breaker = CircuitBreaker::new();
        
        let result = breaker.can_execute().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BreakerError::NotInitialized));
        
        let result = breaker.execute(|| {
            Box::pin(async { Ok::<i32, std::io::Error>(42) })
        }).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BreakerError::NotInitialized));
    }
    
    #[test]
    fn test_breaker_config_default() {
        let config = BreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(config.enabled);
    }
    
    #[test]
    fn test_circuit_state() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_eq!(CircuitState::Open, CircuitState::Open);
        assert_eq!(CircuitState::HalfOpen, CircuitState::HalfOpen);
    }
    
    #[test]
    fn test_breaker_error_display() {
        let error = BreakerError::CircuitOpen;
        let error_string = format!("{}", error);
        assert!(error_string.contains("Circuit breaker is open"));
    }
}
