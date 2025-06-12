#!/bin/bash

# Create GIFs from Screenshots
# This script creates various GIF animations from captured screenshots

echo "🎬 Creating GIFs from Todo App Screenshots"
echo "========================================"
echo ""

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "❌ ImageMagick is not installed."
    echo ""
    echo "To install on macOS:"
    echo "  brew install imagemagick"
    echo ""
    echo "To install on Ubuntu/Debian:"
    echo "  sudo apt-get install imagemagick"
    echo ""
    exit 1
fi

# Create output directory for GIFs
mkdir -p screenshots/gifs

# Function to create a GIF with progress indicator
create_gif() {
    local input_pattern=$1
    local output_file=$2
    local description=$3
    local delay=${4:-100}
    
    echo "Creating: $description..."
    convert -delay $delay -loop 0 $input_pattern -resize 800x600 \
            -coalesce -layers OptimizePlus "screenshots/gifs/$output_file"
    echo "✅ Created: screenshots/gifs/$output_file"
    echo ""
}

# Check if demo screenshots exist
if [ ! -d "screenshots/demo" ]; then
    echo "❌ Demo screenshots not found. Run capture-all-screenshots.sh first."
    exit 1
fi

cd screenshots/demo

# 1. Complete demo GIF (all screenshots)
create_gif "*.png" "todo-app-complete-demo.gif" "Complete app demo" 100

# 2. Quick overview GIF (key frames only)
create_gif "001-empty-state.png 004-first-todo-added.png 008-multiple-todos.png 010-multiple-completed.png 011-filter-active.png 015-todo-deleted.png 017-realistic-final-state.png" \
           "todo-app-overview.gif" "Quick overview" 150

# 3. Add todo feature GIF
create_gif "001-empty-state.png 002-input-focused.png 003-typing-first-todo.png 004-first-todo-added.png 005-typing-second-todo.png 006-second-todo-added.png" \
           "todo-add-feature.gif" "Add todo feature" 100

# 4. Filter feature GIF
create_gif "008-multiple-todos.png 009-first-todo-completed.png 010-multiple-completed.png 011-filter-active.png 012-filter-completed.png 013-filter-all.png" \
           "todo-filter-feature.gif" "Filter feature" 120

# 5. Delete and clear GIF
create_gif "013-filter-all.png 014-hover-delete-button.png 015-todo-deleted.png 016-cleared-completed.png" \
           "todo-delete-clear-feature.gif" "Delete and clear features" 120

# 6. Small header GIF for README
convert -delay 80 -loop 0 \
        001-empty-state.png \
        008-multiple-todos.png \
        010-multiple-completed.png \
        017-realistic-final-state.png \
        -resize 600x400 -colors 128 \
        ../../screenshots/gifs/todo-app-header.gif
echo "✅ Created: screenshots/gifs/todo-app-header.gif (optimized for README)"
echo ""

cd ../..

# Create GIFs from realistic demos if they exist
if [ -d "screenshots/realistic-demo" ]; then
    echo "Creating GIFs from realistic demos..."
    cd screenshots/realistic-demo
    
    # Work tasks GIF
    create_gif "work-tasks.png mixed-tasks.png active-tasks-only.png completed-tasks-only.png" \
               "work-flow.gif" "Work task management flow" 150
    
    # Full workflow GIF
    create_gif "*.png" "realistic-workflow.gif" "Complete realistic workflow" 120
    
    cd ../..
fi

# Summary
echo "📊 GIF Creation Summary"
echo "====================="
echo "✅ Complete demo: screenshots/gifs/todo-app-complete-demo.gif"
echo "✅ Quick overview: screenshots/gifs/todo-app-overview.gif"
echo "✅ Add feature: screenshots/gifs/todo-add-feature.gif"
echo "✅ Filter feature: screenshots/gifs/todo-filter-feature.gif"
echo "✅ Delete/Clear: screenshots/gifs/todo-delete-clear-feature.gif"
echo "✅ README header: screenshots/gifs/todo-app-header.gif"

if [ -d "screenshots/realistic-demo" ]; then
    echo "✅ Work flow: screenshots/gifs/work-flow.gif"
    echo "✅ Realistic workflow: screenshots/gifs/realistic-workflow.gif"
fi

echo ""
echo "🎉 All GIFs created successfully!"
echo ""
echo "To view a GIF:"
echo "  open screenshots/gifs/todo-app-complete-demo.gif"
echo ""
echo "To optimize file size further, see CREATE_GIFS_GUIDE.md"