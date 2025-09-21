//! Circuit breaker pattern for fault tolerance

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, limited requests are allowed
    HalfOpen,
}

/// Circuit breaker status information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    /// Current state
    pub state: CircuitBreakerState,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Number of consecutive successes
    pub success_count: u32,
    /// Last failure time (as Unix timestamp)
    #[serde(skip)]
    pub last_failure_time: Option<Instant>,
    /// Next attempt time (as Unix timestamp)
    #[serde(skip)]
    pub next_attempt_time: Option<Instant>,
}

/// Circuit breaker policy configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBreakerPolicy {
    /// Failure threshold to open the circuit
    pub failure_threshold: u32,
    /// Success threshold to close the circuit (from half-open)
    pub success_threshold: u32,
    /// Timeout before attempting to close the circuit
    pub timeout: Duration,
    /// Whether the circuit breaker is enabled
    pub enabled: bool,
}

impl CircuitBreakerPolicy {
    /// Create a new circuit breaker policy
    pub fn new() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            enabled: true,
        }
    }
    
    /// Create a new circuit breaker policy with custom configuration
    pub fn with_config(
        failure_threshold: u32,
        success_threshold: u32,
        timeout: Duration,
        enabled: bool,
    ) -> Self {
        Self {
            failure_threshold,
            success_threshold,
            timeout,
            enabled,
        }
    }
    
    /// Check if the circuit breaker should allow the request
    pub fn should_allow_request(&self, status: &CircuitBreakerStatus) -> bool {
        if !self.enabled {
            return true;
        }
        
        match status.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if timeout has passed
                if let Some(next_attempt) = status.next_attempt_time {
                    Instant::now() >= next_attempt
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Allow limited requests in half-open state
                status.success_count < self.success_threshold
            }
        }
    }
    
    /// Update circuit breaker state based on operation result
    pub fn update_state(&self, status: &mut CircuitBreakerStatus, success: bool) {
        if !self.enabled {
            return;
        }
        
        match status.state {
            CircuitBreakerState::Closed => {
                if success {
                    status.failure_count = 0;
                } else {
                    status.failure_count += 1;
                    status.last_failure_time = Some(Instant::now());
                    
                    if status.failure_count >= self.failure_threshold {
                        status.state = CircuitBreakerState::Open;
                        status.next_attempt_time = Some(Instant::now() + self.timeout);
                    }
                }
            }
            CircuitBreakerState::Open => {
                // In open state, we only transition to half-open when timeout passes
                if let Some(next_attempt) = status.next_attempt_time {
                    if Instant::now() >= next_attempt {
                        status.state = CircuitBreakerState::HalfOpen;
                        status.failure_count = 0;
                        status.success_count = 0;
                        status.next_attempt_time = None;
                    }
                }
            }
            CircuitBreakerState::HalfOpen => {
                if success {
                    status.success_count += 1;
                    if status.success_count >= self.success_threshold {
                        status.state = CircuitBreakerState::Closed;
                        status.failure_count = 0;
                        status.success_count = 0;
                    }
                } else {
                    // Any failure in half-open state opens the circuit
                    status.state = CircuitBreakerState::Open;
                    status.failure_count = self.failure_threshold;
                    status.success_count = 0;
                    status.last_failure_time = Some(Instant::now());
                    status.next_attempt_time = Some(Instant::now() + self.timeout);
                }
            }
        }
    }
    
    /// Get the current circuit breaker status
    pub fn get_status(&self) -> CircuitBreakerStatus {
        CircuitBreakerStatus {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            next_attempt_time: None,
        }
    }
    
    /// Reset the circuit breaker to closed state
    pub fn reset(&self, status: &mut CircuitBreakerStatus) {
        status.state = CircuitBreakerState::Closed;
        status.failure_count = 0;
        status.success_count = 0;
        status.last_failure_time = None;
        status.next_attempt_time = None;
    }
    
    /// Check if the circuit breaker is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Get failure threshold
    pub fn failure_threshold(&self) -> u32 {
        self.failure_threshold
    }
    
    /// Get success threshold
    pub fn success_threshold(&self) -> u32 {
        self.success_threshold
    }
    
    /// Get timeout duration
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl Default for CircuitBreakerPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_circuit_breaker_policy_creation() {
        let policy = CircuitBreakerPolicy::new();
        assert_eq!(policy.failure_threshold(), 5);
        assert_eq!(policy.success_threshold(), 3);
        assert_eq!(policy.timeout(), Duration::from_secs(60));
        assert!(policy.is_enabled());
    }
    
    #[test]
    fn test_circuit_breaker_policy_with_config() {
        let policy = CircuitBreakerPolicy::with_config(10, 5, Duration::from_secs(30), false);
        assert_eq!(policy.failure_threshold(), 10);
        assert_eq!(policy.success_threshold(), 5);
        assert_eq!(policy.timeout(), Duration::from_secs(30));
        assert!(!policy.is_enabled());
    }
    
    #[test]
    fn test_should_allow_request_closed() {
        let policy = CircuitBreakerPolicy::new();
        let status = CircuitBreakerStatus {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            next_attempt_time: None,
        };
        
        assert!(policy.should_allow_request(&status));
    }
    
    #[test]
    fn test_should_allow_request_open() {
        let policy = CircuitBreakerPolicy::new();
        let status = CircuitBreakerStatus {
            state: CircuitBreakerState::Open,
            failure_count: 5,
            success_count: 0,
            last_failure_time: Some(Instant::now()),
            next_attempt_time: Some(Instant::now() + Duration::from_secs(60)),
        };
        
        assert!(!policy.should_allow_request(&status));
    }
    
    #[test]
    fn test_should_allow_request_half_open() {
        let policy = CircuitBreakerPolicy::new();
        let status = CircuitBreakerStatus {
            state: CircuitBreakerState::HalfOpen,
            failure_count: 0,
            success_count: 1,
            last_failure_time: None,
            next_attempt_time: None,
        };
        
        assert!(policy.should_allow_request(&status));
    }
    
    #[test]
    fn test_update_state_closed_success() {
        let policy = CircuitBreakerPolicy::new();
        let mut status = policy.get_status();
        status.failure_count = 2;
        
        policy.update_state(&mut status, true);
        
        assert_eq!(status.failure_count, 0);
        assert_eq!(status.state, CircuitBreakerState::Closed);
    }
    
    #[test]
    fn test_update_state_closed_failure() {
        let policy = CircuitBreakerPolicy::with_config(3, 2, Duration::from_secs(1), true);
        let mut status = policy.get_status();
        status.failure_count = 2;
        
        policy.update_state(&mut status, false);
        
        assert_eq!(status.failure_count, 3);
        assert_eq!(status.state, CircuitBreakerState::Open);
        assert!(status.next_attempt_time.is_some());
    }
    
    #[test]
    fn test_update_state_half_open_success() {
        let policy = CircuitBreakerPolicy::with_config(3, 2, Duration::from_secs(1), true);
        let mut status = CircuitBreakerStatus {
            state: CircuitBreakerState::HalfOpen,
            failure_count: 0,
            success_count: 1,
            last_failure_time: None,
            next_attempt_time: None,
        };
        
        policy.update_state(&mut status, true);
        
        assert_eq!(status.success_count, 2);
        assert_eq!(status.state, CircuitBreakerState::Closed);
    }
    
    #[test]
    fn test_update_state_half_open_failure() {
        let policy = CircuitBreakerPolicy::new();
        let mut status = CircuitBreakerStatus {
            state: CircuitBreakerState::HalfOpen,
            failure_count: 0,
            success_count: 1,
            last_failure_time: None,
            next_attempt_time: None,
        };
        
        policy.update_state(&mut status, false);
        
        assert_eq!(status.state, CircuitBreakerState::Open);
        assert_eq!(status.failure_count, 5);
        assert!(status.next_attempt_time.is_some());
    }
    
    #[test]
    fn test_reset() {
        let policy = CircuitBreakerPolicy::new();
        let mut status = CircuitBreakerStatus {
            state: CircuitBreakerState::Open,
            failure_count: 5,
            success_count: 0,
            last_failure_time: Some(Instant::now()),
            next_attempt_time: Some(Instant::now() + Duration::from_secs(60)),
        };
        
        policy.reset(&mut status);
        
        assert_eq!(status.state, CircuitBreakerState::Closed);
        assert_eq!(status.failure_count, 0);
        assert_eq!(status.success_count, 0);
        assert!(status.last_failure_time.is_none());
        assert!(status.next_attempt_time.is_none());
    }
    
    #[test]
    fn test_disabled_circuit_breaker() {
        let policy = CircuitBreakerPolicy::with_config(1, 1, Duration::from_secs(1), false);
        let status = CircuitBreakerStatus {
            state: CircuitBreakerState::Open,
            failure_count: 1,
            success_count: 0,
            last_failure_time: Some(Instant::now()),
            next_attempt_time: Some(Instant::now() + Duration::from_secs(60)),
        };
        
        assert!(policy.should_allow_request(&status));
        
        let mut status = status;
        policy.update_state(&mut status, false);
        assert_eq!(status.state, CircuitBreakerState::Open); // Should not change when disabled
    }
}
