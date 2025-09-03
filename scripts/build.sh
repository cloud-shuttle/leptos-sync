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
