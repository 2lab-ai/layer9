# Layer9 Todo App - Comprehensive Testing Guide

This guide provides step-by-step instructions for manually testing all features of the Layer9 Todo App.

## Setup

1. Start the HTTP server:
   ```bash
   cd tests
   python3 serve.py
   ```

2. Open your browser and navigate to: `http://localhost:8080`

## Test Cases

### 1. Initial Load
- [ ] Page loads without errors
- [ ] Loading spinner appears briefly
- [ ] Loading spinner disappears once WASM loads
- [ ] Main app container appears

### 2. Component Verification
Verify these components are visible:
- [ ] Header with "Layer9 Todo" title
- [ ] Subtitle: "A beautiful todo app built with Rust and WASM"
- [ ] Input field with placeholder "What needs to be done?"
- [ ] "Add" button next to input
- [ ] Empty state message: "No todos yet!"
- [ ] Footer with "Built with Layer9" link

### 3. Adding Todos
- [ ] Type "Complete testing" in input field
- [ ] Click "Add" button
- [ ] Todo appears in the list
- [ ] Input field is cleared
- [ ] Empty state message disappears
- [ ] Todo shows creation timestamp

### 4. Adding Multiple Todos
Add these todos:
- [ ] "Write documentation"
- [ ] "Fix bugs"
- [ ] "Deploy to production"

Verify:
- [ ] All todos appear in the list
- [ ] Todos are ordered by creation time
- [ ] Each todo has a unique timestamp

### 5. Toggle Completion
- [ ] Click checkbox next to first todo
- [ ] Todo text gets strikethrough styling
- [ ] Todo item gets "completed" class
- [ ] Click checkbox again
- [ ] Strikethrough is removed
- [ ] Todo returns to normal state

### 6. Delete Todo
- [ ] Hover over a todo item
- [ ] Delete button (×) appears on the right
- [ ] Click delete button
- [ ] Todo is removed from list
- [ ] Remaining todos stay in place

### 7. Filter Functionality
- [ ] Mark 2 todos as completed
- [ ] Footer shows correct counts (e.g., "2 active • 2 completed")

Test filters:
- [ ] Click "active" filter - only uncompleted todos shown
- [ ] Click "completed" filter - only completed todos shown
- [ ] Click "all" filter - all todos shown
- [ ] Active filter button is highlighted

### 8. Clear Completed
- [ ] Ensure some todos are marked as completed
- [ ] "Clear completed" button appears in footer
- [ ] Click "Clear completed"
- [ ] All completed todos are removed
- [ ] Active todos remain
- [ ] Button disappears if no completed todos

### 9. Empty Input Validation
- [ ] Try to add empty todo (just spaces)
- [ ] Todo is not added
- [ ] Input remains focused

### 10. Responsive Design
- [ ] Resize browser window
- [ ] App remains centered
- [ ] Components scale appropriately
- [ ] No horizontal scrolling

### 11. Keyboard Navigation
- [ ] Press Enter in input field to add todo
- [ ] Tab navigation works through interactive elements

### 12. Performance
- [ ] Adding/removing todos is instant
- [ ] No lag when toggling completion
- [ ] Filters apply immediately

## Browser Compatibility
Test in multiple browsers:
- [ ] Chrome/Chromium
- [ ] Firefox
- [ ] Safari
- [ ] Edge

## Developer Tools Checks
Open browser developer tools and verify:
- [ ] No console errors
- [ ] WASM module loads successfully
- [ ] Network tab shows all resources loaded
- [ ] No 404 errors

## Screenshots to Capture
1. Initial empty state
2. Multiple todos added
3. Mix of completed and active todos
4. Active filter applied
5. Completed filter applied
6. After clearing completed todos

## Expected Console Output
You should see these messages in the browser console:
```
Layer9 Todo App starting...
Layer9 Todo App mounted successfully!
```

## Common Issues and Solutions

### WASM Not Loading
- Check CORS headers are set correctly
- Verify WASM file exists in pkg/ directory
- Check browser supports WASM

### Todos Not Persisting
- This app doesn't include persistence
- Todos are stored in memory only
- Refreshing page will clear all todos

### Styling Issues
- Clear browser cache
- Check all CSS is embedded correctly
- Verify no conflicting styles

## Automated Test Status
While automated Puppeteer/Playwright tests are available, they may have compatibility issues on some systems. The manual tests above cover all functionality comprehensively.