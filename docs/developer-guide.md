# Developer Guide for Leptos-Sync

## Table of Contents
1. [Getting Started](#getting-started)
2. [Development Environment](#development-environment)  
3. [Project Structure](#project-structure)
4. [Contributing Guidelines](#contributing-guidelines)
5. [API Reference](#api-reference)
6. [Testing Guide](#testing-guide)
7. [Release Process](#release-process)
8. [Troubleshooting](#troubleshooting)

## Getting Started

### Prerequisites

**Required Tools (Updated for 2025):**
- Rust 1.75+ with `wasm32-unknown-unknown` target
- Node.js 18+ (for WASM tooling)
- Git 2.30+
- Cargo 1.75+

**Install Dependencies:**
```bash
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install Leptos CLI (updated for 0.8.x)
cargo install cargo-leptos

# Install WASM tools
cargo install wasm-pack
npm install -g wasm-opt

# Install additional development tools
cargo install cargo-audit
cargo install cargo-tarpaulin  # For test coverage
cargo install cargo-deny        # For dependency analysis
```

### Quick Start

**Clone and Setup:**
```bash
git clone https://github.com/your-org/leptos-sync.git
cd leptos-sync

# Install dependencies
cargo check

# Run tests
cargo test

# Start development server
cargo leptos watch
```

**Create Your First Local-First App:**
```rust
use leptos::*;
use leptos_sync::*;

#[derive(LocalFirst, Serialize, Deserialize, Clone, Debug)]
#[local_first(crdt = "lww")]
struct Task {
    #[local_first(id)]
    id: Uuid,
    title: String,
    completed: bool,
    #[local_first(version)]
    version: u64,
}

#[component]
fn App() -> impl IntoView {
    // Initialize collection
    let tasks = use_local_first_collection::<Task>("tasks");
    
    // Create reactive query
    let incomplete_tasks = tasks
        .query()
        .filter("completed", Op::Eq, false.into())
        .watch();
    
    view! {
        <LocalFirstProvider>
            <h1>"My Tasks"</h1>
            <SyncStatusIndicator />
            <TaskList tasks=incomplete_tasks />
        </LocalFirstProvider>
    }
}
```

## Development Environment

### Recommended IDE Setup

**VS Code Extensions (2025):**
```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "serayuzgur.crates",
    "tamasfe.even-better-toml",
    "ms-vscode.wasm-dwarf-debugging",
    "ms-vscode.vscode-json",
    "bradlc.vscode-tailwindcss"
  ]
}
```

**VS Code Settings (Updated for 2025):**
```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.cargo.buildScripts.enable": true,
  "rust-analyzer.lens.enable": true,
  "files.associations": {
    "*.rs": "rust",
    "*.toml": "toml"
  },
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll": "explicit",
    "source.organizeImports": "explicit"
  }
}
```

### Environment Configuration

**Development Environment (2025):**
```bash
# .env.development
RUST_LOG=leptos_sync=debug,leptos=info
LEPTOS_SITE_ADDR=127.0.0.1:3000
LEPTOS_RELOAD_PORT=3001
DATABASE_URL=postgresql://localhost/leptos_sync_dev
REDIS_URL=redis://localhost:6379/0
WASM_PACK_TARGET=web
```

**Docker Development Setup (Updated for 2025):**
```yaml
# docker-compose.dev.yml
version: '3.8'
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: leptos_sync_dev
      POSTGRES_USER: developer
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U developer -d leptos_sync_dev"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:
```

### Development Scripts

**package.json (for tooling - 2025):**
```json
{
  "scripts": {
    "dev": "cargo leptos watch",
    "build": "cargo leptos build --release",
    "test": "cargo test && wasm-pack test --node",
    "test:browser": "wasm-pack test --chrome --headless",
    "test:coverage": "cargo tarpaulin --out html",
    "lint": "cargo clippy -- -D warnings",
    "format": "cargo fmt -- --check",
    "docs": "cargo doc --open --no-deps",
    "audit": "cargo audit",
    "deny": "cargo deny check",
    "wasm:build": "wasm-pack build --target web",
    "wasm:optimize": "wasm-opt -O4 -o dist/leptos_sync_opt.wasm dist/leptos_sync.wasm"
  },
  "devDependencies": {
    "wasm-pack": "^0.12.0",
    "wasm-opt": "^0.113.0"
  }
}
```

**Makefile (for common tasks - 2025):**
```makefile
.PHONY: dev build test lint format docs setup clean audit coverage

dev:
	cargo leptos watch

build:
	cargo leptos build --release

test:
	cargo test
	wasm-pack test --node

test-browser:
	wasm-pack test --chrome --headless

test-coverage:
	cargo tarpaulin --out html

lint:
	cargo clippy -- -D warnings
	
format:
	cargo fmt -- --check

docs:
	cargo doc --open --no-deps

audit:
	cargo audit

deny:
	cargo deny check

setup:
	rustup target add wasm32-unknown-unknown
	cargo install cargo-leptos wasm-pack cargo-audit cargo-tarpaulin cargo-deny
	docker-compose -f docker-compose.dev.yml up -d

clean:
	cargo clean
	rm -rf dist/ target/ coverage/

wasm-build:
	wasm-pack build --target web

wasm-optimize:
	wasm-opt -O4 -o dist/leptos_sync_opt.wasm dist/leptos_sync.wasm
```

## Project Structure

```
leptos-sync/
├── Cargo.toml              # Workspace configuration
├── README.md
├── LICENSE
├── .gitignore
├── .github/
│   ├── workflows/          # CI/CD pipelines
│   └── ISSUE_TEMPLATE/     # Issue templates
│
├── docs/                   # Documentation
│   ├── design.md
│   ├── architecture.md
│   ├── testing-strategy.md
│   └── deployment-operations.md
│
├── crates/
│   ├── leptos-sync-core/   # Core library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── collection/
│   │   │   ├── crdt/
│   │   │   ├── storage/
│   │   │   ├── sync/
│   │   │   └── transport/
│   │   └── tests/
│   │
│   ├── leptos-sync-components/ # UI Components
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── provider.rs
│   │   │   ├── status.rs
│   │   │   └── conflict_resolver.rs
│   │   └── tests/
│   │
│   ├── leptos-sync-macros/     # Derive macros
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── local_first.rs
│   │
│   └── leptos-sync-server/     # Server components
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── sync_coordinator.rs
│           └── websocket.rs
│
├── examples/
│   ├── todo-app/           # Complete todo app example
│   ├── collaborative-editor/  # Rich text editor
│   └── offline-blog/       # Blog with offline support
│
├── benchmarks/
│   ├── crdt_performance/
│   ├── storage_benchmarks/
│   └── sync_latency/
│
├── scripts/
│   ├── setup.sh           # Development setup
│   ├── test.sh            # Test runner
│   └── release.sh         # Release automation
│
└── .cargo/
    └── config.toml        # Cargo configuration for WASM
```

### Core Module Architecture

**leptos-sync-core/src/lib.rs (Updated for 2025):**
```rust
//! Leptos-Sync: Local-first library for Leptos applications
//! 
//! This crate provides the core functionality for building local-first,
//! offline-capable web applications with real-time synchronization.
//! 
//! # Features
//! 
//! - Local-first data storage with offline support
//! - CRDT-based conflict resolution
//! - Real-time synchronization
//! - Reactive queries and updates
//! - Progressive enhancement for older browsers

pub mod collection;     // LocalFirstCollection API
pub mod crdt;          // CRDT implementations
pub mod storage;       // Storage abstractions
pub mod sync;          // Synchronization engine
pub mod transport;     // Network transport layer
pub mod query;         // Query builder and execution
pub mod conflict;      // Conflict resolution
pub mod error;         // Error types

// Re-exports for convenience
pub use collection::LocalFirstCollection;
pub use crdt::{Mergeable, LwwValue};
pub use storage::{LocalStorage, HybridStorage};
pub use sync::{SyncManager, SyncStatus};
pub use query::{QueryBuilder, Op, Order};

// Derive macro re-export
pub use leptos_sync_macros::LocalFirst;

// Component re-exports
pub use leptos_sync_components::{
    LocalFirstProvider, SyncStatusIndicator, ConflictResolver
};

// Re-export common dependencies
pub use serde;
pub use serde_json;
pub use uuid;
```

## Contributing Guidelines

### Code Style and Standards

**Rust Style Guide (2025):**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Pass `cargo clippy -- -D warnings`
- Document all public APIs with examples
- Use `thiserror` for error types
- Prefer `Arc<dyn Trait>` over `Box<dyn Trait>` for shared ownership

**Example Documentation (Updated for 2025):**
```rust
/// Creates a new local-first collection with the specified name and configuration.
/// 
/// This function initializes a collection with the given name and optional configuration.
/// The collection will automatically handle storage selection, synchronization, and
/// conflict resolution based on the provided configuration.
/// 
/// # Arguments
/// 
/// * `name` - Unique identifier for the collection
/// * `config` - Optional configuration for storage and sync behavior
/// 
/// # Examples
/// 
/// ```rust
/// use leptos_sync::*;
/// 
/// #[derive(LocalFirst, Serialize, Deserialize, Clone)]
/// struct Task {
///     id: Uuid,
///     title: String,
/// }
/// 
/// let tasks = LocalFirstCollection::<Task>::new("tasks", None)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
/// 
/// # Errors
/// 
/// Returns `Err` if the collection name is invalid or storage initialization fails.
/// 
/// # Panics
/// 
/// This function will panic if the underlying storage system cannot be initialized.
pub fn new(name: &str, config: Option<Config>) -> Result<Self, Error> {
    // Implementation...
}
```

### Commit Message Convention

Use [Conventional Commits](https://www.conventionalcommits.org/) (Updated for 2025):

```
type(scope): description

[optional body]

[optional footer(s)]
```

**Examples:**
```
feat(crdt): add multi-value register CRDT implementation
fix(sync): resolve race condition in conflict resolution
docs(api): add examples for query builder
test(storage): add browser compatibility tests
perf(query): optimize query execution with indexing
refactor(transport): simplify WebSocket connection logic
```

**Types (2025):**
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `test`: Test additions/modifications
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks
- `ci`: CI/CD changes
- `build`: Build system changes

### Pull Request Process

1. **Fork and Branch:**
   ```bash
   git fork https://github.com/your-org/leptos-sync
   git checkout -b feature/new-crdt-type
   ```

2. **Develop and Test:**
   ```bash
   # Make changes
   cargo test
   cargo clippy -- -D warnings
   cargo fmt -- --check
   cargo audit
   ```

3. **Create PR:**
   - Clear title and description
   - Link related issues
   - Include breaking change notes
   - Add tests for new functionality

4. **PR Template (Updated for 2025):**
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update
   - [ ] Performance improvement

   ## Testing
   - [ ] Unit tests added/updated
   - [ ] Integration tests pass
   - [ ] Browser tests pass
   - [ ] WASM tests pass
   - [ ] Performance benchmarks updated

   ## Breaking Changes
   List any breaking changes

   ## Related Issues
   Fixes #123

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Documentation updated
   - [ ] Changelog entry added
   ```

### Code Review Guidelines

**For Authors:**
- Keep PRs focused and reasonably sized
- Write clear commit messages
- Add comprehensive tests
- Update documentation
- Include performance impact analysis

**For Reviewers:**
- Focus on correctness, performance, and maintainability
- Check for security implications
- Verify test coverage
- Ensure API consistency
- Review WASM compatibility

## API Reference

### Core Types (Updated for 2025)

```rust
use leptos_sync::*;

// Main collection type
pub struct LocalFirstCollection<T: Mergeable + 'static> { /* ... */ }

// CRDT trait for mergeable types
pub trait Mergeable: Clone + Serialize + DeserializeOwned + Send + Sync + PartialEq {
    fn merge(&mut self, other: &Self);
    fn conflicts(&self, other: &Self) -> Vec<Conflict>;
    // ... other methods
}

// Derive macro for automatic CRDT implementation
#[derive(LocalFirst)]
#[local_first(crdt = "lww")]
struct MyType {
    #[local_first(id)]
    id: Uuid,
    data: String,
}
```

### Query API (Updated for 2025)

```rust
// Build queries with method chaining
let results = collection
    .query()
    .filter("completed", Op::Eq, false.into())
    .filter("priority", Op::Gte, Priority::High.into())
    .sort_by("created_at", Order::Desc)
    .limit(50)
    .watch();  // Returns Signal<Vec<T>>

// Supported filter operations (2025)
pub enum Op {
    Eq,      // Equal
    Ne,      // Not equal
    Lt,      // Less than
    Le,      // Less than or equal
    Gt,      // Greater than
    Ge,      // Greater than or equal
    In,      // In array
    NotIn,   // Not in array
    Contains, // String contains
    Matches, // Regex match
    Exists,  // Field exists
    NotExists, // Field doesn't exist
}
```

### Sync API (Updated for 2025)

```rust
// Manual sync
let result = collection.sync().await?;

// Automatic sync configuration
let collection = LocalFirstCollection::builder("tasks")
    .with_auto_sync(Duration::from_secs(30))
    .with_transport(WebSocketTransport::new("wss://api.example.com/sync"))
    .with_storage(HybridStorage::new())
    .build()?;

// Sync status monitoring
let status = collection.sync_status(); // Returns Signal<SyncStatus>

// Conflict resolution
let conflicts = collection.conflicts();
if !conflicts.is_empty() {
    // Handle conflicts
    collection.resolve_conflicts(conflicts).await?;
}
```

## Testing Guide

### Unit Testing (Updated for 2025)

**Test Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use test_log::test; // For logging in tests
    
    #[test]
    fn test_crdt_merge_idempotent() {
        let mut item = create_test_item();
        let original = item.clone();
        
        item.merge(&original);
        
        assert_eq!(item, original, "CRDT merge should be idempotent");
    }
    
    // Property-based testing for CRDT laws
    proptest! {
        #[test]
        fn crdt_merge_commutative(a: TestItem, b: TestItem) {
            let mut ab = a.clone();
            ab.merge(&b);
            
            let mut ba = b.clone();
            ba.merge(&a);
            
            prop_assert_eq!(ab, ba);
        }
    }
}
```

### Integration Testing (Updated for 2025)

**Browser Testing:**
```rust
#[cfg(test)]
mod browser_tests {
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    async fn test_indexeddb_storage() {
        let storage = IndexedDbStorage::new("test-db").await.unwrap();
        
        let test_data = TestItem { id: 1, name: "Test".to_string() };
        storage.set("key1", &test_data).await.unwrap();
        
        let retrieved: TestItem = storage.get("key1").await.unwrap().unwrap();
        assert_eq!(retrieved, test_data);
    }
}
```

**WASM Testing:**
```rust
#[cfg(test)]
mod wasm_tests {
    use wasm_bindgen_test::*;
    
    #[wasm_bindgen_test]
    fn test_wasm_compilation() {
        // Test that WASM compilation works
        let collection = LocalFirstCollection::<TestItem>::new("test", None).unwrap();
        assert_eq!(collection.name(), "test");
    }
}
```

### Test Utilities (Updated for 2025)

```rust
// Test fixtures and utilities
pub mod test_utils {
    use super::*;
    
    pub fn create_test_collection<T: Mergeable + 'static>() -> LocalFirstCollection<T> {
        LocalFirstCollection::new_with_storage("test", MemoryStorage::new())
    }
    
    pub async fn create_mock_network() -> MockNetwork {
        MockNetwork::new().with_latency(Duration::from_millis(100))
    }
    
    pub fn assert_eventually_consistent<T: Mergeable + PartialEq>(
        items: &[T],
        timeout: Duration,
    ) {
        // Wait for all items to converge to same state
        let start = Instant::now();
        while start.elapsed() < timeout {
            if items.iter().all(|item| item == &items[0]) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("Items did not converge within timeout");
    }
    
    // Browser capability detection for tests
    pub async fn detect_browser_capabilities() -> BrowserCapabilities {
        // Detect OPFS, IndexedDB, WebRTC support
        BrowserCapabilities::detect().await
    }
}
```

## Release Process

### Versioning Strategy (Updated for 2025)

We follow [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR**: Breaking API changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

### Release Checklist (2025)

1. **Pre-release Testing:**
   ```bash
   # Run full test suite
   cargo test --all-features
   wasm-pack test --chrome --firefox --headless
   
   # Performance regression tests
   cargo bench
   
   # Security audit
   cargo audit
   
   # Dependency analysis
   cargo deny check
   
   # Documentation build
   cargo doc --no-deps
   
   # WASM build verification
   wasm-pack build --target web
   ```

2. **Version Bump:**
   ```bash
   # Update Cargo.toml versions
   cargo release patch --dry-run  # Review changes
   cargo release patch           # Execute release
   ```

3. **Release Notes (2025 format):**
   ```markdown
   ## v0.2.0 - 2025-03-01
   
   ### ✨ New Features
   - Added WebRTC peer-to-peer transport (#45)
   - Implemented presence awareness system (#52)
   - Added OPFS storage support for Chrome 108+ (#55)
   
   ### 🐛 Bug Fixes
   - Fixed race condition in sync coordinator (#48)
   - Resolved memory leak in query cache (#51)
   - Fixed IndexedDB compatibility with Safari 16+ (#54)
   
   ### 🔄 Changes
   - Improved CRDT merge performance by 40% (#49)
   - Updated Leptos dependency to 0.8.1 (#53)
   - Enhanced browser compatibility matrix (#56)
   
   ### ⚠️ Breaking Changes
   - `SyncTransport` trait now requires `Send + Sync` (#50)
   - Renamed `Collection::get_all()` to `Collection::list()` (#47)
   
   ### 🌐 Browser Support
   - Added Chrome 108+ with OPFS support
   - Enhanced Firefox 110+ compatibility
   - Improved Safari 16+ IndexedDB support
   ```

### Publication (2025)

```bash
# Publish to crates.io
cargo publish -p leptos-sync-core
cargo publish -p leptos-sync-macros
cargo publish -p leptos-sync-components
cargo publish -p leptos-sync  # Main crate last

# Update documentation site
# Update browser compatibility matrix
# Announce on social media and forums
```

## Troubleshooting

### Common Issues (Updated for 2025)

**1. WASM Build Failures:**
```bash
# Ensure wasm32 target installed
rustup target add wasm32-unknown-unknown

# Clear cache and rebuild
cargo clean
rm -rf ~/.cargo/registry/cache
cargo build --target wasm32-unknown-unknown

# Check WASM toolchain
wasm-pack --version
wasm-opt --version
```

**2. Storage Access Issues (2025):**
```javascript
// Check browser storage capabilities
console.log('OPFS:', 'storage' in navigator && 'getDirectory' in navigator.storage);
console.log('IndexedDB:', 'indexedDB' in window);
console.log('LocalStorage:', 'localStorage' in window);

// Check browser version
console.log('Chrome:', navigator.userAgent.includes('Chrome'));
console.log('Firefox:', navigator.userAgent.includes('Firefox'));
console.log('Safari:', navigator.userAgent.includes('Safari'));
```

**3. Sync Connection Problems:**
```rust
// Enable debug logging
RUST_LOG=leptos_sync::sync=debug,leptos_sync::transport=debug

// Check WebSocket connection
if let Err(e) = transport.connect("wss://api.example.com/sync").await {
    log::error!("Failed to connect: {:?}", e);
}

// Check browser WebSocket support
if !supports_websocket() {
    log::warn!("WebSocket not supported, falling back to HTTP polling");
}
```

### Debug Tools (Updated for 2025)

**Browser DevTools:**
```javascript
// Access collection state from console
window.leptosSyncDebug = {
    getCollection: (name) => /* internal accessor */,
    getSyncStatus: () => /* sync status */,
    forceSync: () => /* trigger manual sync */,
    getStorageInfo: () => /* storage capabilities */,
    getBrowserInfo: () => /* browser version and features */,
};
```

**Logging Configuration (2025):**
```bash
# Detailed logging
RUST_LOG=leptos_sync=trace,leptos=debug

# Component-specific logging
RUST_LOG=leptos_sync::crdt=debug,leptos_sync::sync=info

# WASM-specific logging
RUST_LOG=leptos_sync::storage=debug,leptos_sync::transport=debug
```

### Performance Profiling (Updated for 2025)

**WASM Profiling:**
```bash
# Build with debug info
cargo build --target wasm32-unknown-unknown --profile release-with-debug

# Use browser profiler
# Chrome: DevTools > Performance > Record
# Firefox: DevTools > Performance > Record
# Safari: Web Inspector > Timeline
```

**Memory Analysis (2025):**
```rust
// Track memory usage
#[cfg(feature = "memory-profiling")]
fn log_memory_usage() {
    if let Some(performance) = web_sys::window()
        .and_then(|w| w.performance())
        .and_then(|p| p.memory())
    {
        let used = performance.used_js_heap_size() / 1_000_000;
        let total = performance.total_js_heap_size() / 1_000_000;
        log::info!("Memory: {} MB used / {} MB total", used, total);
    }
}

// WASM memory profiling
#[cfg(feature = "wasm-memory-profiling")]
fn log_wasm_memory() {
    if let Some(memory) = wasm_bindgen::memory() {
        let size = memory.buffer().byte_length() / 1_000_000;
        log::info!("WASM Memory: {} MB", size);
    }
}
```

### Browser Compatibility Testing (2025)

**Automated Testing:**
```bash
# Test across multiple browsers
npm run test:browsers

# Test specific browser versions
npm run test:chrome:108
npm run test:firefox:110
npm run test:safari:16
```

**Manual Testing Checklist:**
- [ ] Chrome 108+ (OPFS, IndexedDB, WebRTC)
- [ ] Firefox 110+ (IndexedDB, WebRTC)
- [ ] Safari 16+ (IndexedDB)
- [ ] Edge 108+ (OPFS, IndexedDB, WebRTC)
- [ ] Mobile browsers (iOS Safari, Chrome Mobile)

This developer guide provides everything needed to contribute effectively to the Leptos-Sync project, from initial setup through advanced debugging and release management, updated for 2025 standards.