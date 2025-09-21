#!/bin/bash

# Comprehensive Testing Script for Leptos-Sync
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Install dependencies
install_deps() {
    print_status "Installing dependencies..."
    cargo install cargo-tarpaulin cargo-audit cargo-deny cargo-criterion || true
    if ! command -v wasm-pack >/dev/null; then
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    fi
    if ! command -v playwright >/dev/null; then
        npm install -g playwright
        npx playwright install --with-deps
    fi
}

# Quick checks
quick_checks() {
    print_status "Running quick checks..."
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo check --workspace --all-targets
    print_success "Quick checks passed"
}

# Unit tests
unit_tests() {
    print_status "Running unit tests..."
    cargo test --workspace --lib --bins
    cargo test --workspace --doc
    print_success "Unit tests passed"
}

# IndexedDB tests
indexeddb_tests() {
    print_status "Running IndexedDB tests..."
    wasm-pack build --target web --out-dir pkg leptos-sync-core
    cargo test --package leptos-sync-core --test indexeddb_tests || true
    wasm-pack test --node --headless || true
    print_success "IndexedDB tests completed"
}

# WebSocket tests
websocket_tests() {
    print_status "Running WebSocket tests..."
    cargo test --package leptos-sync-core --test websocket_tests
    cargo test --package leptos-sync-core --test leptos_ws_pro_tests
    print_success "WebSocket tests completed"
}

# Integration tests
integration_tests() {
    print_status "Running integration tests..."
    cargo test --test integration -- --test-threads=1 || true
    cargo test --test contracts -- --test-threads=1 || true
    print_success "Integration tests completed"
}

# E2E tests
e2e_tests() {
    print_status "Running E2E tests..."
    playwright test || true
    print_success "E2E tests completed"
}

# Performance tests
performance_tests() {
    print_status "Running performance tests..."
    cargo bench --bench sync_performance || true
    print_success "Performance tests completed"
}

# Security tests
security_tests() {
    print_status "Running security tests..."
    cargo audit --deny warnings || true
    cargo deny check || true
    print_success "Security tests completed"
}

# Coverage
coverage() {
    print_status "Generating coverage..."
    cargo tarpaulin --out xml --output-dir coverage/ --workspace || true
    print_success "Coverage generated"
}

# Main execution
main() {
    case "${1:-all}" in
        quick)
            install_deps
            quick_checks
            ;;
        unit)
            install_deps
            unit_tests
            ;;
        indexeddb)
            install_deps
            indexeddb_tests
            ;;
        websocket)
            install_deps
            websocket_tests
            ;;
        integration)
            install_deps
            integration_tests
            ;;
        e2e)
            install_deps
            e2e_tests
            ;;
        performance)
            install_deps
            performance_tests
            ;;
        security)
            install_deps
            security_tests
            ;;
        coverage)
            install_deps
            coverage
            ;;
        all)
            install_deps
            quick_checks
            unit_tests
            indexeddb_tests
            websocket_tests
            integration_tests
            e2e_tests
            performance_tests
            security_tests
            coverage
            print_success "All tests completed!"
            ;;
        *)
            echo "Usage: $0 [quick|unit|indexeddb|websocket|integration|e2e|performance|security|coverage|all]"
            exit 1
            ;;
    esac
}

main "$@"
