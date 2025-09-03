#!/bin/bash
set -e

echo "🔨 Building WASM packages..."

# Build core library WASM
echo "📦 Building core library WASM..."
cd leptos-sync-core
wasm-pack build --target web --out-dir ../../pkg/core
cd ..

# Build components WASM
echo "🎨 Building components WASM..."
cd leptos-sync-components
wasm-pack build --target web --out-dir ../../pkg/components
cd ..

echo "✅ WASM build completed successfully!"
echo "📁 Packages available in ./pkg/"
