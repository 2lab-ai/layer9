# Creating GIFs from Todo App Screenshots

This guide explains how to convert the captured screenshots into animated GIFs for documentation and demos.

## Prerequisites

You'll need one of these tools installed:
- ImageMagick (recommended)
- FFmpeg
- GIMP (for manual creation)

## Method 1: Using ImageMagick (Recommended)

### Installation

**macOS:**
```bash
brew install imagemagick
```

**Linux:**
```bash
sudo apt-get install imagemagick  # Ubuntu/Debian
sudo yum install ImageMagick       # RHEL/CentOS
```

### Creating GIFs

1. **Basic GIF from all screenshots:**
```bash
cd screenshots/demo
convert -delay 100 -loop 0 *.png todo-app-demo.gif
```

2. **Optimized GIF with better quality:**
```bash
convert -delay 100 -loop 0 -resize 800x600 *.png -coalesce -layers OptimizePlus todo-app-demo-optimized.gif
```

3. **Custom delays for specific frames:**
```bash
convert \
  -delay 100 001-*.png \
  -delay 150 002-*.png \
  -delay 100 003-*.png \
  -delay 200 004-*.png \
  -delay 100 005-*.png \
  -delay 100 006-*.png \
  -delay 100 007-*.png \
  -delay 100 008-*.png \
  -delay 150 009-*.png \
  -delay 150 010-*.png \
  -delay 150 011-*.png \
  -delay 150 012-*.png \
  -delay 100 013-*.png \
  -delay 150 014-*.png \
  -delay 200 015-*.png \
  -delay 100 016-*.png \
  -delay 150 017-*.png \
  -loop 0 \
  todo-app-demo-custom.gif
```

4. **Smaller file size (reduced colors):**
```bash
convert -delay 100 -loop 0 *.png -colors 128 -fuzz 10% todo-app-demo-small.gif
```

## Method 2: Using FFmpeg

### Installation

**macOS:**
```bash
brew install ffmpeg
```

**Linux:**
```bash
sudo apt-get install ffmpeg  # Ubuntu/Debian
```

### Creating GIFs

1. **Basic GIF:**
```bash
ffmpeg -framerate 1 -pattern_type glob -i '*.png' -vf "fps=1,scale=800:-1:flags=lanczos" todo-app-demo.gif
```

2. **High quality GIF with palette:**
```bash
# Generate palette
ffmpeg -i %03d-*.png -vf "fps=1,scale=800:-1:flags=lanczos,palettegen" palette.png

# Create GIF using palette
ffmpeg -framerate 1 -pattern_type glob -i '*.png' -i palette.png -lavfi "fps=1,scale=800:-1:flags=lanczos[x];[x][1:v]paletteuse" todo-app-demo-hq.gif
```

## Method 3: Creating Specific Feature GIFs

### Adding Todos GIF
```bash
convert -delay 100 -loop 0 \
  001-empty-state.png \
  002-input-focused.png \
  003-typing-first-todo.png \
  004-first-todo-added.png \
  005-typing-second-todo.png \
  006-second-todo-added.png \
  todo-add-feature.gif
```

### Filter Feature GIF
```bash
convert -delay 150 -loop 0 \
  008-multiple-todos.png \
  009-first-todo-completed.png \
  010-multiple-completed.png \
  011-filter-active.png \
  012-filter-completed.png \
  013-filter-all.png \
  todo-filter-feature.gif
```

### Delete and Clear GIF
```bash
convert -delay 150 -loop 0 \
  013-filter-all.png \
  014-hover-delete-button.png \
  015-todo-deleted.png \
  016-cleared-completed.png \
  todo-delete-feature.gif
```

## Method 4: Online Tools

If you prefer not to install software, you can use online tools:

1. **ezgif.com** - Upload multiple images and create GIF
2. **gifmaker.me** - Simple online GIF creator
3. **imgflip.com/gif-maker** - Advanced options available

## Tips for Better GIFs

1. **Optimal Delays:**
   - Use 100 (1 second) for normal transitions
   - Use 150-200 (1.5-2 seconds) for important states
   - Use 50 (0.5 seconds) for quick actions

2. **File Size Optimization:**
   - Reduce colors: `-colors 64` or `-colors 128`
   - Add fuzz factor: `-fuzz 5%`
   - Resize if needed: `-resize 600x450`

3. **Loop Options:**
   - `-loop 0` = infinite loop
   - `-loop 1` = play once
   - `-loop 3` = play 3 times

4. **Quality vs Size:**
   - For documentation: prioritize quality
   - For README: prioritize smaller size
   - For demos: balance both

## Example Commands for Common Scenarios

### Full Demo GIF (All Features)
```bash
convert -delay 100 -loop 0 -resize 800x600 *.png -colors 256 todo-app-complete-demo.gif
```

### Quick Overview GIF (Key Frames Only)
```bash
convert -delay 200 -loop 0 \
  001-empty-state.png \
  004-first-todo-added.png \
  008-multiple-todos.png \
  010-multiple-completed.png \
  011-filter-active.png \
  015-todo-deleted.png \
  017-realistic-final-state.png \
  todo-app-overview.gif
```

### README Header GIF (Small, Fast)
```bash
convert -delay 80 -loop 0 -resize 600x400 \
  001-empty-state.png \
  008-multiple-todos.png \
  010-multiple-completed.png \
  017-realistic-final-state.png \
  -colors 64 \
  todo-app-header.gif
```

## Viewing Your GIFs

After creating GIFs, you can:
1. Open them in a web browser
2. Use Preview (macOS) or Image Viewer (Linux)
3. Drag into a markdown file to preview
4. Upload to GitHub and reference in README

## Troubleshooting

- **GIF too large:** Reduce colors, resize, or use fewer frames
- **GIF too fast/slow:** Adjust delay values
- **Poor quality:** Use palette generation (FFmpeg) or increase colors
- **Not looping:** Ensure `-loop 0` is included