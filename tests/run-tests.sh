#!/bin/bash

# Layer9 Test Runner Script
# This script starts the HTTP server and runs all tests

echo "🚀 Layer9 Test Suite Runner"
echo "=========================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in the tests directory
if [ ! -f "package.json" ]; then
    echo -e "${RED}Error: This script must be run from the tests directory${NC}"
    exit 1
fi

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing dependencies...${NC}"
    npm install
    if [ $? -ne 0 ]; then
        echo -e "${RED}Failed to install dependencies${NC}"
        exit 1
    fi
fi

# Check if server is already running
if curl -s http://localhost:8000 > /dev/null; then
    echo -e "${GREEN}✅ Server is already running${NC}"
else
    echo -e "${YELLOW}Starting HTTP server...${NC}"
    # Start server in background from parent directory
    cd .. && python3 -m http.server 8000 &
    SERVER_PID=$!
    cd tests
    
    # Wait for server to start
    sleep 2
    
    # Verify server started
    if curl -s http://localhost:8000 > /dev/null; then
        echo -e "${GREEN}✅ Server started successfully (PID: $SERVER_PID)${NC}"
    else
        echo -e "${RED}Failed to start server${NC}"
        exit 1
    fi
fi

echo ""
echo "Running tests..."
echo ""

# Run the tests
npm test

TEST_EXIT_CODE=$?

# Kill server if we started it
if [ ! -z "$SERVER_PID" ]; then
    echo ""
    echo -e "${YELLOW}Stopping server...${NC}"
    kill $SERVER_PID 2>/dev/null
fi

echo ""
if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ All tests completed successfully!${NC}"
else
    echo -e "${RED}❌ Some tests failed${NC}"
fi

exit $TEST_EXIT_CODE