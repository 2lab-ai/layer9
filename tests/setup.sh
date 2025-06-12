#!/bin/bash

# Layer9 Test Suite Setup Script

echo "🚀 Layer9 Test Suite Setup"
echo "========================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check Node.js
echo -e "${BLUE}Checking Node.js installation...${NC}"
if command -v node &> /dev/null; then
    NODE_VERSION=$(node -v)
    echo -e "${GREEN}✅ Node.js ${NODE_VERSION} installed${NC}"
else
    echo -e "${YELLOW}⚠️  Node.js not found. Please install Node.js first.${NC}"
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

# Check npm
echo -e "${BLUE}Checking npm installation...${NC}"
if command -v npm &> /dev/null; then
    NPM_VERSION=$(npm -v)
    echo -e "${GREEN}✅ npm ${NPM_VERSION} installed${NC}"
else
    echo -e "${YELLOW}⚠️  npm not found. Please install npm first.${NC}"
    exit 1
fi

# Install dependencies
echo ""
echo -e "${BLUE}Installing test dependencies...${NC}"
npm install

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Dependencies installed successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Failed to install dependencies${NC}"
    exit 1
fi

# Create directories
echo ""
echo -e "${BLUE}Creating directories...${NC}"
mkdir -p screenshots/{counter,async-counter,todo-app,memory-game}
mkdir -p doc-screenshots
echo -e "${GREEN}✅ Directories created${NC}"

# Check if examples are built
echo ""
echo -e "${BLUE}Checking if examples are built...${NC}"
EXAMPLES_BUILT=true

if [ ! -f "../examples/counter/pkg/layer9_example_counter_bg.wasm" ]; then
    echo -e "${YELLOW}⚠️  Counter example not built${NC}"
    EXAMPLES_BUILT=false
fi

if [ ! -f "../examples/async-counter/pkg/async_counter_bg.wasm" ]; then
    echo -e "${YELLOW}⚠️  Async counter example not built${NC}"
    EXAMPLES_BUILT=false
fi

if [ ! -f "../examples/todo-app/pkg/layer9_example_todo_bg.wasm" ]; then
    echo -e "${YELLOW}⚠️  Todo app example not built${NC}"
    EXAMPLES_BUILT=false
fi

if [ ! -f "../examples/memory-game/pkg/layer9_example_memory_game_bg.wasm" ]; then
    echo -e "${YELLOW}⚠️  Memory game example not built${NC}"
    EXAMPLES_BUILT=false
fi

if [ "$EXAMPLES_BUILT" = true ]; then
    echo -e "${GREEN}✅ All examples are built${NC}"
else
    echo ""
    echo -e "${YELLOW}Some examples need to be built. Run:${NC}"
    echo -e "${BLUE}  cd .. && ./build.sh${NC}"
fi

# Setup complete
echo ""
echo -e "${GREEN}✅ Setup complete!${NC}"
echo ""
echo "Next steps:"
echo -e "1. Start the HTTP server: ${BLUE}cd .. && python3 -m http.server 8000${NC}"
echo -e "2. Run all tests: ${BLUE}npm test${NC}"
echo -e "   Or use the convenient script: ${BLUE}./run-tests.sh${NC}"
echo ""
echo "For more information, see README.md"