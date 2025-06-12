#!/bin/bash

# Main script to capture screenshots and create GIFs for all Layer9 examples

echo "🎬 Layer9 Automated GIF Creation Pipeline"
echo "========================================"
echo ""

# Change to script directory
cd "$(dirname "$0")"

# Check dependencies
check_dependencies() {
    echo "🔍 Checking dependencies..."
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        echo "❌ Node.js is not installed"
        exit 1
    fi
    
    # Check npm packages
    if [ ! -f "package.json" ]; then
        echo "📦 Creating package.json..."
        cat > package.json << EOF
{
  "name": "layer9-gif-capture",
  "version": "1.0.0",
  "description": "Automated GIF capture for Layer9 examples",
  "scripts": {
    "capture": "node capture-all-examples.js",
    "create-gifs": "node create-gifs.js",
    "start-servers": "./start-servers.sh",
    "stop-servers": "./stop-servers.sh"
  },
  "dependencies": {
    "puppeteer": "^21.0.0"
  }
}
EOF
    fi
    
    # Install dependencies
    if [ ! -d "node_modules" ]; then
        echo "📦 Installing npm dependencies..."
        npm install
    fi
    
    # Check ImageMagick
    if ! command -v convert &> /dev/null; then
        echo "❌ ImageMagick is not installed"
        echo "Install with:"
        echo "  macOS: brew install imagemagick"
        echo "  Linux: sudo apt-get install imagemagick"
        exit 1
    fi
    
    echo "✅ All dependencies satisfied"
    echo ""
}

# Make scripts executable
chmod +x *.sh

# Check dependencies
check_dependencies

# Option parsing
SKIP_BUILD=false
SKIP_CAPTURE=false
SKIP_GIFS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-capture)
            SKIP_CAPTURE=true
            shift
            ;;
        --skip-gifs)
            SKIP_GIFS=true
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --skip-build    Skip building examples"
            echo "  --skip-capture  Skip screenshot capture"
            echo "  --skip-gifs     Skip GIF creation"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Step 1: Build examples
if [ "$SKIP_BUILD" = false ]; then
    echo "🔨 Building Layer9 examples..."
    cd ../..
    
    # Build each example
    for example in counter async-counter todo-app memory-game; do
        if [ -d "examples/$example" ]; then
            echo "  Building $example..."
            cd "examples/$example"
            wasm-pack build --target web --out-dir pkg 2>/dev/null || {
                echo "  ⚠️  Failed to build $example"
            }
            cd ../..
        fi
    done
    
    cd scripts/gif-capture
    echo "✅ Build complete"
    echo ""
fi

# Step 2: Start servers
echo "🚀 Starting example servers..."
./stop-servers.sh > /dev/null 2>&1
./start-servers.sh
echo ""

# Wait for servers to start
echo "⏳ Waiting for servers to start..."
sleep 3

# Step 3: Capture screenshots
if [ "$SKIP_CAPTURE" = false ]; then
    echo "📸 Capturing screenshots..."
    node capture-all-examples.js
    
    if [ $? -ne 0 ]; then
        echo "❌ Screenshot capture failed"
        ./stop-servers.sh
        exit 1
    fi
    echo ""
fi

# Step 4: Create GIFs
if [ "$SKIP_GIFS" = false ]; then
    echo "🎨 Creating GIFs..."
    node create-gifs.js
    
    if [ $? -ne 0 ]; then
        echo "❌ GIF creation failed"
        ./stop-servers.sh
        exit 1
    fi
    echo ""
fi

# Step 5: Stop servers
echo "🛑 Stopping servers..."
./stop-servers.sh
echo ""

# Summary
echo "🎉 GIF creation pipeline complete!"
echo ""
echo "📁 Output locations:"
echo "  Screenshots: examples/*/screenshots/"
echo "  GIFs:        assets/gifs/"
echo ""
echo "📊 Next steps:"
echo "  1. Review the generated GIFs"
echo "  2. Update README.md with GIF links"
echo "  3. Optimize GIF sizes if needed"
echo ""
echo "To view the showcase GIF:"
echo "  open ../../assets/gifs/layer9-examples-showcase.gif"