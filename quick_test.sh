#!/bin/bash

echo "=== Quick Layer9 Examples Functionality Test ==="
echo

# Test counter example
echo "1. Testing Counter Example..."
curl -s http://localhost:8888/examples/counter/ > /tmp/counter.html
if grep -q "layer9_example_counter.js" /tmp/counter.html && grep -q "app" /tmp/counter.html; then
    echo "   ✅ Counter HTML loads correctly"
    echo "   ✅ References WASM module"
else
    echo "   ❌ Counter HTML issues"
fi

# Check if WASM loads
wasm_status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8888/examples/counter/pkg/layer9_example_counter_bg.wasm)
if [ "$wasm_status" = "200" ]; then
    echo "   ✅ WASM file accessible"
else
    echo "   ❌ WASM file not accessible (HTTP $wasm_status)"
fi

echo

# Test async-counter
echo "2. Testing Async Counter Example..."
curl -s http://localhost:8888/examples/async-counter/ > /tmp/async-counter.html
if grep -q "async_counter.js" /tmp/async-counter.html && grep -q "app" /tmp/async-counter.html; then
    echo "   ✅ Async Counter HTML loads correctly"
    echo "   ✅ References WASM module"
else
    echo "   ❌ Async Counter HTML issues"
fi

echo

# Test todo-app
echo "3. Testing Todo App Example..."
curl -s http://localhost:8888/examples/todo-app/ > /tmp/todo.html
if grep -q "layer9_example_todo.js" /tmp/todo.html && grep -q "app" /tmp/todo.html; then
    echo "   ✅ Todo App HTML loads correctly"
    echo "   ✅ References WASM module"
else
    echo "   ❌ Todo App HTML issues"
fi

echo

# Test memory-game
echo "4. Testing Memory Game Example..."
curl -s http://localhost:8888/examples/memory-game/ > /tmp/memory.html
if grep -q "layer9_example_memory_game.js" /tmp/memory.html && grep -q "app" /tmp/memory.html; then
    echo "   ✅ Memory Game HTML loads correctly"
    echo "   ✅ References WASM module"
else
    echo "   ❌ Memory Game HTML issues"
fi

echo

# Check for common issues in HTML files
echo "5. Checking for common issues..."
for file in /tmp/counter.html /tmp/async-counter.html /tmp/todo.html /tmp/memory.html; do
    if [ -f "$file" ]; then
        example=$(basename $file .html)
        echo -n "   $example: "
        
        # Check for proper module initialization
        if grep -q "init()" "$file" || grep -q "wasm_bindgen" "$file"; then
            echo "Has WASM initialization ✅"
        else
            echo "Missing WASM initialization ❌"
        fi
    fi
done

echo
echo "=== Browser Test Instructions ==="
echo "To test in browser:"
echo "1. Open http://localhost:8888/simple_test.html"
echo "2. Or test individual examples:"
echo "   - Counter: http://localhost:8888/examples/counter/"
echo "   - Async Counter: http://localhost:8888/examples/async-counter/"
echo "   - Todo App: http://localhost:8888/examples/todo-app/"
echo "   - Memory Game: http://localhost:8888/examples/memory-game/"
echo
echo "Check browser console (F12) for any errors!"