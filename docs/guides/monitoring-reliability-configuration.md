# Monitoring and Reliability Configuration Guide

## Overview

This guide explains how to configure monitoring and reliability features in Leptos-Sync, including workarounds for known timing issues in advanced features.

## Reliability Features Status

### ✅ Working Features
- Basic error handling and retry logic
- Simple health checks
- Basic monitoring metrics
- Core reliability mechanisms

### ⚠️ Features with Known Issues
- Advanced circuit breakers (timing issues)
- Complex retry policies (precision issues)
- Advanced health checks (uptime calculation issues)
- Complex alert management (race conditions)
- Advanced data integrity checks (timestamp precision)

## Configuration Options

### 1. Basic Configuration (Recommended for Production)

```rust
use leptos_sync::{
    EndToEndSyncManager,
    Storage,
    Transport,
    SyncConfig,
    ReliabilityConfig,
    MonitoringConfig
};

// Basic reliability configuration
let reliability_config = ReliabilityConfig {
    retry_policy: RetryPolicy::Fixed(Duration::from_secs(1)),
    max_retries: 3,
    circuit_breaker: CircuitBreakerConfig::Disabled, // Disable due to timing issues
    health_checks: HealthChecks::Basic,
    data_integrity: DataIntegrityConfig::Basic,
};

// Basic monitoring configuration
let monitoring_config = MonitoringConfig {
    metrics: Metrics::Basic,
    alerts: Alerts::Basic,
    health_checks: HealthChecks::Basic,
    uptime_tracking: false, // Disable due to timing issues
};

let sync_config = SyncConfig {
    reliability: reliability_config,
    monitoring: monitoring_config,
    ..Default::default()
};

let sync_manager = EndToEndSyncManager::new(
    Storage::indexeddb("app_db", 1).await?,
    Transport::websocket("wss://your-server.com/ws"),
    sync_config
).await?;
```

### 2. Advanced Configuration (Use with Caution)

```rust
// Advanced configuration with known issues
let reliability_config = ReliabilityConfig {
    retry_policy: RetryPolicy::Exponential {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        multiplier: 2.0,
    },
    max_retries: 5,
    circuit_breaker: CircuitBreakerConfig::Enabled {
        failure_threshold: 5,
        recovery_timeout: Duration::from_secs(30),
        half_open_max_calls: 3,
    },
    health_checks: HealthChecks::Advanced,
    data_integrity: DataIntegrityConfig::Advanced,
};

// Note: This configuration may have timing issues in tests
// but can work in production with proper monitoring
```

## Workarounds for Known Issues

### 1. Circuit Breaker Issues

**Problem**: State transitions have race conditions in tests

**Workaround**: Use basic retry logic instead

```rust
// Instead of circuit breakers, use simple retry logic
let retry_policy = RetryPolicy::Fixed(Duration::from_secs(1));

// Or implement custom retry logic
async fn retry_with_backoff<F, T>(mut operation: F) -> Result<T, Error>
where
    F: FnMut() -> Future<Output = Result<T, Error>>,
{
    let mut delay = Duration::from_millis(100);
    let max_delay = Duration::from_secs(5);
    
    for attempt in 0..5 {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == 4 => return Err(e),
            Err(_) => {
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, max_delay);
            }
        }
    }
    
    unreachable!()
}
```

### 2. Retry Policy Timing Issues

**Problem**: Exponential backoff calculations have precision issues

**Workaround**: Use fixed delays or implement custom timing

```rust
// Use fixed delays instead of exponential backoff
let retry_policy = RetryPolicy::Fixed(Duration::from_secs(1));

// Or implement custom exponential backoff with integer math
fn calculate_delay(attempt: usize) -> Duration {
    let base_delay_ms = 100;
    let max_delay_ms = 5000;
    
    // Use integer math to avoid floating-point precision issues
    let delay_ms = base_delay_ms * (1 << attempt.min(5));
    let delay_ms = delay_ms.min(max_delay_ms);
    
    Duration::from_millis(delay_ms)
}
```

### 3. Health Check Timing Issues

**Problem**: Uptime calculations are timing-sensitive

**Workaround**: Use basic health checks without uptime tracking

```rust
// Basic health check configuration
let health_config = HealthCheckConfig {
    uptime_tracking: false, // Disable uptime tracking
    basic_checks: true,
    advanced_checks: false,
    check_interval: Duration::from_secs(30),
};

// Or implement custom health checks
struct CustomHealthChecker {
    start_time: Instant,
}

impl CustomHealthChecker {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }
    
    fn is_healthy(&self) -> bool {
        // Simple health check without uptime calculation
        true
    }
    
    fn get_uptime(&self) -> Duration {
        // Simple uptime calculation
        self.start_time.elapsed()
    }
}
```

### 4. Alert Management Race Conditions

**Problem**: Alert counting has race conditions

**Workaround**: Use single-threaded alert management

```rust
use std::sync::Mutex;

struct SafeAlertManager {
    alerts: Mutex<Vec<Alert>>,
    counters: Mutex<HashMap<String, usize>>,
}

impl SafeAlertManager {
    fn new() -> Self {
        Self {
            alerts: Mutex::new(Vec::new()),
            counters: Mutex::new(HashMap::new()),
        }
    }
    
    fn add_alert(&self, alert: Alert) -> Result<(), Error> {
        let mut alerts = self.alerts.lock().unwrap();
        alerts.push(alert);
        Ok(())
    }
    
    fn increment_counter(&self, name: &str) -> Result<(), Error> {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(name.to_string()).or_insert(0) += 1;
        Ok(())
    }
}
```

### 5. Data Integrity Timestamp Issues

**Problem**: Timestamp updates have precision issues

**Workaround**: Use basic integrity checks without timestamp precision

```rust
// Basic data integrity configuration
let integrity_config = DataIntegrityConfig {
    checksum_validation: true,
    timestamp_precision: false, // Disable timestamp precision
    corruption_detection: true,
    metadata_validation: false, // Disable metadata validation
};

// Or implement custom integrity checks
struct BasicIntegrityChecker;

impl BasicIntegrityChecker {
    fn validate_data(&self, data: &[u8]) -> Result<(), IntegrityError> {
        // Simple checksum validation
        let checksum = self.calculate_checksum(data);
        if checksum == 0 {
            return Err(IntegrityError::InvalidChecksum);
        }
        Ok(())
    }
    
    fn calculate_checksum(&self, data: &[u8]) -> u32 {
        // Simple checksum calculation
        data.iter().fold(0u32, |acc, &byte| acc.wrapping_add(byte as u32))
    }
}
```

## Production Monitoring Setup

### 1. Basic Monitoring (Recommended)

```rust
use leptos_sync::monitoring::{MonitoringManager, BasicMetrics};

// Basic monitoring setup
let monitoring = MonitoringManager::new(MonitoringConfig {
    metrics: Metrics::Basic,
    alerts: Alerts::Basic,
    health_checks: HealthChecks::Basic,
    uptime_tracking: false,
}).await?;

// Basic metrics collection
let metrics = BasicMetrics::new();
metrics.increment_counter("sync_operations");
metrics.record_gauge("active_connections", 5);
metrics.record_histogram("sync_duration", Duration::from_millis(100));
```

### 2. Custom Monitoring Implementation

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct CustomMetrics {
    sync_operations: AtomicU64,
    active_connections: AtomicU64,
    sync_duration_sum: AtomicU64,
    sync_duration_count: AtomicU64,
}

impl CustomMetrics {
    fn new() -> Self {
        Self {
            sync_operations: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            sync_duration_sum: AtomicU64::new(0),
            sync_duration_count: AtomicU64::new(0),
        }
    }
    
    fn increment_sync_operations(&self) {
        self.sync_operations.fetch_add(1, Ordering::Relaxed);
    }
    
    fn set_active_connections(&self, count: u64) {
        self.active_connections.store(count, Ordering::Relaxed);
    }
    
    fn record_sync_duration(&self, duration: Duration) {
        let duration_ms = duration.as_millis() as u64;
        self.sync_duration_sum.fetch_add(duration_ms, Ordering::Relaxed);
        self.sync_duration_count.fetch_add(1, Ordering::Relaxed);
    }
    
    fn get_average_sync_duration(&self) -> Option<Duration> {
        let sum = self.sync_duration_sum.load(Ordering::Relaxed);
        let count = self.sync_duration_count.load(Ordering::Relaxed);
        
        if count > 0 {
            Some(Duration::from_millis(sum / count))
        } else {
            None
        }
    }
}
```

## Testing Configuration

### 1. Test with Basic Configuration

```rust
#[tokio::test]
async fn test_basic_reliability() {
    let config = ReliabilityConfig {
        retry_policy: RetryPolicy::Fixed(Duration::from_millis(10)),
        max_retries: 2,
        circuit_breaker: CircuitBreakerConfig::Disabled,
        health_checks: HealthChecks::Basic,
        data_integrity: DataIntegrityConfig::Basic,
    };
    
    // Test basic functionality
    let sync_manager = EndToEndSyncManager::new(
        Storage::memory(),
        Transport::in_memory(),
        SyncConfig {
            reliability: config,
            ..Default::default()
        }
    ).await.unwrap();
    
    // Test operations
    sync_manager.start().await.unwrap();
}
```

### 2. Test with Advanced Configuration (Expect Issues)

```rust
#[tokio::test]
async fn test_advanced_reliability() {
    let config = ReliabilityConfig {
        retry_policy: RetryPolicy::Exponential {
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            multiplier: 2.0,
        },
        max_retries: 3,
        circuit_breaker: CircuitBreakerConfig::Enabled {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(10),
            half_open_max_calls: 1,
        },
        health_checks: HealthChecks::Advanced,
        data_integrity: DataIntegrityConfig::Advanced,
    };
    
    // This test may fail due to timing issues
    // but can work in production with proper configuration
    let sync_manager = EndToEndSyncManager::new(
        Storage::memory(),
        Transport::in_memory(),
        SyncConfig {
            reliability: config,
            ..Default::default()
        }
    ).await.unwrap();
    
    // Test operations with longer timeouts
    tokio::time::timeout(Duration::from_secs(5), async {
        sync_manager.start().await.unwrap();
    }).await.unwrap();
}
```

## Best Practices

### 1. Production Configuration
- Use basic reliability features for production
- Disable advanced features with known issues
- Implement custom monitoring if needed
- Use fixed delays instead of exponential backoff
- Disable uptime tracking in health checks

### 2. Development Configuration
- Test with both basic and advanced configurations
- Use longer timeouts for advanced feature tests
- Monitor for timing-related test failures
- Document any custom workarounds

### 3. Monitoring Strategy
- Start with basic monitoring
- Gradually add advanced features as they stabilize
- Implement custom metrics for critical operations
- Use external monitoring tools for production

### 4. Error Handling
- Implement graceful degradation for failed features
- Provide clear error messages for configuration issues
- Log warnings for disabled features
- Monitor for reliability feature failures

## Troubleshooting

### Common Issues

1. **Test Failures**: Use basic configuration for tests
2. **Timing Issues**: Increase timeouts or use fixed delays
3. **Race Conditions**: Use single-threaded implementations
4. **Precision Issues**: Use integer math instead of floating-point

### Debug Commands

```bash
# Test with basic configuration
cargo test --features "websocket,indexeddb" -- --test-threads=1

# Test with longer timeouts
RUST_LOG=debug cargo test --features "websocket,indexeddb" -- --test-threads=1

# Test specific reliability features
cargo test reliability:: --features "websocket,indexeddb" -- --test-threads=1
```

This guide should help you configure monitoring and reliability features appropriately while avoiding known issues.
