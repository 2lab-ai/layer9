#!/bin/bash
# Build all examples in release mode with optimizations

set -e

echo "🚀 Building all examples in release mode..."

# Array of examples to build
EXAMPLES=(
    "counter"
    "todo-app"
    "async-counter"
    "memory-game"
    "forms-demo"
    "middleware-test"
)

# Build each example
for example in "${EXAMPLES[@]}"; do
    if [ -d "examples/$example" ]; then
        echo "📦 Building $example..."
        cd "examples/$example"
        
        # Build in release mode
        wasm-pack build --target web --out-dir pkg --release
        
        # Get before size
        if [ -f "pkg/${example//-/_}_bg.wasm" ]; then
            BEFORE_SIZE=$(ls -lh "pkg/${example//-/_}_bg.wasm" | awk '{print $5}')
        else
            # Handle naming variations
            WASM_FILE=$(find pkg -name "*_bg.wasm" | head -1)
            if [ -n "$WASM_FILE" ]; then
                BEFORE_SIZE=$(ls -lh "$WASM_FILE" | awk '{print $5}')
            fi
        fi
        
        # Optimize with wasm-opt if available
        if command -v wasm-opt &> /dev/null; then
            WASM_FILE=$(find pkg -name "*_bg.wasm" | head -1)
            if [ -n "$WASM_FILE" ]; then
                echo "  Optimizing with wasm-opt -Oz..."
                wasm-opt -Oz "$WASM_FILE" -o "${WASM_FILE}.opt"
                mv "${WASM_FILE}.opt" "$WASM_FILE"
                
                # Get after size
                AFTER_SIZE=$(ls -lh "$WASM_FILE" | awk '{print $5}')
                echo "  ✅ $example: $BEFORE_SIZE → $AFTER_SIZE"
            fi
        fi
        
        cd ../..
    fi
done

echo ""
echo "📊 Final bundle sizes:"
find examples -name "*_bg.wasm" -exec ls -lh {} \; | awk '{print $5, $9}' | sort -h

echo ""
echo "✅ All examples built in release mode!"