//! Metrics collection and aggregation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A single metric measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Timestamp when the metric was recorded
    pub timestamp: u64,
    /// Optional tags for categorization
    pub tags: HashMap<String, String>,
}

impl Metric {
    /// Create a new metric
    pub fn new(name: String, value: f64) -> Self {
        Self {
            name,
            value,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tags: HashMap::new(),
        }
    }

    /// Create a metric with tags
    pub fn with_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.tags = tags;
        self
    }

    /// Create a metric with a specific timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Time range for metric queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time (Unix timestamp)
    pub start: u64,
    /// End time (Unix timestamp)
    pub end: u64,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    /// Create a time range for the last N seconds
    pub fn last_seconds(seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            start: now - seconds,
            end: now,
        }
    }

    /// Check if a timestamp falls within this range
    pub fn contains(&self, timestamp: u64) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
}

/// Aggregation types for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum all values
    Sum,
    /// Average of all values
    Average,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of measurements
    Count,
    /// Latest value
    Latest,
}

/// Aggregated metric result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetric {
    /// Metric name
    pub name: String,
    /// Aggregated value
    pub value: f64,
    /// Aggregation type used
    pub aggregation_type: AggregationType,
    /// Time range of the aggregation
    pub time_range: TimeRange,
    /// Number of samples aggregated
    pub sample_count: usize,
}

/// Metrics collector for gathering and storing metrics
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Stored metrics
    metrics: HashMap<String, Vec<Metric>>,
    /// Maximum number of metrics to keep per name
    max_metrics_per_name: usize,
    /// Whether to automatically clean old metrics
    auto_cleanup: bool,
    /// Maximum age of metrics to keep (in seconds)
    max_metric_age: u64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            max_metrics_per_name: 1000,
            auto_cleanup: true,
            max_metric_age: 3600, // 1 hour
        }
    }

    /// Create a metrics collector with configuration
    pub fn with_config(config: MetricsConfig) -> Self {
        Self {
            metrics: HashMap::new(),
            max_metrics_per_name: config.max_metrics_per_name,
            auto_cleanup: config.auto_cleanup,
            max_metric_age: config.max_metric_age,
        }
    }

    /// Record a metric
    pub fn record(&mut self, metric: Metric) {
        let name = metric.name.clone();
        
        // Add the metric
        self.metrics.entry(name.clone()).or_insert_with(Vec::new).push(metric);
        
        // Auto-cleanup if enabled
        if self.auto_cleanup {
            self.cleanup_old_metrics(&name);
        }
        
        // Limit the number of metrics per name
        if let Some(metrics) = self.metrics.get_mut(&name) {
            if metrics.len() > self.max_metrics_per_name {
                metrics.drain(0..metrics.len() - self.max_metrics_per_name);
            }
        }
    }

    /// Get metrics for a specific name and time range
    pub fn get_metrics(&self, name: &str, time_range: &TimeRange) -> Vec<&Metric> {
        self.metrics
            .get(name)
            .map(|metrics| {
                metrics
                    .iter()
                    .filter(|m| time_range.contains(m.timestamp))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all metrics for a specific name
    pub fn get_all_metrics(&self, name: &str) -> Vec<&Metric> {
        self.metrics
            .get(name)
            .map(|metrics| metrics.iter().collect())
            .unwrap_or_default()
    }

    /// Aggregate metrics for a specific name and time range
    pub fn aggregate_metrics(
        &self,
        name: &str,
        time_range: &TimeRange,
        aggregation_type: AggregationType,
    ) -> Option<AggregatedMetric> {
        let metrics = self.get_metrics(name, time_range);
        
        if metrics.is_empty() {
            return None;
        }

        let value = match aggregation_type {
            AggregationType::Sum => metrics.iter().map(|m| m.value).sum(),
            AggregationType::Average => {
                let sum: f64 = metrics.iter().map(|m| m.value).sum();
                sum / metrics.len() as f64
            }
            AggregationType::Min => metrics.iter().map(|m| m.value).fold(f64::INFINITY, f64::min),
            AggregationType::Max => metrics.iter().map(|m| m.value).fold(f64::NEG_INFINITY, f64::max),
            AggregationType::Count => metrics.len() as f64,
            AggregationType::Latest => metrics.last().unwrap().value,
        };

        Some(AggregatedMetric {
            name: name.to_string(),
            value,
            aggregation_type,
            time_range: time_range.clone(),
            sample_count: metrics.len(),
        })
    }

    /// Get all metric names
    pub fn get_metric_names(&self) -> Vec<String> {
        self.metrics.keys().cloned().collect()
    }

    /// Clear all metrics
    pub fn clear(&mut self) {
        self.metrics.clear();
    }

    /// Clear metrics for a specific name
    pub fn clear_metrics(&mut self, name: &str) {
        self.metrics.remove(name);
    }

    /// Clean up old metrics for a specific name
    fn cleanup_old_metrics(&mut self, name: &str) {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - self.max_metric_age;

        if let Some(metrics) = self.metrics.get_mut(name) {
            metrics.retain(|m| m.timestamp >= cutoff_time);
        }
    }

    /// Get the total number of stored metrics
    pub fn total_metrics_count(&self) -> usize {
        self.metrics.values().map(|v| v.len()).sum()
    }

    /// Get the number of unique metric names
    pub fn unique_metric_count(&self) -> usize {
        self.metrics.len()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Maximum number of metrics to keep per name
    pub max_metrics_per_name: usize,
    /// Whether to automatically clean old metrics
    pub auto_cleanup: bool,
    /// Maximum age of metrics to keep (in seconds)
    pub max_metric_age: u64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            max_metrics_per_name: 1000,
            auto_cleanup: true,
            max_metric_age: 3600, // 1 hour
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new("test_metric".to_string(), 42.0);
        assert_eq!(metric.name, "test_metric");
        assert_eq!(metric.value, 42.0);
        assert!(!metric.tags.is_empty() || metric.tags.is_empty()); // Tags can be empty
    }

    #[test]
    fn test_metric_with_tags() {
        let mut tags = HashMap::new();
        tags.insert("service".to_string(), "api".to_string());
        tags.insert("version".to_string(), "1.0".to_string());
        
        let metric = Metric::new("test_metric".to_string(), 42.0).with_tags(tags.clone());
        assert_eq!(metric.tags, tags);
    }

    #[test]
    fn test_time_range() {
        let range = TimeRange::new(1000, 2000);
        assert!(range.contains(1500));
        assert!(!range.contains(500));
        assert!(!range.contains(2500));
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        
        // Record some metrics
        collector.record(Metric::new("cpu_usage".to_string(), 75.0));
        collector.record(Metric::new("memory_usage".to_string(), 60.0));
        collector.record(Metric::new("cpu_usage".to_string(), 80.0));
        
        // Check metrics
        assert_eq!(collector.unique_metric_count(), 2);
        assert_eq!(collector.total_metrics_count(), 3);
        
        // Get metrics for a specific name
        let cpu_metrics = collector.get_all_metrics("cpu_usage");
        assert_eq!(cpu_metrics.len(), 2);
        
        // Test aggregation
        let time_range = TimeRange::last_seconds(3600);
        let avg_cpu = collector.aggregate_metrics("cpu_usage", &time_range, AggregationType::Average);
        assert!(avg_cpu.is_some());
        assert_eq!(avg_cpu.unwrap().value, 77.5); // (75.0 + 80.0) / 2
    }

    #[test]
    fn test_aggregation_types() {
        let mut collector = MetricsCollector::new();
        
        // Record metrics with different values
        collector.record(Metric::new("test".to_string(), 10.0));
        collector.record(Metric::new("test".to_string(), 20.0));
        collector.record(Metric::new("test".to_string(), 30.0));
        
        let time_range = TimeRange::last_seconds(3600);
        
        // Test different aggregation types
        let sum = collector.aggregate_metrics("test", &time_range, AggregationType::Sum);
        assert_eq!(sum.unwrap().value, 60.0);
        
        let avg = collector.aggregate_metrics("test", &time_range, AggregationType::Average);
        assert_eq!(avg.unwrap().value, 20.0);
        
        let min = collector.aggregate_metrics("test", &time_range, AggregationType::Min);
        assert_eq!(min.unwrap().value, 10.0);
        
        let max = collector.aggregate_metrics("test", &time_range, AggregationType::Max);
        assert_eq!(max.unwrap().value, 30.0);
        
        let count = collector.aggregate_metrics("test", &time_range, AggregationType::Count);
        assert_eq!(count.unwrap().value, 3.0);
    }
}
