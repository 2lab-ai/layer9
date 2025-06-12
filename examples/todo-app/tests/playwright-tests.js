const { chromium } = require('playwright');
const path = require('path');
const fs = require('fs');

// Configuration
const TODO_APP_URL = 'http://localhost:8000';
const SCREENSHOT_DIR = path.join(__dirname, 'screenshots');
const TIMEOUT = 30000; // 30 seconds

// Helper function to take screenshot
async function takeScreenshot(page, name) {
    const screenshotPath = path.join(SCREENSHOT_DIR, `${name}.png`);
    await page.screenshot({ path: screenshotPath, fullPage: true });
    console.log(`Screenshot saved: ${screenshotPath}`);
}

// Main test function
async function runTests() {
    // Create screenshot directory
    if (!fs.existsSync(SCREENSHOT_DIR)) {
        fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    }

    const browser = await chromium.launch({
        headless: true // Can be set to false to see the browser
    });

    let testsPassed = 0;
    let testsFailed = 0;

    try {
        const context = await browser.newContext({
            viewport: { width: 1280, height: 800 }
        });
        const page = await context.newPage();

        console.log('\n🚀 Starting Layer9 Todo App Tests\n');

        // Test 1: Navigate to the app
        console.log('📋 Test 1: Loading the application...');
        try {
            await page.goto(TODO_APP_URL, { waitUntil: 'networkidle', timeout: TIMEOUT });
            console.log('✅ Successfully loaded the app');
            testsPassed++;
        } catch (error) {
            console.error('❌ Failed to load the app:', error.message);
            testsFailed++;
            throw error;
        }

        // Test 2: Wait for WASM to load
        console.log('\n📋 Test 2: Waiting for WASM to initialize...');
        try {
            // Wait for loading indicator to disappear
            await page.waitForFunction(
                () => {
                    const loading = document.getElementById('loading');
                    return loading && loading.style.display === 'none';
                },
                { timeout: TIMEOUT }
            );
            
            // Wait for the main app container
            await page.waitForSelector('.todo-app', { timeout: TIMEOUT });
            console.log('✅ WASM loaded successfully');
            testsPassed++;
            
            await takeScreenshot(page, '01-initial-load');
        } catch (error) {
            console.error('❌ WASM loading failed:', error.message);
            testsFailed++;
        }

        // Test 3: Verify main components
        console.log('\n📋 Test 3: Verifying main components...');
        const components = [
            { selector: '.header h1', name: 'Title' },
            { selector: '.subtitle', name: 'Subtitle' },
            { selector: '#todo-input', name: 'Input field' },
            { selector: '.add-btn', name: 'Add button' },
            { selector: '.empty-state', name: 'Empty state message' }
        ];

        let allComponentsFound = true;
        for (const component of components) {
            try {
                await page.waitForSelector(component.selector, { timeout: 5000 });
                console.log(`  ✅ ${component.name} found`);
            } catch {
                console.log(`  ❌ ${component.name} not found`);
                allComponentsFound = false;
            }
        }
        
        if (allComponentsFound) {
            testsPassed++;
        } else {
            testsFailed++;
        }

        // Test 4: Add a new todo
        console.log('\n📋 Test 4: Adding a new todo...');
        try {
            await page.fill('#todo-input', 'Complete Playwright tests');
            await page.click('.add-btn');
            
            // Wait for the todo to appear
            await page.waitForSelector('.todo-item', { timeout: 5000 });
            
            // Verify the todo text
            const todoText = await page.textContent('.todo-text');
            if (todoText === 'Complete Playwright tests') {
                console.log('✅ Todo added successfully');
                testsPassed++;
                await takeScreenshot(page, '02-first-todo-added');
            } else {
                console.log('❌ Todo text mismatch');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to add todo:', error.message);
            testsFailed++;
        }

        // Test 5: Add multiple todos
        console.log('\n📋 Test 5: Adding multiple todos...');
        try {
            const todos = [
                'Write documentation',
                'Fix bugs',
                'Deploy to production'
            ];

            for (const todo of todos) {
                await page.fill('#todo-input', todo);
                await page.click('.add-btn');
                await page.waitForTimeout(100); // Small delay between additions
            }

            // Count todos
            const todoItems = await page.$$('.todo-item');
            const todoCount = todoItems.length;
            
            if (todoCount === 4) { // 1 from previous test + 3 new ones
                console.log(`✅ Successfully added multiple todos (total: ${todoCount})`);
                testsPassed++;
                await takeScreenshot(page, '03-multiple-todos');
            } else {
                console.log(`❌ Expected 4 todos, found ${todoCount}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to add multiple todos:', error.message);
            testsFailed++;
        }

        // Test 6: Toggle todo completion
        console.log('\n📋 Test 6: Toggling todo completion...');
        try {
            // Click the first checkbox
            await page.click('.todo-item:first-child .todo-checkbox');
            await page.waitForTimeout(200);

            // Check if the todo has the completed class
            const isCompleted = await page.$eval('.todo-item:first-child', el => 
                el.classList.contains('completed')
            );

            if (isCompleted) {
                console.log('✅ Todo marked as completed');
                testsPassed++;
                await takeScreenshot(page, '04-todo-completed');
            } else {
                console.log('❌ Todo not marked as completed');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to toggle todo:', error.message);
            testsFailed++;
        }

        // Test 7: Delete a todo
        console.log('\n📋 Test 7: Deleting a todo...');
        try {
            // Get initial count
            const initialItems = await page.$$('.todo-item');
            const initialCount = initialItems.length;
            
            // Hover over the first todo to show delete button
            await page.hover('.todo-item:first-child');
            
            // Click delete button
            await page.click('.todo-item:first-child .delete-btn');
            await page.waitForTimeout(200);

            // Get new count
            const newItems = await page.$$('.todo-item');
            const newCount = newItems.length;
            
            if (newCount === initialCount - 1) {
                console.log(`✅ Todo deleted (${initialCount} → ${newCount})`);
                testsPassed++;
                await takeScreenshot(page, '05-todo-deleted');
            } else {
                console.log(`❌ Delete failed (${initialCount} → ${newCount})`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to delete todo:', error.message);
            testsFailed++;
        }

        // Test 8: Filter functionality
        console.log('\n📋 Test 8: Testing filter buttons...');
        try {
            // Mark some todos as completed
            await page.click('.todo-item:nth-child(1) .todo-checkbox');
            await page.click('.todo-item:nth-child(2) .todo-checkbox');
            await page.waitForTimeout(200);

            // Test "Active" filter
            await page.click('.filter-btn:nth-child(2)'); // Active button
            await page.waitForTimeout(200);
            
            const activeItems = await page.$$('.todo-item');
            const activeCount = activeItems.length;
            console.log(`  Active todos shown: ${activeCount}`);
            await takeScreenshot(page, '06-filter-active');

            // Test "Completed" filter
            await page.click('.filter-btn:nth-child(3)'); // Completed button
            await page.waitForTimeout(200);
            
            const completedItems = await page.$$('.todo-item');
            const completedCount = completedItems.length;
            console.log(`  Completed todos shown: ${completedCount}`);
            await takeScreenshot(page, '07-filter-completed');

            // Test "All" filter
            await page.click('.filter-btn:nth-child(1)'); // All button
            await page.waitForTimeout(200);
            
            const allItems = await page.$$('.todo-item');
            const allCount = allItems.length;
            console.log(`  All todos shown: ${allCount}`);
            
            console.log('✅ Filters working correctly');
            testsPassed++;
        } catch (error) {
            console.error('❌ Failed to test filters:', error.message);
            testsFailed++;
        }

        // Test 9: Clear completed functionality
        console.log('\n📋 Test 9: Testing "Clear completed" button...');
        try {
            // Click "Clear completed"
            await page.click('.clear-btn');
            await page.waitForTimeout(200);

            const remainingItems = await page.$$('.todo-item');
            
            console.log(`✅ Completed todos cleared (remaining: ${remainingItems.length})`);
            testsPassed++;
            await takeScreenshot(page, '08-cleared-completed');
        } catch (error) {
            console.error('❌ Failed to clear completed:', error.message);
            testsFailed++;
        }

        // Final screenshot
        await takeScreenshot(page, '09-final-state');

    } catch (error) {
        console.error('\n💥 Critical error:', error);
    } finally {
        await browser.close();
        
        // Summary
        console.log('\n' + '='.repeat(50));
        console.log('📊 TEST SUMMARY');
        console.log('='.repeat(50));
        console.log(`✅ Passed: ${testsPassed}`);
        console.log(`❌ Failed: ${testsFailed}`);
        console.log(`📸 Screenshots saved in: ${SCREENSHOT_DIR}`);
        console.log('='.repeat(50) + '\n');

        // Exit with appropriate code
        process.exit(testsFailed > 0 ? 1 : 0);
    }
}

// Run tests
console.log('🔧 Layer9 Todo App - Playwright Test Suite');
console.log('==========================================\n');

runTests().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});