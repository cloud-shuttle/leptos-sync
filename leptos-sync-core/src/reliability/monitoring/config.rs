//! Configuration types for monitoring system

use serde::{Deserialize, Serialize};
use super::alerts::AlertConfig;
use super::health::HealthConfig;
use super::metrics::MetricsConfig;

/// Main configuration for the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Metrics collection configuration
    pub metrics_config: MetricsConfig,
    /// Alert management configuration
    pub alert_config: AlertConfig,
    /// Health reporting configuration
    pub health_config: HealthConfig,
    /// Whether the monitoring system is enabled
    pub enabled: bool,
    /// Global monitoring interval (in seconds)
    pub monitoring_interval_seconds: u64,
    /// Maximum number of monitoring statistics to keep
    pub max_stats_history: usize,
}

impl MonitorConfig {
    /// Create a new monitoring configuration
    pub fn new() -> Self {
        Self {
            metrics_config: MetricsConfig::default(),
            alert_config: AlertConfig::default(),
            health_config: HealthConfig::default(),
            enabled: true,
            monitoring_interval_seconds: 60,
            max_stats_history: 1000,
        }
    }

    /// Create a configuration with custom settings
    pub fn with_settings(
        metrics_config: MetricsConfig,
        alert_config: AlertConfig,
        health_config: HealthConfig,
    ) -> Self {
        Self {
            metrics_config,
            alert_config,
            health_config,
            enabled: true,
            monitoring_interval_seconds: 60,
            max_stats_history: 1000,
        }
    }

    /// Set the monitoring interval
    pub fn with_monitoring_interval(mut self, interval_seconds: u64) -> Self {
        self.monitoring_interval_seconds = interval_seconds;
        self
    }

    /// Set the maximum stats history
    pub fn with_max_stats_history(mut self, max_history: usize) -> Self {
        self.max_stats_history = max_history;
        self
    }

    /// Enable or disable monitoring
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Monitoring statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    /// Total number of metrics collected
    pub total_metrics_collected: u64,
    /// Total number of alerts triggered
    pub total_alerts_triggered: u64,
    /// Total number of health checks performed
    pub total_health_checks: u64,
    /// System uptime in seconds
    pub system_uptime_seconds: u64,
    /// Last monitoring update timestamp
    pub last_update_timestamp: u64,
    /// Monitoring system start time
    pub start_time: u64,
}

impl MonitoringStats {
    /// Create new monitoring statistics
    pub fn new() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            total_metrics_collected: 0,
            total_alerts_triggered: 0,
            total_health_checks: 0,
            system_uptime_seconds: 0,
            last_update_timestamp: now,
            start_time: now,
        }
    }

    /// Update the statistics
    pub fn update(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.system_uptime_seconds = now - self.start_time;
        self.last_update_timestamp = now;
    }

    /// Increment metrics collected count
    pub fn increment_metrics_collected(&mut self) {
        self.total_metrics_collected += 1;
        self.update();
    }

    /// Increment alerts triggered count
    pub fn increment_alerts_triggered(&mut self) {
        self.total_alerts_triggered += 1;
        self.update();
    }

    /// Increment health checks count
    pub fn increment_health_checks(&mut self) {
        self.total_health_checks += 1;
        self.update();
    }

    /// Get the monitoring system uptime
    pub fn get_uptime(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.system_uptime_seconds)
    }

    /// Get metrics collection rate (metrics per minute)
    pub fn get_metrics_rate(&self) -> f64 {
        if self.system_uptime_seconds == 0 {
            return 0.0;
        }
        (self.total_metrics_collected as f64) / (self.system_uptime_seconds as f64 / 60.0)
    }

    /// Get alert rate (alerts per hour)
    pub fn get_alert_rate(&self) -> f64 {
        if self.system_uptime_seconds == 0 {
            return 0.0;
        }
        (self.total_alerts_triggered as f64) / (self.system_uptime_seconds as f64 / 3600.0)
    }

    /// Get health check rate (checks per minute)
    pub fn get_health_check_rate(&self) -> f64 {
        if self.system_uptime_seconds == 0 {
            return 0.0;
        }
        (self.total_health_checks as f64) / (self.system_uptime_seconds as f64 / 60.0)
    }
}

impl Default for MonitoringStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Whether to enable performance monitoring
    pub enabled: bool,
    /// Sampling rate for performance metrics (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Maximum number of performance samples to keep
    pub max_samples: usize,
    /// Performance threshold for warnings (in milliseconds)
    pub warning_threshold_ms: u64,
    /// Performance threshold for errors (in milliseconds)
    pub error_threshold_ms: u64,
}

impl PerformanceConfig {
    /// Create a new performance configuration
    pub fn new() -> Self {
        Self {
            enabled: true,
            sampling_rate: 1.0,
            max_samples: 10000,
            warning_threshold_ms: 1000,
            error_threshold_ms: 5000,
        }
    }

    /// Set the sampling rate
    pub fn with_sampling_rate(mut self, rate: f64) -> Self {
        self.sampling_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Set the maximum samples
    pub fn with_max_samples(mut self, max_samples: usize) -> Self {
        self.max_samples = max_samples;
        self
    }

    /// Set the performance thresholds
    pub fn with_thresholds(mut self, warning_ms: u64, error_ms: u64) -> Self {
        self.warning_threshold_ms = warning_ms;
        self.error_threshold_ms = error_ms;
        self
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Whether to monitor CPU usage
    pub monitor_cpu: bool,
    /// Whether to monitor memory usage
    pub monitor_memory: bool,
    /// Whether to monitor disk usage
    pub monitor_disk: bool,
    /// Whether to monitor network usage
    pub monitor_network: bool,
    /// CPU usage warning threshold (percentage)
    pub cpu_warning_threshold: f64,
    /// CPU usage error threshold (percentage)
    pub cpu_error_threshold: f64,
    /// Memory usage warning threshold (percentage)
    pub memory_warning_threshold: f64,
    /// Memory usage error threshold (percentage)
    pub memory_error_threshold: f64,
    /// Disk usage warning threshold (percentage)
    pub disk_warning_threshold: f64,
    /// Disk usage error threshold (percentage)
    pub disk_error_threshold: f64,
}

impl ResourceConfig {
    /// Create a new resource configuration
    pub fn new() -> Self {
        Self {
            monitor_cpu: true,
            monitor_memory: true,
            monitor_disk: true,
            monitor_network: false,
            cpu_warning_threshold: 80.0,
            cpu_error_threshold: 95.0,
            memory_warning_threshold: 85.0,
            memory_error_threshold: 95.0,
            disk_warning_threshold: 80.0,
            disk_error_threshold: 90.0,
        }
    }

    /// Enable or disable CPU monitoring
    pub fn with_cpu_monitoring(mut self, enabled: bool) -> Self {
        self.monitor_cpu = enabled;
        self
    }

    /// Enable or disable memory monitoring
    pub fn with_memory_monitoring(mut self, enabled: bool) -> Self {
        self.monitor_memory = enabled;
        self
    }

    /// Enable or disable disk monitoring
    pub fn with_disk_monitoring(mut self, enabled: bool) -> Self {
        self.monitor_disk = enabled;
        self
    }

    /// Enable or disable network monitoring
    pub fn with_network_monitoring(mut self, enabled: bool) -> Self {
        self.monitor_network = enabled;
        self
    }

    /// Set CPU thresholds
    pub fn with_cpu_thresholds(mut self, warning: f64, error: f64) -> Self {
        self.cpu_warning_threshold = warning.clamp(0.0, 100.0);
        self.cpu_error_threshold = error.clamp(0.0, 100.0);
        self
    }

    /// Set memory thresholds
    pub fn with_memory_thresholds(mut self, warning: f64, error: f64) -> Self {
        self.memory_warning_threshold = warning.clamp(0.0, 100.0);
        self.memory_error_threshold = error.clamp(0.0, 100.0);
        self
    }

    /// Set disk thresholds
    pub fn with_disk_thresholds(mut self, warning: f64, error: f64) -> Self {
        self.disk_warning_threshold = warning.clamp(0.0, 100.0);
        self.disk_error_threshold = error.clamp(0.0, 100.0);
        self
    }
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedMonitorConfig {
    /// Base monitoring configuration
    pub base_config: MonitorConfig,
    /// Performance monitoring configuration
    pub performance_config: PerformanceConfig,
    /// Resource monitoring configuration
    pub resource_config: ResourceConfig,
    /// Whether to enable detailed logging
    pub detailed_logging: bool,
    /// Log level for monitoring (debug, info, warn, error)
    pub log_level: String,
}

impl ExtendedMonitorConfig {
    /// Create a new extended monitoring configuration
    pub fn new() -> Self {
        Self {
            base_config: MonitorConfig::new(),
            performance_config: PerformanceConfig::new(),
            resource_config: ResourceConfig::new(),
            detailed_logging: false,
            log_level: "info".to_string(),
        }
    }

    /// Enable detailed logging
    pub fn with_detailed_logging(mut self, enabled: bool) -> Self {
        self.detailed_logging = enabled;
        self
    }

    /// Set the log level
    pub fn with_log_level(mut self, level: String) -> Self {
        self.log_level = level;
        self
    }
}

impl Default for ExtendedMonitorConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_config_creation() {
        let config = MonitorConfig::new();
        assert!(config.enabled);
        assert_eq!(config.monitoring_interval_seconds, 60);
        assert_eq!(config.max_stats_history, 1000);
    }

    #[test]
    fn test_monitor_config_customization() {
        let config = MonitorConfig::new()
            .with_monitoring_interval(30)
            .with_max_stats_history(500)
            .set_enabled(false);

        assert!(!config.enabled);
        assert_eq!(config.monitoring_interval_seconds, 30);
        assert_eq!(config.max_stats_history, 500);
    }

    #[test]
    fn test_monitoring_stats() {
        let mut stats = MonitoringStats::new();
        
        assert_eq!(stats.total_metrics_collected, 0);
        assert_eq!(stats.total_alerts_triggered, 0);
        assert_eq!(stats.total_health_checks, 0);

        // Increment counters
        stats.increment_metrics_collected();
        stats.increment_alerts_triggered();
        stats.increment_health_checks();

        assert_eq!(stats.total_metrics_collected, 1);
        assert_eq!(stats.total_alerts_triggered, 1);
        assert_eq!(stats.total_health_checks, 1);

        // Test rates
        let metrics_rate = stats.get_metrics_rate();
        let alert_rate = stats.get_alert_rate();
        let health_check_rate = stats.get_health_check_rate();

        assert!(metrics_rate >= 0.0);
        assert!(alert_rate >= 0.0);
        assert!(health_check_rate >= 0.0);
    }

    #[test]
    fn test_performance_config() {
        let config = PerformanceConfig::new()
            .with_sampling_rate(0.5)
            .with_max_samples(5000)
            .with_thresholds(500, 2000);

        assert_eq!(config.sampling_rate, 0.5);
        assert_eq!(config.max_samples, 5000);
        assert_eq!(config.warning_threshold_ms, 500);
        assert_eq!(config.error_threshold_ms, 2000);
    }

    #[test]
    fn test_resource_config() {
        let config = ResourceConfig::new()
            .with_cpu_monitoring(true)
            .with_memory_monitoring(false)
            .with_cpu_thresholds(70.0, 90.0)
            .with_memory_thresholds(80.0, 95.0);

        assert!(config.monitor_cpu);
        assert!(!config.monitor_memory);
        assert_eq!(config.cpu_warning_threshold, 70.0);
        assert_eq!(config.cpu_error_threshold, 90.0);
        assert_eq!(config.memory_warning_threshold, 80.0);
        assert_eq!(config.memory_error_threshold, 95.0);
    }

    #[test]
    fn test_extended_monitor_config() {
        let config = ExtendedMonitorConfig::new()
            .with_detailed_logging(true)
            .with_log_level("debug".to_string());

        assert!(config.detailed_logging);
        assert_eq!(config.log_level, "debug");
        assert!(config.base_config.enabled);
        assert!(config.performance_config.enabled);
        assert!(config.resource_config.monitor_cpu);
    }

    #[test]
    fn test_config_serialization() {
        let config = MonitorConfig::new();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: MonitorConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.monitoring_interval_seconds, deserialized.monitoring_interval_seconds);
    }
}
