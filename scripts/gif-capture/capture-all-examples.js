const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');
const { execSync } = require('child_process');

// Configuration
const BASE_URL = 'http://localhost:8080';
const EXAMPLES = {
    counter: {
        port: 8080,
        name: 'Counter',
        scenarios: ['increment', 'decrement', 'reset', 'quick-actions']
    },
    'async-counter': {
        port: 8081,
        name: 'Async Counter',
        scenarios: ['loading', 'async-increment', 'async-decrement', 'error-handling']
    },
    'todo-app': {
        port: 8082,
        name: 'Todo App',
        scenarios: ['add-todos', 'complete-todos', 'filter', 'delete-clear']
    },
    'memory-game': {
        port: 8083,
        name: 'Memory Game',
        scenarios: ['start', 'flip-cards', 'match', 'win']
    }
};

const SCREENSHOT_BASE_DIR = path.join(__dirname, '../../examples');
const DELAY_BETWEEN_ACTIONS = 800;
const TYPING_DELAY = 80;

// Helper functions
function ensureDirectory(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

async function takeScreenshot(page, example, scenario, counter, description) {
    const dir = path.join(SCREENSHOT_BASE_DIR, example, 'screenshots', scenario);
    ensureDirectory(dir);
    
    const filename = `${String(counter).padStart(3, '0')}-${description}.png`;
    const screenshotPath = path.join(dir, filename);
    
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
    
    console.log(`  📸 ${filename}`);
    return counter + 1;
}

async function typeText(page, selector, text) {
    await page.click(selector);
    await page.evaluate((selector) => {
        const element = document.querySelector(selector);
        if (element) element.value = '';
    }, selector);
    await page.type(selector, text, { delay: TYPING_DELAY });
}

// Capture functions for each example
async function captureCounter(page, example) {
    console.log('\n🎯 Capturing Counter Example');
    
    await page.goto(`http://localhost:${EXAMPLES[example].port}`, { waitUntil: 'networkidle2' });
    
    // Wait for loading to complete
    await page.waitForFunction(
        () => {
            const loading = document.querySelector('.loading');
            return !loading || loading.style.display === 'none';
        },
        { timeout: 30000 }
    );
    await page.waitForTimeout(1000);
    
    // Scenario 1: Increment
    console.log('  📂 Scenario: increment');
    let counter = 1;
    counter = await takeScreenshot(page, example, 'increment', counter, 'initial-state');
    
    for (let i = 0; i < 3; i++) {
        await page.click('button:has-text("Increment")');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, example, 'increment', counter, `increment-${i + 1}`);
    }
    
    // Scenario 2: Decrement
    console.log('  📂 Scenario: decrement');
    counter = 1;
    counter = await takeScreenshot(page, example, 'decrement', counter, 'start-at-3');
    
    for (let i = 0; i < 3; i++) {
        await page.click('button:has-text("Decrement")');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, example, 'decrement', counter, `decrement-${i + 1}`);
    }
    
    // Scenario 3: Reset
    console.log('  📂 Scenario: reset');
    counter = 1;
    await page.click('button:has-text("Increment")');
    await page.click('button:has-text("Increment")');
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'reset', counter, 'before-reset');
    
    await page.click('button:has-text("Reset")');
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'reset', counter, 'after-reset');
    
    // Scenario 4: Quick actions
    console.log('  📂 Scenario: quick-actions');
    counter = 1;
    counter = await takeScreenshot(page, example, 'quick-actions', counter, 'initial');
    
    // Quick increments
    for (let i = 0; i < 5; i++) {
        await page.click('button:has-text("Increment")');
        await page.waitForTimeout(200);
    }
    counter = await takeScreenshot(page, example, 'quick-actions', counter, 'after-quick-increments');
    
    // Quick decrements
    for (let i = 0; i < 3; i++) {
        await page.click('button:has-text("Decrement")');
        await page.waitForTimeout(200);
    }
    counter = await takeScreenshot(page, example, 'quick-actions', counter, 'final-state');
}

async function captureAsyncCounter(page, example) {
    console.log('\n⏱️  Capturing Async Counter Example');
    
    await page.goto(`http://localhost:${EXAMPLES[example].port}`, { waitUntil: 'networkidle2' });
    
    // Wait for loading to complete
    await page.waitForFunction(
        () => {
            const loading = document.querySelector('.loading');
            return !loading || loading.style.display === 'none';
        },
        { timeout: 30000 }
    );
    await page.waitForTimeout(1000);
    
    // Scenario 1: Loading states
    console.log('  📂 Scenario: loading');
    let counter = 1;
    counter = await takeScreenshot(page, example, 'loading', counter, 'initial-state');
    
    // Click and capture loading state quickly
    const incrementButton = await page.$('button:has-text("Async Increment")');
    if (incrementButton) {
        await incrementButton.click();
        await page.waitForTimeout(100); // Capture loading state
        counter = await takeScreenshot(page, example, 'loading', counter, 'loading-state');
        
        await page.waitForTimeout(2000); // Wait for operation to complete
        counter = await takeScreenshot(page, example, 'loading', counter, 'operation-complete');
    }
    
    // Scenario 2: Async increment
    console.log('  📂 Scenario: async-increment');
    counter = 1;
    counter = await takeScreenshot(page, example, 'async-increment', counter, 'start');
    
    for (let i = 0; i < 2; i++) {
        await page.click('button:has-text("Async Increment")');
        await page.waitForTimeout(100);
        counter = await takeScreenshot(page, example, 'async-increment', counter, `loading-${i + 1}`);
        await page.waitForTimeout(2000);
        counter = await takeScreenshot(page, example, 'async-increment', counter, `complete-${i + 1}`);
    }
    
    // Scenario 3: Async decrement
    console.log('  📂 Scenario: async-decrement');
    counter = 1;
    counter = await takeScreenshot(page, example, 'async-decrement', counter, 'start');
    
    await page.click('button:has-text("Async Decrement")');
    await page.waitForTimeout(100);
    counter = await takeScreenshot(page, example, 'async-decrement', counter, 'loading');
    await page.waitForTimeout(2000);
    counter = await takeScreenshot(page, example, 'async-decrement', counter, 'complete');
}

async function captureTodoApp(page, example) {
    console.log('\n📝 Capturing Todo App Example');
    
    await page.goto(`http://localhost:${EXAMPLES[example].port}`, { waitUntil: 'networkidle2' });
    
    // Wait for WASM to load
    await page.waitForFunction(
        () => {
            const loading = document.getElementById('loading');
            return loading && loading.style.display === 'none';
        },
        { timeout: 30000 }
    );
    await page.waitForSelector('.todo-app', { timeout: 30000 });
    
    // Scenario 1: Add todos
    console.log('  📂 Scenario: add-todos');
    let counter = 1;
    counter = await takeScreenshot(page, example, 'add-todos', counter, 'empty-state');
    
    const todos = ['Buy groceries', 'Write documentation', 'Review pull requests'];
    for (let i = 0; i < todos.length; i++) {
        await typeText(page, '#todo-input', todos[i]);
        counter = await takeScreenshot(page, example, 'add-todos', counter, `typing-todo-${i + 1}`);
        
        await page.click('.add-btn');
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, example, 'add-todos', counter, `todo-${i + 1}-added`);
    }
    
    // Scenario 2: Complete todos
    console.log('  📂 Scenario: complete-todos');
    counter = 1;
    counter = await takeScreenshot(page, example, 'complete-todos', counter, 'todos-list');
    
    await page.click('.todo-item:nth-child(1) .todo-checkbox');
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'complete-todos', counter, 'first-completed');
    
    await page.click('.todo-item:nth-child(3) .todo-checkbox');
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'complete-todos', counter, 'multiple-completed');
    
    // Scenario 3: Filter
    console.log('  📂 Scenario: filter');
    counter = 1;
    counter = await takeScreenshot(page, example, 'filter', counter, 'all-todos');
    
    await page.click('.filter-btn:nth-child(2)'); // Active
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'filter', counter, 'active-only');
    
    await page.click('.filter-btn:nth-child(3)'); // Completed
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'filter', counter, 'completed-only');
    
    await page.click('.filter-btn:nth-child(1)'); // All
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'filter', counter, 'back-to-all');
    
    // Scenario 4: Delete and clear
    console.log('  📂 Scenario: delete-clear');
    counter = 1;
    counter = await takeScreenshot(page, example, 'delete-clear', counter, 'before-delete');
    
    await page.hover('.todo-item:nth-child(2)');
    await page.waitForTimeout(500);
    counter = await takeScreenshot(page, example, 'delete-clear', counter, 'hover-delete');
    
    await page.click('.todo-item:nth-child(2) .delete-btn');
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'delete-clear', counter, 'todo-deleted');
    
    await page.click('.clear-btn');
    await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    counter = await takeScreenshot(page, example, 'delete-clear', counter, 'completed-cleared');
}

async function captureMemoryGame(page, example) {
    console.log('\n🎮 Capturing Memory Game Example');
    
    await page.goto(`http://localhost:${EXAMPLES[example].port}`, { waitUntil: 'networkidle2' });
    
    // Wait for loading to complete
    await page.waitForFunction(
        () => {
            const loading = document.querySelector('.loading');
            return !loading || loading.style.display === 'none';
        },
        { timeout: 30000 }
    );
    await page.waitForTimeout(1000);
    
    // Scenario 1: Start game
    console.log('  📂 Scenario: start');
    let counter = 1;
    counter = await takeScreenshot(page, example, 'start', counter, 'initial-state');
    
    // If there's a start button, click it
    const startButton = await page.$('button:has-text("Start"), button:has-text("New Game")');
    if (startButton) {
        await startButton.click();
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, example, 'start', counter, 'game-started');
    }
    
    // Scenario 2: Flip cards
    console.log('  📂 Scenario: flip-cards');
    counter = 1;
    counter = await takeScreenshot(page, example, 'flip-cards', counter, 'all-cards-hidden');
    
    // Flip first card
    const cards = await page.$$('.card, .memory-card');
    if (cards.length > 0) {
        await cards[0].click();
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, example, 'flip-cards', counter, 'first-card-flipped');
        
        // Flip second card (non-matching)
        await cards[3].click();
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, example, 'flip-cards', counter, 'two-cards-flipped');
        
        // Wait for cards to flip back
        await page.waitForTimeout(1500);
        counter = await takeScreenshot(page, example, 'flip-cards', counter, 'cards-flipped-back');
    }
    
    // Scenario 3: Match cards
    console.log('  📂 Scenario: match');
    counter = 1;
    
    // Reset game if possible
    const resetButton = await page.$('button:has-text("Reset"), button:has-text("New Game")');
    if (resetButton) {
        await resetButton.click();
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
    }
    
    counter = await takeScreenshot(page, example, 'match', counter, 'game-reset');
    
    // Try to find matching cards (this is simplified - in reality would need to track card values)
    if (cards.length >= 2) {
        await cards[0].click();
        await page.waitForTimeout(600);
        counter = await takeScreenshot(page, example, 'match', counter, 'first-card');
        
        await cards[1].click();
        await page.waitForTimeout(DELAY_BETWEEN_ACTIONS);
        counter = await takeScreenshot(page, example, 'match', counter, 'potential-match');
        
        // Simulate finding a match (would need actual game logic)
        await page.waitForTimeout(1000);
        counter = await takeScreenshot(page, example, 'match', counter, 'match-result');
    }
    
    // Scenario 4: Win state (simulated)
    console.log('  📂 Scenario: win');
    counter = 1;
    // This would require playing through the entire game
    // For now, just capture the current state
    counter = await takeScreenshot(page, example, 'win', counter, 'game-progress');
}

// Main capture function
async function captureAllExamples() {
    const browser = await puppeteer.launch({
        headless: true,
        args: [
            '--window-size=800,600',
            '--no-sandbox',
            '--disable-setuid-sandbox',
            '--disable-dev-shm-usage',
            '--disable-accelerated-2d-canvas',
            '--disable-gpu'
        ]
    });
    
    try {
        const page = await browser.newPage();
        await page.setViewport({ width: 800, height: 600 });
        
        console.log('🎬 Layer9 Examples GIF Capture Tool');
        console.log('===================================\n');
        
        for (const [example, config] of Object.entries(EXAMPLES)) {
            try {
                // Check if server is running
                const response = await fetch(`http://localhost:${config.port}`);
                if (!response.ok) {
                    console.log(`⚠️  Skipping ${config.name} - server not running on port ${config.port}`);
                    continue;
                }
                
                // Capture based on example type
                switch (example) {
                    case 'counter':
                        await captureCounter(page, example);
                        break;
                    case 'async-counter':
                        await captureAsyncCounter(page, example);
                        break;
                    case 'todo-app':
                        await captureTodoApp(page, example);
                        break;
                    case 'memory-game':
                        await captureMemoryGame(page, example);
                        break;
                }
                
                console.log(`✅ Completed ${config.name}\n`);
            } catch (error) {
                console.error(`❌ Error capturing ${config.name}:`, error.message);
            }
        }
        
    } catch (error) {
        console.error('Fatal error:', error);
    } finally {
        await browser.close();
    }
}

// Check which servers are running
async function checkServers() {
    console.log('🔍 Checking example servers...\n');
    
    const running = [];
    for (const [example, config] of Object.entries(EXAMPLES)) {
        try {
            const response = await fetch(`http://localhost:${config.port}`);
            if (response.ok) {
                running.push(`✅ ${config.name} (port ${config.port})`);
            } else {
                running.push(`❌ ${config.name} (port ${config.port})`);
            }
        } catch (error) {
            running.push(`❌ ${config.name} (port ${config.port})`);
        }
    }
    
    console.log(running.join('\n'));
    console.log('\nMake sure to start the servers for the examples you want to capture.\n');
}

// Run
if (process.argv.includes('--check')) {
    checkServers();
} else {
    captureAllExamples().catch(console.error);
}