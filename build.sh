#!/bin/bash

# Layer9 Build Script

set -e

echo "🚀 Building Layer9 Framework..."

# Install wasm-pack if not installed
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the example
echo "🔨 Building Counter Example..."
cd examples/counter
wasm-pack build --target web --out-dir pkg --release

# Serve the example
echo "🌐 Starting development server..."
echo "📍 Open http://localhost:8080 in your browser"

# Simple Python server
python3 -m http.server 8080