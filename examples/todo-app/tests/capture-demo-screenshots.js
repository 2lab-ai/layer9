const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// Configuration
const TODO_APP_URL = 'http://localhost:8082';
const SCREENSHOT_DIR = path.join(__dirname, 'screenshots', 'demo');
const DELAY_BETWEEN_ACTIONS = 1000; // 1 second between actions for smooth GIF
const TYPING_DELAY = 100; // Delay between keystrokes for realistic typing

// Helper function to create directory
function ensureDirectory(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

// Helper function to take screenshot with counter
async function takeScreenshot(page, counter, description) {
    const filename = `${String(counter).padStart(3, '0')}-${description}.png`;
    const screenshotPath = path.join(SCREENSHOT_DIR, filename);
    await page.screenshot({ 
        path: screenshotPath, 
        fullPage: false,
        clip: {
            x: 0,
            y: 0,
            width: 800,
            height: 600
        }
    });
    console.log(`📸 ${filename} - ${description}`);
    return counter + 1;
}

// Helper function to type text with realistic timing
async function typeText(page, selector, text) {
    await page.click(selector);
    await page.evaluate((selector) => {
        document.querySelector(selector).value = '';
    }, selector);
    await page.type(selector, text, { delay: TYPING_DELAY });
}

// Main capture function
async function captureDemo() {
    ensureDirectory(SCREENSHOT_DIR);
    
    const browser = await puppeteer.launch({
        headless: false,
        args: ['--window-size=800,600']
    });

    try {
        const page = await browser.newPage();
        await page.setViewport({ width: 800, height: 600 });
        
        console.log('\n🎬 Starting Todo App Demo Screenshot Capture\n');
        
        let counter = 1;

        // Navigate to the app
        console.log('Loading application...');
        await page.goto(TODO_APP_URL, { waitUntil: 'networkidle2' });
        
        // Wait for WASM to load
        await page.waitForFunction(
            () => document.getElementById('loading').style.display === 'none',
            { timeout: 30000 }
        );
        await page.waitForSelector('.todo-app');
        
        // 1. Initial empty state
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'empty-state');
        
        // 2. Focus on input field
        await page.click('#todo-input');
        await page.waitForTimeout(500);
        counter = await takeScreenshot(page, counter, 'input-focused');
        
        // 3. Type first todo
        await typeText(page, '#todo-input', 'Buy groceries');
        await page.waitForTimeout(500);
        counter = await takeScreenshot(page, counter, 'typing-first-todo');
        
        // 4. Add first todo
        await page.click('.add-btn');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'first-todo-added');
        
        // 5. Add second todo
        await typeText(page, '#todo-input', 'Write project documentation');
        await page.waitForTimeout(500);
        counter = await takeScreenshot(page, counter, 'typing-second-todo');
        
        await page.click('.add-btn');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'second-todo-added');
        
        // 6. Add third todo
        await typeText(page, '#todo-input', 'Call dentist for appointment');
        await page.click('.add-btn');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'third-todo-added');
        
        // 7. Add fourth todo
        await typeText(page, '#todo-input', 'Prepare presentation slides');
        await page.click('.add-btn');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'multiple-todos');
        
        // 8. Complete first todo
        await page.click('.todo-item:nth-child(1) .todo-checkbox');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'first-todo-completed');
        
        // 9. Complete third todo
        await page.click('.todo-item:nth-child(3) .todo-checkbox');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'multiple-completed');
        
        // 10. Show Active filter
        await page.click('.filter-btn:nth-child(2)'); // Active button
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'filter-active');
        
        // 11. Show Completed filter
        await page.click('.filter-btn:nth-child(3)'); // Completed button
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'filter-completed');
        
        // 12. Back to All filter
        await page.click('.filter-btn:nth-child(1)'); // All button
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'filter-all');
        
        // 13. Hover to show delete button
        await page.hover('.todo-item:nth-child(2)');
        await page.waitForTimeout(500);
        counter = await takeScreenshot(page, counter, 'hover-delete-button');
        
        // 14. Delete a todo
        await page.click('.todo-item:nth-child(2) .delete-btn');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'todo-deleted');
        
        // 15. Clear completed
        await page.click('.clear-btn');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'cleared-completed');
        
        // 16. Add more realistic todos
        const moreTodos = [
            'Review pull requests',
            'Update project README',
            'Test new features',
            'Deploy to staging'
        ];
        
        for (const todo of moreTodos) {
            await typeText(page, '#todo-input', todo);
            await page.click('.add-btn');
            await page.waitForTimeout(500);
        }
        
        counter = await takeScreenshot(page, counter, 'final-state');
        
        // 17. Complete some todos for a realistic final view
        await page.click('.todo-item:nth-child(1) .todo-checkbox');
        await page.waitForTimeout(300);
        await page.click('.todo-item:nth-child(3) .todo-checkbox');
        await page.waitForTimeout(300);
        await page.click('.todo-item:nth-child(5) .todo-checkbox');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, counter, 'realistic-final-state');

        console.log(`\n✅ Captured ${counter - 1} screenshots successfully!`);
        console.log(`📁 Screenshots saved in: ${SCREENSHOT_DIR}`);
        
    } catch (error) {
        console.error('❌ Error during capture:', error);
    } finally {
        await browser.close();
    }
}

// Check if server is running
async function checkServer() {
    try {
        const response = await fetch(TODO_APP_URL);
        return response.ok;
    } catch (error) {
        return false;
    }
}

// Run capture
console.log('🎬 Todo App Demo Screenshot Capture Tool');
console.log('=====================================\n');

checkServer().then(isRunning => {
    if (!isRunning) {
        console.error(`❌ Server is not running at ${TODO_APP_URL}`);
        console.error('Please start the server first with the correct port.');
        process.exit(1);
    }
    return captureDemo();
}).catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});