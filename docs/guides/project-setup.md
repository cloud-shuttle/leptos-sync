# Project Setup Guide for Leptos-Sync

## Overview

**Last Updated:** September 3rd, 2025  
**Target:** Getting developers up and running with Leptos-Sync development  
**Prerequisites:** Rust 1.75+, Node.js 18+, Git  
**Estimated Setup Time:** 15-30 minutes

This guide walks you through setting up the Leptos-Sync development environment from scratch, including project creation, dependency installation, and initial configuration.

## Prerequisites

### 1. **System Requirements**

**Operating Systems:**
- ✅ **macOS**: 12.0+ (Monterey)
- ✅ **Linux**: Ubuntu 20.04+, Debian 11+, CentOS 8+
- ✅ **Windows**: Windows 10/11 with WSL2 (recommended)

**Hardware Requirements:**
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 10GB free space minimum
- **CPU**: Multi-core processor (2+ cores recommended)

### 2. **Required Software**

**Core Dependencies:**
```bash
# Rust toolchain (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
rustup target add wasm32-unknown-unknown

# Node.js (18+)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 18
nvm use 18

# Git
# macOS: brew install git
# Ubuntu/Debian: sudo apt install git
# Windows: https://git-scm.com/download/win
```

**Development Tools:**
```bash
# WASM tools
cargo install wasm-pack
cargo install wasm-opt

# Development utilities
cargo install cargo-audit
cargo install cargo-tarpaulin
cargo install cargo-deny
cargo install cargo-flamegraph

# Leptos CLI
cargo install cargo-leptos

# Code quality tools
cargo install cargo-fmt
cargo install cargo-clippy
```

## Project Creation

### 1. **Repository Setup**

**Clone the Repository:**
```bash
# Clone the main repository
git clone https://github.com/your-org/leptos-sync.git
cd leptos-sync

# Set up git hooks (optional but recommended)
git config core.hooksPath .githooks
chmod +x .githooks/*
```

**Verify Setup:**
```bash
# Check Rust version
rustc --version  # Should be 1.75.0 or higher

# Check WASM target
rustup target list | grep wasm32-unknown-unknown

# Check Node.js version
node --version  # Should be 18.0.0 or higher

# Check cargo-leptos
cargo leptos --version
```

### 2. **Project Structure**

**Initial Directory Structure:**
```
leptos-sync/
├── Cargo.toml                 # Workspace configuration
├── .cargo/                    # Cargo configuration
│   └── config.toml
├── leptos-sync-core/          # Core library crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── storage/
│   │   ├── crdt/
│   │   ├── sync/
│   │   └── error.rs
│   └── tests/
├── leptos-sync-macros/        # Derive macros crate
│   ├── Cargo.toml
│   └── src/
├── leptos-sync-components/    # Leptos components crate
│   ├── Cargo.toml
│   └── src/
├── leptos-sync-examples/      # Example applications
│   ├── Cargo.toml
│   └── src/
├── docs/                      # Documentation
├── scripts/                   # Build and deployment scripts
├── .github/                   # GitHub workflows
├── k8s/                       # Kubernetes manifests
└── infrastructure/            # Terraform configurations
```

## Development Environment Setup

### 1. **IDE Configuration**

**VS Code Setup:**
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": true,
        "source.organizeImports": true
    },
    "files.associations": {
        "*.rs": "rust",
        "*.toml": "toml",
        "*.md": "markdown"
    }
}
```

**Recommended VS Code Extensions:**
```json
// .vscode/extensions.json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "tamasfe.even-better-toml",
        "ms-vscode.vscode-json",
        "bradlc.vscode-tailwindcss",
        "esbenp.prettier-vscode",
        "ms-vscode.vscode-typescript-next",
        "ms-vscode.vscode-js-debug"
    ]
}
```

### 2. **Cargo Configuration**

**Workspace Configuration:**
```toml
# Cargo.toml
[workspace]
members = [
    "leptos-sync-core",
    "leptos-sync-macros", 
    "leptos-sync-components",
    "leptos-sync-examples"
]

resolver = "2"

[workspace.dependencies]
leptos = "0.8"
leptos-ws = "0.8"
web-sys = "0.3"
wasm-bindgen = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "chrono"] }
redis = { version = "0.23", features = ["tokio-comp"] }
prometheus = "0.13"
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Local-first, offline-capable data synchronization for Leptos applications"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-org/leptos-sync"
keywords = ["leptos", "sync", "crdt", "local-first", "offline"]
categories = ["web-programming", "asynchronous", "data-structures"]
```

**Cargo Configuration:**
```toml
# .cargo/config.toml
[build]
target = "wasm32-unknown-unknown"

[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "link-arg=--import-memory",
    "-C", "link-arg=--initial-memory=65536",
    "-C", "link-arg=--max-memory=65536",
    "-C", "link-arg=--stack-first",
    "-C", "link-arg=--export-table",
]

[alias]
test-all = "test --all --all-features"
test-wasm = "wasm-pack test --node --headless"
test-browser = "wasm-pack test --chrome --headless"
bench-all = "bench --all --all-features"
check-all = "check --all --all-features"
clippy-all = "clippy --all --all-features -- -D warnings"
fmt-all = "fmt --all"
audit-all = "audit --all"
```

### 3. **Environment Variables**

**Development Environment:**
```bash
# .env.development
ENVIRONMENT=development
LOG_LEVEL=debug
RUST_LOG=leptos_sync=debug,tokio=info

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/leptos_sync_dev
REDIS_URL=redis://localhost:6379/0

# Sync
SYNC_ENDPOINT=ws://localhost:8080/sync
SYNC_INTERVAL=30

# Security
JWT_SECRET=your-dev-secret-key
ENCRYPTION_KEY=your-dev-encryption-key

# Performance
CACHE_TTL=300
MAX_CONNECTIONS=10
```

**Production Environment:**
```bash
# .env.production
ENVIRONMENT=production
LOG_LEVEL=info
RUST_LOG=leptos_sync=info

# Database
DATABASE_URL=postgresql://user:pass@prod-db:5432/leptos_sync
REDIS_URL=redis://prod-redis:6379/0

# Sync
SYNC_ENDPOINT=wss://sync.yourdomain.com
SYNC_INTERVAL=60

# Security
JWT_SECRET=${JWT_SECRET}
ENCRYPTION_KEY=${ENCRYPTION_KEY}

# Performance
CACHE_TTL=1800
MAX_CONNECTIONS=100
```

## Initial Development Setup

### 1. **Core Library Setup**

**Create Core Library:**
```bash
# Create the core library directory
mkdir -p leptos-sync-core/src
cd leptos-sync-core

# Initialize Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "leptos-sync-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
description = "Core synchronization library for Leptos applications"
license.workspace = true
repository.workspace = true
keywords.workspace = true
categories.workspace = true

[dependencies]
leptos.workspace = true
leptos-ws.workspace = true
web-sys.workspace = true
wasm-bindgen.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
uuid.workspace = true
thiserror.workspace = true
tracing.workspace = true
chrono.workspace = true

[dev-dependencies]
tokio-test = "0.4"
proptest = "1.0"
criterion = "0.5"
wasm-bindgen-test = "0.3"
EOF

# Create initial lib.rs
cat > src/lib.rs << 'EOF'
//! Leptos-Sync Core Library
//! 
//! This library provides local-first, offline-capable data synchronization
//! for Leptos applications using CRDTs and conflict resolution.

pub mod storage;
pub mod crdt;
pub mod sync;
pub mod error;
pub mod collection;
pub mod query;
pub mod transport;

pub use collection::LocalFirstCollection;
pub use error::{Error, Result};

/// Re-export common dependencies for convenience
pub use serde;
pub use serde_json;
pub use uuid;

/// Features available in this crate
pub mod features {
    /// Enable encryption support
    pub const ENCRYPTION: &str = "encryption";
    
    /// Enable compression support
    pub const COMPRESSION: &str = "compression";
    
    /// Enable metrics collection
    pub const METRICS: &str = "metrics";
    
    /// Enable distributed tracing
    pub const TRACING: &str = "tracing";
}
EOF
```

### 2. **Macros Library Setup**

**Create Macros Library:**
```bash
# Create the macros library directory
mkdir -p leptos-sync-macros/src
cd leptos-sync-macros

# Initialize Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "leptos-sync-macros"
version.workspace = true
edition.workspace = true
authors.workspace = true
description = "Derive macros for Leptos-Sync"
license.workspace = true
repository.workspace = true

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
EOF

# Create initial lib.rs
cat > src/lib.rs << 'EOF'
//! Leptos-Sync Macros
//! 
//! This crate provides derive macros for automatic CRDT implementation
//! and local-first collection setup.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for automatic CRDT implementation
#[proc_macro_derive(LocalFirst, attributes(local_first))]
pub fn derive_local_first(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Implementation will be added here
    let name = input.ident;
    
    let expanded = quote! {
        impl LocalFirst for #name {
            // Auto-generated implementation
        }
    };
    
    TokenStream::from(expanded)
}
EOF
```

### 3. **Components Library Setup**

**Create Components Library:**
```bash
# Create the components library directory
mkdir -p leptos-sync-components/src
cd leptos-sync-components

# Initialize Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "leptos-sync-components"
version.workspace = true
edition.workspace = true
authors.workspace = true
description = "Leptos components for synchronization UI"
license.workspace = true
repository.workspace = true

[dependencies]
leptos.workspace = true
leptos-sync-core = { path = "../leptos-sync-core" }
EOF

# Create initial lib.rs
cat > src/lib.rs << 'EOF'
//! Leptos-Sync Components
//! 
//! This crate provides Leptos components for building synchronization
//! user interfaces.

pub mod provider;
pub mod status;
pub mod resolver;

pub use provider::LocalFirstProvider;
pub use status::SyncStatusIndicator;
pub use resolver::ConflictResolver;
EOF
```

## Build and Test Setup

### 1. **Build Scripts**

**Create Build Scripts:**
```bash
# Create scripts directory
mkdir -p scripts
chmod +x scripts/*

# Create main build script
cat > scripts/build.sh << 'EOF'
#!/bin/bash
set -e

echo "🔨 Building Leptos-Sync..."

# Build core library
echo "📦 Building core library..."
cargo build --package leptos-sync-core

# Build macros
echo "🔧 Building macros..."
cargo build --package leptos-sync-macros

# Build components
echo "🎨 Building components..."
cargo build --package leptos-sync-components

# Build examples
echo "📚 Building examples..."
cargo build --package leptos-sync-examples

echo "✅ Build completed successfully!"
EOF

# Create WASM build script
cat > scripts/build-wasm.sh << 'EOF'
#!/bin/bash
set -e

echo "🌐 Building WASM packages..."

# Build core WASM
echo "📦 Building core WASM..."
cd leptos-sync-core
wasm-pack build --target web --out-dir ../../pkg/core

# Build components WASM
echo "🎨 Building components WASM..."
cd ../leptos-sync-components
wasm-pack build --target web --out-dir ../../pkg/components

echo "✅ WASM build completed successfully!"
EOF

# Create test script
cat > scripts/test.sh << 'EOF'
#!/bin/bash
set -e

echo "🧪 Running tests..."

# Run unit tests
echo "📝 Running unit tests..."
cargo test --all

# Run WASM tests
echo "🌐 Running WASM tests..."
wasm-pack test --node --headless

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test integration

echo "✅ All tests passed!"
EOF
```

### 2. **Makefile for Common Tasks**

**Create Makefile:**
```makefile
# Makefile
.PHONY: help build test clean wasm docs

help: ## Show this help message
	@echo "Leptos-Sync Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build all crates
	cargo build --all

build-release: ## Build all crates in release mode
	cargo build --all --release

test: ## Run all tests
	cargo test --all

test-wasm: ## Run WASM tests
	wasm-pack test --node --headless

test-browser: ## Run browser tests
	wasm-pack test --chrome --headless

clean: ## Clean build artifacts
	cargo clean
	rm -rf pkg/
	rm -rf target/

wasm: ## Build WASM packages
	./scripts/build-wasm.sh

docs: ## Generate documentation
	cargo doc --all --no-deps --open

check: ## Run all checks
	cargo check --all
	cargo clippy --all -- -D warnings
	cargo fmt --all -- --check

audit: ## Security audit
	cargo audit

coverage: ## Generate test coverage
	cargo tarpaulin --out html --output-dir coverage

install-deps: ## Install development dependencies
	cargo install wasm-pack
	cargo install wasm-opt
	cargo install cargo-audit
	cargo install cargo-tarpaulin
	cargo install cargo-deny
	cargo install cargo-flamegraph
	cargo install cargo-leptos

setup: install-deps ## Complete development setup
	rustup target add wasm32-unknown-unknown
	git config core.hooksPath .githooks
	chmod +x .githooks/*
```

### 3. **Package.json for Node.js Tools**

**Create Package.json:**
```json
{
  "name": "leptos-sync",
  "version": "0.1.0",
  "description": "Local-first, offline-capable data synchronization for Leptos applications",
  "scripts": {
    "build": "make build",
    "build:wasm": "make wasm",
    "test": "make test",
    "test:wasm": "make test-wasm",
    "test:browser": "make test-browser",
    "clean": "make clean",
    "check": "make check",
    "audit": "make audit",
    "coverage": "make coverage",
    "docs": "make docs",
    "setup": "make setup",
    "install-deps": "make install-deps"
  },
  "devDependencies": {
    "wasm-opt": "^0.4.0"
  },
  "engines": {
    "node": ">=18.0.0",
    "npm": ">=8.0.0"
  }
}
```

## Docker Development Setup

### 1. **Docker Compose for Services**

**Create Docker Compose:**
```yaml
# docker-compose.yml
version: '3.8'

services:
  # PostgreSQL database
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: leptos_sync_dev
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U user -d leptos_sync_dev"]
      interval: 10s
      timeout: 5s
      retries: 5

  # Redis cache
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  # Development API server
  api:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "8080:8080"
    environment:
      - ENVIRONMENT=development
      - DATABASE_URL=postgresql://user:pass@postgres:5432/leptos_sync_dev
      - REDIS_URL=redis://redis:6379/0
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    volumes:
      - .:/app
      - cargo_cache:/usr/local/cargo/registry
      - target_cache:/app/target

volumes:
  postgres_data:
  redis_data:
  cargo_cache:
  target_cache:
```

### 2. **Development Dockerfile**

**Create Development Dockerfile:**
```dockerfile
# Dockerfile.dev
FROM rust:1.75-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Install WASM tools
RUN cargo install wasm-pack wasm-opt

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY leptos-sync-core/Cargo.toml ./leptos-sync-core/
COPY leptos-sync-macros/Cargo.toml ./leptos-sync-macros/
COPY leptos-sync-components/Cargo.toml ./leptos-sync-components/

# Pre-build dependencies
RUN cargo build --all

# Copy source code
COPY . .

# Build the application
RUN cargo build --all

# Expose port
EXPOSE 8080

# Start development server
CMD ["cargo", "run", "--package", "leptos-sync-examples", "--bin", "api-server"]
```

## Getting Started

### 1. **First Run**

**Complete Setup:**
```bash
# 1. Clone and setup
git clone https://github.com/your-org/leptos-sync.git
cd leptos-sync

# 2. Install dependencies
make install-deps

# 3. Setup development environment
make setup

# 4. Start development services
docker-compose up -d

# 5. Build the project
make build

# 6. Run tests
make test

# 7. Start development server
cargo run --package leptos-sync-examples --bin todo-app
```

### 2. **Development Workflow**

**Typical Development Session:**
```bash
# 1. Start development services
docker-compose up -d

# 2. Make code changes
# ... edit files ...

# 3. Run checks
make check

# 4. Run tests
make test

# 5. Build and test WASM
make wasm
make test-wasm

# 6. Run examples
cargo run --package leptos-sync-examples --bin todo-app

# 7. Generate documentation
make docs
```

### 3. **Common Commands**

**Development Commands:**
```bash
# Build everything
make build

# Run all tests
make test

# Check code quality
make check

# Build WASM packages
make wasm

# Generate coverage report
make coverage

# Security audit
make audit

# Clean everything
make clean

# View help
make help
```

## Troubleshooting

### 1. **Common Issues**

**WASM Build Issues:**
```bash
# If wasm-pack is not found
cargo install wasm-pack

# If WASM target is missing
rustup target add wasm32-unknown-unknown

# If Node.js version is too old
nvm install 18
nvm use 18
```

**Database Connection Issues:**
```bash
# Check if PostgreSQL is running
docker-compose ps postgres

# Check database logs
docker-compose logs postgres

# Reset database
docker-compose down -v
docker-compose up -d
```

**Build Performance Issues:**
```bash
# Use cargo cache in Docker
# The docker-compose.yml already includes volume mounts for caching

# Enable parallel compilation
export CARGO_BUILD_JOBS=$(nproc)

# Use release builds for testing
cargo build --release
```

### 2. **Performance Optimization**

**Cargo Configuration:**
```toml
# .cargo/config.toml
[build]
jobs = 0  # Use all available cores

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=native"]

[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "target-cpu=mvp",
    "-C", "link-arg=--import-memory",
    "-C", "link-arg=--initial-memory=65536",
    "-C", "link-arg=--max-memory=65536",
    "-C", "link-arg=--stack-first",
    "-C", "link-arg=--export-table",
]
```

## Conclusion

This setup guide provides everything needed to get started with Leptos-Sync development. The environment is configured for:

- ✅ **Fast Development**: Hot reloading and incremental builds
- ✅ **Comprehensive Testing**: Unit, integration, and WASM tests
- ✅ **Quality Assurance**: Linting, formatting, and security audits
- ✅ **Performance**: Optimized builds and caching
- ✅ **Documentation**: Auto-generated API documentation
- ✅ **Containerization**: Docker development environment

Once setup is complete, you can begin developing local-first, offline-capable applications with real-time synchronization capabilities.

**Next Steps:**
1. Review the [API Reference](api-reference.md) for available functionality
2. Check the [Testing Strategy](testing-strategy.md) for testing guidelines
3. Explore the [Security Guide](security-privacy.md) for security considerations
4. Review the [Deployment Guide](deployment-operations.md) for production deployment

Happy coding! 🚀
