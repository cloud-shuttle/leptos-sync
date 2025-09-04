# DevTools Guide

The Leptos-Sync DevTools provide comprehensive debugging and monitoring capabilities for your collaborative applications. This guide shows you how to use DevTools to debug sync issues, monitor performance, and analyze your application's behavior.

## 🚀 Quick Start

### Basic Setup

```rust
use leptos_sync_core::{DevTools, DevToolsConfig, CrdtInspector};

// Create DevTools with default configuration
let devtools_config = DevToolsConfig {
    enable_crdt_inspection: true,
    enable_sync_monitoring: true,
    enable_transport_monitoring: true,
    enable_performance_metrics: true,
    max_events: 1000,
};

let devtools = Arc::new(DevTools::new(devtools_config));
let inspector = CrdtInspector::new(devtools.clone());
```

### Configuration Options

```rust
let config = DevToolsConfig {
    // Enable CRDT state inspection
    enable_crdt_inspection: true,
    
    // Monitor sync operations (merge, conflict resolution, etc.)
    enable_sync_monitoring: true,
    
    // Track transport layer events (connection, messages, errors)
    enable_transport_monitoring: true,
    
    // Collect performance metrics (memory, CPU, throughput)
    enable_performance_metrics: true,
    
    // Maximum events to keep in memory (prevents memory leaks)
    max_events: 1000,
};
```

## 📊 CRDT Monitoring

### Recording CRDT Operations

```rust
use leptos_sync_core::crdt::{LwwRegister, ReplicaId};

let mut register = LwwRegister::new("initial_value".to_string(), replica_id.clone());

// Record operations as they happen
devtools.record_crdt_operation(
    "user-profile".to_string(),
    "update".to_string(),
    replica_id.clone()
).await;

register.update("new_value".to_string(), replica_id.clone());
```

### Inspecting CRDT State

```rust
// Get detailed information about a CRDT
let inspection = inspector.inspect_crdt(&register, "user-profile".to_string()).await;

println!("CRDT Type: {}", inspection.type_name);
println!("Memory Usage: {} bytes", inspection.memory_usage_bytes);
println!("Operations: {}", inspection.operation_count);
println!("Last Operation: {}", inspection.last_operation_at);
```

### CRDT Inspection Details

The `CrdtInspection` provides:
- **Type Information**: Exact CRDT type and generic parameters
- **Memory Usage**: Current memory footprint in bytes
- **Operation Count**: Number of operations performed
- **Last Activity**: Timestamp of most recent operation
- **State Summary**: Human-readable state description

## 🔄 Sync Operations Monitoring

### Recording Sync Events

```rust
// Record successful sync operations
devtools.record_sync_operation(
    "sync-123".to_string(),
    "merge".to_string(),
    "success".to_string(),
    Some(150) // duration in milliseconds
).await;

// Record failed operations
devtools.record_sync_operation(
    "sync-124".to_string(),
    "merge".to_string(),
    "failed".to_string(),
    Some(500)
).await;
```

### Sync Statistics

```rust
let stats = devtools.get_sync_stats().await;

println!("Total Operations: {}", stats.total_operations);
println!("Success Rate: {:.1}%", 
    (stats.successful_operations as f64 / stats.total_operations as f64) * 100.0);
println!("Average Duration: {:.1}ms", stats.avg_duration_ms);
println!("Last Sync: {}", stats.last_sync_at);
```

## 🌐 Transport Monitoring

### Recording Transport Events

```rust
// Connection events
devtools.record_transport_event(
    "websocket".to_string(),
    "connected".to_string(),
    "Connected to ws://api.example.com/sync".to_string()
).await;

// Error events
devtools.record_transport_event(
    "websocket".to_string(),
    "connection_failed".to_string(),
    "Connection timeout after 30s".to_string()
).await;

// Message events
devtools.record_transport_event(
    "websocket".to_string(),
    "message_sent".to_string(),
    "Sent 1024 bytes to server".to_string()
).await;
```

### Transport Statistics

```rust
let transport_stats = devtools.get_transport_stats().await;

for (transport_type, stats) in transport_stats {
    println!("Transport: {}", transport_type);
    println!("  Messages Sent: {}", stats.messages_sent);
    println!("  Messages Received: {}", stats.messages_received);
    println!("  Connected: {}", stats.is_connected);
    println!("  Errors: {}", stats.error_count);
    println!("  Last Activity: {}", stats.last_activity_at);
}
```

## ⚡ Performance Monitoring

### Recording Performance Metrics

```rust
// Memory usage
devtools.record_performance_metric(
    "memory_usage".to_string(),
    1024.0,
    "bytes".to_string()
).await;

// CPU usage
devtools.record_performance_metric(
    "cpu_usage".to_string(),
    25.5,
    "percent".to_string()
).await;

// Throughput
devtools.record_performance_metric(
    "operations_per_second".to_string(),
    150.0,
    "ops/sec".to_string()
).await;
```

### Performance Analysis

```rust
let metrics = devtools.get_performance_metrics().await;

println!("CRDT Merges/sec: {:.1}", metrics.crdt_merges_per_second);
println!("Sync Operations/sec: {:.1}", metrics.sync_operations_per_second);
println!("Memory Usage: {} bytes", metrics.memory_usage_bytes);
println!("CPU Usage: {:.1}%", metrics.cpu_usage_percent);
```

## 📈 Event Analysis

### Filtering Events

```rust
// Get all events
let all_events = devtools.get_events().await;

// Filter by event type
let crdt_events = devtools.get_events_by_type("crdt_operation").await;
let sync_events = devtools.get_events_by_type("sync_operation").await;
let transport_events = devtools.get_events_by_type("transport_event").await;
let perf_events = devtools.get_events_by_type("performance_metric").await;

// Get recent events
let recent_events = devtools.get_recent_events(10).await;
```

### Event Counts

```rust
let event_counts = devtools.get_event_counts().await;

for (event_type, count) in event_counts {
    println!("{}: {} events", event_type, count);
}
```

## 💾 Data Export

### Complete Data Export

```rust
// Export all DevTools data as JSON
let export_json = devtools.export_data().await?;

// Save to file for analysis
std::fs::write("devtools-export.json", export_json)?;

// Parse and analyze
let export_data: DevToolsExport = serde_json::from_str(&export_json)?;
```

### Export Structure

The exported data includes:
- **Events**: All recorded events with timestamps
- **CRDT Inspections**: Current state of all monitored CRDTs
- **Sync Statistics**: Aggregated sync performance data
- **Transport Statistics**: Per-transport performance metrics
- **Performance Metrics**: System resource usage
- **Configuration**: DevTools settings used

## 🔍 Debugging Workflows

### Debugging Sync Issues

```rust
// 1. Check sync statistics
let sync_stats = devtools.get_sync_stats().await;
if sync_stats.failed_operations > 0 {
    println!("⚠️  {} sync operations failed", sync_stats.failed_operations);
}

// 2. Look at recent sync events
let recent_sync_events = devtools.get_events_by_type("sync_operation").await;
for event in recent_sync_events.iter().rev().take(5) {
    if let DevToolsEvent::SyncOperation { status, operation_type, .. } = event {
        if status == "failed" {
            println!("❌ Failed {} operation", operation_type);
        }
    }
}

// 3. Check transport health
let transport_stats = devtools.get_transport_stats().await;
for (transport_type, stats) in transport_stats {
    if !stats.is_connected {
        println!("🔌 {} transport disconnected", transport_type);
    }
    if stats.error_count > 0 {
        println!("⚠️  {} transport has {} errors", transport_type, stats.error_count);
    }
}
```

### Performance Debugging

```rust
// 1. Check memory usage
let metrics = devtools.get_performance_metrics().await;
if metrics.memory_usage_bytes > 100_000_000 { // 100MB
    println!("⚠️  High memory usage: {} bytes", metrics.memory_usage_bytes);
}

// 2. Check operation throughput
if metrics.crdt_merges_per_second < 10.0 {
    println!("⚠️  Low CRDT merge performance: {:.1} merges/sec", 
        metrics.crdt_merges_per_second);
}

// 3. Analyze recent performance events
let perf_events = devtools.get_events_by_type("performance_metric").await;
for event in perf_events.iter().rev().take(10) {
    if let DevToolsEvent::PerformanceMetric { metric_name, value, unit, .. } = event {
        println!("📊 {}: {:.1} {}", metric_name, value, unit);
    }
}
```

### CRDT State Debugging

```rust
// 1. Inspect all CRDTs
let all_inspections = inspector.get_all_inspections().await;
for (crdt_id, inspection) in all_inspections {
    println!("CRDT '{}':", crdt_id);
    println!("  Type: {}", inspection.type_name);
    println!("  Memory: {} bytes", inspection.memory_usage_bytes);
    println!("  Operations: {}", inspection.operation_count);
    
    // Flag potential issues
    if inspection.memory_usage_bytes > 10_000 {
        println!("  ⚠️  High memory usage");
    }
    if inspection.operation_count > 1000 {
        println!("  ⚠️  High operation count");
    }
}

// 2. Check for CRDT conflicts
let crdt_events = devtools.get_events_by_type("crdt_operation").await;
for event in crdt_events.iter().rev().take(20) {
    if let DevToolsEvent::CrdtOperation { operation, crdt_id, .. } = event {
        if operation.contains("conflict") {
            println!("⚠️  Conflict detected in CRDT '{}'", crdt_id);
        }
    }
}
```

## 🎯 Best Practices

### 1. Selective Monitoring

```rust
// Only enable monitoring for development
#[cfg(debug_assertions)]
let devtools_config = DevToolsConfig {
    enable_crdt_inspection: true,
    enable_sync_monitoring: true,
    enable_transport_monitoring: true,
    enable_performance_metrics: true,
    max_events: 1000,
};

#[cfg(not(debug_assertions))]
let devtools_config = DevToolsConfig {
    enable_crdt_inspection: false,
    enable_sync_monitoring: false,
    enable_transport_monitoring: false,
    enable_performance_metrics: false,
    max_events: 0,
};
```

### 2. Event Cleanup

```rust
// Periodically clear old events to prevent memory growth
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
    loop {
        interval.tick().await;
        devtools.clear_events().await;
    }
});
```

### 3. Performance Impact

```rust
// Use async operations to avoid blocking
devtools.record_crdt_operation(crdt_id, operation, replica_id).await;

// Batch operations when possible
let mut batch_operations = Vec::new();
// ... collect operations ...
for (crdt_id, operation, replica_id) in batch_operations {
    devtools.record_crdt_operation(crdt_id, operation, replica_id).await;
}
```

### 4. Production Monitoring

```rust
// Export data periodically for analysis
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600)); // 1 hour
    loop {
        interval.tick().await;
        if let Ok(export_data) = devtools.export_data().await {
            // Send to monitoring service
            send_to_monitoring_service(export_data).await;
        }
    }
});
```

## 🚀 Running the Demo

See DevTools in action with our comprehensive demo:

```bash
# Run the DevTools demo
cargo run --bin devtools-demo
```

The demo shows:
- CRDT operations monitoring
- Sync operation tracking
- Transport event logging
- Performance metrics collection
- Event filtering and analysis
- Complete data export
- Real-time monitoring simulation

## 📚 Integration with Your App

### Leptos Integration

```rust
use leptos::*;
use leptos_sync_core::{DevTools, DevToolsConfig};

#[component]
fn MyApp() -> impl IntoView {
    let devtools = use_context::<Arc<DevTools>>()
        .expect("DevTools context not found");
    
    let debug_info = create_rw_signal(String::new());
    
    let export_debug_data = move |_| {
        let devtools = devtools.clone();
        spawn_local(async move {
            if let Ok(export_json) = devtools.export_data().await {
                debug_info.set(export_json);
            }
        });
    };
    
    view! {
        <div>
            <button on:click=export_debug_data>
                "Export Debug Data"
            </button>
            <pre>{debug_info}</pre>
        </div>
    }
}
```

### Server-Side Integration

```rust
// In your sync server
use leptos_sync_core::{DevTools, DevToolsConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devtools = Arc::new(DevTools::new(DevToolsConfig::default()));
    
    // Monitor server-side sync operations
    let sync_engine = SyncEngine::new()
        .with_devtools(devtools.clone())
        .build()?;
    
    // ... rest of server setup
}
```

---

**DevTools make debugging distributed systems manageable!** Use them to understand what's happening in your sync system, identify performance bottlenecks, and resolve conflicts quickly.

For more examples, see the `examples/devtools_demo` directory in the repository.
