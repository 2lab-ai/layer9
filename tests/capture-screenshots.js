const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// Configuration
const EXAMPLES = [
    {
        name: 'counter',
        url: 'http://localhost:8000/examples/counter/',
        actions: async (page) => {
            // Initial state
            await page.waitForTimeout(1000);
            await screenshot(page, 'counter-initial');
            
            // After increment
            await page.click('.btn-increment');
            await page.waitForTimeout(500);
            await screenshot(page, 'counter-increment');
            
            // After using quick buttons
            await page.click('.quick-btn:nth-child(4)'); // +10
            await page.waitForTimeout(500);
            await screenshot(page, 'counter-quick-action');
            
            // Negative state
            for (let i = 0; i < 15; i++) {
                await page.click('.btn-decrement');
            }
            await page.waitForTimeout(500);
            await screenshot(page, 'counter-negative');
        }
    },
    {
        name: 'async-counter',
        url: 'http://localhost:8000/examples/async-counter/',
        actions: async (page) => {
            // Loading state
            await screenshot(page, 'async-counter-loading');
            
            // Wait for initial load
            await page.waitForFunction(
                () => document.querySelector('.counter-value')?.textContent === '42',
                { timeout: 5000 }
            );
            await screenshot(page, 'async-counter-loaded');
            
            // Async operation
            await page.click('.btn-async:nth-child(2)'); // Random
            await page.waitForTimeout(500);
            await screenshot(page, 'async-counter-async-operation');
        }
    },
    {
        name: 'todo-app',
        url: 'http://localhost:8000/examples/todo-app/',
        actions: async (page) => {
            // Wait for load
            await page.waitForFunction(
                () => document.getElementById('loading').style.display === 'none',
                { timeout: 5000 }
            );
            
            // Empty state
            await screenshot(page, 'todo-empty');
            
            // Add todos
            const todos = ['Build with Layer9', 'Test everything', 'Deploy to production'];
            for (const todo of todos) {
                await page.type('#todo-input', todo);
                await page.click('.add-btn');
                await page.waitForTimeout(200);
            }
            await screenshot(page, 'todo-with-items');
            
            // Mark some as completed
            await page.click('.todo-item:nth-child(1) .todo-checkbox');
            await page.click('.todo-item:nth-child(2) .todo-checkbox');
            await page.waitForTimeout(300);
            await screenshot(page, 'todo-completed');
            
            // Filter active
            await page.click('.filter-btn:nth-child(2)');
            await page.waitForTimeout(300);
            await screenshot(page, 'todo-filtered');
        }
    },
    {
        name: 'memory-game',
        url: 'http://localhost:8000/examples/memory-game/',
        actions: async (page) => {
            // Initial state
            await page.waitForSelector('.game-board');
            await screenshot(page, 'memory-initial');
            
            // Flip some cards
            await page.click('.card:nth-child(1)');
            await page.waitForTimeout(300);
            await page.click('.card:nth-child(2)');
            await page.waitForTimeout(300);
            await screenshot(page, 'memory-cards-flipped');
            
            // Wait for them to flip back
            await page.waitForTimeout(1500);
            
            // Try to find a match
            for (let i = 1; i <= 4; i++) {
                await page.click(`.card:nth-child(${i})`);
                await page.waitForTimeout(400);
            }
            await screenshot(page, 'memory-in-progress');
        }
    }
];

const DOC_SCREENSHOTS_DIR = path.join(__dirname, 'doc-screenshots');

async function screenshot(page, filename) {
    const filepath = path.join(DOC_SCREENSHOTS_DIR, `${filename}.png`);
    await page.screenshot({ path: filepath, fullPage: true });
    console.log(`📸 Captured: ${filename}.png`);
}

async function captureScreenshots() {
    console.log('📸 Layer9 Documentation Screenshot Capture\n');
    
    // Create directory
    if (!fs.existsSync(DOC_SCREENSHOTS_DIR)) {
        fs.mkdirSync(DOC_SCREENSHOTS_DIR, { recursive: true });
    }
    
    // Check server
    try {
        const response = await fetch('http://localhost:8000');
        if (!response.ok) throw new Error('Server not responding');
    } catch (error) {
        console.error('❌ Server not running at http://localhost:8000');
        console.error('Please start the server first: python3 -m http.server 8000');
        process.exit(1);
    }
    
    const browser = await puppeteer.launch({
        headless: 'new',
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    try {
        for (const example of EXAMPLES) {
            console.log(`\n📌 Capturing ${example.name}...`);
            
            const page = await browser.newPage();
            await page.setViewport({ width: 1280, height: 800 });
            
            // Navigate to example
            await page.goto(example.url, { waitUntil: 'networkidle2' });
            
            // Run example-specific actions
            await example.actions(page);
            
            await page.close();
        }
        
        console.log(`\n✅ Screenshots saved to: ${DOC_SCREENSHOTS_DIR}`);
        
    } catch (error) {
        console.error('❌ Error:', error);
    } finally {
        await browser.close();
    }
}

// Run capture
captureScreenshots().catch(console.error);