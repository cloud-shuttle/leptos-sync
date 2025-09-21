//! Alert management and notification system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Unique identifier for the rule
    pub id: String,
    /// Human-readable name for the rule
    pub name: String,
    /// Description of what this rule monitors
    pub description: String,
    /// Metric name to monitor
    pub metric_name: String,
    /// Condition that triggers the alert
    pub condition: AlertCondition,
    /// Severity level of the alert
    pub severity: AlertSeverity,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Cooldown period between alerts (in seconds)
    pub cooldown_seconds: u64,
}

impl AlertRule {
    /// Create a new alert rule
    pub fn new(
        id: String,
        name: String,
        metric_name: String,
        condition: AlertCondition,
        severity: AlertSeverity,
    ) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            metric_name,
            condition,
            severity,
            enabled: true,
            cooldown_seconds: 300, // 5 minutes default
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Set the cooldown period
    pub fn with_cooldown(mut self, cooldown_seconds: u64) -> Self {
        self.cooldown_seconds = cooldown_seconds;
        self
    }

    /// Enable or disable the rule
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Alert condition for triggering alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Threshold value
    pub threshold: f64,
    /// Duration the condition must be true before triggering (in seconds)
    pub duration_seconds: u64,
}

impl AlertCondition {
    /// Create a new alert condition
    pub fn new(operator: ComparisonOperator, threshold: f64, duration_seconds: u64) -> Self {
        Self {
            operator,
            threshold,
            duration_seconds,
        }
    }

    /// Check if a value satisfies this condition
    pub fn is_satisfied(&self, value: f64) -> bool {
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal to
    Equal,
    /// Not equal to
    NotEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - warning
    Medium,
    /// High severity - error
    High,
    /// Critical severity - system failure
    Critical,
}

/// An active alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique identifier for the alert
    pub id: String,
    /// Rule that triggered this alert
    pub rule_id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Timestamp when the alert was triggered
    pub triggered_at: u64,
    /// Timestamp when the alert was resolved (if resolved)
    pub resolved_at: Option<u64>,
    /// Whether the alert is currently active
    pub is_active: bool,
    /// Additional context data
    pub context: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        id: String,
        rule_id: String,
        severity: AlertSeverity,
        message: String,
    ) -> Self {
        Self {
            id,
            rule_id,
            severity,
            message,
            triggered_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            resolved_at: None,
            is_active: true,
            context: HashMap::new(),
        }
    }

    /// Add context data to the alert
    pub fn with_context(mut self, context: HashMap<String, String>) -> Self {
        self.context = context;
        self
    }

    /// Resolve the alert
    pub fn resolve(&mut self) {
        self.is_active = false;
        self.resolved_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    /// Get the duration the alert has been active
    pub fn duration(&self) -> Duration {
        let end_time = self.resolved_at.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        Duration::from_secs(end_time - self.triggered_at)
    }
}

/// Alert manager for handling alert rules and notifications
#[derive(Debug, Clone)]
pub struct AlertManager {
    /// Alert rules
    rules: HashMap<String, AlertRule>,
    /// Active alerts
    active_alerts: HashMap<String, Alert>,
    /// Alert history
    alert_history: Vec<Alert>,
    /// Maximum number of alerts to keep in history
    max_history_size: usize,
    /// Last trigger times for cooldown tracking
    last_trigger_times: HashMap<String, u64>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            active_alerts: HashMap::new(),
            alert_history: Vec::new(),
            max_history_size: 1000,
            last_trigger_times: HashMap::new(),
        }
    }

    /// Create an alert manager with configuration
    pub fn with_config(config: AlertConfig) -> Self {
        Self {
            rules: HashMap::new(),
            active_alerts: HashMap::new(),
            alert_history: Vec::new(),
            max_history_size: config.max_history_size,
            last_trigger_times: HashMap::new(),
        }
    }

    /// Add an alert rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Remove an alert rule
    pub fn remove_rule(&mut self, rule_id: &str) {
        self.rules.remove(rule_id);
        // Also remove any active alerts for this rule
        self.active_alerts.retain(|_, alert| alert.rule_id != rule_id);
    }

    /// Update an alert rule
    pub fn update_rule(&mut self, rule: AlertRule) {
        self.rules.insert(rule.id.clone(), rule);
    }

    /// Get all alert rules
    pub fn get_rules(&self) -> Vec<&AlertRule> {
        self.rules.values().collect()
    }

    /// Get a specific alert rule
    pub fn get_rule(&self, rule_id: &str) -> Option<&AlertRule> {
        self.rules.get(rule_id)
    }

    /// Check if a metric value should trigger an alert
    pub fn check_metric(&mut self, metric_name: &str, value: f64) -> Vec<Alert> {
        let mut new_alerts = Vec::new();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Find rules that match this metric
        let rule_ids: Vec<String> = self.rules.keys().cloned().collect();
        let mut rules_to_resolve = Vec::new();
        
        for rule_id in rule_ids {
            if let Some(rule) = self.rules.get(&rule_id) {
                if !rule.enabled || rule.metric_name != metric_name {
                    continue;
                }

                // Check cooldown
                if let Some(last_trigger) = self.last_trigger_times.get(&rule.id) {
                    if current_time - last_trigger < rule.cooldown_seconds {
                        continue;
                    }
                }

                // Check if condition is satisfied
                if rule.condition.is_satisfied(value) {
                    // Check if there's already an active alert for this rule
                    let has_active_alert = self.active_alerts.values()
                        .any(|alert| alert.rule_id == rule.id && alert.is_active);

                    if !has_active_alert {
                        // Create new alert
                        let alert_id = format!("{}_{}", rule.id, current_time);
                        let message = format!(
                            "Alert triggered: {} (value: {}, threshold: {})",
                            rule.name, value, rule.condition.threshold
                        );

                        let mut alert = Alert::new(
                            alert_id,
                            rule.id.clone(),
                            rule.severity.clone(),
                            message,
                        );

                        // Add context
                        let mut context = HashMap::new();
                        context.insert("metric_name".to_string(), metric_name.to_string());
                        context.insert("value".to_string(), value.to_string());
                        context.insert("threshold".to_string(), rule.condition.threshold.to_string());
                        alert.context = context;

                        self.active_alerts.insert(alert.id.clone(), alert.clone());
                        self.alert_history.push(alert.clone());
                        self.last_trigger_times.insert(rule.id.clone(), current_time);
                        new_alerts.push(alert);
                    }
                } else {
                    // Condition not satisfied, mark for resolution
                    rules_to_resolve.push(rule.id.clone());
                }
            }
        }
        
        // Resolve alerts for rules that no longer match
        for rule_id in rules_to_resolve {
            self.resolve_alerts_for_rule(&rule_id);
        }

        // Clean up old history
        self.cleanup_history();

        new_alerts
    }

    /// Get all active alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        self.active_alerts.values().filter(|alert| alert.is_active).collect()
    }

    /// Get alerts by severity
    pub fn get_alerts_by_severity(&self, severity: &AlertSeverity) -> Vec<&Alert> {
        self.active_alerts
            .values()
            .filter(|alert| alert.is_active && &alert.severity == severity)
            .collect()
    }

    /// Resolve an alert by ID
    pub fn resolve_alert(&mut self, alert_id: &str) -> bool {
        if let Some(alert) = self.active_alerts.get_mut(alert_id) {
            alert.resolve();
            true
        } else {
            false
        }
    }

    /// Resolve all alerts for a specific rule
    fn resolve_alerts_for_rule(&mut self, rule_id: &str) {
        for alert in self.active_alerts.values_mut() {
            if alert.rule_id == rule_id && alert.is_active {
                alert.resolve();
            }
        }
    }

    /// Get alert history
    pub fn get_alert_history(&self, limit: Option<usize>) -> Vec<&Alert> {
        let limit = limit.unwrap_or(self.alert_history.len());
        self.alert_history
            .iter()
            .rev()
            .take(limit)
            .collect()
    }

    /// Clear alert history
    pub fn clear_history(&mut self) {
        self.alert_history.clear();
    }

    /// Clean up old history entries
    fn cleanup_history(&mut self) {
        if self.alert_history.len() > self.max_history_size {
            let excess = self.alert_history.len() - self.max_history_size;
            self.alert_history.drain(0..excess);
        }
    }

    /// Get alert statistics
    pub fn get_stats(&self) -> AlertStats {
        let active_count = self.active_alerts.values().filter(|a| a.is_active).count();
        let critical_count = self.get_alerts_by_severity(&AlertSeverity::Critical).len();
        let high_count = self.get_alerts_by_severity(&AlertSeverity::High).len();
        let medium_count = self.get_alerts_by_severity(&AlertSeverity::Medium).len();
        let low_count = self.get_alerts_by_severity(&AlertSeverity::Low).len();

        AlertStats {
            total_rules: self.rules.len(),
            active_alerts: active_count,
            critical_alerts: critical_count,
            high_alerts: high_count,
            medium_alerts: medium_count,
            low_alerts: low_count,
            total_history: self.alert_history.len(),
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    /// Total number of alert rules
    pub total_rules: usize,
    /// Number of active alerts
    pub active_alerts: usize,
    /// Number of critical alerts
    pub critical_alerts: usize,
    /// Number of high severity alerts
    pub high_alerts: usize,
    /// Number of medium severity alerts
    pub medium_alerts: usize,
    /// Number of low severity alerts
    pub low_alerts: usize,
    /// Total number of alerts in history
    pub total_history: usize,
}

/// Configuration for alert management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Maximum number of alerts to keep in history
    pub max_history_size: usize,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_rule_creation() {
        let condition = AlertCondition::new(ComparisonOperator::GreaterThan, 80.0, 60);
        let rule = AlertRule::new(
            "cpu_high".to_string(),
            "High CPU Usage".to_string(),
            "cpu_usage".to_string(),
            condition,
            AlertSeverity::High,
        );

        assert_eq!(rule.id, "cpu_high");
        assert_eq!(rule.metric_name, "cpu_usage");
        assert!(rule.enabled);
    }

    #[test]
    fn test_alert_condition() {
        let condition = AlertCondition::new(ComparisonOperator::GreaterThan, 80.0, 60);
        
        assert!(condition.is_satisfied(85.0));
        assert!(!condition.is_satisfied(75.0));
        assert!(!condition.is_satisfied(80.0));
    }

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(
            "alert_1".to_string(),
            "rule_1".to_string(),
            AlertSeverity::High,
            "Test alert".to_string(),
        );

        assert_eq!(alert.id, "alert_1");
        assert!(alert.is_active);
        assert!(alert.resolved_at.is_none());
    }

    #[test]
    fn test_alert_resolution() {
        let mut alert = Alert::new(
            "alert_1".to_string(),
            "rule_1".to_string(),
            AlertSeverity::High,
            "Test alert".to_string(),
        );

        assert!(alert.is_active);
        alert.resolve();
        assert!(!alert.is_active);
        assert!(alert.resolved_at.is_some());
    }

    #[test]
    fn test_alert_manager() {
        let mut manager = AlertManager::new();
        
        // Add a rule
        let condition = AlertCondition::new(ComparisonOperator::GreaterThan, 80.0, 60);
        let rule = AlertRule::new(
            "cpu_high".to_string(),
            "High CPU Usage".to_string(),
            "cpu_usage".to_string(),
            condition,
            AlertSeverity::High,
        );
        manager.add_rule(rule);

        // Check metric that should trigger alert
        let alerts = manager.check_metric("cpu_usage", 85.0);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::High);

        // Check that we have an active alert
        let active_alerts = manager.get_active_alerts();
        assert_eq!(active_alerts.len(), 1);

        // Check metric that should not trigger alert
        let alerts = manager.check_metric("cpu_usage", 75.0);
        assert_eq!(alerts.len(), 0);

        // Check that the alert was resolved
        let active_alerts = manager.get_active_alerts();
        assert_eq!(active_alerts.len(), 0);
    }

    #[test]
    fn test_alert_cooldown() {
        let mut manager = AlertManager::new();
        
        let condition = AlertCondition::new(ComparisonOperator::GreaterThan, 80.0, 60);
        let rule = AlertRule::new(
            "cpu_high".to_string(),
            "High CPU Usage".to_string(),
            "cpu_usage".to_string(),
            condition,
            AlertSeverity::High,
        ).with_cooldown(300); // 5 minutes
        manager.add_rule(rule);

        // First trigger should create alert
        let alerts = manager.check_metric("cpu_usage", 85.0);
        assert_eq!(alerts.len(), 1);

        // Second trigger within cooldown should not create alert
        let alerts = manager.check_metric("cpu_usage", 90.0);
        assert_eq!(alerts.len(), 0);
    }
}
