#!/bin/bash

echo "🚀 Layer9 Todo App - Simple Tests"
echo "================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Start server
echo -e "${YELLOW}Starting HTTP server...${NC}"
cd "$(dirname "$0")"
python3 serve.py > server.log 2>&1 &
SERVER_PID=$!

# Wait for server
sleep 3

# Check if server is running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}❌ Failed to start server${NC}"
    cat server.log
    exit 1
fi

echo -e "${GREEN}✅ Server started (PID: $SERVER_PID)${NC}"
echo ""

# Run tests
echo -e "${YELLOW}Running tests...${NC}"
echo ""

# Test 1: Check if index.html is served
echo "📋 Test 1: Checking if index.html is served..."
if curl -s http://localhost:8082 | grep -q "Layer9 Todo App"; then
    echo -e "${GREEN}✅ index.html served correctly${NC}"
else
    echo -e "${RED}❌ Failed to serve index.html${NC}"
fi

# Test 2: Check if WASM file is accessible
echo ""
echo "📋 Test 2: Checking if WASM file is accessible..."
if curl -s -I http://localhost:8082/pkg/layer9_example_todo_bg.wasm | grep -q "200 OK"; then
    echo -e "${GREEN}✅ WASM file accessible${NC}"
else
    echo -e "${RED}❌ WASM file not accessible${NC}"
fi

# Test 3: Check if JavaScript module is accessible
echo ""
echo "📋 Test 3: Checking if JS module is accessible..."
if curl -s -I http://localhost:8082/pkg/layer9_example_todo.js | grep -q "200 OK"; then
    echo -e "${GREEN}✅ JavaScript module accessible${NC}"
else
    echo -e "${RED}❌ JavaScript module not accessible${NC}"
fi

# Test 4: Check CORS headers
echo ""
echo "📋 Test 4: Checking CORS headers for WASM..."
HEADERS=$(curl -s -I http://localhost:8082/pkg/layer9_example_todo_bg.wasm)
if echo "$HEADERS" | grep -q "Cross-Origin-Embedder-Policy: require-corp"; then
    echo -e "${GREEN}✅ COEP header present${NC}"
else
    echo -e "${RED}❌ COEP header missing${NC}"
fi

if echo "$HEADERS" | grep -q "Cross-Origin-Opener-Policy: same-origin"; then
    echo -e "${GREEN}✅ COOP header present${NC}"
else
    echo -e "${RED}❌ COOP header missing${NC}"
fi

# Test 5: Check content type for WASM
echo ""
echo "📋 Test 5: Checking WASM content type..."
if curl -s -I http://localhost:8082/pkg/layer9_example_todo_bg.wasm | grep -q "application/wasm"; then
    echo -e "${GREEN}✅ Correct WASM content type${NC}"
else
    echo -e "${RED}❌ Incorrect WASM content type${NC}"
fi

# Create screenshots directory
echo ""
echo "📋 Creating screenshots directory..."
mkdir -p screenshots
echo -e "${GREEN}✅ Screenshots directory created${NC}"

# Summary
echo ""
echo "================================="
echo "📊 TEST SUMMARY"
echo "================================="
echo "All basic server tests completed!"
echo ""
echo "To run full browser tests, you'll need to:"
echo "1. Keep this server running"
echo "2. Install Playwright: npm install playwright"
echo "3. Run: npm run test:playwright"
echo ""

# Keep server running for manual testing
echo "Server is running at http://localhost:8082"
echo "Press Ctrl+C to stop..."

# Wait for Ctrl+C
trap "echo ''; echo '👋 Stopping server...'; kill $SERVER_PID 2>/dev/null; exit 0" INT
wait