//! Health reporting and status monitoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Health check definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Unique identifier for the health check
    pub id: String,
    /// Human-readable name for the health check
    pub name: String,
    /// Description of what this health check monitors
    pub description: String,
    /// Function to perform the health check
    pub check_fn: String, // In real implementation, this would be a function pointer
    /// Timeout for the health check (in seconds)
    pub timeout_seconds: u64,
    /// Whether the health check is enabled
    pub enabled: bool,
    /// Interval between health checks (in seconds)
    pub interval_seconds: u64,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(
        id: String,
        name: String,
        description: String,
        check_fn: String,
    ) -> Self {
        Self {
            id,
            name,
            description,
            check_fn,
            timeout_seconds: 30,
            enabled: true,
            interval_seconds: 60,
        }
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Set the interval
    pub fn with_interval(mut self, interval_seconds: u64) -> Self {
        self.interval_seconds = interval_seconds;
        self
    }

    /// Enable or disable the health check
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Health status levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System has minor issues
    Degraded,
    /// System has major issues
    Unhealthy,
    /// System status is unknown
    Unknown,
}

impl HealthStatus {
    /// Get a human-readable description of the status
    pub fn description(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "System is operating normally",
            HealthStatus::Degraded => "System is experiencing minor issues",
            HealthStatus::Unhealthy => "System is experiencing major issues",
            HealthStatus::Unknown => "System status is unknown",
        }
    }

    /// Get the status color for UI display
    pub fn color(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "green",
            HealthStatus::Degraded => "yellow",
            HealthStatus::Unhealthy => "red",
            HealthStatus::Unknown => "gray",
        }
    }
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Overall system health status
    pub status: HealthStatus,
    /// Timestamp when the status was last updated
    pub last_updated: u64,
    /// Individual health check results
    pub health_checks: HashMap<String, HealthCheckResult>,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Additional system information
    pub system_info: HashMap<String, String>,
}

impl SystemStatus {
    /// Create a new system status
    pub fn new() -> Self {
        Self {
            status: HealthStatus::Unknown,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            health_checks: HashMap::new(),
            uptime_seconds: 0,
            system_info: HashMap::new(),
        }
    }

    /// Update the system status
    pub fn update_status(&mut self, status: HealthStatus) {
        self.status = status;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Add a health check result
    pub fn add_health_check_result(&mut self, check_id: String, result: HealthCheckResult) {
        self.health_checks.insert(check_id, result);
    }

    /// Get the overall health status based on individual checks
    pub fn calculate_overall_status(&self) -> HealthStatus {
        if self.health_checks.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for result in self.health_checks.values() {
            match result.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => continue,
                HealthStatus::Unknown => has_degraded = true,
            }
        }

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Health check ID
    pub check_id: String,
    /// Status of the health check
    pub status: HealthStatus,
    /// Timestamp when the check was performed
    pub checked_at: u64,
    /// Duration the check took to complete (in milliseconds)
    pub duration_ms: u64,
    /// Optional message describing the result
    pub message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl HealthCheckResult {
    /// Create a new health check result
    pub fn new(
        check_id: String,
        status: HealthStatus,
        duration_ms: u64,
    ) -> Self {
        Self {
            check_id,
            status,
            checked_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            duration_ms,
            message: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a message to the result
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Health reporter for monitoring system health
#[derive(Debug, Clone)]
pub struct HealthReporter {
    /// Health checks to perform
    health_checks: HashMap<String, HealthCheck>,
    /// Current system status
    system_status: SystemStatus,
    /// System start time for uptime calculation
    system_start_time: u64,
}

impl HealthReporter {
    /// Create a new health reporter
    pub fn new() -> Self {
        Self {
            health_checks: HashMap::new(),
            system_status: SystemStatus::new(),
            system_start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Create a health reporter with configuration
    pub fn with_config(config: HealthConfig) -> Self {
        Self {
            health_checks: HashMap::new(),
            system_status: SystemStatus::new(),
            system_start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Add a health check
    pub fn add_health_check(&mut self, health_check: HealthCheck) {
        self.health_checks.insert(health_check.id.clone(), health_check);
    }

    /// Remove a health check
    pub fn remove_health_check(&mut self, check_id: &str) {
        self.health_checks.remove(check_id);
        self.system_status.health_checks.remove(check_id);
    }

    /// Get all health checks
    pub fn get_health_checks(&self) -> Vec<&HealthCheck> {
        self.health_checks.values().collect()
    }

    /// Get a specific health check
    pub fn get_health_check(&self, check_id: &str) -> Option<&HealthCheck> {
        self.health_checks.get(check_id)
    }

    /// Perform a health check
    pub fn perform_health_check(&mut self, check_id: &str) -> Option<HealthCheckResult> {
        let health_check = self.health_checks.get(check_id)?;
        
        if !health_check.enabled {
            return None;
        }

        let start_time = SystemTime::now();
        
        // In a real implementation, this would call the actual health check function
        // For now, we'll simulate a health check
        let result = self.simulate_health_check(health_check);
        
        let duration = start_time.elapsed().unwrap_or_default();
        let duration_ms = duration.as_millis() as u64;

        let mut health_result = HealthCheckResult::new(
            check_id.to_string(),
            result,
            duration_ms,
        );

        // Add the result to system status
        self.system_status.add_health_check_result(check_id.to_string(), health_result.clone());
        
        // Update overall status
        let overall_status = self.system_status.calculate_overall_status();
        self.system_status.update_status(overall_status);

        Some(health_result)
    }

    /// Perform all enabled health checks
    pub fn perform_all_health_checks(&mut self) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();
        
        let check_ids: Vec<String> = self.health_checks.keys().cloned().collect();
        for check_id in check_ids {
            if let Some(result) = self.perform_health_check(&check_id) {
                results.push(result);
            }
        }
        
        results
    }

    /// Get the current system status
    pub fn get_system_status(&self) -> &SystemStatus {
        &self.system_status
    }

    /// Get the current system status with updated uptime
    pub fn get_current_system_status(&mut self) -> SystemStatus {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.system_status.uptime_seconds = current_time - self.system_start_time;
        self.system_status.clone()
    }

    /// Get health check results by status
    pub fn get_health_checks_by_status(&self, status: &HealthStatus) -> Vec<&HealthCheckResult> {
        self.system_status
            .health_checks
            .values()
            .filter(|result| &result.status == status)
            .collect()
    }

    /// Get health statistics
    pub fn get_health_stats(&self) -> HealthStats {
        let total_checks = self.health_checks.len();
        let healthy_checks = self.get_health_checks_by_status(&HealthStatus::Healthy).len();
        let degraded_checks = self.get_health_checks_by_status(&HealthStatus::Degraded).len();
        let unhealthy_checks = self.get_health_checks_by_status(&HealthStatus::Unhealthy).len();
        let unknown_checks = self.get_health_checks_by_status(&HealthStatus::Unknown).len();

        HealthStats {
            total_checks,
            healthy_checks,
            degraded_checks,
            unhealthy_checks,
            unknown_checks,
            overall_status: self.system_status.status.clone(),
            uptime_seconds: self.system_status.uptime_seconds,
        }
    }

    /// Simulate a health check (for testing purposes)
    fn simulate_health_check(&self, health_check: &HealthCheck) -> HealthStatus {
        // In a real implementation, this would call the actual health check function
        // For now, we'll return a random status for testing
        match health_check.id.as_str() {
            "database" => HealthStatus::Healthy,
            "redis" => HealthStatus::Healthy,
            "external_api" => HealthStatus::Degraded,
            "disk_space" => HealthStatus::Unhealthy,
            _ => HealthStatus::Unknown,
        }
    }
}

impl Default for HealthReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Health statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStats {
    /// Total number of health checks
    pub total_checks: usize,
    /// Number of healthy checks
    pub healthy_checks: usize,
    /// Number of degraded checks
    pub degraded_checks: usize,
    /// Number of unhealthy checks
    pub unhealthy_checks: usize,
    /// Number of unknown checks
    pub unknown_checks: usize,
    /// Overall system status
    pub overall_status: HealthStatus,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

/// Configuration for health reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Default timeout for health checks (in seconds)
    pub default_timeout_seconds: u64,
    /// Default interval between health checks (in seconds)
    pub default_interval_seconds: u64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: 30,
            default_interval_seconds: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_creation() {
        let health_check = HealthCheck::new(
            "database".to_string(),
            "Database Health".to_string(),
            "Checks database connectivity".to_string(),
            "check_database".to_string(),
        );

        assert_eq!(health_check.id, "database");
        assert_eq!(health_check.name, "Database Health");
        assert!(health_check.enabled);
    }

    #[test]
    fn test_health_status() {
        assert_eq!(HealthStatus::Healthy.description(), "System is operating normally");
        assert_eq!(HealthStatus::Healthy.color(), "green");
        assert!(HealthStatus::Healthy < HealthStatus::Degraded);
        assert!(HealthStatus::Degraded < HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult::new(
            "database".to_string(),
            HealthStatus::Healthy,
            150,
        ).with_message("Database is responding normally".to_string());

        assert_eq!(result.check_id, "database");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.duration_ms, 150);
        assert!(result.message.is_some());
    }

    #[test]
    fn test_system_status() {
        let mut status = SystemStatus::new();
        
        // Add some health check results
        let result1 = HealthCheckResult::new(
            "database".to_string(),
            HealthStatus::Healthy,
            100,
        );
        let result2 = HealthCheckResult::new(
            "redis".to_string(),
            HealthStatus::Degraded,
            200,
        );

        status.add_health_check_result("database".to_string(), result1);
        status.add_health_check_result("redis".to_string(), result2);

        // Calculate overall status
        let overall_status = status.calculate_overall_status();
        assert_eq!(overall_status, HealthStatus::Degraded);
    }

    #[test]
    fn test_health_reporter() {
        let mut reporter = HealthReporter::new();
        
        // Add a health check
        let health_check = HealthCheck::new(
            "database".to_string(),
            "Database Health".to_string(),
            "Checks database connectivity".to_string(),
            "check_database".to_string(),
        );
        reporter.add_health_check(health_check);

        // Perform the health check
        let result = reporter.perform_health_check("database");
        assert!(result.is_some());
        
        let result = result.unwrap();
        assert_eq!(result.check_id, "database");

        // Check system status
        let status = reporter.get_current_system_status();
        assert!(status.uptime_seconds > 0);
    }

    #[test]
    fn test_health_stats() {
        let mut reporter = HealthReporter::new();
        
        // Add multiple health checks
        let checks = vec![
            ("database", HealthStatus::Healthy),
            ("redis", HealthStatus::Healthy),
            ("external_api", HealthStatus::Degraded),
            ("disk_space", HealthStatus::Unhealthy),
        ];

        for (id, status) in checks {
            let health_check = HealthCheck::new(
                id.to_string(),
                format!("{} Health", id),
                format!("Checks {} connectivity", id),
                format!("check_{}", id),
            );
            reporter.add_health_check(health_check);
            
            // Simulate the health check result
            let result = HealthCheckResult::new(id.to_string(), status, 100);
            reporter.system_status.add_health_check_result(id.to_string(), result);
        }

        let stats = reporter.get_health_stats();
        assert_eq!(stats.total_checks, 4);
        assert_eq!(stats.healthy_checks, 2);
        assert_eq!(stats.degraded_checks, 1);
        assert_eq!(stats.unhealthy_checks, 1);
    }
}
