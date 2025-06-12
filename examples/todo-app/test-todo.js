const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

async function testTodoApp() {
  console.log('Testing Todo App Example...');
  
  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  
  const page = await browser.newPage();
  
  // Set up console message listener
  const consoleLogs = [];
  page.on('console', msg => {
    consoleLogs.push({
      type: msg.type(),
      text: msg.text()
    });
  });
  
  // Set up error listener
  const pageErrors = [];
  page.on('pageerror', error => {
    pageErrors.push(error.toString());
  });
  
  try {
    console.log('Navigating to http://localhost:8082');
    await page.goto('http://localhost:8082', {
      waitUntil: 'networkidle0',
      timeout: 30000
    });
    
    // Wait for WASM to initialize
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    // Take initial screenshot
    await page.screenshot({
      path: 'screenshots/todo-initial.png',
      fullPage: true
    });
    console.log('Initial screenshot saved');
    
    // Check if app rendered
    const appElement = await page.$('#app, #root, .todo-app');
    if (!appElement) {
      console.log('Warning: Main app element not found, continuing anyway...');
    }
    
    // Test adding a todo
    console.log('\nTesting todo creation...');
    
    // Find input field
    const inputSelector = 'input[type="text"], input[placeholder*="todo"], input[placeholder*="task"]';
    const inputField = await page.$(inputSelector);
    
    if (inputField) {
      await inputField.type('Test todo item');
      
      // Click the Add button
      const addButton = await page.$('button:not(.delete)');
      if (addButton) {
        await addButton.click();
        await new Promise(resolve => setTimeout(resolve, 1000));
        console.log('Added test todo item');
      } else {
        // Try pressing Enter if no Add button found
        await page.keyboard.press('Enter');
        await new Promise(resolve => setTimeout(resolve, 1000));
        console.log('Added test todo item via Enter');
      }
      
      await page.screenshot({
        path: 'screenshots/todo-with-item.png',
        fullPage: true
      });
      
      // Check if todo was added
      const todoItems = await page.$$eval('li, .todo-item, [data-todo]', items => items.length);
      console.log('Number of todo items found:', todoItems);
      
      // Try to toggle the todo
      const todoCheckbox = await page.$('input[type="checkbox"], .todo-checkbox');
      if (todoCheckbox) {
        await todoCheckbox.click();
        await new Promise(resolve => setTimeout(resolve, 500));
        console.log('Toggled todo checkbox');
        
        await page.screenshot({
          path: 'screenshots/todo-toggled.png',
          fullPage: true
        });
      }
      
      // Add another todo
      await inputField.click({ clickCount: 3 }); // Select all
      await inputField.type('Another todo item');
      
      const addButton2 = await page.$('button:not(.delete)');
      if (addButton2) {
        await addButton2.click();
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
      
      // Try to delete a todo
      const deleteButton = await page.evaluateHandle(() => {
        const buttons = Array.from(document.querySelectorAll('button'));
        return buttons.find(btn => 
          btn.textContent.includes('×') || 
          btn.textContent.includes('Delete') ||
          btn.classList.contains('delete')
        );
      });
      
      if (deleteButton && deleteButton.asElement()) {
        await deleteButton.click();
        await new Promise(resolve => setTimeout(resolve, 500));
        console.log('Deleted a todo item');
      }
      
      await page.screenshot({
        path: 'screenshots/todo-final.png',
        fullPage: true
      });
      
    } else {
      console.log('Todo input field not found');
    }
    
    // Report results
    console.log('\n=== Test Results ===');
    console.log('Page loaded successfully: YES');
    console.log('Console errors:', pageErrors.length);
    console.log('Console warnings:', consoleLogs.filter(log => log.type === 'warning').length);
    
    if (pageErrors.length > 0) {
      console.log('\nErrors found:');
      pageErrors.forEach(err => console.log(' -', err));
    }
    
    const result = {
      success: pageErrors.length === 0,
      errors: pageErrors,
      warnings: consoleLogs.filter(log => log.type === 'warning'),
      logs: consoleLogs
    };
    
    fs.writeFileSync('test-results-todo.json', JSON.stringify(result, null, 2));
    
  } catch (error) {
    console.error('Test failed:', error);
    await page.screenshot({
      path: 'screenshots/todo-error.png',
      fullPage: true
    });
    
    const result = {
      success: false,
      error: error.toString(),
      errors: pageErrors,
      logs: consoleLogs
    };
    
    fs.writeFileSync('test-results-todo.json', JSON.stringify(result, null, 2));
  } finally {
    await browser.close();
  }
}

// Create screenshots directory
if (!fs.existsSync('screenshots')) {
  fs.mkdirSync('screenshots');
}

testTodoApp().catch(console.error);