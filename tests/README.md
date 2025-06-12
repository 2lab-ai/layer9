# Layer9 Comprehensive Test Suite

This directory contains comprehensive Puppeteer tests for all working Layer9 examples.

## Overview

The test suite verifies:
- WASM module loading
- Component rendering
- Interactive features
- State management
- UI responsiveness
- Console errors

## Prerequisites

1. Node.js and npm installed
2. HTTP server running (Python or any other)
3. Layer9 examples built (WASM files generated)

## Installation

```bash
# From this directory
npm install
```

## Running Tests

### Start the HTTP Server

First, start an HTTP server from the project root:

```bash
# From the layer9 root directory
cd ..
python3 -m http.server 8000
```

### Run All Tests

```bash
npm test
```

### Run Individual Tests

```bash
# Counter example
npm run test:counter

# Async counter example
npm run test:async-counter

# Todo app example
npm run test:todo

# Memory game example
npm run test:memory
```

## Test Structure

```
tests/
├── e2e/                      # End-to-end test files
│   ├── counter.test.js       # Counter example tests
│   ├── async-counter.test.js # Async counter tests
│   ├── todo-app.test.js      # Todo app tests
│   └── memory-game.test.js   # Memory game tests
├── screenshots/              # Generated screenshots
│   ├── counter/             # Counter screenshots
│   ├── async-counter/       # Async counter screenshots
│   ├── todo-app/            # Todo app screenshots
│   └── memory-game/         # Memory game screenshots
├── test-runner.js           # Main test runner
├── test-report.json         # Generated test report
├── package.json             # Dependencies
└── README.md               # This file
```

## What Each Test Covers

### Counter Tests (14 tests)
- WASM initialization
- Component rendering (title, buttons, stats, etc.)
- Increment/decrement functionality
- Quick action buttons (+5, -5, +10, -10)
- Reset functionality
- Statistics updates
- Counter color states (positive/negative/zero)
- Animation classes
- Console error checking

### Async Counter Tests (11 tests)
- WASM initialization with loading state
- Initial async data loading
- Component rendering
- Sync increment/decrement
- Dynamic message updates
- Async reset with loading states
- Random number generation
- Button disabled states during async operations
- Animations
- Console error checking

### Todo App Tests (14 tests)
- WASM initialization
- Component rendering
- Empty state display
- Adding single and multiple todos
- Todo completion toggle
- Todo deletion
- Filter functionality (All/Active/Completed)
- Clear completed feature
- Statistics display
- Empty input validation
- Keyboard interaction (Enter key)
- Console error checking

### Memory Game Tests (14 tests)
- WASM initialization
- Component rendering
- 4x4 grid verification (16 cards)
- Card flipping mechanics
- Mismatch behavior
- Move counter
- Finding and matching pairs
- Matches counter
- New Game functionality
- Click prevention during animations
- Card flip animations
- Win screen capability
- Console error checking

## Screenshots

Each test captures screenshots at key moments:
- Initial load
- After user interactions
- Error states (if any)
- Final state

Screenshots are saved in `screenshots/<example-name>/` directories.

## Test Report

After running all tests, a `test-report.json` file is generated containing:
- Test results (passed/failed)
- Execution time for each test
- Total duration
- Screenshot paths
- Timestamp

## Troubleshooting

### Server Not Running
If you see "HTTP server is not running", start the server:
```bash
cd .. && python3 -m http.server 8000
```

### WASM Files Not Found
Ensure examples are built:
```bash
cd .. && ./build.sh
```

### Puppeteer Issues
If Puppeteer fails to launch:
```bash
# Install system dependencies
sudo apt-get install -y chromium-browser  # Linux
brew install chromium  # macOS
```

### Timeout Errors
Increase the TIMEOUT constant in test files if needed (default: 30 seconds).

## Contributing

When adding new examples:
1. Create a new test file in `e2e/`
2. Follow the existing test structure
3. Add the test to `test-runner.js`
4. Update this README

## CI/CD Integration

These tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Install dependencies
  run: cd tests && npm install
  
- name: Start server
  run: python3 -m http.server 8000 &
  
- name: Run tests
  run: cd tests && npm test
```