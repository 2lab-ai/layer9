#!/bin/bash
# WASM optimization script

set -e

echo "🔧 Optimizing WASM bundles..."

# Install wasm-opt if not present
if ! command -v wasm-opt &> /dev/null; then
    echo "Installing wasm-opt..."
    npm install -g wasm-opt
fi

# Optimize counter example
if [ -f "examples/counter/pkg/layer9_example_counter_bg.wasm" ]; then
    echo "Optimizing counter example..."
    wasm-opt -Oz         examples/counter/pkg/layer9_example_counter_bg.wasm         -o examples/counter/pkg/layer9_example_counter_bg_opt.wasm
    mv examples/counter/pkg/layer9_example_counter_bg_opt.wasm        examples/counter/pkg/layer9_example_counter_bg.wasm
fi

# Add tree shaking to build
echo "✅ Bundle optimization complete"
