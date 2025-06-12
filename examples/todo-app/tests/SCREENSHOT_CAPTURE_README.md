# Todo App Screenshot Capture & GIF Creation

This directory contains scripts to capture screenshots of the Todo App in various states and convert them into animated GIFs for documentation and demos.

## Quick Start

1. **Ensure the todo app is running on port 8082:**
   ```bash
   # In the parent directory, update serve.py to use port 8082
   # Then run:
   python3 serve.py
   ```

2. **Capture all screenshots:**
   ```bash
   ./capture-all-screenshots.sh
   ```

3. **Create GIFs from screenshots:**
   ```bash
   # Install ImageMagick first if needed:
   # macOS: brew install imagemagick
   # Linux: sudo apt-get install imagemagick
   
   ./create-gifs.sh
   ```

## Scripts Overview

### 1. `capture-demo-screenshots.js`
Captures a sequence of screenshots showing all app features:
- Empty state
- Adding todos
- Completing todos
- Using filters
- Deleting todos
- Clearing completed

**Output:** `screenshots/demo/`

### 2. `generate-realistic-demo.js`
Creates realistic usage scenarios:
- Daily work tasks
- Mixed personal & work items
- Learning goals tracking
- End-of-day review
- Project planning
- Weekend planning

**Output:** `screenshots/realistic-demo/`

### 3. `capture-all-screenshots.sh`
Runs both capture scripts automatically.

### 4. `create-gifs.sh`
Converts screenshots into various GIFs:
- Complete demo
- Quick overview
- Feature-specific animations
- Optimized header for README

**Output:** `screenshots/gifs/`

## Manual Usage

### Run Individual Scripts

```bash
# Capture demo screenshots only
node capture-demo-screenshots.js

# Capture realistic scenarios only
node generate-realistic-demo.js
```

### Create Custom GIFs

See `CREATE_GIFS_GUIDE.md` for detailed instructions on creating custom GIFs with:
- Custom timing
- Specific frame selection
- Size optimization
- Quality adjustments

## Directory Structure

```
tests/
├── capture-demo-screenshots.js     # Sequential demo capture
├── generate-realistic-demo.js      # Realistic scenario capture
├── capture-all-screenshots.sh      # Run all captures
├── create-gifs.sh                  # Create GIFs from screenshots
├── CREATE_GIFS_GUIDE.md           # Detailed GIF creation guide
└── screenshots/
    ├── demo/                      # Sequential demo screenshots
    ├── realistic-demo/            # Realistic usage screenshots
    └── gifs/                      # Generated GIF files
```

## Tips

1. **Server Port:** Make sure the app is running on port 8082 (update in scripts if using different port)

2. **Timing:** Scripts use realistic delays between actions for smooth GIFs

3. **Window Size:** Screenshots are captured at 800x600 for demos, 1200x800 for realistic scenarios

4. **File Naming:** Screenshots are numbered sequentially with descriptive names

5. **GIF Optimization:** Use the CREATE_GIFS_GUIDE.md for advanced optimization techniques

## Troubleshooting

- **Server not running:** Start the server with `python3 serve.py` on port 8082
- **Screenshots not capturing:** Check browser permissions and that Puppeteer is installed
- **GIFs too large:** Use color reduction and resize options in create-gifs.sh
- **Missing dependencies:** Run `npm install` to install Puppeteer/Playwright

## Example GIFs Created

1. **todo-app-complete-demo.gif** - Full feature demonstration
2. **todo-app-overview.gif** - Quick overview with key frames
3. **todo-app-header.gif** - Small, optimized for README headers
4. **todo-add-feature.gif** - Adding todos demonstration
5. **todo-filter-feature.gif** - Filter functionality
6. **todo-delete-clear-feature.gif** - Delete and clear features

## Using GIFs in Documentation

```markdown
# In your README.md:

![Todo App Demo](tests/screenshots/gifs/todo-app-header.gif)

## Features

### Adding Todos
![Adding Todos](tests/screenshots/gifs/todo-add-feature.gif)

### Filtering
![Filter Feature](tests/screenshots/gifs/todo-filter-feature.gif)
```