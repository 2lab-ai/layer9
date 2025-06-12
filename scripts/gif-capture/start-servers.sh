#!/bin/bash

# Start all Layer9 example servers for GIF capture

echo "🚀 Starting Layer9 Example Servers"
echo "=================================="
echo ""

# Function to start a server
start_server() {
    local example=$1
    local port=$2
    local name=$3
    
    echo "Starting $name on port $port..."
    
    cd "../../examples/$example" 2>/dev/null || {
        echo "❌ Example directory not found: examples/$example"
        return 1
    }
    
    # Kill any existing process on this port
    lsof -ti:$port | xargs kill -9 2>/dev/null
    
    # Start the server in background
    python3 -m http.server $port > /dev/null 2>&1 &
    
    # Save PID for cleanup
    echo $! >> ../../scripts/gif-capture/.server-pids
    
    echo "✅ $name started (PID: $!)"
    cd - > /dev/null
}

# Clean up any existing PID file
rm -f .server-pids

# Start all servers
start_server "counter" 8080 "Counter"
start_server "async-counter" 8081 "Async Counter"
start_server "todo-app" 8082 "Todo App"
start_server "memory-game" 8083 "Memory Game"

echo ""
echo "✅ All servers started!"
echo ""
echo "Servers are running on:"
echo "  Counter:       http://localhost:8080"
echo "  Async Counter: http://localhost:8081"
echo "  Todo App:      http://localhost:8082"
echo "  Memory Game:   http://localhost:8083"
echo ""
echo "To stop all servers, run: ./stop-servers.sh"
echo ""
echo "Ready for GIF capture! Run:"
echo "  node capture-all-examples.js"