const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// Configuration
const COUNTER_URL = 'http://localhost:8000/examples/counter/';
const SCREENSHOT_DIR = path.join(__dirname, '../screenshots/counter');
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
async function runCounterTests() {
    console.log('\n🚀 Starting Layer9 Counter Tests\n');
    
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

        // Test 1: Navigate to counter page
        console.log('📋 Test 1: Loading the counter application...');
        try {
            await page.goto(COUNTER_URL, { waitUntil: 'networkidle2', timeout: TIMEOUT });
            console.log('✅ Successfully loaded the counter app');
            testsPassed++;
        } catch (error) {
            console.error('❌ Failed to load the counter app:', error.message);
            testsFailed++;
            throw error;
        }

        // Test 2: Verify WASM loads correctly
        console.log('\n📋 Test 2: Verifying WASM initialization...');
        try {
            // Wait for loading to complete
            await page.waitForFunction(
                () => !document.querySelector('.loading'),
                { timeout: TIMEOUT }
            );
            
            // Check that the main counter content is visible
            await waitForElement(page, '.beautiful-counter');
            await waitForElement(page, '.counter-content');
            
            console.log('✅ WASM loaded successfully');
            testsPassed++;
            await takeScreenshot(page, '01-initial-load');
        } catch (error) {
            console.error('❌ WASM loading failed:', error.message);
            testsFailed++;
        }

        // Test 3: Verify all main components render
        console.log('\n📋 Test 3: Verifying all counter components...');
        const components = [
            { selector: 'h1', name: 'Title with Layer9 branding' },
            { selector: '.subtitle', name: 'Subtitle' },
            { selector: '.counter-display', name: 'Counter display' },
            { selector: '.counter-value', name: 'Counter value' },
            { selector: '.counter-label', name: 'Counter label' },
            { selector: '.quick-actions', name: 'Quick action buttons' },
            { selector: '.main-controls', name: 'Main control buttons' },
            { selector: '.btn-increment', name: 'Increment button' },
            { selector: '.btn-decrement', name: 'Decrement button' },
            { selector: '.btn-reset', name: 'Reset button' },
            { selector: '.stats', name: 'Statistics section' },
            { selector: 'footer', name: 'Footer' }
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

        // Test 4: Verify initial state
        console.log('\n📋 Test 4: Verifying initial counter state...');
        try {
            const counterValue = await page.$eval('.counter-value', el => el.textContent);
            const statusStat = await page.$eval('.stat-value', el => el.textContent);
            
            if (counterValue === '0') {
                console.log('✅ Initial counter value is 0');
                testsPassed++;
            } else {
                console.log(`❌ Expected counter value 0, got ${counterValue}`);
                testsFailed++;
            }
            
            if (statusStat === 'Zero') {
                console.log('✅ Initial status shows "Zero"');
            }
        } catch (error) {
            console.error('❌ Failed to verify initial state:', error.message);
            testsFailed++;
        }

        // Test 5: Test increment functionality
        console.log('\n📋 Test 5: Testing increment button...');
        try {
            await page.click('.btn-increment');
            await page.waitForTimeout(500); // Wait for animation
            
            const newValue = await page.$eval('.counter-value', el => el.textContent);
            if (newValue === '2') {
                console.log('✅ Increment button works correctly');
                testsPassed++;
                await takeScreenshot(page, '02-after-increment');
            } else {
                console.log(`❌ Expected value 2 after increment, got ${newValue}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test increment:', error.message);
            testsFailed++;
        }

        // Test 6: Test decrement functionality
        console.log('\n📋 Test 6: Testing decrement button...');
        try {
            await page.click('.btn-decrement');
            await page.click('.btn-decrement');
            await page.waitForTimeout(500);
            
            const newValue = await page.$eval('.counter-value', el => el.textContent);
            if (newValue === '-2') {
                console.log('✅ Decrement button works correctly');
                testsPassed++;
                await takeScreenshot(page, '03-after-decrement');
            } else {
                console.log(`❌ Expected value -2 after decrement, got ${newValue}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test decrement:', error.message);
            testsFailed++;
        }

        // Test 7: Test quick action buttons
        console.log('\n📋 Test 7: Testing quick action buttons...');
        try {
            // Click +10 button
            await page.click('.quick-btn:nth-child(4)'); // +10 button
            await page.waitForTimeout(300);
            
            let value = await page.$eval('.counter-value', el => el.textContent);
            if (value === '8') {
                console.log('✅ +10 quick button works');
            } else {
                console.log(`❌ Expected 8 after +10, got ${value}`);
            }
            
            // Click -5 button
            await page.click('.quick-btn:nth-child(2)'); // -5 button
            await page.waitForTimeout(300);
            
            value = await page.$eval('.counter-value', el => el.textContent);
            if (value === '3') {
                console.log('✅ -5 quick button works');
                testsPassed++;
                await takeScreenshot(page, '04-after-quick-actions');
            } else {
                console.log(`❌ Expected 3 after -5, got ${value}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test quick actions:', error.message);
            testsFailed++;
        }

        // Test 8: Test reset functionality
        console.log('\n📋 Test 8: Testing reset button...');
        try {
            await page.click('.btn-reset');
            await page.waitForTimeout(600); // Wait for reset animation
            
            const value = await page.$eval('.counter-value', el => el.textContent);
            if (value === '0') {
                console.log('✅ Reset button works correctly');
                testsPassed++;
                await takeScreenshot(page, '05-after-reset');
            } else {
                console.log(`❌ Expected 0 after reset, got ${value}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test reset:', error.message);
            testsFailed++;
        }

        // Test 9: Test statistics update
        console.log('\n📋 Test 9: Testing statistics update...');
        try {
            // Set counter to various values and check stats
            for (let i = 0; i < 5; i++) {
                await page.click('.btn-increment');
            }
            await page.waitForTimeout(500);
            
            const stats = await page.$$eval('.stat-value', elements => 
                elements.map(el => el.textContent)
            );
            
            if (stats[0] === 'Positive' && stats[1] === '10' && stats[2] === '100') {
                console.log('✅ Statistics update correctly');
                console.log(`  Status: ${stats[0]}, Distance: ${stats[1]}, Square: ${stats[2]}`);
                testsPassed++;
            } else {
                console.log('❌ Statistics not updating as expected');
                console.log(`  Got - Status: ${stats[0]}, Distance: ${stats[1]}, Square: ${stats[2]}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test statistics:', error.message);
            testsFailed++;
        }

        // Test 10: Test counter color changes
        console.log('\n📋 Test 10: Testing counter color states...');
        try {
            // Test positive state
            const positiveClass = await page.$eval('.counter-display', el => el.classList.contains('positive'));
            console.log(`  Positive state class: ${positiveClass ? '✅' : '❌'}`);
            
            // Go to zero
            await page.click('.btn-reset');
            await page.waitForTimeout(600);
            const zeroClass = await page.$eval('.counter-display', el => el.classList.contains('zero'));
            console.log(`  Zero state class: ${zeroClass ? '✅' : '❌'}`);
            
            // Go negative
            await page.click('.btn-decrement');
            await page.waitForTimeout(300);
            const negativeClass = await page.$eval('.counter-display', el => el.classList.contains('negative'));
            console.log(`  Negative state class: ${negativeClass ? '✅' : '❌'}`);
            
            if (positiveClass && zeroClass && negativeClass) {
                console.log('✅ Counter color states work correctly');
                testsPassed++;
                await takeScreenshot(page, '06-color-states');
            } else {
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test color states:', error.message);
            testsFailed++;
        }

        // Test 11: Check for console errors
        console.log('\n📋 Test 11: Checking for console errors...');
        if (consoleErrors.length === 0) {
            console.log('✅ No console errors detected');
            testsPassed++;
        } else {
            console.log(`❌ Found ${consoleErrors.length} console errors:`);
            consoleErrors.forEach(error => console.log(`   - ${error}`));
            testsFailed++;
        }

        // Final screenshot
        await takeScreenshot(page, '07-final-state');

    } catch (error) {
        console.error('\n💥 Critical error:', error);
        errors.push(error.message);
    } finally {
        await browser.close();
        
        // Summary
        console.log('\n' + '='.repeat(50));
        console.log('📊 COUNTER TEST SUMMARY');
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
        const response = await fetch(COUNTER_URL);
        return response.ok;
    } catch (error) {
        return false;
    }
}

// Run tests
console.log('🔧 Layer9 Counter - Comprehensive Puppeteer Test Suite');
console.log('=====================================================\n');

checkServer().then(isRunning => {
    if (!isRunning) {
        console.error('❌ HTTP server is not running at', COUNTER_URL);
        console.error('Please start the server first from the project root:');
        console.error('  cd .. && python3 -m http.server 8000');
        process.exit(1);
    }
    return runCounterTests();
}).catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});