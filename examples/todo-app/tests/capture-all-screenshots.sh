#!/bin/bash

# Capture All Screenshots Script
# This script runs all screenshot capture scripts for the Todo App

echo "🎬 Todo App Screenshot Capture Suite"
echo "==================================="
echo ""

# Check if node is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js first."
    exit 1
fi

# Check if the server is running
echo "🔍 Checking if server is running on port 8082..."
if ! curl -s http://localhost:8082 > /dev/null; then
    echo "❌ Server is not running on http://localhost:8082"
    echo "Please start the server first with: python3 serve.py (and ensure it's on port 8082)"
    exit 1
fi
echo "✅ Server is running"
echo ""

# Create main screenshots directory if it doesn't exist
mkdir -p screenshots

# Run demo screenshot capture
echo "📸 Capturing demo screenshots for GIF creation..."
echo "================================================"
node capture-demo-screenshots.js
echo ""

# Run realistic demo capture
echo "📸 Capturing realistic demo screenshots..."
echo "========================================"
node generate-realistic-demo.js
echo ""

# Summary
echo "📊 Screenshot Capture Summary"
echo "============================"
echo "✅ Demo screenshots: screenshots/demo/"
echo "✅ Realistic demos: screenshots/realistic-demo/"
echo ""
echo "Next steps:"
echo "1. Review the screenshots in the directories above"
echo "2. Use CREATE_GIFS_GUIDE.md to create animated GIFs"
echo "3. Choose the best screenshots for your documentation"
echo ""
echo "🎉 All screenshots captured successfully!"