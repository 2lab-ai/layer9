#!/bin/bash
# Build all WASM examples

set -e

echo "🔨 Building all WASM examples..."
echo "================================"

EXAMPLES=(
    "counter"
    "async-counter"
    "todo-app"
    "memory-game"
)

for example in "${EXAMPLES[@]}"; do
    echo ""
    echo "📦 Building $example..."
    if [ -d "examples/$example" ] && [ -f "examples/$example/Cargo.toml" ]; then
        wasm-pack build --target web --out-dir pkg examples/$example
        echo "✅ $example built successfully"
    else
        echo "⚠️  Skipping $example (not found)"
    fi
done

echo ""
echo "================================"
echo "✅ All WASM builds complete!"