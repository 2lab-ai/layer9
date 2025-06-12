const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// Configuration
const TODO_URL = 'http://localhost:8000/examples/todo-app/';
const SCREENSHOT_DIR = path.join(__dirname, '../screenshots/todo-app');
const TIMEOUT = 30000;

// Helper functions
async function waitForElement(page, selector, options = {}) {
    try {
        await page.waitForSelector(selector, { timeout: TIMEOUT, ...options });
        return true;
    } catch (error) {
        console.error(`Failed to find element: ${selector}`);
        return false;
    }
}

async function takeScreenshot(page, name) {
    if (!fs.existsSync(SCREENSHOT_DIR)) {
        fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    }
    const screenshotPath = path.join(SCREENSHOT_DIR, `${name}.png`);
    await page.screenshot({ path: screenshotPath, fullPage: true });
    console.log(`📸 Screenshot saved: ${name}.png`);
}

async function checkConsoleErrors(page) {
    const errors = [];
    page.on('console', msg => {
        if (msg.type() === 'error') {
            errors.push(msg.text());
        }
    });
    return errors;
}

// Main test function
async function runTodoTests() {
    console.log('\n🚀 Starting Layer9 Todo App Tests\n');
    
    const browser = await puppeteer.launch({
        headless: 'new',
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });

    let testsPassed = 0;
    let testsFailed = 0;
    const errors = [];

    try {
        const page = await browser.newPage();
        await page.setViewport({ width: 1280, height: 800 });
        
        // Monitor console errors
        const consoleErrors = await checkConsoleErrors(page);

        // Test 1: Navigate to todo app
        console.log('📋 Test 1: Loading the todo application...');
        try {
            await page.goto(TODO_URL, { waitUntil: 'networkidle2', timeout: TIMEOUT });
            console.log('✅ Successfully loaded the todo app');
            testsPassed++;
        } catch (error) {
            console.error('❌ Failed to load the todo app:', error.message);
            testsFailed++;
            throw error;
        }

        // Test 2: Verify WASM loads correctly
        console.log('\n📋 Test 2: Verifying WASM initialization...');
        try {
            // Wait for loading to complete
            await page.waitForFunction(
                () => {
                    const loading = document.getElementById('loading');
                    return loading && loading.style.display === 'none';
                },
                { timeout: TIMEOUT }
            );
            
            await waitForElement(page, '.todo-app');
            console.log('✅ WASM loaded successfully');
            testsPassed++;
            await takeScreenshot(page, '01-initial-load');
        } catch (error) {
            console.error('❌ WASM loading failed:', error.message);
            testsFailed++;
        }

        // Test 3: Verify all main components render
        console.log('\n📋 Test 3: Verifying all todo app components...');
        const components = [
            { selector: '.header h1', name: 'Title' },
            { selector: '.subtitle', name: 'Subtitle' },
            { selector: '#todo-input', name: 'Input field' },
            { selector: '.add-btn', name: 'Add button' },
            { selector: '.empty-state', name: 'Empty state message' },
            { selector: '.filters', name: 'Filter buttons' },
            { selector: '.filter-btn', name: 'Filter button (All)' },
            { selector: '.stats', name: 'Statistics section' },
            { selector: '.clear-btn', name: 'Clear completed button' }
        ];

        let allComponentsFound = true;
        for (const component of components) {
            const found = await waitForElement(page, component.selector);
            if (found) {
                console.log(`  ✅ ${component.name} found`);
            } else {
                console.log(`  ❌ ${component.name} not found`);
                allComponentsFound = false;
            }
        }
        
        if (allComponentsFound) {
            testsPassed++;
        } else {
            testsFailed++;
        }

        // Test 4: Verify initial empty state
        console.log('\n📋 Test 4: Verifying initial empty state...');
        try {
            const emptyStateText = await page.$eval('.empty-state p', el => el.textContent);
            if (emptyStateText.includes('No todos yet')) {
                console.log('✅ Empty state message displayed correctly');
                testsPassed++;
            } else {
                console.log('❌ Empty state message not as expected');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to verify empty state:', error.message);
            testsFailed++;
        }

        // Test 5: Add a single todo
        console.log('\n📋 Test 5: Adding a single todo...');
        try {
            await page.type('#todo-input', 'Write Layer9 tests');
            await page.click('.add-btn');
            await page.waitForTimeout(300);
            
            // Verify todo appears
            await waitForElement(page, '.todo-item');
            const todoText = await page.$eval('.todo-text', el => el.textContent);
            
            if (todoText === 'Write Layer9 tests') {
                console.log('✅ Todo added successfully');
                testsPassed++;
                await takeScreenshot(page, '02-first-todo');
            } else {
                console.log(`❌ Todo text mismatch: expected "Write Layer9 tests", got "${todoText}"`);
                testsFailed++;
            }
            
            // Verify input is cleared
            const inputValue = await page.$eval('#todo-input', el => el.value);
            if (inputValue === '') {
                console.log('✅ Input field cleared after adding todo');
            }
        } catch (error) {
            console.error('❌ Failed to add todo:', error.message);
            testsFailed++;
        }

        // Test 6: Add multiple todos
        console.log('\n📋 Test 6: Adding multiple todos...');
        try {
            const todos = [
                'Deploy to production',
                'Write documentation',
                'Fix performance issues',
                'Add more features'
            ];
            
            for (const todo of todos) {
                await page.type('#todo-input', todo);
                await page.click('.add-btn');
                await page.waitForTimeout(200);
            }
            
            const todoCount = await page.$$eval('.todo-item', items => items.length);
            if (todoCount === 5) { // 1 from previous test + 4 new
                console.log(`✅ Successfully added multiple todos (total: ${todoCount})`);
                testsPassed++;
                await takeScreenshot(page, '03-multiple-todos');
            } else {
                console.log(`❌ Expected 5 todos, found ${todoCount}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to add multiple todos:', error.message);
            testsFailed++;
        }

        // Test 7: Toggle todo completion
        console.log('\n📋 Test 7: Testing todo completion toggle...');
        try {
            // Click first checkbox
            await page.click('.todo-item:nth-child(1) .todo-checkbox');
            await page.waitForTimeout(300);
            
            // Check if completed
            const isCompleted = await page.$eval('.todo-item:nth-child(1)', el => 
                el.classList.contains('completed')
            );
            
            if (isCompleted) {
                console.log('✅ Todo marked as completed');
                
                // Click again to uncomplete
                await page.click('.todo-item:nth-child(1) .todo-checkbox');
                await page.waitForTimeout(300);
                
                const isUncompleted = await page.$eval('.todo-item:nth-child(1)', el => 
                    !el.classList.contains('completed')
                );
                
                if (isUncompleted) {
                    console.log('✅ Todo unmarked successfully');
                    testsPassed++;
                } else {
                    console.log('❌ Failed to unmark todo');
                    testsFailed++;
                }
            } else {
                console.log('❌ Todo not marked as completed');
                testsFailed++;
            }
            
            await takeScreenshot(page, '04-todo-completion');
        } catch (error) {
            console.error('❌ Failed to toggle todo:', error.message);
            testsFailed++;
        }

        // Test 8: Delete a todo
        console.log('\n📋 Test 8: Testing todo deletion...');
        try {
            const initialCount = await page.$$eval('.todo-item', items => items.length);
            
            // Hover to show delete button
            await page.hover('.todo-item:nth-child(1)');
            await page.waitForTimeout(200);
            
            // Click delete
            await page.click('.todo-item:nth-child(1) .delete-btn');
            await page.waitForTimeout(300);
            
            const newCount = await page.$$eval('.todo-item', items => items.length);
            
            if (newCount === initialCount - 1) {
                console.log(`✅ Todo deleted successfully (${initialCount} → ${newCount})`);
                testsPassed++;
                await takeScreenshot(page, '05-after-delete');
            } else {
                console.log(`❌ Delete failed (${initialCount} → ${newCount})`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to delete todo:', error.message);
            testsFailed++;
        }

        // Test 9: Test filter functionality
        console.log('\n📋 Test 9: Testing filter buttons...');
        try {
            // Mark some todos as completed
            await page.click('.todo-item:nth-child(1) .todo-checkbox');
            await page.click('.todo-item:nth-child(2) .todo-checkbox');
            await page.waitForTimeout(300);
            
            // Test Active filter
            await page.click('.filter-btn:nth-child(2)'); // Active
            await page.waitForTimeout(300);
            const activeCount = await page.$$eval('.todo-item:not([style*="display: none"])', items => items.length);
            console.log(`  Active todos shown: ${activeCount}`);
            await takeScreenshot(page, '06-filter-active');
            
            // Test Completed filter
            await page.click('.filter-btn:nth-child(3)'); // Completed
            await page.waitForTimeout(300);
            const completedCount = await page.$$eval('.todo-item:not([style*="display: none"])', items => items.length);
            console.log(`  Completed todos shown: ${completedCount}`);
            await takeScreenshot(page, '07-filter-completed');
            
            // Test All filter
            await page.click('.filter-btn:nth-child(1)'); // All
            await page.waitForTimeout(300);
            const allCount = await page.$$eval('.todo-item:not([style*="display: none"])', items => items.length);
            console.log(`  All todos shown: ${allCount}`);
            
            if (activeCount < allCount && completedCount < allCount) {
                console.log('✅ Filters working correctly');
                testsPassed++;
            } else {
                console.log('❌ Filter counts don\'t match expected values');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test filters:', error.message);
            testsFailed++;
        }

        // Test 10: Test clear completed functionality
        console.log('\n📋 Test 10: Testing "Clear completed" button...');
        try {
            // Ensure we're on All filter
            await page.click('.filter-btn:nth-child(1)');
            await page.waitForTimeout(200);
            
            const beforeCount = await page.$$eval('.todo-item', items => items.length);
            const completedBefore = await page.$$eval('.todo-item.completed', items => items.length);
            
            await page.click('.clear-btn');
            await page.waitForTimeout(300);
            
            const afterCount = await page.$$eval('.todo-item', items => items.length);
            
            if (afterCount === beforeCount - completedBefore) {
                console.log(`✅ Completed todos cleared (${beforeCount} → ${afterCount})`);
                testsPassed++;
                await takeScreenshot(page, '08-after-clear-completed');
            } else {
                console.log('❌ Clear completed didn\'t work as expected');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to clear completed:', error.message);
            testsFailed++;
        }

        // Test 11: Test statistics update
        console.log('\n📋 Test 11: Verifying statistics display...');
        try {
            const stats = await page.$$eval('.stats span', spans => 
                spans.map(span => span.textContent).filter(text => text !== '•')
            );
            
            console.log(`  Stats found: ${stats.join(', ')}`);
            
            if (stats.length >= 2 && stats.some(s => s.includes('active')) && stats.some(s => s.includes('completed'))) {
                console.log('✅ Statistics displaying correctly');
                testsPassed++;
            } else {
                console.log('❌ Statistics not displaying as expected');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to verify statistics:', error.message);
            testsFailed++;
        }

        // Test 12: Test empty input validation
        console.log('\n📋 Test 12: Testing empty input validation...');
        try {
            const beforeCount = await page.$$eval('.todo-item', items => items.length);
            
            // Try to add empty todo
            await page.focus('#todo-input');
            await page.keyboard.press('Space');
            await page.keyboard.press('Space');
            await page.click('.add-btn');
            await page.waitForTimeout(300);
            
            const afterCount = await page.$$eval('.todo-item', items => items.length);
            
            if (beforeCount === afterCount) {
                console.log('✅ Empty todos are not added');
                testsPassed++;
            } else {
                console.log('❌ Empty todo was added');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test empty input:', error.message);
            testsFailed++;
        }

        // Test 13: Test keyboard interaction
        console.log('\n📋 Test 13: Testing keyboard interaction...');
        try {
            await page.type('#todo-input', 'Test Enter key');
            await page.keyboard.press('Enter');
            await page.waitForTimeout(300);
            
            const lastTodo = await page.$$eval('.todo-item', items => {
                const last = items[items.length - 1];
                return last?.querySelector('.todo-text')?.textContent;
            });
            
            if (lastTodo === 'Test Enter key') {
                console.log('✅ Enter key adds todo');
                testsPassed++;
            } else {
                console.log('❌ Enter key does not add todo');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test keyboard interaction:', error.message);
            testsFailed++;
        }

        // Test 14: Check for console errors
        console.log('\n📋 Test 14: Checking for console errors...');
        if (consoleErrors.length === 0) {
            console.log('✅ No console errors detected');
            testsPassed++;
        } else {
            console.log(`❌ Found ${consoleErrors.length} console errors:`);
            consoleErrors.forEach(error => console.log(`   - ${error}`));
            testsFailed++;
        }

        // Final screenshot
        await takeScreenshot(page, '09-final-state');

    } catch (error) {
        console.error('\n💥 Critical error:', error);
        errors.push(error.message);
    } finally {
        await browser.close();
        
        // Summary
        console.log('\n' + '='.repeat(50));
        console.log('📊 TODO APP TEST SUMMARY');
        console.log('='.repeat(50));
        console.log(`✅ Passed: ${testsPassed}`);
        console.log(`❌ Failed: ${testsFailed}`);
        console.log(`📸 Screenshots saved in: ${SCREENSHOT_DIR}`);
        if (errors.length > 0) {
            console.log(`⚠️  Critical errors: ${errors.length}`);
        }
        console.log('='.repeat(50) + '\n');

        process.exit(testsFailed > 0 || errors.length > 0 ? 1 : 0);
    }
}

// Check if server is running
async function checkServer() {
    try {
        const response = await fetch(TODO_URL);
        return response.ok;
    } catch (error) {
        return false;
    }
}

// Run tests
console.log('🔧 Layer9 Todo App - Comprehensive Puppeteer Test Suite');
console.log('======================================================\n');

checkServer().then(isRunning => {
    if (!isRunning) {
        console.error('❌ HTTP server is not running at', TODO_URL);
        console.error('Please start the server first from the project root:');
        console.error('  cd .. && python3 -m http.server 8000');
        process.exit(1);
    }
    return runTodoTests();
}).catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});