# Dependency Updates & Modernization - Medium Priority

## Overview
Update dependencies to their latest stable versions as of September 2025 and resolve version compatibility issues.

## Current Dependency Status

### Rust Ecosystem (September 2025)
Based on current dependency analysis:

#### Core Dependencies - Status Check
```toml
# Current versions in Cargo.toml vs Expected Latest (Sept 2025)
leptos = "0.8.6"              # Latest likely: 0.9.x or 1.0.x
leptos-ws-pro = "0.10.0"      # Check for latest version
tokio = "1.0"                 # Latest: 1.40+ (should update to 1.x latest)  
uuid = "1.0"                  # Latest: 1.10+
serde = "1.0"                 # Latest: 1.0.210+
chrono = "0.4"                # Latest: 0.4.38+
sqlx = "0.7"                  # Latest: 0.8+
redis = "0.23"                # Latest: 0.26+
```

#### Development Dependencies
```toml
# Testing & Development
wasm-bindgen-test = "0.3"     # Check latest
criterion = "0.5"             # Latest: 0.5.1+
proptest = "1.0"              # Latest: 1.5+
```

## Implementation Plan

### Phase 1: Dependency Audit (Week 1)
**File**: `docs/remediation/dependency-audit.md` (< 200 lines)

#### Step 1: Current Version Analysis
```bash
# Generate dependency tree and outdated analysis
cargo tree > dependency-tree.txt
cargo audit > security-audit.txt

# Check for outdated dependencies
cargo outdated > outdated-deps.txt
```

#### Step 2: Breaking Changes Analysis
For each major dependency:
- Review CHANGELOG for breaking changes
- Identify required code changes
- Assess migration effort and risks

#### Step 3: Version Compatibility Matrix
Create compatibility matrix for major dependencies:

| Dependency | Current | Target | Breaking Changes | Migration Effort |
|------------|---------|--------|------------------|------------------|
| leptos     | 0.8.6   | 0.9.x  | TBD              | Medium           |
| tokio      | 1.0     | 1.40   | None expected    | Low              |
| sqlx       | 0.7     | 0.8    | Query API changes| Medium           |

### Phase 2: Low-Risk Updates (Week 1)
Update dependencies with no breaking changes:

```toml
# Cargo.toml - Low-risk updates
uuid = { version = "1.10", features = ["v4", "serde"] }
chrono = { version = "0.4.38", features = ["serde"] }
serde = { version = "1.0.210", features = ["derive"] }
serde_json = "1.0.128"
tracing = "0.1.40"
thiserror = "1.0.64"
```

### Phase 3: Medium-Risk Updates (Week 2)
Update dependencies with potential breaking changes:

#### Tokio Update
```toml
tokio = { version = "1.40", features = ["full"] }
```
**Migration Requirements:**
- Review async runtime initialization
- Check for deprecated APIs
- Update test harness usage

#### SQLX Update  
```toml
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "chrono"] }
```
**Migration Requirements:**
- Update query macro usage
- Review connection pool configuration  
- Test database migration compatibility

### Phase 4: High-Risk Updates (Week 2)
Update core framework dependencies:

#### Leptos Framework Update
```toml
leptos = "0.9.0"  # Or latest stable
leptos_ws = "0.9.0"
leptos-ws-pro = "0.11.0"  # Check latest
```

**Migration Strategy:**
1. Create feature branch for leptos migration
2. Update core leptos usage patterns
3. Fix component macro changes  
4. Update SSR/CSR configuration
5. Test all examples and demos

#### WebAssembly Dependencies
```toml
wasm-bindgen = "0.2.95"      # Latest as of Sept 2025
js-sys = "0.3.72"
web-sys = { version = "0.3.72", features = [
    "Window",
    "Storage", 
    "console",
    "IdbDatabase",           # For IndexedDB
    "IdbObjectStore",
    "IdbTransaction",
] }
```

### Phase 5: Development Dependencies (Week 1)
Update testing and development tools:

```toml
# Testing
wasm-bindgen-test = "0.3.45"
criterion = { version = "0.5.1", features = ["html_reports"] }
proptest = "1.5.0"
tokio-test = "0.4.4"

# Development
cargo-leptos = "0.2.20"      # Check latest
wasm-pack = "0.13.0"         # System dependency
```

## Migration Implementation

### Leptos 0.8 → 0.9 Migration
**File**: `migration-guides/leptos-0.9-migration.md` (< 250 lines)

#### Component Syntax Changes
```rust
// Before (0.8)
#[component]
pub fn MyComponent(cx: Scope) -> impl IntoView {
    view! { cx,
        <div>"Hello"</div>  
    }
}

// After (0.9) - if syntax changed
#[component]
pub fn MyComponent() -> impl IntoView {
    view! {
        <div>"Hello"</div>
    }
}
```

#### Server Function Updates
```rust
// Update server function syntax if changed
#[server(UpdateCollection, "/api")]
pub async fn update_collection(
    collection_id: String, 
    data: String
) -> Result<(), ServerFnError> {
    // Implementation
}
```

### SQLX Migration Script
**File**: `scripts/migrate-sqlx.sh` (< 100 lines)

```bash
#!/bin/bash
# Script to help migrate SQLX 0.7 → 0.8

echo "Migrating SQLX from 0.7 to 0.8..."

# Update query macros
find . -name "*.rs" -exec sed -i 's/sqlx::query!/sqlx::query!/g' {} \;

# Update connection pool syntax (if changed)
# Add specific migration commands based on SQLX changelog

echo "Migration complete. Please review changes and run tests."
```

## Testing Strategy During Updates

### Phase 1: Isolated Testing
```bash
# Test each dependency update in isolation
cargo update <package_name>
cargo test --workspace
cargo check --workspace
```

### Phase 2: Integration Testing  
```bash
# Full integration test after each update
make test-all
cargo bench --bench sync_performance
```

### Phase 3: Browser Testing
```bash
# WASM compatibility after web dependencies update
wasm-pack test --chrome --headless
wasm-pack test --firefox --headless
```

## Version Pinning Strategy

### Production Dependencies
```toml
# Pin exact versions for reproducible builds
[workspace.dependencies]
leptos = "=0.9.0"
tokio = "=1.40.0" 
uuid = "=1.10.0"
serde = "=1.0.210"
```

### Development Dependencies
```toml
# Allow patch updates for dev tools
[workspace.dev-dependencies]
criterion = "~0.5.1"
proptest = "~1.5.0"
wasm-bindgen-test = "~0.3.45"
```

### CI/CD Pipeline Updates
```yaml
# .github/workflows/dependency-updates.yml
name: Dependency Updates

on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday
  workflow_dispatch:

jobs:
  update-dependencies:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Update dependencies
        run: |
          cargo update
          cargo check --workspace
          cargo test --workspace
          
      - name: Create PR for updates
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: 'chore: update dependencies'
          title: 'Automated dependency updates'
          body: |
            Automated dependency updates:
            - Run `cargo update`
            - Verified compilation and tests pass
            
            Please review changes and merge if appropriate.
```

## Rust Edition Update

### Current Issue: Edition 2024
```toml
# Current (problematic)
edition = "2024"  # Nightly only

# Target (stable compatible)
edition = "2021"  # Stable as of Rust 1.56+
```

### Edition 2021 Features Used
- Disjoint captures in closures
- IntoIterator for arrays
- Cargo resolver version 2
- Panic macro consistency

### Future Edition Planning
```toml
# When Rust edition 2024 becomes stable (likely 2024-2025)
# Plan migration path:
# 1. Test with nightly
# 2. Update when stable
# 3. Use new features incrementally
```

## Security Updates

### Vulnerability Monitoring
```bash
# Regular security audits
cargo audit --db security-advisory-db
cargo audit --stale
```

### High-Priority Security Dependencies
- `tokio` - Runtime security
- `serde` - Serialization vulnerabilities  
- `sqlx` - SQL injection prevention
- `uuid` - Cryptographic randomness
- `chrono` - Time handling security

## Compatibility Matrix

### Rust Version Support
| Rust Version | Support Status | Notes |
|--------------|----------------|-------|  
| 1.75+        | ✅ Full       | Current MSRV |
| 1.70-1.74    | ⚠️ Limited    | Some features missing |
| <1.70        | ❌ None       | Too old |

### Platform Support  
| Platform | Status | Dependencies |
|----------|--------|--------------|
| Linux    | ✅ Full | All deps supported |
| macOS    | ✅ Full | All deps supported |
| Windows  | ✅ Full | Some WASM limitations |
| WASM32   | ⚠️ Limited | No tokio, limited std |

## Rollback Strategy

### Dependency Rollback Plan
```bash
# If updates cause issues, rollback process:
git checkout HEAD~1 -- Cargo.toml Cargo.lock
cargo check --workspace
cargo test --workspace

# Or rollback specific dependency:
cargo update --package <package_name> --precise <old_version>
```

### Testing Rollback
- Automated rollback if CI fails
- Manual rollback procedures documented
- Known-good dependency versions recorded

## Acceptance Criteria

### Version Updates
- [ ] All dependencies updated to latest stable versions
- [ ] No security vulnerabilities in dependency tree
- [ ] All tests pass with updated dependencies
- [ ] Performance benchmarks show no regression
- [ ] WASM compatibility maintained

### Process Improvements  
- [ ] Automated dependency update pipeline
- [ ] Regular security audit schedule
- [ ] Migration guides for major updates
- [ ] Rollback procedures tested and documented

### Documentation
- [ ] Dependency compatibility matrix maintained
- [ ] Migration guides for breaking changes
- [ ] Version pinning strategy documented
- [ ] Security update procedures defined

## Time Estimate: 2-3 weeks
## Dependencies: Compilation fixes (01)
## Risk: Medium - potential breaking changes in major dependencies
## Benefits: Security updates, performance improvements, access to latest features
