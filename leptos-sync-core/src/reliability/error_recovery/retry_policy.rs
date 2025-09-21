//! Retry policy configuration and strategies

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Retry strategy types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// Exponential backoff with jitter
    ExponentialBackoff,
    /// Linear backoff
    LinearBackoff,
    /// Fixed delay between retries
    FixedDelay,
    /// No retries
    NoRetry,
}

/// Configuration for exponential backoff retry strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExponentialBackoffConfig {
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Multiplier for each retry
    pub multiplier: f64,
    /// Whether to add jitter
    pub jitter: bool,
}

impl Default for ExponentialBackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Configuration for linear backoff retry strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearBackoffConfig {
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Increment per retry in milliseconds
    pub increment_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
}

impl Default for LinearBackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            increment_ms: 1000,
            max_delay_ms: 10000,
        }
    }
}

/// Configuration for fixed delay retry strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixedDelayConfig {
    /// Fixed delay in milliseconds
    pub delay_ms: u64,
}

impl Default for FixedDelayConfig {
    fn default() -> Self {
        Self {
            delay_ms: 1000,
        }
    }
}

/// Retry policy configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Retry strategy
    pub strategy: RetryStrategy,
    /// Exponential backoff configuration
    pub exponential_config: ExponentialBackoffConfig,
    /// Linear backoff configuration
    pub linear_config: LinearBackoffConfig,
    /// Fixed delay configuration
    pub fixed_config: FixedDelayConfig,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            strategy: RetryStrategy::ExponentialBackoff,
            exponential_config: ExponentialBackoffConfig::default(),
            linear_config: LinearBackoffConfig::default(),
            fixed_config: FixedDelayConfig::default(),
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_retries: usize, strategy: RetryStrategy) -> Self {
        Self {
            max_retries,
            strategy,
            exponential_config: ExponentialBackoffConfig::default(),
            linear_config: LinearBackoffConfig::default(),
            fixed_config: FixedDelayConfig::default(),
        }
    }
    
    /// Calculate delay for a given retry attempt
    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }
        
        match self.strategy {
            RetryStrategy::ExponentialBackoff => {
                let delay_ms = (self.exponential_config.initial_delay_ms as f64
                    * self.exponential_config.multiplier.powi(attempt as i32 - 1))
                    .min(self.exponential_config.max_delay_ms as f64) as u64;
                
                let mut delay = Duration::from_millis(delay_ms);
                
                if self.exponential_config.jitter {
                    // Add jitter: ±25% of the delay
                    let jitter_range = delay_ms / 4;
                    let jitter = (rand::random::<u64>() % (jitter_range * 2)) as i64 - jitter_range as i64;
                    let adjusted_ms = (delay_ms as i64 + jitter).max(0) as u64;
                    delay = Duration::from_millis(adjusted_ms);
                }
                
                delay
            }
            RetryStrategy::LinearBackoff => {
                let delay_ms = (self.linear_config.initial_delay_ms
                    + (attempt - 1) as u64 * self.linear_config.increment_ms)
                    .min(self.linear_config.max_delay_ms);
                Duration::from_millis(delay_ms)
            }
            RetryStrategy::FixedDelay => {
                Duration::from_millis(self.fixed_config.delay_ms)
            }
            RetryStrategy::NoRetry => {
                Duration::from_millis(0)
            }
        }
    }
    
    /// Check if retry should be attempted
    pub fn should_retry(&self, attempt: usize) -> bool {
        attempt < self.max_retries && self.strategy != RetryStrategy::NoRetry
    }
    
    /// Get the maximum number of retries
    pub fn max_retries(&self) -> usize {
        self.max_retries
    }
    
    /// Get the retry strategy
    pub fn strategy(&self) -> &RetryStrategy {
        &self.strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_policy_creation() {
        let policy = RetryPolicy::new(5, RetryStrategy::ExponentialBackoff);
        assert_eq!(policy.max_retries(), 5);
        assert_eq!(policy.strategy(), &RetryStrategy::ExponentialBackoff);
    }
    
    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries(), 3);
        assert_eq!(policy.strategy(), &RetryStrategy::ExponentialBackoff);
    }
    
    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy::new(3, RetryStrategy::ExponentialBackoff);
        
        assert!(policy.should_retry(0));
        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(!policy.should_retry(3));
        assert!(!policy.should_retry(4));
    }
    
    #[test]
    fn test_should_retry_no_retry_strategy() {
        let policy = RetryPolicy::new(3, RetryStrategy::NoRetry);
        
        assert!(!policy.should_retry(0));
        assert!(!policy.should_retry(1));
        assert!(!policy.should_retry(2));
    }
    
    #[test]
    fn test_calculate_delay_exponential() {
        let policy = RetryPolicy::new(5, RetryStrategy::ExponentialBackoff);
        
        let delay0 = policy.calculate_delay(0);
        assert_eq!(delay0, Duration::from_millis(0));
        
        let delay1 = policy.calculate_delay(1);
        assert_eq!(delay1, Duration::from_millis(1000)); // initial_delay
        
        let delay2 = policy.calculate_delay(2);
        assert_eq!(delay2, Duration::from_millis(2000)); // initial_delay * multiplier
        
        let delay3 = policy.calculate_delay(3);
        assert_eq!(delay3, Duration::from_millis(4000)); // initial_delay * multiplier^2
    }
    
    #[test]
    fn test_calculate_delay_linear() {
        let policy = RetryPolicy::new(5, RetryStrategy::LinearBackoff);
        
        let delay0 = policy.calculate_delay(0);
        assert_eq!(delay0, Duration::from_millis(0));
        
        let delay1 = policy.calculate_delay(1);
        assert_eq!(delay1, Duration::from_millis(1000)); // initial_delay
        
        let delay2 = policy.calculate_delay(2);
        assert_eq!(delay2, Duration::from_millis(2000)); // initial_delay + increment
        
        let delay3 = policy.calculate_delay(3);
        assert_eq!(delay3, Duration::from_millis(3000)); // initial_delay + 2*increment
    }
    
    #[test]
    fn test_calculate_delay_fixed() {
        let policy = RetryPolicy::new(5, RetryStrategy::FixedDelay);
        
        let delay0 = policy.calculate_delay(0);
        assert_eq!(delay0, Duration::from_millis(0));
        
        let delay1 = policy.calculate_delay(1);
        assert_eq!(delay1, Duration::from_millis(1000)); // fixed_delay
        
        let delay2 = policy.calculate_delay(2);
        assert_eq!(delay2, Duration::from_millis(1000)); // fixed_delay
        
        let delay3 = policy.calculate_delay(3);
        assert_eq!(delay3, Duration::from_millis(1000)); // fixed_delay
    }
    
    #[test]
    fn test_calculate_delay_no_retry() {
        let policy = RetryPolicy::new(5, RetryStrategy::NoRetry);
        
        let delay0 = policy.calculate_delay(0);
        assert_eq!(delay0, Duration::from_millis(0));
        
        let delay1 = policy.calculate_delay(1);
        assert_eq!(delay1, Duration::from_millis(0));
        
        let delay2 = policy.calculate_delay(2);
        assert_eq!(delay2, Duration::from_millis(0));
    }
    
    #[test]
    fn test_max_delay_limit() {
        let mut policy = RetryPolicy::new(10, RetryStrategy::ExponentialBackoff);
        policy.exponential_config.max_delay_ms = 5000;
        
        // This should be capped at max_delay_ms
        let delay = policy.calculate_delay(10);
        assert!(delay <= Duration::from_millis(5000));
    }
}
