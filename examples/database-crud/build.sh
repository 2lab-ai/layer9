#!/bin/bash

# Build script for database CRUD example

set -e

echo "Building database CRUD example..."

# Build WASM
wasm-pack build --target web --out-dir pkg

# Create dist directory
mkdir -p dist

# Copy files
cp index.html dist/
cp -r pkg dist/

echo "Build complete! Run the server with:"
echo "cargo run --bin server --features layer9-core/ssr"