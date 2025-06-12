# Layer9 GIF Capture Scripts

Automated GIF creation pipeline for Layer9 examples. This system captures screenshots of all examples in action and converts them to optimized GIFs for documentation.

## Quick Start

```bash
# Run the complete pipeline
./capture-and-create-gifs.sh

# Check server status
node capture-all-examples.js --check
```

## Features

- **Automated Screenshot Capture**: Uses Puppeteer to interact with examples and capture key moments
- **Smart GIF Creation**: Converts screenshots to optimized GIFs with configurable timing
- **Multiple Scenarios**: Captures different use cases for each example
- **Size Optimization**: Automatically optimizes GIFs for web use
- **Batch Processing**: Handles all examples in one run

## Examples Covered

### 1. Counter
- **Increment**: Shows counting up animation
- **Decrement**: Shows counting down animation
- **Reset**: Demonstrates reset functionality
- **Quick Actions**: Rapid button clicks

### 2. Async Counter
- **Loading States**: Captures loading indicators
- **Async Operations**: Shows async increment/decrement
- **Error Handling**: Demonstrates error states

### 3. Todo App
- **Add Todos**: Shows adding new items
- **Complete Todos**: Checking off items
- **Filters**: All/Active/Completed views
- **Delete & Clear**: Removing items

### 4. Memory Game
- **Start Game**: Initial game state
- **Flip Cards**: Card flipping animations
- **Match Cards**: Successful matches
- **Win State**: Game completion

## Scripts

### capture-and-create-gifs.sh
Main orchestration script that runs the entire pipeline:
```bash
./capture-and-create-gifs.sh [options]

Options:
  --skip-build    Skip building examples
  --skip-capture  Skip screenshot capture
  --skip-gifs     Skip GIF creation
```

### capture-all-examples.js
Captures screenshots for all examples:
```bash
node capture-all-examples.js       # Capture screenshots
node capture-all-examples.js --check  # Check server status
```

### create-gifs.js
Converts screenshots to optimized GIFs:
```bash
node create-gifs.js
```

### start-servers.sh / stop-servers.sh
Manage example servers:
```bash
./start-servers.sh  # Start all servers
./stop-servers.sh   # Stop all servers
```

## Output Structure

```
examples/
  counter/
    screenshots/
      increment/
        001-initial-state.png
        002-increment-1.png
        ...
      decrement/
      reset/
      quick-actions/
  async-counter/
    screenshots/
      ...
  todo-app/
    screenshots/
      ...
  memory-game/
    screenshots/
      ...

assets/
  gifs/
    counter-increment.gif
    counter-overview.gif
    async-counter-loading.gif
    todo-app-overview.gif
    memory-game-overview.gif
    layer9-examples-showcase.gif
```

## GIF Optimization

GIFs are automatically optimized for size:
- Color reduction (128 colors)
- Frame optimization
- Configurable delays
- Resize options

To further optimize:
```bash
# Install gifsicle
npm install -g gifsicle

# Manual optimization
gifsicle -O3 --colors 64 input.gif > output.gif
```

## Customization

Edit `GIF_CONFIGS` in `create-gifs.js` to customize:
- Frame delays
- Output sizes
- Which frames to include
- Output filenames

Example:
```javascript
counter: {
    scenarios: {
        increment: {
            delay: 60,        // 60/100 seconds between frames
            resize: '800x600', // Output size
            output: 'counter-increment.gif'
        }
    }
}
```

## Requirements

- Node.js 16+
- ImageMagick (`brew install imagemagick`)
- Puppeteer (auto-installed)
- Optional: gifsicle for better optimization

## Troubleshooting

### "Server not running" errors
Make sure to build examples first:
```bash
cd ../../examples/counter
wasm-pack build --target web
```

### ImageMagick not found
Install ImageMagick:
```bash
# macOS
brew install imagemagick

# Ubuntu/Debian
sudo apt-get install imagemagick
```

### GIFs too large
- Reduce frame count
- Lower resolution
- Increase color reduction
- Use gifsicle optimization

## Adding New Examples

1. Add example configuration to `EXAMPLES` in `capture-all-examples.js`
2. Create capture function (e.g., `captureNewExample`)
3. Add GIF configuration to `GIF_CONFIGS` in `create-gifs.js`
4. Update server start script if needed

## Best Practices

1. **Timing**: Use appropriate delays between actions for smooth GIFs
2. **Frame Selection**: Capture key moments, not every frame
3. **Size**: Keep GIFs under 5MB for README use
4. **Consistency**: Use same dimensions across related GIFs
5. **Accessibility**: Ensure GIFs clearly show the interaction flow