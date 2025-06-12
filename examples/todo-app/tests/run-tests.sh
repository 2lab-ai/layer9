#!/bin/bash

echo "🚀 Layer9 Todo App - Test Runner"
echo "================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm is not installed. Please install Node.js and npm first.${NC}"
    exit 1
fi

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 is not installed. Please install Python 3 first.${NC}"
    exit 1
fi

# Install dependencies
echo -e "${YELLOW}📦 Installing Puppeteer dependencies...${NC}"
cd "$(dirname "$0")"
npm install

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Failed to install dependencies${NC}"
    exit 1
fi

# Start the HTTP server in background
echo -e "${YELLOW}🌐 Starting HTTP server...${NC}"
python3 serve.py &
SERVER_PID=$!

# Give the server time to start
sleep 2

# Check if server is running
if ! ps -p $SERVER_PID > /dev/null; then
    echo -e "${RED}❌ Failed to start HTTP server${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Server started (PID: $SERVER_PID)${NC}"
echo ""

# Run the Puppeteer tests
echo -e "${YELLOW}🧪 Running Puppeteer tests...${NC}"
echo ""

npm test

TEST_RESULT=$?

# Kill the server
echo ""
echo -e "${YELLOW}🛑 Stopping HTTP server...${NC}"
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

# Exit with the test result
if [ $TEST_RESULT -eq 0 ]; then
    echo -e "${GREEN}✅ All tests completed successfully!${NC}"
else
    echo -e "${RED}❌ Tests failed with exit code: $TEST_RESULT${NC}"
fi

exit $TEST_RESULT