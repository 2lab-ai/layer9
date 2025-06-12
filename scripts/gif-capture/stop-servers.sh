#!/bin/bash

# Stop all Layer9 example servers

echo "🛑 Stopping Layer9 Example Servers"
echo "=================================="
echo ""

# Kill servers by port
for port in 8080 8081 8082 8083; do
    pid=$(lsof -ti:$port)
    if [ ! -z "$pid" ]; then
        kill -9 $pid 2>/dev/null
        echo "✅ Stopped server on port $port (PID: $pid)"
    else
        echo "⚠️  No server running on port $port"
    fi
done

# Also kill any PIDs saved in the PID file
if [ -f ".server-pids" ]; then
    while read pid; do
        if kill -0 $pid 2>/dev/null; then
            kill -9 $pid 2>/dev/null
            echo "✅ Stopped process $pid"
        fi
    done < .server-pids
    rm -f .server-pids
fi

echo ""
echo "✅ All servers stopped!"