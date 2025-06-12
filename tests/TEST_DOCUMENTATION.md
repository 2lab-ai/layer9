# Layer9 Test Suite Documentation

## Overview

This document provides detailed information about what each test verifies in the Layer9 example applications.

## Counter Example Tests

### Test Coverage

1. **WASM Loading**
   - Verifies the loading spinner disappears
   - Confirms the counter container renders
   - Ensures no JavaScript errors during initialization

2. **Component Structure**
   - Title with "Layer9" gradient text
   - Subtitle "Beautiful reactive state management"
   - Counter display with value and label
   - Quick action buttons (-10, -5, +5, +10)
   - Main control buttons (Decrement, Reset, Increment)
   - Statistics section (Status, Distance from zero, Square)
   - Footer with Layer9 link

3. **Initial State**
   - Counter starts at 0
   - Status shows "Zero"
   - Correct initial statistics

4. **Increment/Decrement**
   - Single increment changes value to 1
   - Single decrement changes value to -1
   - Animations trigger on value changes

5. **Quick Actions**
   - +10 button adds 10 to current value
   - -5 button subtracts 5 from current value
   - All quick action buttons work correctly

6. **Reset Functionality**
   - Reset button sets counter to 0
   - Reset animation plays
   - Statistics update accordingly

7. **Statistics Updates**
   - Status changes: "Zero", "Positive", "Negative"
   - Distance from zero calculates absolute value
   - Square shows value²

8. **Visual States**
   - Positive numbers get green color
   - Negative numbers get red color
   - Zero gets primary color
   - Correct CSS classes applied

## Async Counter Example Tests

### Test Coverage

1. **Async Initialization**
   - Loading spinner shows during initial load
   - Counter loads with value 42 from "server"
   - Initial message confirms server load

2. **Component Structure**
   - Title with "Async" gradient text
   - Subtitle about asynchronous state management
   - Counter display with loading states
   - Message display area
   - Sync control buttons (Increment, Decrement)
   - Async control buttons (Async Reset, Random)

3. **Dynamic Messages**
   - "The answer to everything!" for 42
   - "Zero - The beginning of everything!" for 0
   - "Venturing into negative territory!" for negatives
   - "Perfect ten!" for multiples of 10

4. **Sync Operations**
   - Increment/decrement work immediately
   - No loading state for sync operations

5. **Async Reset**
   - Shows loading state
   - Displays "Resetting..." message
   - Buttons disabled during operation
   - Completes with value 0 and success message

6. **Random Number**
   - Shows loading state
   - Displays "Fetching random number..." message
   - Loads random value 0-99
   - Shows completion message with value

7. **Button States**
   - Async buttons disable during operations
   - Disabled buttons show visual feedback
   - Re-enable after operation completes

## Todo App Example Tests

### Test Coverage

1. **Initial State**
   - Empty state message "No todos yet"
   - Input field and add button visible
   - Filter buttons show (All, Active, Completed)
   - Statistics show "0 active, 0 completed"

2. **Adding Todos**
   - Text input captures user input
   - Add button creates new todo
   - Input field clears after adding
   - Todo appears in list with checkbox and text

3. **Multiple Todos**
   - Can add multiple todos sequentially
   - Each todo gets unique position
   - List updates dynamically

4. **Todo Completion**
   - Checkbox toggles completion state
   - Completed todos get strikethrough style
   - Can uncomplete by clicking again

5. **Todo Deletion**
   - Delete button appears on hover
   - Clicking delete removes todo
   - List reorganizes after deletion

6. **Filtering**
   - "All" shows all todos
   - "Active" shows only uncompleted todos
   - "Completed" shows only completed todos
   - Filter buttons highlight when active

7. **Clear Completed**
   - Removes all completed todos at once
   - Only active todos remain
   - Statistics update

8. **Statistics**
   - Shows "X active, Y completed"
   - Updates in real-time
   - Accurate counts

9. **Input Validation**
   - Empty or whitespace-only input rejected
   - No empty todos added to list

10. **Keyboard Support**
    - Enter key adds todo
    - Same as clicking add button

## Memory Game Example Tests

### Test Coverage

1. **Game Board**
   - 4x4 grid (16 cards total)
   - Cards start face down showing "?"
   - Even distribution of 8 pairs

2. **Card Mechanics**
   - Click flips card to show emoji
   - Second click flips another card
   - Cards stay flipped while checking match

3. **Matching Logic**
   - Matching pairs stay face up
   - Matched cards get special styling
   - Non-matching pairs flip back after delay

4. **Game Statistics**
   - Move counter increments after each pair attempt
   - Matches counter shows "X/8"
   - Both update in real-time

5. **Click Prevention**
   - Can't click more than 2 cards at once
   - Can't click already matched cards
   - Can't click during flip animations

6. **New Game**
   - Resets all cards face down
   - Shuffles card positions
   - Resets moves to 0
   - Clears all matches

7. **Animations**
   - Card flip animation (3D transform)
   - Smooth transitions
   - Visual feedback for interactions

8. **Win Condition**
   - Detects when all pairs matched
   - Shows congratulations message
   - Displays total moves taken
   - Offers play again option

## Common Test Patterns

### All Examples Test For:

1. **WASM Loading**
   - No loading errors
   - Proper initialization
   - Component mounting

2. **Console Errors**
   - No JavaScript errors
   - No warning messages
   - Clean console output

3. **Responsive UI**
   - Buttons respond to clicks
   - State updates reflect in UI
   - Animations play smoothly

4. **Screenshot Capture**
   - Initial state
   - After interactions
   - Various UI states
   - Final state

## Running Individual Test Suites

Each test file can be run independently for debugging:

```bash
# Test specific example
node e2e/counter.test.js
node e2e/async-counter.test.js
node e2e/todo-app.test.js
node e2e/memory-game.test.js
```

## Debugging Failed Tests

1. **Check Screenshots**
   - Look in `screenshots/<example>/` directory
   - Compare with expected UI state
   - Identify visual discrepancies

2. **Console Output**
   - Each test logs what it's checking
   - Failed assertions show expected vs actual
   - Console errors are captured

3. **Timeout Issues**
   - Increase TIMEOUT constant if needed
   - Check for slow WASM loading
   - Verify server response times

4. **Element Not Found**
   - Check selector accuracy
   - Verify component rendered
   - Look for timing issues