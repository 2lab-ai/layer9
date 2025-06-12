#!/bin/bash

echo "🔍 Layer9 Todo App - Setup Verification"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if WASM files exist
echo "Checking WASM build files..."
if [ -f "../pkg/layer9_example_todo_bg.wasm" ]; then
    echo -e "${GREEN}✅ WASM file found${NC}"
    ls -lh ../pkg/layer9_example_todo_bg.wasm
else
    echo -e "${RED}❌ WASM file not found${NC}"
    echo "   Run 'wasm-pack build' in the parent directory first"
fi

echo ""
if [ -f "../pkg/layer9_example_todo.js" ]; then
    echo -e "${GREEN}✅ JS module found${NC}"
    ls -lh ../pkg/layer9_example_todo.js
else
    echo -e "${RED}❌ JS module not found${NC}"
fi

echo ""
echo "Starting temporary server to test..."
cd ..
python3 -m http.server 8080 > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2

if ps -p $SERVER_PID > /dev/null; then
    echo -e "${GREEN}✅ Server started successfully${NC}"
    echo ""
    echo "You can now:"
    echo "1. Open http://localhost:8080 in your browser"
    echo "2. Test all features manually using TESTING_GUIDE.md"
    echo ""
    echo "Server is running. Press Ctrl+C to stop..."
    
    trap "kill $SERVER_PID 2>/dev/null; echo ''; echo 'Server stopped.'; exit 0" INT
    wait
else
    echo -e "${RED}❌ Failed to start server${NC}"
fi