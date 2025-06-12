#!/bin/bash

# Build all Layer9 examples
echo "🚀 Building all Layer9 examples..."
echo "================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track build status
FAILED_BUILDS=""

# Function to build an example
build_example() {
    local example_name=$1
    local example_dir="examples/$example_name"
    
    echo -e "\n${BLUE}Building $example_name...${NC}"
    
    if [ -d "$example_dir" ]; then
        cd "$example_dir" || return
        
        if wasm-pack build --target web --out-dir pkg; then
            echo -e "${GREEN}✅ $example_name built successfully!${NC}"
        else
            echo -e "${RED}❌ $example_name build failed!${NC}"
            FAILED_BUILDS="$FAILED_BUILDS $example_name"
        fi
        
        cd - > /dev/null || return
    else
        echo -e "${RED}❌ $example_name directory not found!${NC}"
        FAILED_BUILDS="$FAILED_BUILDS $example_name"
    fi
}

# Build all examples
build_example "todo-app"
build_example "counter"
build_example "async-counter"
build_example "memory-game"

# Summary
echo -e "\n================================="
echo "📊 Build Summary"
echo "================================="

if [ -z "$FAILED_BUILDS" ]; then
    echo -e "${GREEN}✅ All examples built successfully!${NC}"
    echo ""
    echo "To run the examples:"
    echo "  cd examples/todo-app && python3 -m http.server 8080"
    echo "  cd examples/counter && python3 -m http.server 8081"
    echo "  cd examples/async-counter && python3 -m http.server 8082"
    echo "  cd examples/memory-game && python3 -m http.server 8083"
else
    echo -e "${RED}❌ Failed builds:$FAILED_BUILDS${NC}"
    exit 1
fi