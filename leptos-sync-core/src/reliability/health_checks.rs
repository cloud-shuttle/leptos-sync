//! Health Checks System
//!
//! This module provides comprehensive health checking capabilities including:
//! - System health monitoring
//! - Component health verification
//! - Health status reporting
//! - Health check scheduling and execution

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Health checker for system and component health monitoring
#[derive(Debug, Clone)]
pub struct HealthChecker {
    /// Health checks
    health_checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    /// Health check results
    results: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    /// Health check configuration
    config: HealthConfig,
    /// Whether the system is initialized
    initialized: bool,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            config: HealthConfig::default(),
            initialized: false,
        }
    }
    
    /// Create a new health checker with configuration
    pub fn with_config(config: HealthConfig) -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            config,
            initialized: false,
        }
    }
    
    /// Initialize the health checker
    pub async fn initialize(&mut self) -> Result<(), HealthError> {
        // Initialize default health checks
        self.add_default_health_checks().await;
        
        self.initialized = true;
        Ok(())
    }
    
    /// Shutdown the health checker
    pub async fn shutdown(&mut self) -> Result<(), HealthError> {
        self.initialized = false;
        Ok(())
    }
    
    /// Check if the system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Add a health check
    pub async fn add_health_check(&self, name: String, health_check: HealthCheck) -> Result<(), HealthError> {
        if !self.initialized {
            return Err(HealthError::NotInitialized);
        }
        
        let mut health_checks = self.health_checks.write().await;
        health_checks.insert(name, health_check);
        
        Ok(())
    }
    
    /// Remove a health check
    pub async fn remove_health_check(&self, name: &str) -> Result<(), HealthError> {
        if !self.initialized {
            return Err(HealthError::NotInitialized);
        }
        
        let mut health_checks = self.health_checks.write().await;
        health_checks.remove(name);
        
        let mut results = self.results.write().await;
        results.remove(name);
        
        Ok(())
    }
    
    /// Run a specific health check
    pub async fn run_health_check(&self, name: &str) -> Result<HealthCheckResult, HealthError> {
        if !self.initialized {
            return Err(HealthError::NotInitialized);
        }
        
        let health_checks = self.health_checks.read().await;
        let health_check = health_checks.get(name)
            .ok_or_else(|| HealthError::HealthCheckNotFound(name.to_string()))?;
        
        let result = self.execute_health_check(health_check).await;
        
        // Store the result
        let mut results = self.results.write().await;
        results.insert(name.to_string(), result.clone());
        
        Ok(result)
    }
    
    /// Run all health checks
    pub async fn check_all(&self) -> Result<Vec<HealthCheckResult>, HealthError> {
        if !self.initialized {
            return Err(HealthError::NotInitialized);
        }
        
        let health_checks = self.health_checks.read().await;
        let mut results = Vec::new();
        
        for (name, health_check) in health_checks.iter() {
            let result = self.execute_health_check(health_check).await;
            results.push(result);
        }
        
        // Store all results
        let mut stored_results = self.results.write().await;
        for (name, health_check) in health_checks.iter() {
            let result = self.execute_health_check(health_check).await;
            stored_results.insert(name.clone(), result);
        }
        
        Ok(results)
    }
    
    /// Get health check result
    pub async fn get_health_check_result(&self, name: &str) -> Result<Option<HealthCheckResult>, HealthError> {
        if !self.initialized {
            return Err(HealthError::NotInitialized);
        }
        
        let results = self.results.read().await;
        Ok(results.get(name).cloned())
    }
    
    /// Get all health check results
    pub async fn get_all_results(&self) -> Result<HashMap<String, HealthCheckResult>, HealthError> {
        if !self.initialized {
            return Err(HealthError::NotInitialized);
        }
        
        let results = self.results.read().await;
        Ok(results.clone())
    }
    
    /// Get overall system health
    pub async fn get_system_health(&self) -> Result<SystemHealth, HealthError> {
        if !self.initialized {
            return Err(HealthError::NotInitialized);
        }
        
        let results = self.check_all().await?;
        let overall_status = if results.iter().all(|r| r.is_healthy) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        };
        
        Ok(SystemHealth {
            overall_status,
            health_checks: results,
            last_checked: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        })
    }
    
    /// Execute a health check
    async fn execute_health_check(&self, health_check: &HealthCheck) -> HealthCheckResult {
        let start_time = Instant::now();
        
        match health_check.check_type {
            HealthCheckType::Ping => self.ping_check(health_check).await,
            HealthCheckType::Http => self.http_check(health_check).await,
            HealthCheckType::Database => self.database_check(health_check).await,
            HealthCheckType::Custom => self.custom_check(health_check).await,
        }
    }
    
    /// Ping health check
    async fn ping_check(&self, health_check: &HealthCheck) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simulate ping check
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let duration = start_time.elapsed();
        let is_healthy = duration < Duration::from_millis(100);
        
        HealthCheckResult {
            name: health_check.name.clone(),
            is_healthy,
            message: if is_healthy {
                "Ping successful".to_string()
            } else {
                "Ping timeout".to_string()
            },
            duration,
            last_checked: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details: HashMap::new(),
        }
    }
    
    /// HTTP health check
    async fn http_check(&self, health_check: &HealthCheck) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simulate HTTP check
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let duration = start_time.elapsed();
        let is_healthy = duration < Duration::from_millis(500);
        
        let mut details = HashMap::new();
        details.insert("status_code".to_string(), "200".to_string());
        details.insert("response_time_ms".to_string(), duration.as_millis().to_string());
        
        HealthCheckResult {
            name: health_check.name.clone(),
            is_healthy,
            message: if is_healthy {
                "HTTP check successful".to_string()
            } else {
                "HTTP check failed".to_string()
            },
            duration,
            last_checked: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details,
        }
    }
    
    /// Database health check
    async fn database_check(&self, health_check: &HealthCheck) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simulate database check
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        let duration = start_time.elapsed();
        let is_healthy = duration < Duration::from_millis(200);
        
        let mut details = HashMap::new();
        details.insert("connection_pool_size".to_string(), "10".to_string());
        details.insert("active_connections".to_string(), "5".to_string());
        
        HealthCheckResult {
            name: health_check.name.clone(),
            is_healthy,
            message: if is_healthy {
                "Database check successful".to_string()
            } else {
                "Database check failed".to_string()
            },
            duration,
            last_checked: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details,
        }
    }
    
    /// Custom health check
    async fn custom_check(&self, health_check: &HealthCheck) -> HealthCheckResult {
        let start_time = Instant::now();
        
        // Simulate custom check
        tokio::time::sleep(Duration::from_millis(30)).await;
        
        let duration = start_time.elapsed();
        let is_healthy = duration < Duration::from_millis(300);
        
        HealthCheckResult {
            name: health_check.name.clone(),
            is_healthy,
            message: if is_healthy {
                "Custom check successful".to_string()
            } else {
                "Custom check failed".to_string()
            },
            duration,
            last_checked: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            details: HashMap::new(),
        }
    }
    
    /// Add default health checks
    async fn add_default_health_checks(&self) {
        let default_checks = vec![
            ("system_memory".to_string(), HealthCheck {
                name: "system_memory".to_string(),
                check_type: HealthCheckType::Custom,
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                enabled: true,
            }),
            ("system_cpu".to_string(), HealthCheck {
                name: "system_cpu".to_string(),
                check_type: HealthCheckType::Custom,
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                enabled: true,
            }),
            ("system_disk".to_string(), HealthCheck {
                name: "system_disk".to_string(),
                check_type: HealthCheckType::Custom,
                interval: Duration::from_secs(60),
                timeout: Duration::from_secs(10),
                enabled: true,
            }),
        ];
        
        let mut health_checks = self.health_checks.write().await;
        for (name, health_check) in default_checks {
            health_checks.insert(name, health_check);
        }
    }
}

/// Health check data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Health check name
    pub name: String,
    /// Health check type
    pub check_type: HealthCheckType,
    /// Check interval
    pub interval: Duration,
    /// Check timeout
    pub timeout: Duration,
    /// Whether the check is enabled
    pub enabled: bool,
}

/// Health check types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthCheckType {
    /// Ping check
    Ping,
    /// HTTP check
    Http,
    /// Database check
    Database,
    /// Custom check
    Custom,
}

/// Health check result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Health check name
    pub name: String,
    /// Whether the check is healthy
    pub is_healthy: bool,
    /// Health check message
    pub message: String,
    /// Check duration
    pub duration: Duration,
    /// Last checked timestamp
    pub last_checked: u64,
    /// Additional details
    pub details: HashMap<String, String>,
}

/// System health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall health status
    pub overall_status: HealthStatus,
    /// Individual health check results
    pub health_checks: Vec<HealthCheckResult>,
    /// Last checked timestamp
    pub last_checked: u64,
}

/// Health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is unhealthy
    Unhealthy,
}

/// Health configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health checks
    pub health_checks: Vec<HealthCheck>,
    /// Default check interval
    pub default_interval: Duration,
    /// Default check timeout
    pub default_timeout: Duration,
    /// Enable health checks
    pub enable_health_checks: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            health_checks: Vec::new(),
            default_interval: Duration::from_secs(30),
            default_timeout: Duration::from_secs(5),
            enable_health_checks: true,
        }
    }
}

/// Health errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthError {
    /// System not initialized
    NotInitialized,
    /// Health check not found
    HealthCheckNotFound(String),
    /// Health check execution failed
    HealthCheckExecutionFailed(String),
    /// Health check timeout
    HealthCheckTimeout,
    /// Configuration error
    ConfigurationError(String),
}

impl std::fmt::Display for HealthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthError::NotInitialized => write!(f, "Health checker not initialized"),
            HealthError::HealthCheckNotFound(name) => write!(f, "Health check not found: {}", name),
            HealthError::HealthCheckExecutionFailed(msg) => write!(f, "Health check execution failed: {}", msg),
            HealthError::HealthCheckTimeout => write!(f, "Health check timeout"),
            HealthError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for HealthError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        assert!(!checker.is_initialized());
    }
    
    #[tokio::test]
    async fn test_health_checker_initialization() {
        let mut checker = HealthChecker::new();
        let result = checker.initialize().await;
        assert!(result.is_ok());
        assert!(checker.is_initialized());
    }
    
    #[tokio::test]
    async fn test_health_checker_shutdown() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        let result = checker.shutdown().await;
        assert!(result.is_ok());
        assert!(!checker.is_initialized());
    }
    
    #[tokio::test]
    async fn test_add_health_check() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        let health_check = HealthCheck {
            name: "test_check".to_string(),
            check_type: HealthCheckType::Ping,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        let result = checker.add_health_check("test_check".to_string(), health_check).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_remove_health_check() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        let health_check = HealthCheck {
            name: "test_check".to_string(),
            check_type: HealthCheckType::Ping,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        checker.add_health_check("test_check".to_string(), health_check).await.unwrap();
        
        let result = checker.remove_health_check("test_check").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_run_health_check() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        let health_check = HealthCheck {
            name: "test_check".to_string(),
            check_type: HealthCheckType::Ping,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        checker.add_health_check("test_check".to_string(), health_check).await.unwrap();
        
        let result = checker.run_health_check("test_check").await.unwrap();
        assert_eq!(result.name, "test_check");
        assert!(result.is_healthy);
        assert!(!result.message.is_empty());
    }
    
    #[tokio::test]
    async fn test_run_health_check_not_found() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        let result = checker.run_health_check("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HealthError::HealthCheckNotFound(_)));
    }
    
    #[tokio::test]
    async fn test_check_all() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        let results = checker.check_all().await.unwrap();
        assert!(!results.is_empty());
        
        // Should have default health checks
        assert!(results.iter().any(|r| r.name == "system_memory"));
        assert!(results.iter().any(|r| r.name == "system_cpu"));
        assert!(results.iter().any(|r| r.name == "system_disk"));
    }
    
    #[tokio::test]
    async fn test_get_health_check_result() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        let health_check = HealthCheck {
            name: "test_check".to_string(),
            check_type: HealthCheckType::Ping,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        checker.add_health_check("test_check".to_string(), health_check).await.unwrap();
        checker.run_health_check("test_check").await.unwrap();
        
        let result = checker.get_health_check_result("test_check").await.unwrap();
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.name, "test_check");
        assert!(result.is_healthy);
    }
    
    #[tokio::test]
    async fn test_get_all_results() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        checker.check_all().await.unwrap();
        
        let results = checker.get_all_results().await.unwrap();
        assert!(!results.is_empty());
        
        // Should have default health checks
        assert!(results.contains_key("system_memory"));
        assert!(results.contains_key("system_cpu"));
        assert!(results.contains_key("system_disk"));
    }
    
    #[tokio::test]
    async fn test_get_system_health() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        let system_health = checker.get_system_health().await.unwrap();
        assert_eq!(system_health.overall_status, HealthStatus::Healthy);
        assert!(!system_health.health_checks.is_empty());
        assert!(system_health.last_checked > 0);
    }
    
    #[tokio::test]
    async fn test_health_check_types() {
        let mut checker = HealthChecker::new();
        checker.initialize().await.unwrap();
        
        // Test ping check
        let ping_check = HealthCheck {
            name: "ping_check".to_string(),
            check_type: HealthCheckType::Ping,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        checker.add_health_check("ping_check".to_string(), ping_check).await.unwrap();
        let result = checker.run_health_check("ping_check").await.unwrap();
        assert!(result.is_healthy);
        assert!(result.message.contains("Ping"));
        
        // Test HTTP check
        let http_check = HealthCheck {
            name: "http_check".to_string(),
            check_type: HealthCheckType::Http,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        checker.add_health_check("http_check".to_string(), http_check).await.unwrap();
        let result = checker.run_health_check("http_check").await.unwrap();
        assert!(result.is_healthy);
        assert!(result.message.contains("HTTP"));
        assert!(result.details.contains_key("status_code"));
        
        // Test database check
        let db_check = HealthCheck {
            name: "db_check".to_string(),
            check_type: HealthCheckType::Database,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        checker.add_health_check("db_check".to_string(), db_check).await.unwrap();
        let result = checker.run_health_check("db_check").await.unwrap();
        assert!(result.is_healthy);
        assert!(result.message.contains("Database"));
        assert!(result.details.contains_key("connection_pool_size"));
        
        // Test custom check
        let custom_check = HealthCheck {
            name: "custom_check".to_string(),
            check_type: HealthCheckType::Custom,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        checker.add_health_check("custom_check".to_string(), custom_check).await.unwrap();
        let result = checker.run_health_check("custom_check").await.unwrap();
        assert!(result.is_healthy);
        assert!(result.message.contains("Custom"));
    }
    
    #[test]
    fn test_health_config_default() {
        let config = HealthConfig::default();
        assert_eq!(config.default_interval, Duration::from_secs(30));
        assert_eq!(config.default_timeout, Duration::from_secs(5));
        assert!(config.enable_health_checks);
        assert!(config.health_checks.is_empty());
    }
    
    #[test]
    fn test_health_check_creation() {
        let health_check = HealthCheck {
            name: "test_check".to_string(),
            check_type: HealthCheckType::Ping,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            enabled: true,
        };
        
        assert_eq!(health_check.name, "test_check");
        assert_eq!(health_check.check_type, HealthCheckType::Ping);
        assert_eq!(health_check.interval, Duration::from_secs(30));
        assert_eq!(health_check.timeout, Duration::from_secs(5));
        assert!(health_check.enabled);
    }
    
    #[test]
    fn test_health_check_result_creation() {
        let result = HealthCheckResult {
            name: "test_check".to_string(),
            is_healthy: true,
            message: "Test check passed".to_string(),
            duration: Duration::from_millis(100),
            last_checked: 1234567890,
            details: HashMap::new(),
        };
        
        assert_eq!(result.name, "test_check");
        assert!(result.is_healthy);
        assert_eq!(result.message, "Test check passed");
        assert_eq!(result.duration, Duration::from_millis(100));
        assert_eq!(result.last_checked, 1234567890);
    }
    
    #[test]
    fn test_health_error_display() {
        let error = HealthError::HealthCheckNotFound("test_check".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Health check not found"));
        assert!(error_string.contains("test_check"));
    }
}
