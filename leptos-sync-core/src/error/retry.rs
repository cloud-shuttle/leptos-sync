//! Comprehensive error handling and retry logic

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum RetryError {
    #[error("Max retries exceeded: {0}")]
    MaxRetriesExceeded(usize),
    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),
    #[error("Operation timeout: {0}")]
    Timeout(String),
    #[error("Fatal error: {0}")]
    Fatal(String),
}

/// Retry strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// No retries
    None,
    /// Fixed delay between retries
    Fixed { delay: Duration, max_attempts: usize },
    /// Exponential backoff
    Exponential {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        max_attempts: usize,
    },
    /// Fibonacci backoff
    Fibonacci {
        initial_delay: Duration,
        max_delay: Duration,
        max_attempts: usize,
    },
    /// Custom retry logic
    Custom { max_attempts: usize, custom_logic: String },
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::Exponential {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_attempts: 5,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Failing, reject requests
    HalfOpen,  // Testing if service is recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,
    pub recovery_timeout: Duration,
    pub expected_duration: Duration,
    pub min_calls: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            expected_duration: Duration::from_secs(1),
            min_calls: 3,
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
    failure_count: usize,
    success_count: usize,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            last_state_change: Instant::now(),
        }
    }

    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if self.should_attempt_reset() {
                    self.transition_to_half_open();
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub fn on_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.transition_to_closed();
            }
            CircuitBreakerState::Open => {}
        }
    }

    pub fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.transition_to_open();
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.transition_to_open();
            }
            CircuitBreakerState::Open => {}
        }
    }

    fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = self.last_failure_time {
            last_failure.elapsed() >= self.config.recovery_timeout
        } else {
            false
        }
    }

    fn transition_to_open(&mut self) {
        self.state = CircuitBreakerState::Open;
        self.last_state_change = Instant::now();
    }

    fn transition_to_half_open(&mut self) {
        self.state = CircuitBreakerState::HalfOpen;
        self.last_state_change = Instant::now();
        self.failure_count = 0;
        self.success_count = 0;
    }

    fn transition_to_closed(&mut self) {
        self.state = CircuitBreakerState::Closed;
        self.last_state_change = Instant::now();
        self.failure_count = 0;
        self.success_count = 0;
    }

    pub fn state(&self) -> &CircuitBreakerState {
        &self.state
    }

    pub fn failure_count(&self) -> usize {
        self.failure_count
    }

    pub fn success_count(&self) -> usize {
        self.success_count
    }
}

/// Retry manager with circuit breaker
pub struct RetryManager {
    strategy: RetryStrategy,
    circuit_breaker: CircuitBreaker,
    operation_history: HashMap<String, OperationStats>,
}

/// Operation statistics
#[derive(Debug, Clone)]
pub struct OperationStats {
    pub total_attempts: usize,
    pub successful_attempts: usize,
    pub failed_attempts: usize,
    pub total_duration: Duration,
    pub last_attempt: Option<Instant>,
}

impl RetryManager {
    pub fn new(strategy: RetryStrategy, circuit_breaker_config: CircuitBreakerConfig) -> Self {
        Self {
            strategy,
            circuit_breaker: CircuitBreaker::new(circuit_breaker_config),
            operation_history: HashMap::new(),
        }
    }

    /// Execute an operation with retry logic
    pub async fn execute_with_retry<F, T, E>(
        &mut self,
        operation_name: &str,
        operation: F,
    ) -> Result<T, RetryError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        let mut attempt = 0;

        // Check circuit breaker
        if !self.circuit_breaker.can_execute() {
            return Err(RetryError::CircuitBreakerOpen(
                format!("Circuit breaker is {:?}", self.circuit_breaker.state())
            ));
        }

        loop {
            attempt += 1;
            let attempt_start = Instant::now();

            // Update operation history
            let stats = self.operation_history.entry(operation_name.to_string()).or_insert_with(|| OperationStats {
                total_attempts: 0,
                successful_attempts: 0,
                failed_attempts: 0,
                total_duration: Duration::ZERO,
                last_attempt: None,
            });

            stats.total_attempts += 1;
            stats.last_attempt = Some(Instant::now());

            // Execute operation
            let result = operation().await;

            match result {
                Ok(value) => {
                    // Success
                    self.circuit_breaker.on_success();
                    stats.successful_attempts += 1;
                    stats.total_duration += attempt_start.elapsed();
                    
                    return Ok(value);
                }
                Err(error) => {
                    // Failure
                    self.circuit_breaker.on_failure();
                    stats.failed_attempts += 1;
                    stats.total_duration += attempt_start.elapsed();

                    // Check if we should retry
                    if !self.should_retry(attempt, &error) {
                        return Err(RetryError::MaxRetriesExceeded(attempt));
                    }

                    // Calculate delay for next attempt
                    let delay = self.calculate_delay(attempt);
                    
                    // Wait before retry
                    sleep(delay).await;
                }
            }
        }
    }

    fn should_retry(&self, attempt: usize, error: &dyn std::error::Error) -> bool {
        let max_attempts = match &self.strategy {
            RetryStrategy::None => return false,
            RetryStrategy::Fixed { max_attempts, .. } => *max_attempts,
            RetryStrategy::Exponential { max_attempts, .. } => *max_attempts,
            RetryStrategy::Fibonacci { max_attempts, .. } => *max_attempts,
            RetryStrategy::Custom { max_attempts, .. } => *max_attempts,
        };

        attempt < max_attempts
    }

    fn calculate_delay(&self, attempt: usize) -> Duration {
        match &self.strategy {
            RetryStrategy::None => Duration::ZERO,
            RetryStrategy::Fixed { delay, .. } => *delay,
            RetryStrategy::Exponential { initial_delay, max_delay, multiplier, .. } => {
                let delay = initial_delay.as_millis() as f64 * multiplier.powi(attempt as i32 - 1);
                let delay_ms = delay.min(max_delay.as_millis() as f64) as u64;
                Duration::from_millis(delay_ms)
            }
            RetryStrategy::Fibonacci { initial_delay, max_delay, .. } => {
                            let delay = initial_delay.as_millis() as u64 * fibonacci(attempt);
            Duration::from_millis(delay.min(max_delay.as_millis() as u64))
            }
            RetryStrategy::Custom { .. } => Duration::from_millis(100), // Default delay
        }
    }

    /// Get operation statistics
    pub fn get_operation_stats(&self, operation_name: &str) -> Option<&OperationStats> {
        self.operation_history.get(operation_name)
    }

    /// Get all operation statistics
    pub fn get_all_stats(&self) -> &HashMap<String, OperationStats> {
        &self.operation_history
    }

    /// Reset circuit breaker
    pub fn reset_circuit_breaker(&mut self) {
        self.circuit_breaker = CircuitBreaker::new(self.circuit_breaker.config.clone());
    }

    /// Update retry strategy
    pub fn update_strategy(&mut self, strategy: RetryStrategy) {
        self.strategy = strategy;
    }
}

/// Calculate Fibonacci number
fn fibonacci(n: usize) -> u64 {
    if n <= 1 {
        1
    } else {
        let mut a = 1;
        let mut b = 1;
        for _ in 2..n {
            let temp = a + b;
            a = b;
            b = temp;
        }
        b
    }
}

/// Error classification for retry decisions
pub trait RetryableError {
    fn is_retryable(&self) -> bool;
    fn is_fatal(&self) -> bool;
    fn retry_after(&self) -> Option<Duration>;
}

/// Default error classification
impl<T: std::error::Error> RetryableError for T {
    fn is_retryable(&self) -> bool {
        // Default to retryable for most errors
        true
    }

    fn is_fatal(&self) -> bool {
        // Default to non-fatal
        false
    }

    fn retry_after(&self) -> Option<Duration> {
        // No specific retry delay
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let breaker = CircuitBreaker::new(config);
        assert!(matches!(breaker.state(), CircuitBreakerState::Closed));
    }

    #[tokio::test]
    async fn test_retry_manager_creation() {
        let strategy = RetryStrategy::default();
        let config = CircuitBreakerConfig::default();
        let manager = RetryManager::new(strategy, config);
        assert!(manager.operation_history.is_empty());
    }

    #[tokio::test]
    async fn test_fibonacci_calculation() {
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(2), 1);
        assert_eq!(fibonacci(3), 2);
        assert_eq!(fibonacci(4), 3);
        assert_eq!(fibonacci(5), 5);
    }

    #[tokio::test]
    async fn test_retry_strategy_default() {
        let strategy = RetryStrategy::default();
        match strategy {
            RetryStrategy::Exponential { initial_delay, max_delay, multiplier, max_attempts } => {
                assert_eq!(initial_delay, Duration::from_millis(100));
                assert_eq!(max_delay, Duration::from_secs(30));
                assert_eq!(multiplier, 2.0);
                assert_eq!(max_attempts, 5);
            }
            _ => panic!("Expected exponential strategy"),
        }
    }
}
