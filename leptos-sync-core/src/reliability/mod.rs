//! Production Reliability Module
//!
//! This module provides enterprise-grade reliability features including:
//! - Advanced error recovery with retry policies
//! - Data integrity verification and corruption detection
//! - Comprehensive monitoring and health checks
//! - Circuit breakers and fault tolerance
//! - Backup and restore capabilities

pub mod error_recovery;
pub mod data_integrity;
pub mod monitoring;
pub mod health_checks;
pub mod circuit_breaker;
pub mod backup_restore;

use std::time::{SystemTime, UNIX_EPOCH};

// Re-export main components
pub use error_recovery::{
    ErrorRecovery, RetryPolicy, ExponentialBackoffConfig, CircuitBreakerPolicy,
    RetryStrategy, RecoveryResult, RecoveryError
};
pub use data_integrity::{
    DataIntegrity, ChecksumVerifier, VersionVerifier, CorruptionDetector,
    IntegrityResult, IntegrityError, ChecksumAlgorithm
};
pub use monitoring::{
    ReliabilityMonitor, MetricsCollector, AlertManager, HealthReporter,
    MonitorConfig, AlertConfig
};
pub use health_checks::{
    HealthChecker, HealthStatus, HealthCheck, SystemHealth,
    HealthCheckResult, HealthError
};
pub use circuit_breaker::{
    CircuitBreaker, CircuitState, BreakerConfig, BreakerError
};
pub use backup_restore::{
    BackupManager, RestoreManager, BackupStrategy, RestoreStrategy,
    BackupResult, RestoreResult, BackupError
};

/// Main reliability manager that coordinates all reliability features
#[derive(Debug, Clone)]
pub struct ReliabilityManager {
    /// Error recovery system
    pub error_recovery: ErrorRecovery,
    /// Data integrity system
    pub data_integrity: DataIntegrity,
    /// Monitoring system
    pub monitoring: ReliabilityMonitor,
    /// Health checking system
    pub health_checks: HealthChecker,
    /// Circuit breaker system
    pub circuit_breaker: CircuitBreaker,
    /// Backup and restore system
    pub backup_restore: BackupManager,
}

impl ReliabilityManager {
    /// Create a new reliability manager with default configuration
    pub fn new() -> Self {
        Self {
            error_recovery: ErrorRecovery::new(),
            data_integrity: DataIntegrity::new(),
            monitoring: ReliabilityMonitor::new(),
            health_checks: HealthChecker::new(),
            circuit_breaker: CircuitBreaker::new(),
            backup_restore: BackupManager::new(),
        }
    }
    
    /// Create a reliability manager with custom configuration
    pub fn with_config(config: ReliabilityConfig) -> Self {
        Self {
            error_recovery: ErrorRecovery::with_config(config.error_recovery),
            data_integrity: DataIntegrity::with_config(config.data_integrity),
            monitoring: ReliabilityMonitor::with_config(config.monitoring),
            health_checks: HealthChecker::with_config(config.health_checks),
            circuit_breaker: CircuitBreaker::with_config(config.circuit_breaker),
            backup_restore: BackupManager::with_config(config.backup_restore),
        }
    }
    
    /// Initialize all reliability systems
    pub async fn initialize(&mut self) -> Result<(), ReliabilityError> {
        self.error_recovery.initialize().await?;
        self.data_integrity.initialize().await?;
        self.monitoring.initialize().await?;
        self.health_checks.initialize().await?;
        self.circuit_breaker.initialize().await?;
        self.backup_restore.initialize().await?;
        
        Ok(())
    }
    
    /// Shutdown all reliability systems
    pub async fn shutdown(&mut self) -> Result<(), ReliabilityError> {
        self.error_recovery.shutdown().await?;
        self.data_integrity.shutdown().await?;
        self.monitoring.shutdown().await?;
        self.health_checks.shutdown().await?;
        self.circuit_breaker.shutdown().await?;
        self.backup_restore.shutdown().await?;
        
        Ok(())
    }
    
    /// Get overall system health
    pub async fn get_system_health(&self) -> Result<SystemHealth, ReliabilityError> {
        let health_checks = self.health_checks.check_all().await?;
        let circuit_breaker_status = self.circuit_breaker.get_state();
        let monitoring_status = self.monitoring.get_status().await?;
        
        Ok(SystemHealth {
            overall_status: self.determine_overall_status(&health_checks, circuit_breaker_status.await),
            health_checks,
            last_checked: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        })
    }
    
    /// Determine overall system status based on component health
    fn determine_overall_status(
        &self,
        health_checks: &[HealthCheckResult],
        circuit_breaker_status: CircuitState,
    ) -> HealthStatus {
        // Check if any health checks are failing
        let failing_checks = health_checks.iter().any(|check| !check.is_healthy);
        
        // Check circuit breaker status
        let circuit_breaker_healthy = matches!(circuit_breaker_status, CircuitState::Closed);
        
        if failing_checks || !circuit_breaker_healthy {
            HealthStatus::Unhealthy
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Configuration for the reliability manager
#[derive(Debug, Clone)]
pub struct ReliabilityConfig {
    /// Error recovery configuration
    pub error_recovery: error_recovery::RecoveryConfig,
    /// Data integrity configuration
    pub data_integrity: data_integrity::IntegrityConfig,
    /// Monitoring configuration
    pub monitoring: monitoring::MonitorConfig,
    /// Health checks configuration
    pub health_checks: health_checks::HealthConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: circuit_breaker::BreakerConfig,
    /// Backup and restore configuration
    pub backup_restore: backup_restore::BackupConfig,
}

impl Default for ReliabilityConfig {
    fn default() -> Self {
        Self {
            error_recovery: error_recovery::RecoveryConfig::default(),
            data_integrity: data_integrity::IntegrityConfig::default(),
            monitoring: monitoring::MonitorConfig::default(),
            health_checks: health_checks::HealthConfig::default(),
            circuit_breaker: circuit_breaker::BreakerConfig::default(),
            backup_restore: backup_restore::BackupConfig::default(),
        }
    }
}

/// Error types for reliability operations
#[derive(Debug, Clone, PartialEq)]
pub enum ReliabilityError {
    /// Error recovery failed
    ErrorRecovery(RecoveryError),
    /// Data integrity check failed
    DataIntegrity(IntegrityError),
    /// Monitoring system error
    Monitoring(String),
    /// Health check failed
    HealthCheck(HealthError),
    /// Circuit breaker error
    CircuitBreaker(BreakerError),
    /// Backup/restore error
    BackupRestore(BackupError),
    /// Initialization failed
    Initialization(String),
    /// Shutdown failed
    Shutdown(String),
}

impl std::fmt::Display for ReliabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReliabilityError::ErrorRecovery(e) => write!(f, "Error recovery failed: {}", e),
            ReliabilityError::DataIntegrity(e) => write!(f, "Data integrity failed: {}", e),
            ReliabilityError::Monitoring(e) => write!(f, "Monitoring error: {}", e),
            ReliabilityError::HealthCheck(e) => write!(f, "Health check failed: {}", e),
            ReliabilityError::CircuitBreaker(e) => write!(f, "Circuit breaker error: {}", e),
            ReliabilityError::BackupRestore(e) => write!(f, "Backup/restore error: {}", e),
            ReliabilityError::Initialization(e) => write!(f, "Initialization failed: {}", e),
            ReliabilityError::Shutdown(e) => write!(f, "Shutdown failed: {}", e),
        }
    }
}

impl std::error::Error for ReliabilityError {}

// From implementations for error conversion
impl From<RecoveryError> for ReliabilityError {
    fn from(e: RecoveryError) -> Self {
        ReliabilityError::ErrorRecovery(e)
    }
}

impl From<IntegrityError> for ReliabilityError {
    fn from(e: IntegrityError) -> Self {
        ReliabilityError::DataIntegrity(e)
    }
}

impl From<HealthError> for ReliabilityError {
    fn from(e: HealthError) -> Self {
        ReliabilityError::HealthCheck(e)
    }
}

impl From<BreakerError> for ReliabilityError {
    fn from(e: BreakerError) -> Self {
        ReliabilityError::CircuitBreaker(e)
    }
}

impl From<BackupError> for ReliabilityError {
    fn from(e: BackupError) -> Self {
        ReliabilityError::BackupRestore(e)
    }
}

impl From<String> for ReliabilityError {
    fn from(e: String) -> Self {
        ReliabilityError::Monitoring(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_reliability_manager_creation() {
        let mut manager = ReliabilityManager::new();
        
        // Verify all components are created
        assert!(manager.error_recovery.is_initialized());
        assert!(manager.data_integrity.is_initialized());
        assert!(manager.monitoring.is_initialized());
        assert!(manager.health_checks.is_initialized());
        assert!(manager.circuit_breaker.is_initialized());
        assert!(manager.backup_restore.is_initialized());
    }
    
    #[tokio::test]
    async fn test_reliability_manager_with_config() {
        let config = ReliabilityConfig::default();
        let mut manager = ReliabilityManager::with_config(config);
        
        // Verify all components are created with config
        assert!(manager.error_recovery.is_initialized());
        assert!(manager.data_integrity.is_initialized());
        assert!(manager.monitoring.is_initialized());
        assert!(manager.health_checks.is_initialized());
        assert!(manager.circuit_breaker.is_initialized());
        assert!(manager.backup_restore.is_initialized());
    }
    
    #[tokio::test]
    async fn test_reliability_manager_initialization() {
        let mut manager = ReliabilityManager::new();
        
        // Initialize all systems
        let result = manager.initialize().await;
        assert!(result.is_ok());
        
        // Verify all systems are initialized
        assert!(manager.error_recovery.is_initialized());
        assert!(manager.data_integrity.is_initialized());
        assert!(manager.monitoring.is_initialized());
        assert!(manager.health_checks.is_initialized());
        assert!(manager.circuit_breaker.is_initialized());
        assert!(manager.backup_restore.is_initialized());
    }
    
    #[tokio::test]
    async fn test_reliability_manager_shutdown() {
        let mut manager = ReliabilityManager::new();
        
        // Initialize first
        manager.initialize().await.unwrap();
        
        // Then shutdown
        let result = manager.shutdown().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_system_health_check() {
        let mut manager = ReliabilityManager::new();
        manager.initialize().await.unwrap();
        
        let health = manager.get_system_health().await.unwrap();
        
        // Should be healthy by default
        assert_eq!(health.overall_status, HealthStatus::Healthy);
        assert!(!health.health_checks.is_empty());
    }
    
    #[test]
    fn test_reliability_config_default() {
        let config = ReliabilityConfig::default();
        
        // Verify all configs are created
        assert!(config.error_recovery.retry_policy.max_retries > 0);
        assert!(config.data_integrity.checksum_config.algorithm == ChecksumAlgorithm::Sha256);
        assert!(config.monitoring.metrics_config.max_metrics_per_name > 0);
        assert!(config.health_checks.enable_health_checks);
        assert!(config.circuit_breaker.failure_threshold > 0);
        assert!(config.backup_restore.enable_backups);
    }
    
    #[test]
    fn test_reliability_error_display() {
        let error = ReliabilityError::Initialization("Test error".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Initialization failed"));
        assert!(error_string.contains("Test error"));
    }
    
    #[test]
    fn test_reliability_error_from_conversions() {
        // Test error conversion from RecoveryError
        let recovery_error = RecoveryError::MaxRetriesExceeded;
        let reliability_error: ReliabilityError = recovery_error.into();
        assert!(matches!(reliability_error, ReliabilityError::ErrorRecovery(_)));
        
        // Test error conversion from IntegrityError
        let integrity_error = IntegrityError::ChecksumMismatch;
        let reliability_error: ReliabilityError = integrity_error.into();
        assert!(matches!(reliability_error, ReliabilityError::DataIntegrity(_)));
        
        // Test error conversion from HealthError
        let health_error = HealthError::NotInitialized;
        let reliability_error: ReliabilityError = health_error.into();
        assert!(matches!(reliability_error, ReliabilityError::HealthCheck(_)));
        
        // Test error conversion from BreakerError
        let breaker_error = BreakerError::CircuitOpen;
        let reliability_error: ReliabilityError = breaker_error.into();
        assert!(matches!(reliability_error, ReliabilityError::CircuitBreaker(_)));
        
        // Test error conversion from BackupError
        let backup_error = BackupError::BackupFailed("test".to_string());
        let reliability_error: ReliabilityError = backup_error.into();
        assert!(matches!(reliability_error, ReliabilityError::BackupRestore(_)));
    }
}
