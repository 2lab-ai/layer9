const { chromium } = require('playwright');

async function testExample(name, url) {
    console.log(`\n=== Testing ${name} ===`);
    console.log(`URL: ${url}`);
    
    const browser = await chromium.launch({ headless: true });
    const context = await browser.newContext();
    const page = await context.newPage();
    
    const errors = [];
    const warnings = [];
    const logs = [];
    
    // Capture console messages
    page.on('console', msg => {
        const text = msg.text();
        const type = msg.type();
        
        if (type === 'error') {
            errors.push(text);
        } else if (type === 'warning') {
            warnings.push(text);
        } else {
            logs.push(text);
        }
    });
    
    // Capture page errors
    page.on('pageerror', error => {
        errors.push(`Page error: ${error.message}`);
    });
    
    // Capture network failures
    page.on('requestfailed', request => {
        errors.push(`Network request failed: ${request.url()} - ${request.failure().errorText}`);
    });
    
    try {
        // Navigate to the page
        const response = await page.goto(url, { 
            waitUntil: 'networkidle',
            timeout: 30000 
        });
        
        console.log(`HTTP Status: ${response.status()}`);
        
        // Wait a bit for WASM to initialize
        await page.waitForTimeout(3000);
        
        // Check if the app element exists
        const appExists = await page.$('#app') !== null;
        console.log(`App element exists: ${appExists ? '✅' : '❌'}`);
        
        // Get the app content
        if (appExists) {
            const appContent = await page.$eval('#app', el => el.innerHTML);
            console.log(`App has content: ${appContent.length > 0 ? '✅' : '❌'} (${appContent.length} chars)`);
        }
        
        // Example-specific tests
        if (name === 'Counter') {
            const buttons = await page.$$('button');
            console.log(`Buttons found: ${buttons.length}`);
            
            if (buttons.length > 0) {
                // Try clicking increment
                await buttons[0].click();
                await page.waitForTimeout(500);
                console.log('Clicked first button');
            }
        } else if (name === 'Todo App') {
            const input = await page.$('input[type="text"]');
            console.log(`Input field found: ${input ? '✅' : '❌'}`);
            
            if (input) {
                await input.type('Test todo item');
                const addButton = await page.$('button');
                if (addButton) {
                    await addButton.click();
                    await page.waitForTimeout(500);
                    console.log('Added test todo item');
                }
            }
        } else if (name === 'Memory Game') {
            const cards = await page.$$('.card, [class*="card"]');
            console.log(`Game cards found: ${cards.length}`);
            
            if (cards.length > 0) {
                await cards[0].click();
                await page.waitForTimeout(500);
                console.log('Clicked first card');
            }
        }
        
        // Report errors and warnings
        if (errors.length > 0) {
            console.log('\n❌ Errors:');
            errors.forEach(err => console.log(`  - ${err}`));
        } else {
            console.log('\n✅ No errors');
        }
        
        if (warnings.length > 0) {
            console.log('\n⚠️  Warnings:');
            warnings.forEach(warn => console.log(`  - ${warn}`));
        }
        
        // Take a screenshot
        await page.screenshot({ path: `screenshots/${name.toLowerCase().replace(/\s+/g, '-')}.png` });
        console.log(`📸 Screenshot saved`);
        
    } catch (error) {
        console.log(`\n❌ Failed to test: ${error.message}`);
    } finally {
        await browser.close();
    }
}

async function runTests() {
    console.log('Starting Layer9 Examples Test Suite...');
    
    // Create screenshots directory
    const fs = require('fs');
    if (!fs.existsSync('screenshots')) {
        fs.mkdirSync('screenshots');
    }
    
    const baseUrl = 'http://localhost:8888';
    
    // Test built examples
    await testExample('Counter', `${baseUrl}/examples/counter/`);
    await testExample('Async Counter', `${baseUrl}/examples/async-counter/`);
    await testExample('Todo App', `${baseUrl}/examples/todo-app/`);
    await testExample('Memory Game', `${baseUrl}/examples/memory-game/`);
    
    // Test unbuild examples (will likely fail)
    await testExample('Form Validation', `${baseUrl}/examples/form-validation/`);
    await testExample('GitHub Dashboard', `${baseUrl}/examples/github-dashboard/`);
    
    console.log('\n=== Test Summary ===');
    console.log('✅ Tested all examples');
    console.log('📁 Screenshots saved in ./screenshots/');
}

// Check if playwright is installed
try {
    require('playwright');
    runTests().catch(console.error);
} catch (error) {
    console.log('Installing playwright...');
    const { execSync } = require('child_process');
    execSync('npm install playwright', { stdio: 'inherit' });
    console.log('Please run the script again after installation.');
}