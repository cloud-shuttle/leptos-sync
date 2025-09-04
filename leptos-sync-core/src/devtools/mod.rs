//! Developer Tools for debugging and monitoring Leptos-Sync
//!
//! This module provides comprehensive debugging and monitoring capabilities
//! for CRDTs, sync operations, and transport layer.

use crate::crdt::{CRDT, Mergeable, ReplicaId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// DevTools configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsConfig {
    /// Enable CRDT inspection
    pub enable_crdt_inspection: bool,
    /// Enable sync monitoring
    pub enable_sync_monitoring: bool,
    /// Enable transport monitoring
    pub enable_transport_monitoring: bool,
    /// Maximum number of events to keep in memory
    pub max_events: usize,
    /// Enable performance metrics
    pub enable_performance_metrics: bool,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            enable_crdt_inspection: true,
            enable_sync_monitoring: true,
            enable_transport_monitoring: true,
            max_events: 1000,
            enable_performance_metrics: true,
        }
    }
}

/// Event types that can be monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DevToolsEvent {
    /// CRDT operation performed
    CrdtOperation {
        crdt_id: String,
        operation: String,
        timestamp: u64,
        replica_id: ReplicaId,
    },
    /// Sync operation started/completed
    SyncOperation {
        operation_id: String,
        operation_type: String,
        status: String,
        timestamp: u64,
        duration_ms: Option<u64>,
    },
    /// Transport event
    TransportEvent {
        transport_type: String,
        event_type: String,
        timestamp: u64,
        details: String,
    },
    /// Performance metric
    PerformanceMetric {
        metric_name: String,
        value: f64,
        timestamp: u64,
        unit: String,
    },
}

/// CRDT inspection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtInspection {
    /// CRDT identifier
    pub id: String,
    /// CRDT type name
    pub type_name: String,
    /// Replica ID
    pub replica_id: ReplicaId,
    /// Current state summary
    pub state_summary: String,
    /// Number of operations performed
    pub operation_count: u64,
    /// Last operation timestamp
    pub last_operation_at: u64,
    /// Memory usage estimate
    pub memory_usage_bytes: usize,
}

/// Sync operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    /// Total sync operations
    pub total_operations: u64,
    /// Successful operations
    pub successful_operations: u64,
    /// Failed operations
    pub failed_operations: u64,
    /// Average operation duration
    pub avg_duration_ms: f64,
    /// Last sync timestamp
    pub last_sync_at: u64,
}

/// Transport statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportStats {
    /// Transport type
    pub transport_type: String,
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Connection status
    pub is_connected: bool,
    /// Last activity timestamp
    pub last_activity_at: u64,
    /// Error count
    pub error_count: u64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CRDT merge operations per second
    pub crdt_merges_per_second: f64,
    /// Sync operations per second
    pub sync_operations_per_second: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: usize,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
}

/// Complete DevTools data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsExport {
    /// All recorded events
    pub events: Vec<DevToolsEvent>,
    /// CRDT inspections
    pub crdt_inspections: HashMap<String, CrdtInspection>,
    /// Sync statistics
    pub sync_stats: SyncStats,
    /// Transport statistics
    pub transport_stats: HashMap<String, TransportStats>,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Configuration
    pub config: DevToolsConfig,
}

/// Main DevTools instance
pub struct DevTools {
    config: DevToolsConfig,
    events: Arc<RwLock<Vec<DevToolsEvent>>>,
    crdt_inspections: Arc<RwLock<HashMap<String, CrdtInspection>>>,
    sync_stats: Arc<RwLock<SyncStats>>,
    transport_stats: Arc<RwLock<HashMap<String, TransportStats>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl DevTools {
    /// Create a new DevTools instance
    pub fn new(config: DevToolsConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
            crdt_inspections: Arc::new(RwLock::new(HashMap::new())),
            sync_stats: Arc::new(RwLock::new(SyncStats {
                total_operations: 0,
                successful_operations: 0,
                failed_operations: 0,
                avg_duration_ms: 0.0,
                last_sync_at: 0,
            })),
            transport_stats: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics {
                crdt_merges_per_second: 0.0,
                sync_operations_per_second: 0.0,
                memory_usage_bytes: 0,
                cpu_usage_percent: 0.0,
            })),
        }
    }

    /// Record a CRDT operation
    pub async fn record_crdt_operation(&self, crdt_id: String, operation: String, replica_id: ReplicaId) {
        if !self.config.enable_crdt_inspection {
            return;
        }

        let event = DevToolsEvent::CrdtOperation {
            crdt_id: crdt_id.clone(),
            operation,
            timestamp: self.current_timestamp(),
            replica_id,
        };

        self.add_event(event).await;

        // Update CRDT inspection
        self.update_crdt_inspection(crdt_id, replica_id).await;
    }

    /// Record a sync operation
    pub async fn record_sync_operation(&self, operation_id: String, operation_type: String, status: String, duration_ms: Option<u64>) {
        if !self.config.enable_sync_monitoring {
            return;
        }

        let is_success = status == "success";
        let event = DevToolsEvent::SyncOperation {
            operation_id,
            operation_type,
            status,
            timestamp: self.current_timestamp(),
            duration_ms,
        };

        self.add_event(event).await;
        self.update_sync_stats(is_success, duration_ms).await;
    }

    /// Record a transport event
    pub async fn record_transport_event(&self, transport_type: String, event_type: String, details: String) {
        if !self.config.enable_transport_monitoring {
            return;
        }

        let event = DevToolsEvent::TransportEvent {
            transport_type: transport_type.clone(),
            event_type,
            timestamp: self.current_timestamp(),
            details,
        };

        self.add_event(event).await;
        self.update_transport_stats(transport_type).await;
    }

    /// Record a performance metric
    pub async fn record_performance_metric(&self, metric_name: String, value: f64, unit: String) {
        if !self.config.enable_performance_metrics {
            return;
        }

        let event = DevToolsEvent::PerformanceMetric {
            metric_name,
            value,
            timestamp: self.current_timestamp(),
            unit,
        };

        self.add_event(event).await;
    }

    /// Get all events
    pub async fn get_events(&self) -> Vec<DevToolsEvent> {
        self.events.read().await.clone()
    }

    /// Get CRDT inspections
    pub async fn get_crdt_inspections(&self) -> HashMap<String, CrdtInspection> {
        self.crdt_inspections.read().await.clone()
    }

    /// Get sync statistics
    pub async fn get_sync_stats(&self) -> SyncStats {
        self.sync_stats.read().await.clone()
    }

    /// Get transport statistics
    pub async fn get_transport_stats(&self) -> HashMap<String, TransportStats> {
        self.transport_stats.read().await.clone()
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// Clear all events
    pub async fn clear_events(&self) {
        let mut events = self.events.write().await;
        events.clear();
    }

    /// Get configuration
    pub fn config(&self) -> &DevToolsConfig {
        &self.config
    }

    /// Get events filtered by type
    pub async fn get_events_by_type(&self, event_type: &str) -> Vec<DevToolsEvent> {
        let events = self.events.read().await;
        events.iter()
            .filter(|event| match event {
                DevToolsEvent::CrdtOperation { .. } => event_type == "crdt_operation",
                DevToolsEvent::SyncOperation { .. } => event_type == "sync_operation",
                DevToolsEvent::TransportEvent { .. } => event_type == "transport_event",
                DevToolsEvent::PerformanceMetric { .. } => event_type == "performance_metric",
            })
            .cloned()
            .collect()
    }

    /// Get recent events (last N events)
    pub async fn get_recent_events(&self, count: usize) -> Vec<DevToolsEvent> {
        let events = self.events.read().await;
        let start = if events.len() > count {
            events.len() - count
        } else {
            0
        };
        events[start..].to_vec()
    }

    /// Get event count by type
    pub async fn get_event_counts(&self) -> HashMap<String, usize> {
        let events = self.events.read().await;
        let mut counts = HashMap::new();
        
        for event in events.iter() {
            let event_type = match event {
                DevToolsEvent::CrdtOperation { .. } => "crdt_operation",
                DevToolsEvent::SyncOperation { .. } => "sync_operation",
                DevToolsEvent::TransportEvent { .. } => "transport_event",
                DevToolsEvent::PerformanceMetric { .. } => "performance_metric",
            };
            
            *counts.entry(event_type.to_string()).or_insert(0) += 1;
        }
        
        counts
    }

    /// Export all data as JSON
    pub async fn export_data(&self) -> Result<String, serde_json::Error> {
        let data = DevToolsExport {
            events: self.get_events().await,
            crdt_inspections: self.get_crdt_inspections().await,
            sync_stats: self.get_sync_stats().await,
            transport_stats: self.get_transport_stats().await,
            performance_metrics: self.get_performance_metrics().await,
            config: self.config().clone(),
        };
        
        serde_json::to_string_pretty(&data)
    }

    // Private helper methods
    async fn add_event(&self, event: DevToolsEvent) {
        let mut events = self.events.write().await;
        events.push(event);
        
        // Trim events if we exceed max_events
        if events.len() > self.config.max_events {
            let excess = events.len() - self.config.max_events;
            events.drain(0..excess);
        }
    }

    async fn update_crdt_inspection(&self, crdt_id: String, replica_id: ReplicaId) {
        let mut inspections = self.crdt_inspections.write().await;
        let inspection = inspections.entry(crdt_id.clone()).or_insert(CrdtInspection {
            id: crdt_id,
            type_name: "Unknown".to_string(),
            replica_id,
            state_summary: "Unknown".to_string(),
            operation_count: 0,
            last_operation_at: 0,
            memory_usage_bytes: 0,
        });
        
        inspection.operation_count += 1;
        inspection.last_operation_at = self.current_timestamp();
    }

    async fn update_sync_stats(&self, success: bool, duration_ms: Option<u64>) {
        let mut stats = self.sync_stats.write().await;
        stats.total_operations += 1;
        
        if success {
            stats.successful_operations += 1;
        } else {
            stats.failed_operations += 1;
        }
        
        if let Some(duration) = duration_ms {
            // Update average duration
            let total_duration = stats.avg_duration_ms * (stats.total_operations - 1) as f64 + duration as f64;
            stats.avg_duration_ms = total_duration / stats.total_operations as f64;
        }
        
        stats.last_sync_at = self.current_timestamp();
    }

    async fn update_transport_stats(&self, transport_type: String) {
        let mut stats = self.transport_stats.write().await;
        let transport_type_clone = transport_type.clone();
        let transport_stats = stats.entry(transport_type).or_insert(TransportStats {
            transport_type: transport_type_clone,
            messages_sent: 0,
            messages_received: 0,
            is_connected: false,
            last_activity_at: 0,
            error_count: 0,
        });
        
        transport_stats.last_activity_at = self.current_timestamp();
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

/// CRDT Inspector for detailed CRDT analysis
pub struct CrdtInspector {
    devtools: Arc<DevTools>,
}

impl CrdtInspector {
    /// Create a new CRDT inspector
    pub fn new(devtools: Arc<DevTools>) -> Self {
        Self { devtools }
    }

    /// Inspect a CRDT and return detailed information
    pub async fn inspect_crdt<T: CRDT + Mergeable>(&self, crdt: &T, crdt_id: String) -> CrdtInspection {
        let replica_id = crdt.replica_id().clone();
        let type_name = std::any::type_name::<T>().to_string();
        
        // This would be implemented with actual CRDT inspection logic
        let state_summary = format!("CRDT of type {}", type_name);
        let memory_usage_bytes = std::mem::size_of_val(crdt);
        
        let inspection = CrdtInspection {
            id: crdt_id.clone(),
            type_name,
            replica_id,
            state_summary,
            operation_count: 0, // Would be tracked by the CRDT
            last_operation_at: 0,
            memory_usage_bytes,
        };

        // Record the inspection
        let mut inspections = self.devtools.crdt_inspections.write().await;
        inspections.insert(crdt_id, inspection.clone());
        
        inspection
    }

    /// Get all CRDT inspections
    pub async fn get_all_inspections(&self) -> HashMap<String, CrdtInspection> {
        self.devtools.get_crdt_inspections().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::{LwwRegister, ReplicaId};
    use uuid::Uuid;

    fn create_replica(id: u64) -> ReplicaId {
        ReplicaId::from(Uuid::from_u64_pair(0, id))
    }

    #[tokio::test]
    async fn test_devtools_creation() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        assert!(devtools.config().enable_crdt_inspection);
        assert!(devtools.config().enable_sync_monitoring);
        assert!(devtools.config().enable_transport_monitoring);
        assert_eq!(devtools.config().max_events, 1000);
    }

    #[tokio::test]
    async fn test_record_crdt_operation() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        let replica_id = create_replica(1);
        
        devtools.record_crdt_operation("test-crdt".to_string(), "add".to_string(), replica_id).await;
        
        let events = devtools.get_events().await;
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            DevToolsEvent::CrdtOperation { crdt_id, operation, replica_id: recorded_replica_id, .. } => {
                assert_eq!(crdt_id, "test-crdt");
                assert_eq!(operation, "add");
                assert_eq!(recorded_replica_id, &replica_id);
            }
            _ => panic!("Expected CrdtOperation event"),
        }
    }

    #[tokio::test]
    async fn test_record_sync_operation() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        devtools.record_sync_operation(
            "sync-1".to_string(),
            "merge".to_string(),
            "success".to_string(),
            Some(150)
        ).await;
        
        let events = devtools.get_events().await;
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            DevToolsEvent::SyncOperation { operation_id, operation_type, status, duration_ms, .. } => {
                assert_eq!(operation_id, "sync-1");
                assert_eq!(operation_type, "merge");
                assert_eq!(status, "success");
                assert_eq!(duration_ms, &Some(150));
            }
            _ => panic!("Expected SyncOperation event"),
        }
    }

    #[tokio::test]
    async fn test_record_transport_event() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        devtools.record_transport_event(
            "websocket".to_string(),
            "connected".to_string(),
            "Connected to server".to_string()
        ).await;
        
        let events = devtools.get_events().await;
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            DevToolsEvent::TransportEvent { transport_type, event_type, details, .. } => {
                assert_eq!(transport_type, "websocket");
                assert_eq!(event_type, "connected");
                assert_eq!(details, "Connected to server");
            }
            _ => panic!("Expected TransportEvent"),
        }
    }

    #[tokio::test]
    async fn test_record_performance_metric() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        devtools.record_performance_metric(
            "memory_usage".to_string(),
            1024.0,
            "bytes".to_string()
        ).await;
        
        let events = devtools.get_events().await;
        assert_eq!(events.len(), 1);
        
        match &events[0] {
            DevToolsEvent::PerformanceMetric { metric_name, value, unit, .. } => {
                assert_eq!(metric_name, "memory_usage");
                assert_eq!(*value, 1024.0);
                assert_eq!(unit, "bytes");
            }
            _ => panic!("Expected PerformanceMetric event"),
        }
    }

    #[tokio::test]
    async fn test_sync_stats_tracking() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        // Record successful operation
        devtools.record_sync_operation(
            "sync-1".to_string(),
            "merge".to_string(),
            "success".to_string(),
            Some(100)
        ).await;
        
        // Record failed operation
        devtools.record_sync_operation(
            "sync-2".to_string(),
            "merge".to_string(),
            "failed".to_string(),
            Some(200)
        ).await;
        
        let stats = devtools.get_sync_stats().await;
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.failed_operations, 1);
        assert_eq!(stats.avg_duration_ms, 150.0); // (100 + 200) / 2
    }

    #[tokio::test]
    async fn test_event_limit() {
        let config = DevToolsConfig {
            max_events: 3,
            ..Default::default()
        };
        let devtools = DevTools::new(config);
        
        // Add more events than the limit
        for i in 0..5 {
            devtools.record_crdt_operation(
                format!("crdt-{}", i),
                "add".to_string(),
                create_replica(1)
            ).await;
        }
        
        let events = devtools.get_events().await;
        assert_eq!(events.len(), 3); // Should be limited to max_events
    }

    #[tokio::test]
    async fn test_clear_events() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        // Add some events
        devtools.record_crdt_operation("test".to_string(), "add".to_string(), create_replica(1)).await;
        assert_eq!(devtools.get_events().await.len(), 1);
        
        // Clear events
        devtools.clear_events().await;
        assert_eq!(devtools.get_events().await.len(), 0);
    }

    #[tokio::test]
    async fn test_crdt_inspector() {
        let config = DevToolsConfig::default();
        let devtools = Arc::new(DevTools::new(config));
        let inspector = CrdtInspector::new(devtools.clone());
        
        let replica_id = create_replica(1);
        let crdt = LwwRegister::new("test".to_string(), replica_id.clone());
        
        let inspection = inspector.inspect_crdt(&crdt, "test-crdt".to_string()).await;
        
        assert_eq!(inspection.id, "test-crdt");
        assert!(inspection.type_name.contains("LwwRegister"));
        assert_eq!(inspection.replica_id, replica_id);
        assert!(inspection.memory_usage_bytes > 0);
        
        // Check that inspection was stored
        let inspections = inspector.get_all_inspections().await;
        assert_eq!(inspections.len(), 1);
        assert!(inspections.contains_key("test-crdt"));
    }

    #[tokio::test]
    async fn test_disabled_features() {
        let config = DevToolsConfig {
            enable_crdt_inspection: false,
            enable_sync_monitoring: false,
            enable_transport_monitoring: false,
            enable_performance_metrics: false,
            ..Default::default()
        };
        let devtools = DevTools::new(config);
        
        // These should not record events
        devtools.record_crdt_operation("test".to_string(), "add".to_string(), create_replica(1)).await;
        devtools.record_sync_operation("test".to_string(), "merge".to_string(), "success".to_string(), None).await;
        devtools.record_transport_event("test".to_string(), "connected".to_string(), "test".to_string()).await;
        devtools.record_performance_metric("test".to_string(), 1.0, "test".to_string()).await;
        
        assert_eq!(devtools.get_events().await.len(), 0);
    }

    #[tokio::test]
    async fn test_get_events_by_type() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        // Add different types of events
        devtools.record_crdt_operation("test".to_string(), "add".to_string(), create_replica(1)).await;
        devtools.record_sync_operation("test".to_string(), "merge".to_string(), "success".to_string(), None).await;
        devtools.record_transport_event("test".to_string(), "connected".to_string(), "test".to_string()).await;
        devtools.record_performance_metric("test".to_string(), 1.0, "test".to_string()).await;
        
        // Test filtering by type
        let crdt_events = devtools.get_events_by_type("crdt_operation").await;
        assert_eq!(crdt_events.len(), 1);
        
        let sync_events = devtools.get_events_by_type("sync_operation").await;
        assert_eq!(sync_events.len(), 1);
        
        let transport_events = devtools.get_events_by_type("transport_event").await;
        assert_eq!(transport_events.len(), 1);
        
        let perf_events = devtools.get_events_by_type("performance_metric").await;
        assert_eq!(perf_events.len(), 1);
    }

    #[tokio::test]
    async fn test_get_recent_events() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        // Add 5 events
        for i in 0..5 {
            devtools.record_crdt_operation(format!("test-{}", i), "add".to_string(), create_replica(1)).await;
        }
        
        // Get last 3 events
        let recent = devtools.get_recent_events(3).await;
        assert_eq!(recent.len(), 3);
        
        // Get more events than we have
        let all_recent = devtools.get_recent_events(10).await;
        assert_eq!(all_recent.len(), 5);
    }

    #[tokio::test]
    async fn test_get_event_counts() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        // Add different types of events
        devtools.record_crdt_operation("test".to_string(), "add".to_string(), create_replica(1)).await;
        devtools.record_crdt_operation("test2".to_string(), "remove".to_string(), create_replica(1)).await;
        devtools.record_sync_operation("test".to_string(), "merge".to_string(), "success".to_string(), None).await;
        devtools.record_transport_event("test".to_string(), "connected".to_string(), "test".to_string()).await;
        
        let counts = devtools.get_event_counts().await;
        assert_eq!(counts.get("crdt_operation"), Some(&2));
        assert_eq!(counts.get("sync_operation"), Some(&1));
        assert_eq!(counts.get("transport_event"), Some(&1));
        // Performance metric count should be 0 since we didn't add any
        assert_eq!(counts.get("performance_metric"), None);
    }

    #[tokio::test]
    async fn test_export_data() {
        let config = DevToolsConfig::default();
        let devtools = DevTools::new(config);
        
        // Add some data
        devtools.record_crdt_operation("test".to_string(), "add".to_string(), create_replica(1)).await;
        devtools.record_sync_operation("test".to_string(), "merge".to_string(), "success".to_string(), None).await;
        
        // Export data
        let export_json = devtools.export_data().await.unwrap();
        assert!(export_json.contains("test"));
        assert!(export_json.contains("CrdtOperation"));
        assert!(export_json.contains("SyncOperation"));
        
        // Verify it's valid JSON
        let parsed: DevToolsExport = serde_json::from_str(&export_json).unwrap();
        assert_eq!(parsed.events.len(), 2);
        assert_eq!(parsed.config.max_events, 1000);
    }
}
