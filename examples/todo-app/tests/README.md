# Layer9 Todo App - Puppeteer Tests

This directory contains comprehensive Puppeteer tests for the Layer9 Todo App.

## Prerequisites

- Node.js and npm
- Python 3
- Built WASM files in `../pkg/` directory

## Files

- `puppeteer-tests.js` - Main test suite with 11 comprehensive tests
- `serve.py` - Python HTTP server for serving the todo app
- `run-tests.sh` - Automated test runner script
- `package.json` - Node.js dependencies

## Running Tests

### Option 1: Automated (Recommended)
```bash
./run-tests.sh
```

This will:
1. Install Puppeteer dependencies
2. Start the HTTP server
3. Run all tests
4. Generate screenshots
5. Stop the server

### Option 2: Manual

1. Install dependencies:
   ```bash
   npm install
   ```

2. In one terminal, start the server:
   ```bash
   python3 serve.py
   ```

3. In another terminal, run tests:
   ```bash
   npm test
   ```

## Test Coverage

The test suite covers:

1. **Application Loading** - Verifies the app loads successfully
2. **WASM Initialization** - Ensures WASM modules load properly
3. **Component Verification** - Checks all main UI components render
4. **Adding Todos** - Tests single and multiple todo creation
5. **Toggle Completion** - Tests marking todos as complete/incomplete
6. **Delete Functionality** - Tests todo deletion
7. **Filter Buttons** - Tests All/Active/Completed filters
8. **Clear Completed** - Tests bulk deletion of completed todos
9. **Stats Display** - Verifies active/completed counts
10. **Input Validation** - Ensures empty todos aren't added
11. **Screenshots** - Captures app state at key points

## Screenshots

Screenshots are saved in the `screenshots/` directory:

- `01-initial-load.png` - App after loading
- `02-first-todo-added.png` - After adding first todo
- `03-multiple-todos.png` - Multiple todos added
- `04-todo-completed.png` - Todo marked as completed
- `05-todo-deleted.png` - After deleting a todo
- `06-filter-active.png` - Active filter applied
- `07-filter-completed.png` - Completed filter applied
- `08-cleared-completed.png` - After clearing completed todos
- `09-final-state.png` - Final app state

## Troubleshooting

If tests fail:

1. Ensure the WASM files are built (`wasm-pack build` in parent directory)
2. Check that port 8000 is available
3. Verify Node.js and Python are installed
4. Check console output for specific error messages