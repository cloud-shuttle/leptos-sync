# Compilation Fixes - Critical Priority

## Overview
Fix immediate compilation errors that prevent the codebase from building on stable Rust.

## Critical Issues

### 1. Edition 2024 Dependency
**Problem**: `edition = "2024"` requires nightly Rust, blocking most users
**Location**: `Cargo.toml` line 60
**Solution**: 
- Change to `edition = "2021"` for stable compatibility
- Test all features work on Rust 1.75+
- Document any required nightly features separately

### 2. Generic Parameter Errors  
**Problem**: Unused generic parameters and invalid syntax in sync engine
**Location**: `leptos-sync-core/src/sync/engine.rs` line 373
**Example**: `+ use<Tr>` syntax error
**Solution**:
- Remove unused generic type parameters
- Fix function signature syntax errors
- Add proper trait bounds where needed

### 3. Missing Feature Flags
**Problem**: Documentation references features not in Cargo.toml
**Referenced Features**: `encryption`, `compression`, `metrics`
**Solution**:
- Add feature flags to workspace Cargo.toml
- Conditionally compile relevant modules
- Update documentation to match actual features

## Implementation Steps

### Step 1: Rust Edition Fix
```toml
[workspace.package]
version = "0.8.4"
edition = "2021"  # Changed from "2024"
```

### Step 2: Add Missing Features
```toml
[features]
default = []
encryption = ["aes-gcm", "ring"]
compression = ["flate2", "lz4"]
metrics = ["prometheus"]
websocket = ["leptos-ws-pro"]
indexeddb = ["idb"]
```

### Step 3: Fix Sync Engine Generics
- Remove invalid `+ use<Tr>` syntax
- Add proper trait bounds for generic parameters
- Ensure all type parameters are used or marked with `#[allow(unused)]`

### Step 4: CI Pipeline
Create `.github/workflows/ci.yml`:
```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
```

## Acceptance Criteria
- [ ] `cargo build --workspace` completes without errors
- [ ] `cargo test --workspace` runs successfully  
- [ ] `cargo clippy --workspace` passes without warnings
- [ ] CI pipeline runs successfully on push
- [ ] Code builds on Rust 1.75+ stable

## Time Estimate: 3-5 days
## Dependencies: None
## Risk: Low - purely technical fixes
