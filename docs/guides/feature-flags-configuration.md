# Feature Flags Configuration Guide

## Overview

Leptos-Sync uses feature flags to enable optional functionality and optimize build size. This guide explains how to configure feature flags for different use cases.

## Available Feature Flags

### Core Features (Default)
- `websocket` - WebSocket transport support
- `indexeddb` - IndexedDB storage support

### Optional Features
- `encryption` - AES-GCM encryption support
- `compression` - Data compression support
- `metrics` - Prometheus metrics support
- `validation` - JSON schema validation support

## Configuration Examples

### 1. Minimal Configuration (Smallest Bundle)
```toml
# Cargo.toml
[features]
default = []
websocket = ["leptos-ws-pro"]
indexeddb = ["web-sys"]
```

### 2. Standard Configuration (Recommended)
```toml
# Cargo.toml
[features]
default = ["websocket", "indexeddb"]
encryption = ["aes-gcm"]
compression = ["flate2"]
metrics = ["prometheus"]
validation = ["jsonschema"]
```

### 3. Full Configuration (All Features)
```toml
# Cargo.toml
[features]
default = ["websocket", "indexeddb", "encryption", "compression", "metrics", "validation"]
encryption = ["aes-gcm"]
compression = ["flate2"]
metrics = ["prometheus"]
validation = ["jsonschema"]
```

## Runtime Configuration

### Enable Features at Runtime
```rust
use leptos_sync::{
    EndToEndSyncManager, 
    Storage, 
    Transport, 
    SyncConfig
};

// Check if features are available
#[cfg(feature = "encryption")]
let encryption_enabled = true;

#[cfg(not(feature = "encryption"))]
let encryption_enabled = false;

// Configure based on available features
let config = SyncConfig {
    encryption: encryption_enabled,
    compression: cfg!(feature = "compression"),
    metrics: cfg!(feature = "metrics"),
    validation: cfg!(feature = "validation"),
    ..Default::default()
};
```

### Conditional Compilation
```rust
// Only compile encryption code when feature is enabled
#[cfg(feature = "encryption")]
mod encryption {
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    
    pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        // Encryption implementation
    }
}

#[cfg(not(feature = "encryption"))]
mod encryption {
    pub fn encrypt_data(_data: &[u8], _key: &[u8]) -> Result<Vec<u8>, EncryptionError> {
        Err(EncryptionError::FeatureNotEnabled)
    }
}
```

## Testing with Feature Flags

### Run Tests with Specific Features
```bash
# Test with all features
cargo test --features "encryption,compression,metrics,validation"

# Test with specific features
cargo test --features encryption
cargo test --features compression
cargo test --features metrics

# Test without optional features
cargo test --no-default-features
```

### Test Configuration
```rust
// tests/feature_flags.rs
#[cfg(feature = "encryption")]
#[tokio::test]
async fn test_encryption() {
    // Test encryption functionality
}

#[cfg(not(feature = "encryption"))]
#[tokio::test]
async fn test_encryption_disabled() {
    // Test that encryption is properly disabled
}
```

## Build Optimization

### Size Optimization
```toml
# Cargo.toml - Minimal build
[features]
default = []
websocket = ["leptos-ws-pro"]
indexeddb = ["web-sys"]

# Exclude optional dependencies
[dependencies.leptos-sync-core]
version = "0.9.0"
default-features = false
features = ["websocket", "indexeddb"]
```

### Performance Optimization
```toml
# Cargo.toml - Performance build
[features]
default = ["websocket", "indexeddb", "compression"]
encryption = ["aes-gcm"]
metrics = ["prometheus"]

# Enable optimizations
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

## Platform-Specific Configuration

### Web/WASM Configuration
```toml
# Cargo.toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
leptos-sync-core = { version = "0.9.0", default-features = false, features = ["websocket", "indexeddb"] }
```

### Native Configuration
```toml
# Cargo.toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
leptos-sync-core = { version = "0.9.0", features = ["websocket", "indexeddb", "encryption", "compression", "metrics"] }
```

## Common Issues and Solutions

### Issue: Feature Not Available
```rust
// Error: feature `encryption` is not available
use leptos_sync::encryption::EncryptionManager;

// Solution: Check feature availability
#[cfg(feature = "encryption")]
use leptos_sync::encryption::EncryptionManager;

#[cfg(not(feature = "encryption"))]
// Provide alternative implementation or error handling
```

### Issue: Test Failures with Missing Features
```bash
# Error: test failed because encryption feature is not enabled
cargo test

# Solution: Enable required features for tests
cargo test --features encryption
```

### Issue: Build Size Too Large
```toml
# Problem: Including all features increases bundle size
[features]
default = ["websocket", "indexeddb", "encryption", "compression", "metrics", "validation"]

# Solution: Use only required features
[features]
default = ["websocket", "indexeddb"]
```

## Best Practices

### 1. Feature Selection
- **Always include**: `websocket` and `indexeddb` for basic functionality
- **Include encryption**: For sensitive data applications
- **Include compression**: For bandwidth-constrained environments
- **Include metrics**: For production monitoring
- **Include validation**: For strict data validation requirements

### 2. Testing Strategy
- Test with minimal feature set
- Test with full feature set
- Test feature combinations
- Test feature absence scenarios

### 3. Documentation
- Document which features are required for your use case
- Provide examples for different feature configurations
- Include feature flag information in README

### 4. CI/CD Configuration
```yaml
# .github/workflows/ci.yml
strategy:
  matrix:
    features:
      - "websocket,indexeddb"
      - "websocket,indexeddb,encryption"
      - "websocket,indexeddb,compression"
      - "websocket,indexeddb,metrics"
      - "websocket,indexeddb,validation"
      - "websocket,indexeddb,encryption,compression,metrics,validation"

steps:
  - name: Test with features
    run: cargo test --features ${{ matrix.features }}
```

## Migration Guide

### From v0.8.0 to v0.9.0
```toml
# v0.8.0
[features]
default = ["websocket", "indexeddb"]

# v0.9.0 - No changes required, but new features available
[features]
default = ["websocket", "indexeddb"]
encryption = ["aes-gcm"]
compression = ["flate2"]
metrics = ["prometheus"]
validation = ["jsonschema"]
```

### Adding New Features
```rust
// 1. Add feature flag to Cargo.toml
[features]
new_feature = ["dependency"]

// 2. Use conditional compilation
#[cfg(feature = "new_feature")]
mod new_feature {
    // Implementation
}

// 3. Provide fallback for when feature is disabled
#[cfg(not(feature = "new_feature"))]
mod new_feature {
    // Fallback implementation
}
```

## Troubleshooting

### Common Commands
```bash
# Check available features
cargo tree --features

# Check feature dependencies
cargo tree --features encryption

# Build with specific features
cargo build --features "websocket,indexeddb,encryption"

# Test with specific features
cargo test --features "websocket,indexeddb,encryption"
```

### Debug Feature Issues
```rust
// Check if feature is enabled at runtime
if cfg!(feature = "encryption") {
    println!("Encryption feature is enabled");
} else {
    println!("Encryption feature is disabled");
}

// Check feature availability in tests
#[cfg(feature = "encryption")]
#[test]
fn test_encryption_available() {
    assert!(cfg!(feature = "encryption"));
}
```

This guide should help you configure feature flags appropriately for your use case and avoid common issues.
