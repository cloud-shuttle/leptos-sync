use leptos_sync_core::{
    DevTools, DevToolsConfig, CrdtInspector,
    crdt::{LwwRegister, LwwMap, GCounter, ReplicaId},
    MultiTransport, MultiTransportConfig, TransportType, TransportEnum,
    transport::InMemoryTransport,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Leptos-Sync DevTools Demo");
    println!("============================\n");

    // Create DevTools with comprehensive monitoring
    let devtools_config = DevToolsConfig {
        enable_crdt_inspection: true,
        enable_sync_monitoring: true,
        enable_transport_monitoring: true,
        enable_performance_metrics: true,
        max_events: 50,
    };
    
    let devtools = Arc::new(DevTools::new(devtools_config));
    let inspector = CrdtInspector::new(devtools.clone());

    // Demo 1: CRDT Operations Monitoring
    println!("📊 Demo 1: CRDT Operations Monitoring");
    println!("-------------------------------------");
    
    let replica_id = ReplicaId::from(Uuid::new_v4());
    
    // Create and monitor CRDTs
    let mut register = LwwRegister::new("initial_value".to_string(), replica_id.clone());
    let mut counter = GCounter::new();
    let mut map = LwwMap::new();
    
    // Record CRDT operations
    devtools.record_crdt_operation("register-1".to_string(), "create".to_string(), replica_id.clone()).await;
    devtools.record_crdt_operation("counter-1".to_string(), "create".to_string(), replica_id.clone()).await;
    devtools.record_crdt_operation("map-1".to_string(), "create".to_string(), replica_id.clone()).await;
    
    // Simulate some operations
    register.update("updated_value".to_string(), replica_id.clone());
    devtools.record_crdt_operation("register-1".to_string(), "update".to_string(), replica_id.clone()).await;
    
    counter.increment(replica_id.clone());
    devtools.record_crdt_operation("counter-1".to_string(), "increment".to_string(), replica_id.clone()).await;
    
    map.insert("key1".to_string(), "value1".to_string(), replica_id.clone());
    devtools.record_crdt_operation("map-1".to_string(), "insert".to_string(), replica_id.clone()).await;
    
    // Inspect CRDTs
    let register_inspection = inspector.inspect_crdt(&register, "register-1".to_string()).await;
    let counter_inspection = inspector.inspect_crdt(&counter, "counter-1".to_string()).await;
    let map_inspection = inspector.inspect_crdt(&map, "map-1".to_string()).await;
    
    println!("Register Inspection: {:?}", register_inspection);
    println!("Counter Inspection: {:?}", counter_inspection);
    println!("Map Inspection: {:?}", map_inspection);
    println!();

    // Demo 2: Sync Operations Monitoring
    println!("🔄 Demo 2: Sync Operations Monitoring");
    println!("-------------------------------------");
    
    // Simulate sync operations
    devtools.record_sync_operation(
        "sync-1".to_string(),
        "merge".to_string(),
        "success".to_string(),
        Some(150)
    ).await;
    
    devtools.record_sync_operation(
        "sync-2".to_string(),
        "merge".to_string(),
        "success".to_string(),
        Some(200)
    ).await;
    
    devtools.record_sync_operation(
        "sync-3".to_string(),
        "merge".to_string(),
        "failed".to_string(),
        Some(300)
    ).await;
    
    let sync_stats = devtools.get_sync_stats().await;
    println!("Sync Statistics: {:?}", sync_stats);
    println!();

    // Demo 3: Transport Monitoring
    println!("🌐 Demo 3: Transport Monitoring");
    println!("-------------------------------");
    
    // Create multi-transport setup
    let transport_config = MultiTransportConfig {
        primary: TransportType::WebSocket,
        fallbacks: vec![TransportType::Memory],
        auto_switch: true,
        timeout_ms: 5000,
    };
    
    let mut multi_transport = MultiTransport::new(transport_config);
    
    // Register transports
    let memory_transport = TransportEnum::InMemory(InMemoryTransport::new());
    multi_transport.register_transport(TransportType::Memory, memory_transport);
    
    // Record transport events
    devtools.record_transport_event(
        "websocket".to_string(),
        "connection_attempt".to_string(),
        "Attempting to connect to ws://example.com".to_string()
    ).await;
    
    devtools.record_transport_event(
        "websocket".to_string(),
        "connection_failed".to_string(),
        "Connection failed, switching to fallback".to_string()
    ).await;
    
    devtools.record_transport_event(
        "memory".to_string(),
        "connected".to_string(),
        "Connected to in-memory transport".to_string()
    ).await;
    
    let transport_stats = devtools.get_transport_stats().await;
    println!("Transport Statistics: {:?}", transport_stats);
    println!();

    // Demo 4: Performance Metrics
    println!("⚡ Demo 4: Performance Metrics");
    println!("-----------------------------");
    
    // Record performance metrics
    devtools.record_performance_metric("memory_usage".to_string(), 1024.0, "bytes".to_string()).await;
    devtools.record_performance_metric("cpu_usage".to_string(), 25.5, "percent".to_string()).await;
    devtools.record_performance_metric("operations_per_second".to_string(), 150.0, "ops/sec".to_string()).await;
    
    let performance_metrics = devtools.get_performance_metrics().await;
    println!("Performance Metrics: {:?}", performance_metrics);
    println!();

    // Demo 5: Event Analysis
    println!("📈 Demo 5: Event Analysis");
    println!("-------------------------");
    
    // Get all events
    let all_events = devtools.get_events().await;
    println!("Total events recorded: {}", all_events.len());
    
    // Get events by type
    let crdt_events = devtools.get_events_by_type("crdt_operation").await;
    let sync_events = devtools.get_events_by_type("sync_operation").await;
    let transport_events = devtools.get_events_by_type("transport_event").await;
    let perf_events = devtools.get_events_by_type("performance_metric").await;
    
    println!("CRDT operations: {}", crdt_events.len());
    println!("Sync operations: {}", sync_events.len());
    println!("Transport events: {}", transport_events.len());
    println!("Performance metrics: {}", perf_events.len());
    
    // Get event counts
    let event_counts = devtools.get_event_counts().await;
    println!("Event counts: {:?}", event_counts);
    
    // Get recent events
    let recent_events = devtools.get_recent_events(5).await;
    println!("Recent events (last 5):");
    for (i, event) in recent_events.iter().enumerate() {
        println!("  {}. {:?}", i + 1, event);
    }
    println!();

    // Demo 6: Data Export
    println!("💾 Demo 6: Data Export");
    println!("----------------------");
    
    let export_json = devtools.export_data().await?;
    println!("Exported data size: {} characters", export_json.len());
    println!("First 200 characters of export:");
    println!("{}", &export_json[..200.min(export_json.len())]);
    println!("...");
    println!();

    // Demo 7: Real-time Monitoring Simulation
    println!("⏱️  Demo 7: Real-time Monitoring Simulation");
    println!("--------------------------------------------");
    
    // Simulate real-time operations
    for i in 1..=5 {
        println!("Simulating operation batch {}...", i);
        
        // Simulate CRDT operations
        devtools.record_crdt_operation(
            format!("batch-{}-register", i),
            "update".to_string(),
            replica_id.clone()
        ).await;
        
        // Simulate sync operations
        devtools.record_sync_operation(
            format!("batch-{}-sync", i),
            "merge".to_string(),
            "success".to_string(),
            Some(100 + i * 10)
        ).await;
        
        // Simulate performance metrics
        devtools.record_performance_metric(
            "memory_usage".to_string(),
            1000.0 + i as f64 * 100.0,
            "bytes".to_string()
        ).await;
        
        // Wait a bit to simulate real-time behavior
        sleep(Duration::from_millis(100)).await;
    }
    
    // Show final statistics
    let final_sync_stats = devtools.get_sync_stats().await;
    let final_event_counts = devtools.get_event_counts().await;
    
    println!("Final sync statistics: {:?}", final_sync_stats);
    println!("Final event counts: {:?}", final_event_counts);
    println!();

    // Demo 8: CRDT Inspections Summary
    println!("🔍 Demo 8: CRDT Inspections Summary");
    println!("-----------------------------------");
    
    let all_inspections = inspector.get_all_inspections().await;
    println!("Total CRDTs inspected: {}", all_inspections.len());
    
    for (id, inspection) in all_inspections {
        println!("CRDT '{}':", id);
        println!("  Type: {}", inspection.type_name);
        println!("  Operations: {}", inspection.operation_count);
        println!("  Memory: {} bytes", inspection.memory_usage_bytes);
        println!("  Last operation: {}", inspection.last_operation_at);
        println!();
    }

    println!("✅ DevTools Demo Complete!");
    println!("==========================");
    println!("This demo showed how DevTools can monitor:");
    println!("• CRDT operations and state inspection");
    println!("• Sync operation statistics and performance");
    println!("• Transport layer events and fallbacks");
    println!("• Performance metrics and resource usage");
    println!("• Event filtering and analysis");
    println!("• Complete data export for debugging");
    println!("• Real-time monitoring capabilities");

    Ok(())
}
