#!/bin/bash

echo "=== Layer9 Examples Status Check ==="
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check which examples have pkg directories
echo "Checking build status..."
echo

for dir in examples/*/; do
    if [ -d "$dir" ]; then
        example_name=$(basename "$dir")
        if [ -d "${dir}pkg" ]; then
            echo -e "${GREEN}✓${NC} $example_name - Built (pkg directory exists)"
            # Check for WASM file
            if ls ${dir}pkg/*.wasm 1> /dev/null 2>&1; then
                wasm_file=$(ls ${dir}pkg/*.wasm | head -1)
                wasm_size=$(ls -lh "$wasm_file" | awk '{print $5}')
                echo "  └─ WASM file: $(basename $wasm_file) ($wasm_size)"
            fi
        else
            echo -e "${RED}✗${NC} $example_name - Not built (no pkg directory)"
        fi
        
        # Check for index.html
        if [ -f "${dir}index.html" ]; then
            echo "  └─ index.html: Found"
        else
            echo -e "  └─ index.html: ${RED}Missing${NC}"
        fi
    fi
done

echo
echo "=== Testing HTTP access ==="
echo

# Test each example via HTTP
for example in counter async-counter todo-app memory-game form-validation github-dashboard; do
    echo -n "Testing $example... "
    
    # Check if index.html is accessible
    status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8888/examples/$example/)
    
    if [ "$status_code" = "200" ]; then
        echo -e "${GREEN}✓${NC} (HTTP 200)"
        
        # Check if WASM file is accessible
        if [ -d "examples/$example/pkg" ]; then
            wasm_file=$(ls examples/$example/pkg/*.wasm 2>/dev/null | head -1)
            if [ -n "$wasm_file" ]; then
                wasm_name=$(basename "$wasm_file")
                wasm_status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8888/examples/$example/pkg/$wasm_name)
                if [ "$wasm_status" = "200" ]; then
                    echo "  └─ WASM accessible: ✓"
                else
                    echo -e "  └─ WASM accessible: ${RED}✗${NC} (HTTP $wasm_status)"
                fi
            fi
        fi
    else
        echo -e "${RED}✗${NC} (HTTP $status_code)"
    fi
done

echo
echo "=== Quick content check ==="
echo

# Check if examples are trying to load their WASM modules
for example in counter async-counter todo-app memory-game; do
    if [ -f "examples/$example/index.html" ]; then
        echo -n "Checking $example index.html... "
        
        # Look for wasm references
        if grep -q "wasm\|pkg" "examples/$example/index.html"; then
            echo -e "${GREEN}✓${NC} (References WASM/pkg)"
            
            # Extract the module path
            module_ref=$(grep -oE './pkg/[^"]+\.js' "examples/$example/index.html" | head -1)
            if [ -n "$module_ref" ]; then
                echo "  └─ Loading: $module_ref"
            fi
        else
            echo -e "${YELLOW}?${NC} (No WASM reference found)"
        fi
    fi
done

echo
echo "Server running at: http://localhost:8888"
echo "Test page available at: http://localhost:8888/simple_test.html"