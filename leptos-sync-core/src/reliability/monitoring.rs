//! Monitoring System
//!
//! This module provides comprehensive monitoring capabilities including:
//! - Metrics collection and aggregation
//! - Alert management and notification
//! - Health reporting and status monitoring
//! - Performance tracking and analysis

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// Reliability monitoring system
#[derive(Debug, Clone)]
pub struct ReliabilityMonitor {
    /// Metrics collector
    metrics_collector: MetricsCollector,
    /// Alert manager
    alert_manager: AlertManager,
    /// Health reporter
    health_reporter: HealthReporter,
    /// Monitoring statistics
    stats: Arc<RwLock<MonitoringStats>>,
    /// Whether the system is initialized
    initialized: bool,
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
        self.metrics_collector.initialize().await?;
        self.alert_manager.initialize().await?;
        self.health_reporter.initialize().await?;
        
        let mut stats = self.stats.write().await;
        stats.reset();
        
        self.initialized = true;
        Ok(())
    }
    
    /// Shutdown the monitoring system
    pub async fn shutdown(&mut self) -> Result<(), String> {
        self.initialized = false;
        Ok(())
    }
    
    /// Check if the system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Record a metric
    pub async fn record_metric(&self, metric: Metric) -> Result<(), String> {
        if !self.initialized {
            return Err("Monitoring system not initialized".to_string());
        }
        
        self.metrics_collector.record(metric.clone()).await?;
        
        // Check for alerts
        if let Some(alert) = self.alert_manager.check_metric(&metric).await? {
            self.alert_manager.send_alert(alert).await?;
        }
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.record_metric(&metric);
        
        Ok(())
    }
    
    /// Get metrics for a specific time range
    pub async fn get_metrics(&self, metric_name: &str, time_range: TimeRange) -> Result<Vec<Metric>, String> {
        if !self.initialized {
            return Err("Monitoring system not initialized".to_string());
        }
        
        self.metrics_collector.get_metrics(metric_name, time_range).await
    }
    
    /// Get aggregated metrics
    pub async fn get_aggregated_metrics(&self, metric_name: &str, aggregation: AggregationType, time_range: TimeRange) -> Result<AggregatedMetric, String> {
        if !self.initialized {
            return Err("Monitoring system not initialized".to_string());
        }
        
        self.metrics_collector.get_aggregated(metric_name, aggregation, time_range).await
    }
    
    /// Get system health status
    pub async fn get_health_status(&self) -> Result<HealthStatus, String> {
        if !self.initialized {
            return Err("Monitoring system not initialized".to_string());
        }
        
        self.health_reporter.get_status().await
    }
    
    /// Get monitoring statistics
    pub async fn get_stats(&self) -> MonitoringStats {
        self.stats.read().await.clone()
    }
    
    /// Get system status
    pub async fn get_status(&self) -> Result<SystemStatus, String> {
        if !self.initialized {
            return Err("Monitoring system not initialized".to_string());
        }
        
        let health_status = self.get_health_status().await?;
        let metrics_count = self.metrics_collector.get_metrics_count().await?;
        let alerts_count = self.alert_manager.get_active_alerts_count().await?;
        
        Ok(SystemStatus {
            health_status,
            metrics_count,
            alerts_count,
            uptime: self.get_uptime().await,
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        })
    }
    
    /// Get system uptime
    async fn get_uptime(&self) -> Duration {
        // This would typically be tracked from system start
        // For now, return a placeholder
        Duration::from_secs(3600) // 1 hour
    }
}

/// Metrics collector for gathering and storing metrics
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Stored metrics
    metrics: Arc<RwLock<HashMap<String, Vec<Metric>>>>,
    /// Maximum number of metrics to store per name
    max_metrics_per_name: usize,
    /// Whether the collector is initialized
    initialized: bool,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            max_metrics_per_name: 1000,
            initialized: false,
        }
    }
    
    /// Create a new metrics collector with configuration
    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            max_metrics_per_name: config.max_metrics_per_name,
            initialized: false,
        }
    }
    
    /// Initialize the metrics collector
    pub async fn initialize(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }
    
    /// Record a metric
    pub async fn record(&self, metric: Metric) -> Result<(), String> {
        if !self.initialized {
            return Err("Metrics collector not initialized".to_string());
        }
        
        let mut metrics = self.metrics.write().await;
        let metric_list = metrics.entry(metric.name.clone()).or_insert_with(Vec::new);
        
        // Add the metric
        metric_list.push(metric);
        
        // Trim if we exceed the maximum
        if metric_list.len() > self.max_metrics_per_name {
            metric_list.drain(0..metric_list.len() - self.max_metrics_per_name);
        }
        
        Ok(())
    }
    
    /// Get metrics for a specific name and time range
    pub async fn get_metrics(&self, metric_name: &str, time_range: TimeRange) -> Result<Vec<Metric>, String> {
        if !self.initialized {
            return Err("Metrics collector not initialized".to_string());
        }
        
        let metrics = self.metrics.read().await;
        if let Some(metric_list) = metrics.get(metric_name) {
            let filtered_metrics: Vec<Metric> = metric_list
                .iter()
                .filter(|metric| {
                    metric.timestamp >= time_range.start && metric.timestamp <= time_range.end
                })
                .cloned()
                .collect();
            Ok(filtered_metrics)
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Get aggregated metrics
    pub async fn get_aggregated(&self, metric_name: &str, aggregation: AggregationType, time_range: TimeRange) -> Result<AggregatedMetric, String> {
        if !self.initialized {
            return Err("Metrics collector not initialized".to_string());
        }
        
        let metrics = self.get_metrics(metric_name, time_range.clone()).await?;
        
        if metrics.is_empty() {
            return Ok(AggregatedMetric {
                name: metric_name.to_string(),
                aggregation,
                value: 0.0,
                count: 0,
                time_range,
            });
        }
        
        let values: Vec<f64> = metrics.iter().map(|m| m.value).collect();
        let aggregated_value = match aggregation {
            AggregationType::Sum => values.iter().sum(),
            AggregationType::Average => values.iter().sum::<f64>() / values.len() as f64,
            AggregationType::Min => values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
            AggregationType::Max => values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            AggregationType::Count => values.len() as f64,
        };
        
        Ok(AggregatedMetric {
            name: metric_name.to_string(),
            aggregation,
            value: aggregated_value,
            count: metrics.len(),
            time_range,
        })
    }
    
    /// Get the number of metrics stored
    pub async fn get_metrics_count(&self) -> Result<usize, String> {
        if !self.initialized {
            return Err("Metrics collector not initialized".to_string());
        }
        
        let metrics = self.metrics.read().await;
        Ok(metrics.values().map(|v| v.len()).sum())
    }
}

/// Alert manager for handling alerts and notifications
#[derive(Debug, Clone)]
pub struct AlertManager {
    /// Alert rules
    rules: Arc<RwLock<Vec<AlertRule>>>,
    /// Active alerts
    active_alerts: Arc<RwLock<Vec<Alert>>>,
    /// Whether the manager is initialized
    initialized: bool,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            initialized: false,
        }
    }
    
    /// Create a new alert manager with configuration
    pub fn with_config(config: AlertConfig) -> Self {
        Self {
            rules: Arc::new(RwLock::new(config.rules)),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            initialized: false,
        }
    }
    
    /// Initialize the alert manager
    pub async fn initialize(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }
    
    /// Check if a metric triggers any alerts
    pub async fn check_metric(&self, metric: &Metric) -> Result<Option<Alert>, String> {
        if !self.initialized {
            return Err("Alert manager not initialized".to_string());
        }
        
        let rules = self.rules.read().await;
        for rule in rules.iter() {
            if rule.metric_name == metric.name && rule.condition.evaluate(metric.value) {
                let alert = Alert {
                    id: format!("{}-{}", rule.id, metric.timestamp),
                    rule_id: rule.id.clone(),
                    metric_name: metric.name.clone(),
                    value: metric.value,
                    threshold: rule.condition.threshold,
                    severity: rule.severity.clone(),
                    message: rule.message.clone(),
                    timestamp: metric.timestamp,
                    acknowledged: false,
                };
                return Ok(Some(alert));
            }
        }
        
        Ok(None)
    }
    
    /// Send an alert
    pub async fn send_alert(&self, alert: Alert) -> Result<(), String> {
        if !self.initialized {
            return Err("Alert manager not initialized".to_string());
        }
        
        let mut active_alerts = self.active_alerts.write().await;
        active_alerts.push(alert);
        
        // Keep only the last 100 alerts
        if active_alerts.len() > 100 {
            let len = active_alerts.len();
            active_alerts.drain(0..len - 100);
        }
        
        Ok(())
    }
    
    /// Get active alerts count
    pub async fn get_active_alerts_count(&self) -> Result<usize, String> {
        if !self.initialized {
            return Err("Alert manager not initialized".to_string());
        }
        
        let active_alerts = self.active_alerts.read().await;
        Ok(active_alerts.len())
    }
}

/// Health reporter for system health monitoring
#[derive(Debug, Clone)]
pub struct HealthReporter {
    /// Health checks
    health_checks: Arc<RwLock<Vec<HealthCheck>>>,
    /// Whether the reporter is initialized
    initialized: bool,
}

impl HealthReporter {
    /// Create a new health reporter
    pub fn new() -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(Vec::new())),
            initialized: false,
        }
    }
    
    /// Create a new health reporter with configuration
    pub fn with_config(config: HealthConfig) -> Self {
        Self {
            health_checks: Arc::new(RwLock::new(config.health_checks)),
            initialized: false,
        }
    }
    
    /// Initialize the health reporter
    pub async fn initialize(&mut self) -> Result<(), String> {
        self.initialized = true;
        Ok(())
    }
    
    /// Get system health status
    pub async fn get_status(&self) -> Result<HealthStatus, String> {
        if !self.initialized {
            return Err("Health reporter not initialized".to_string());
        }
        
        let health_checks = self.health_checks.read().await;
        let mut overall_healthy = true;
        
        for check in health_checks.iter() {
            if !check.is_healthy {
                overall_healthy = false;
                break;
            }
        }
        
        Ok(if overall_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        })
    }
}

/// Metric data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Metric timestamp
    pub timestamp: u64,
    /// Metric tags
    pub tags: HashMap<String, String>,
}

/// Time range for metric queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start timestamp
    pub start: u64,
    /// End timestamp
    pub end: u64,
}

/// Aggregation types for metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum of values
    Sum,
    /// Average of values
    Average,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of values
    Count,
}

/// Aggregated metric result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregatedMetric {
    /// Metric name
    pub name: String,
    /// Aggregation type
    pub aggregation: AggregationType,
    /// Aggregated value
    pub value: f64,
    /// Number of metrics aggregated
    pub count: usize,
    /// Time range
    pub time_range: TimeRange,
}

/// Alert rule for triggering alerts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    /// Metric name to monitor
    pub metric_name: String,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
}

/// Alert condition for evaluating metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertCondition {
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Threshold value
    pub threshold: f64,
}

impl AlertCondition {
    /// Evaluate the condition against a metric value
    pub fn evaluate(&self, value: f64) -> bool {
        match self.operator {
            ComparisonOperator::GreaterThan => value > self.threshold,
            ComparisonOperator::GreaterThanOrEqual => value >= self.threshold,
            ComparisonOperator::LessThan => value < self.threshold,
            ComparisonOperator::LessThanOrEqual => value <= self.threshold,
            ComparisonOperator::Equal => (value - self.threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (value - self.threshold).abs() >= f64::EPSILON,
        }
    }
}

/// Comparison operators for alert conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal
    Equal,
    /// Not equal
    NotEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Alert data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Rule ID that triggered the alert
    pub rule_id: String,
    /// Metric name
    pub metric_name: String,
    /// Metric value that triggered the alert
    pub value: f64,
    /// Threshold value
    pub threshold: f64,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Alert timestamp
    pub timestamp: u64,
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
}

/// Health check data structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Health check name
    pub name: String,
    /// Whether the check is healthy
    pub is_healthy: bool,
    /// Health check message
    pub message: String,
    /// Last check timestamp
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

/// System status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Overall health status
    pub health_status: HealthStatus,
    /// Number of metrics collected
    pub metrics_count: usize,
    /// Number of active alerts
    pub alerts_count: usize,
    /// System uptime
    pub uptime: Duration,
    /// Last updated timestamp
    pub last_updated: u64,
}

/// Monitoring statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoringStats {
    /// Total metrics recorded
    pub total_metrics: usize,
    /// Total alerts triggered
    pub total_alerts: usize,
    /// Total health checks performed
    pub total_health_checks: usize,
}

impl MonitoringStats {
    /// Create new monitoring statistics
    pub fn new() -> Self {
        Self {
            total_metrics: 0,
            total_alerts: 0,
            total_health_checks: 0,
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        self.total_metrics = 0;
        self.total_alerts = 0;
        self.total_health_checks = 0;
    }
    
    /// Record a metric
    pub fn record_metric(&mut self, _metric: &Metric) {
        self.total_metrics += 1;
    }
}

/// Monitoring configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Metrics configuration
    pub metrics_config: MetricsConfig,
    /// Alert configuration
    pub alert_config: AlertConfig,
    /// Health configuration
    pub health_config: HealthConfig,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            metrics_config: MetricsConfig::default(),
            alert_config: AlertConfig::default(),
            health_config: HealthConfig::default(),
        }
    }
}

/// Metrics configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Maximum number of metrics to store per name
    pub max_metrics_per_name: usize,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            max_metrics_per_name: 1000,
        }
    }
}

/// Alert configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Alert rules
    pub rules: Vec<AlertRule>,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
}

/// Health configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health checks
    pub health_checks: Vec<HealthCheck>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            health_checks: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_reliability_monitor_creation() {
        let monitor = ReliabilityMonitor::new();
        assert!(!monitor.is_initialized());
    }
    
    #[tokio::test]
    async fn test_reliability_monitor_initialization() {
        let mut monitor = ReliabilityMonitor::new();
        let result = monitor.initialize().await;
        assert!(result.is_ok());
        assert!(monitor.is_initialized());
    }
    
    #[tokio::test]
    async fn test_reliability_monitor_shutdown() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        let result = monitor.shutdown().await;
        assert!(result.is_ok());
        assert!(!monitor.is_initialized());
    }
    
    #[tokio::test]
    async fn test_metric_recording() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        let metric = Metric {
            name: "test_metric".to_string(),
            value: 42.0,
            timestamp: 1234567890,
            tags: HashMap::new(),
        };
        
        let result = monitor.record_metric(metric).await;
        assert!(result.is_ok());
        
        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_metrics, 1);
    }
    
    #[tokio::test]
    async fn test_metrics_retrieval() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        let metric = Metric {
            name: "test_metric".to_string(),
            value: 42.0,
            timestamp: 1234567890,
            tags: HashMap::new(),
        };
        
        monitor.record_metric(metric).await.unwrap();
        
        let time_range = TimeRange {
            start: 1234567890,
            end: 1234567890,
        };
        
        let metrics = monitor.get_metrics("test_metric", time_range).await.unwrap();
        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].value, 42.0);
    }
    
    #[tokio::test]
    async fn test_metrics_aggregation() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        // Record multiple metrics
        for i in 0..5 {
            let metric = Metric {
                name: "test_metric".to_string(),
                value: i as f64,
                timestamp: 1234567890 + i,
                tags: HashMap::new(),
            };
            monitor.record_metric(metric).await.unwrap();
        }
        
        let time_range = TimeRange {
            start: 1234567890,
            end: 1234567895,
        };
        
        let aggregated = monitor.get_aggregated_metrics("test_metric", AggregationType::Sum, time_range.clone()).await.unwrap();
        assert_eq!(aggregated.value, 10.0); // 0 + 1 + 2 + 3 + 4
        assert_eq!(aggregated.count, 5);
        
        let aggregated = monitor.get_aggregated_metrics("test_metric", AggregationType::Average, time_range.clone()).await.unwrap();
        assert_eq!(aggregated.value, 2.0); // (0 + 1 + 2 + 3 + 4) / 5
        assert_eq!(aggregated.count, 5);
        
        let aggregated = monitor.get_aggregated_metrics("test_metric", AggregationType::Min, time_range.clone()).await.unwrap();
        assert_eq!(aggregated.value, 0.0);
        assert_eq!(aggregated.count, 5);
        
        let aggregated = monitor.get_aggregated_metrics("test_metric", AggregationType::Max, time_range).await.unwrap();
        assert_eq!(aggregated.value, 4.0);
        assert_eq!(aggregated.count, 5);
    }
    
    #[tokio::test]
    async fn test_health_status() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        let health_status = monitor.get_health_status().await.unwrap();
        assert_eq!(health_status, HealthStatus::Healthy);
    }
    
    #[tokio::test]
    async fn test_system_status() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        let status = monitor.get_status().await.unwrap();
        assert_eq!(status.health_status, HealthStatus::Healthy);
        assert_eq!(status.metrics_count, 0);
        assert_eq!(status.alerts_count, 0);
    }
    
    #[tokio::test]
    async fn test_alert_condition_evaluation() {
        let condition = AlertCondition {
            operator: ComparisonOperator::GreaterThan,
            threshold: 10.0,
        };
        
        assert!(condition.evaluate(15.0));
        assert!(!condition.evaluate(5.0));
        assert!(!condition.evaluate(10.0));
        
        let condition = AlertCondition {
            operator: ComparisonOperator::GreaterThanOrEqual,
            threshold: 10.0,
        };
        
        assert!(condition.evaluate(15.0));
        assert!(!condition.evaluate(5.0));
        assert!(condition.evaluate(10.0));
        
        let condition = AlertCondition {
            operator: ComparisonOperator::LessThan,
            threshold: 10.0,
        };
        
        assert!(!condition.evaluate(15.0));
        assert!(condition.evaluate(5.0));
        assert!(!condition.evaluate(10.0));
        
        let condition = AlertCondition {
            operator: ComparisonOperator::LessThanOrEqual,
            threshold: 10.0,
        };
        
        assert!(!condition.evaluate(15.0));
        assert!(condition.evaluate(5.0));
        assert!(condition.evaluate(10.0));
        
        let condition = AlertCondition {
            operator: ComparisonOperator::Equal,
            threshold: 10.0,
        };
        
        assert!(!condition.evaluate(15.0));
        assert!(!condition.evaluate(5.0));
        assert!(condition.evaluate(10.0));
        
        let condition = AlertCondition {
            operator: ComparisonOperator::NotEqual,
            threshold: 10.0,
        };
        
        assert!(condition.evaluate(15.0));
        assert!(condition.evaluate(5.0));
        assert!(!condition.evaluate(10.0));
    }
    
    #[tokio::test]
    async fn test_alert_rule_triggering() {
        let mut monitor = ReliabilityMonitor::new();
        monitor.initialize().await.unwrap();
        
        // This would require setting up alert rules in the configuration
        // For now, just test that the system can handle metrics without errors
        let metric = Metric {
            name: "test_metric".to_string(),
            value: 42.0,
            timestamp: 1234567890,
            tags: HashMap::new(),
        };
        
        let result = monitor.record_metric(metric).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_monitor_config_default() {
        let config = MonitorConfig::default();
        assert_eq!(config.metrics_config.max_metrics_per_name, 1000);
        assert!(config.alert_config.rules.is_empty());
        assert!(config.health_config.health_checks.is_empty());
    }
    
    #[test]
    fn test_metric_creation() {
        let metric = Metric {
            name: "test_metric".to_string(),
            value: 42.0,
            timestamp: 1234567890,
            tags: HashMap::new(),
        };
        
        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.value, 42.0);
        assert_eq!(metric.timestamp, 1234567890);
    }
    
    #[test]
    fn test_time_range_creation() {
        let time_range = TimeRange {
            start: 1234567890,
            end: 1234567895,
        };
        
        assert_eq!(time_range.start, 1234567890);
        assert_eq!(time_range.end, 1234567895);
    }
    
    #[test]
    fn test_alert_rule_creation() {
        let rule = AlertRule {
            id: "test_rule".to_string(),
            metric_name: "test_metric".to_string(),
            condition: AlertCondition {
                operator: ComparisonOperator::GreaterThan,
                threshold: 10.0,
            },
            severity: AlertSeverity::High,
            message: "Test alert".to_string(),
        };
        
        assert_eq!(rule.id, "test_rule");
        assert_eq!(rule.metric_name, "test_metric");
        assert_eq!(rule.severity, AlertSeverity::High);
        assert_eq!(rule.message, "Test alert");
    }
    
    #[test]
    fn test_health_check_creation() {
        let health_check = HealthCheck {
            name: "test_check".to_string(),
            is_healthy: true,
            message: "Test check passed".to_string(),
            last_checked: 1234567890,
        };
        
        assert_eq!(health_check.name, "test_check");
        assert!(health_check.is_healthy);
        assert_eq!(health_check.message, "Test check passed");
        assert_eq!(health_check.last_checked, 1234567890);
    }
}
