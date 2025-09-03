#!/bin/bash
set -e

echo "🧪 Running tests..."

# Run unit tests
echo "�� Running unit tests..."
cargo test --all

# Run WASM tests
echo "�� Running WASM tests..."
wasm-pack test --node --headless

echo "✅ All tests passed!"
