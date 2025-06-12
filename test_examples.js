// Test script to check if Layer9 examples work
const puppeteer = require('puppeteer');

async function testExample(name, path, tests) {
    console.log(`\n=== Testing ${name} ===`);
    const browser = await puppeteer.launch({ headless: 'new' });
    const page = await browser.newPage();
    
    // Capture console messages
    const consoleLogs = [];
    page.on('console', msg => {
        consoleLogs.push({
            type: msg.type(),
            text: msg.text()
        });
    });
    
    // Capture page errors
    const pageErrors = [];
    page.on('pageerror', error => {
        pageErrors.push(error.toString());
    });
    
    try {
        await page.goto(`http://localhost:8888${path}`, { waitUntil: 'networkidle0', timeout: 10000 });
        
        // Wait for WASM to load
        await page.waitForTimeout(2000);
        
        // Run specific tests for each example
        if (tests) {
            await tests(page);
        }
        
        // Check for errors
        const errors = consoleLogs.filter(log => log.type === 'error');
        if (errors.length > 0) {
            console.log('❌ JavaScript errors found:');
            errors.forEach(err => console.log(`   ${err.text}`));
        } else {
            console.log('✅ No JavaScript errors');
        }
        
        if (pageErrors.length > 0) {
            console.log('❌ Page errors found:');
            pageErrors.forEach(err => console.log(`   ${err}`));
        } else {
            console.log('✅ No page errors');
        }
        
        // Take screenshot
        await page.screenshot({ path: `/tmp/${name}-test.png` });
        console.log(`📸 Screenshot saved to /tmp/${name}-test.png`);
        
    } catch (error) {
        console.log(`❌ Failed to load: ${error.message}`);
    } finally {
        await browser.close();
    }
}

async function runAllTests() {
    // Test Counter example
    await testExample('Counter', '/examples/counter/', async (page) => {
        const hasCounter = await page.$('#app') !== null;
        console.log(hasCounter ? '✅ App container found' : '❌ App container not found');
        
        // Check if buttons exist
        const incrementButton = await page.$('button');
        if (incrementButton) {
            console.log('✅ Button found');
            // Click and check if it works
            await incrementButton.click();
            await page.waitForTimeout(500);
        } else {
            console.log('❌ No buttons found');
        }
    });
    
    // Test Async Counter example
    await testExample('Async Counter', '/examples/async-counter/', async (page) => {
        const hasApp = await page.$('#app') !== null;
        console.log(hasApp ? '✅ App container found' : '❌ App container not found');
    });
    
    // Test Todo App example
    await testExample('Todo App', '/examples/todo-app/', async (page) => {
        const hasApp = await page.$('#app') !== null;
        console.log(hasApp ? '✅ App container found' : '❌ App container not found');
        
        // Check for input field
        const input = await page.$('input[type="text"]');
        if (input) {
            console.log('✅ Input field found');
            await input.type('Test todo item');
            // Look for add button
            const addButton = await page.$('button');
            if (addButton) {
                await addButton.click();
                await page.waitForTimeout(500);
                console.log('✅ Added test todo item');
            }
        } else {
            console.log('❌ No input field found');
        }
    });
    
    // Test Memory Game example
    await testExample('Memory Game', '/examples/memory-game/', async (page) => {
        const hasApp = await page.$('#app') !== null;
        console.log(hasApp ? '✅ App container found' : '❌ App container not found');
        
        // Check for game cards
        const cards = await page.$$('.card');
        console.log(cards.length > 0 ? `✅ Found ${cards.length} game cards` : '❌ No game cards found');
        
        if (cards.length > 0) {
            // Try clicking a card
            await cards[0].click();
            await page.waitForTimeout(500);
            console.log('✅ Clicked first card');
        }
    });
    
    console.log('\n=== Test Summary ===');
    console.log('Checked examples: Counter, Async Counter, Todo App, Memory Game');
    console.log('Missing builds: Form Validation (compilation errors), GitHub Dashboard (workspace config issue)');
}

// Check if puppeteer is installed
try {
    runAllTests().catch(console.error);
} catch (error) {
    console.log('Puppeteer not installed. Installing...');
    const { execSync } = require('child_process');
    execSync('npm install puppeteer', { stdio: 'inherit' });
    runAllTests().catch(console.error);
}