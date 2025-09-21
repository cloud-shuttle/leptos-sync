//! Monitoring System
//!
//! This module provides comprehensive monitoring capabilities including:
//! - Metrics collection and aggregation
//! - Alert management and notification
//! - Health reporting and status monitoring
//! - Performance tracking and analysis

pub mod metrics;
pub mod alerts;
pub mod health;
pub mod config;

// Re-export main types for convenience
pub use metrics::{
    Metric, TimeRange, AggregationType, AggregatedMetric, MetricsCollector, MetricsConfig,
};
pub use alerts::{
    AlertRule, AlertCondition, ComparisonOperator, AlertSeverity, Alert, AlertManager,
    AlertStats, AlertConfig,
};
pub use health::{
    HealthCheck, HealthStatus, SystemStatus, HealthCheckResult, HealthReporter, HealthStats,
    HealthConfig,
};
pub use config::{
    MonitorConfig, MonitoringStats, PerformanceConfig, ResourceConfig, ExtendedMonitorConfig,
};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Reliability monitoring system
#[derive(Debug, Clone)]
pub struct ReliabilityMonitor {
    /// Metrics collector
    pub metrics_collector: MetricsCollector,
    /// Alert manager
    pub alert_manager: AlertManager,
    /// Health reporter
    pub health_reporter: HealthReporter,
    /// Monitoring statistics
    pub stats: Arc<RwLock<MonitoringStats>>,
    /// Whether the system is initialized
    pub initialized: bool,
}

impl ReliabilityMonitor {
    /// Create a new reliability monitor
    pub fn new() -> Self {
        Self {
            metrics_collector: MetricsCollector::new(),
            alert_manager: AlertManager::new(),
            health_reporter: HealthReporter::new(),
            stats: Arc::new(RwLock::new(MonitoringStats::new())),
            initialized: false,
        }
    }
    
    /// Create a new reliability monitor with configuration
    pub fn with_config(config: MonitorConfig) -> Self {
        Self {
            metrics_collector: MetricsCollector::with_config(config.metrics_config),
            alert_manager: AlertManager::with_config(config.alert_config),
            health_reporter: HealthReporter::with_config(config.health_config),
            stats: Arc::new(RwLock::new(MonitoringStats::new())),
            initialized: false,
        }
    }

    /// Initialize the monitoring system
    pub async fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Err("Monitoring system is already initialized".to_string());
        }

        // Initialize components
        self.metrics_collector = MetricsCollector::new();
        self.alert_manager = AlertManager::new();
        self.health_reporter = HealthReporter::new();

        self.initialized = true;
        Ok(())
    }

    /// Record a metric
    pub fn record_metric(&mut self, metric: Metric) {
        if !self.initialized {
            return;
        }

        self.metrics_collector.record(metric.clone());
        
        // Check if this metric should trigger any alerts
        let alerts = self.alert_manager.check_metric(&metric.name, metric.value);
        
        // Update statistics
        if let Ok(mut stats) = self.stats.try_write() {
            stats.increment_metrics_collected();
            if !alerts.is_empty() {
                stats.increment_alerts_triggered();
            }
        }
    }

    /// Add an alert rule
    pub fn add_alert_rule(&mut self, rule: AlertRule) {
        if !self.initialized {
            return;
        }
        self.alert_manager.add_rule(rule);
    }

    /// Add a health check
    pub fn add_health_check(&mut self, health_check: HealthCheck) {
        if !self.initialized {
            return;
        }
        self.health_reporter.add_health_check(health_check);
    }

    /// Perform all health checks
    pub fn perform_health_checks(&mut self) -> Vec<HealthCheckResult> {
        if !self.initialized {
            return Vec::new();
        }

        let results = self.health_reporter.perform_all_health_checks();
        
        // Update statistics
        if let Ok(mut stats) = self.stats.try_write() {
            stats.increment_health_checks();
        }

        results
    }

    /// Get the current system status
    pub fn get_system_status(&mut self) -> SystemStatus {
        if !self.initialized {
            return SystemStatus::new();
        }
        self.health_reporter.get_current_system_status()
    }

    /// Get monitoring statistics
    pub async fn get_stats(&self) -> Result<MonitoringStats, String> {
        if !self.initialized {
            return Err("Monitoring system is not initialized".to_string());
        }

        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    /// Get alert statistics
    pub fn get_alert_stats(&self) -> AlertStats {
        if !self.initialized {
            return AlertStats {
                total_rules: 0,
                active_alerts: 0,
                critical_alerts: 0,
                high_alerts: 0,
                medium_alerts: 0,
                low_alerts: 0,
                total_history: 0,
            };
        }
        self.alert_manager.get_stats()
    }

    /// Get health statistics
    pub fn get_health_stats(&self) -> HealthStats {
        if !self.initialized {
            return HealthStats {
                total_checks: 0,
                healthy_checks: 0,
                degraded_checks: 0,
                unhealthy_checks: 0,
                unknown_checks: 0,
                overall_status: HealthStatus::Unknown,
                uptime_seconds: 0,
            };
        }
        self.health_reporter.get_health_stats()
    }

    /// Get metrics for a specific name and time range
    pub fn get_metrics(&self, name: &str, time_range: &TimeRange) -> Vec<&Metric> {
        if !self.initialized {
            return Vec::new();
        }
        self.metrics_collector.get_metrics(name, time_range)
    }

    /// Get aggregated metrics
    pub fn get_aggregated_metrics(
        &self,
        name: &str,
        time_range: &TimeRange,
        aggregation_type: AggregationType,
    ) -> Option<AggregatedMetric> {
        if !self.initialized {
            return None;
        }
        self.metrics_collector.aggregate_metrics(name, time_range, aggregation_type)
    }

    /// Get all active alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        if !self.initialized {
            return Vec::new();
        }
        self.alert_manager.get_active_alerts()
    }

    /// Resolve an alert
    pub fn resolve_alert(&mut self, alert_id: &str) -> bool {
        if !self.initialized {
            return false;
        }
        self.alert_manager.resolve_alert(alert_id)
    }

    /// Get all metric names
    pub fn get_metric_names(&self) -> Vec<String> {
        if !self.initialized {
            return Vec::new();
        }
        self.metrics_collector.get_metric_names()
    }

    /// Clear all metrics
    pub fn clear_metrics(&mut self) {
        if !self.initialized {
            return;
        }
        self.metrics_collector.clear();
    }

    /// Clear alert history
    pub fn clear_alert_history(&mut self) {
        if !self.initialized {
            return;
        }
        self.alert_manager.clear_history();
    }

    /// Shutdown the monitoring system
    pub async fn shutdown(&mut self) -> Result<(), String> {
        if !self.initialized {
            return Err("Monitoring system is not initialized".to_string());
        }

        // Perform final health checks
        self.perform_health_checks();

        // Clear all data
        self.clear_metrics();
        self.clear_alert_history();

        self.initialized = false;
        Ok(())
    }
}

impl Default for ReliabilityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_reliability_monitor_creation() {
        let monitor = ReliabilityMonitor::new();
        assert!(!monitor.initialized);
    }

    #[test]
    fn test_reliability_monitor_with_config() {
        let config = MonitorConfig::new();
        let monitor = ReliabilityMonitor::with_config(config);
        assert!(!monitor.initialized);
    }

    #[tokio::test]
    async fn test_reliability_monitor_initialization() {
        let mut monitor = ReliabilityMonitor::new();
        
        // Should not be initialized initially
        assert!(!monitor.initialized);
        
        // Initialize the monitor
        let result = monitor.initialize().await;
        assert!(result.is_ok());
        assert!(monitor.initialized);
        
        // Try to initialize again (should fail)
        let result = monitor.initialize().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reliability_monitor_operations() {
        let mut monitor = ReliabilityMonitor::new();
        
        // Operations should not work when not initialized
        let metric = Metric::new("test_metric".to_string(), 42.0);
        monitor.record_metric(metric);
        
        let metrics = monitor.get_metrics("test_metric", &TimeRange::last_seconds(3600));
        assert!(metrics.is_empty());
        
        // Initialize the monitor
        let result = monitor.initialize();
        assert!(result.await.is_ok());
        
        // Now operations should work
        let metric = Metric::new("test_metric".to_string(), 42.0);
        monitor.record_metric(metric);
        
        let metrics = monitor.get_metrics("test_metric", &TimeRange::last_seconds(3600));
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].value, 42.0);
    }

    #[tokio::test]
    async fn test_reliability_monitor_alert_integration() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        // Add an alert rule
        let condition = AlertCondition::new(ComparisonOperator::GreaterThan, 80.0, 60);
        let rule = AlertRule::new(
            "cpu_high".to_string(),
            "High CPU Usage".to_string(),
            "cpu_usage".to_string(),
            condition,
            AlertSeverity::High,
        );
        monitor.add_alert_rule(rule);
        
        // Record a metric that should trigger the alert
        let metric = Metric::new("cpu_usage".to_string(), 85.0);
        monitor.record_metric(metric);
        
        // Check that we have an active alert
        let active_alerts = monitor.get_active_alerts();
        assert_eq!(active_alerts.len(), 1);
        assert_eq!(active_alerts[0].severity, AlertSeverity::High);
    }

    #[tokio::test]
    async fn test_reliability_monitor_health_integration() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        // Add a health check
        let health_check = HealthCheck::new(
            "database".to_string(),
            "Database Health".to_string(),
            "Checks database connectivity".to_string(),
            "check_database".to_string(),
        );
        monitor.add_health_check(health_check);
        
        // Perform health checks
        let results = monitor.perform_health_checks();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].check_id, "database");
        
        // Get system status
        let status = monitor.get_system_status();
        assert!(status.uptime_seconds > 0);
    }

    #[tokio::test]
    async fn test_reliability_monitor_statistics() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        // Record some metrics
        monitor.record_metric(Metric::new("metric1".to_string(), 10.0));
        monitor.record_metric(Metric::new("metric2".to_string(), 20.0));
        
        // Get statistics
        let alert_stats = monitor.get_alert_stats();
        let health_stats = monitor.get_health_stats();
        
        assert_eq!(alert_stats.total_rules, 0);
        assert_eq!(health_stats.total_checks, 0);
        
        // Get metric names
        let metric_names = monitor.get_metric_names();
        assert_eq!(metric_names.len(), 2);
        assert!(metric_names.contains(&"metric1".to_string()));
        assert!(metric_names.contains(&"metric2".to_string()));
    }

    #[tokio::test]
    async fn test_reliability_monitor_shutdown() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        assert!(monitor.initialized);
        
        // Shutdown the monitor
        let result = monitor.shutdown().await;
        assert!(result.is_ok());
        assert!(!monitor.initialized);
        
        // Try to shutdown again (should fail)
        let result = monitor.shutdown().await;
        assert!(result.is_err());
    }
}
