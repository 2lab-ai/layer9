const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// Configuration
const ASYNC_COUNTER_URL = 'http://localhost:8000/examples/async-counter/';
const SCREENSHOT_DIR = path.join(__dirname, '../screenshots/async-counter');
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
async function runAsyncCounterTests() {
    console.log('\n🚀 Starting Layer9 Async Counter Tests\n');
    
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

        // Test 1: Navigate to async counter page
        console.log('📋 Test 1: Loading the async counter application...');
        try {
            await page.goto(ASYNC_COUNTER_URL, { waitUntil: 'networkidle2', timeout: TIMEOUT });
            console.log('✅ Successfully loaded the async counter app');
            testsPassed++;
        } catch (error) {
            console.error('❌ Failed to load the async counter app:', error.message);
            testsFailed++;
            throw error;
        }

        // Test 2: Verify WASM loads and initial loading state
        console.log('\n📋 Test 2: Verifying WASM initialization and loading state...');
        try {
            // Check for loading spinner initially
            const hasLoadingSpinner = await waitForElement(page, '.loading-spinner', { timeout: 5000 });
            if (hasLoadingSpinner) {
                console.log('✅ Loading spinner displayed during initialization');
                await takeScreenshot(page, '01-loading-state');
            }
            
            // Wait for loading to complete (should show initial value of 42)
            await page.waitForFunction(
                () => document.querySelector('.counter-value')?.textContent === '42',
                { timeout: TIMEOUT }
            );
            
            console.log('✅ WASM loaded successfully with initial value from "server"');
            testsPassed++;
            await takeScreenshot(page, '02-initial-load-complete');
        } catch (error) {
            console.error('❌ WASM loading failed:', error.message);
            testsFailed++;
        }

        // Test 3: Verify all main components render
        console.log('\n📋 Test 3: Verifying all async counter components...');
        const components = [
            { selector: 'h1', name: 'Title with Async branding' },
            { selector: '.subtitle', name: 'Subtitle' },
            { selector: '.counter-display', name: 'Counter display' },
            { selector: '.counter-value', name: 'Counter value' },
            { selector: '.message-display', name: 'Message display' },
            { selector: '.sync-controls', name: 'Sync control buttons' },
            { selector: '.async-controls', name: 'Async control buttons' },
            { selector: '.btn-increment', name: 'Increment button' },
            { selector: '.btn-decrement', name: 'Decrement button' },
            { selector: '.btn-async:nth-child(1)', name: 'Async Reset button' },
            { selector: '.btn-async:nth-child(2)', name: 'Random button' },
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

        // Test 4: Verify initial message
        console.log('\n📋 Test 4: Verifying initial message...');
        try {
            const message = await page.$eval('.message', el => el.textContent);
            if (message.includes('Initial count loaded from server!') || message.includes('The answer to everything!')) {
                console.log(`✅ Initial message displayed: "${message}"`);
                testsPassed++;
            } else {
                console.log(`❌ Unexpected initial message: "${message}"`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to verify initial message:', error.message);
            testsFailed++;
        }

        // Test 5: Test sync increment/decrement
        console.log('\n📋 Test 5: Testing sync increment/decrement buttons...');
        try {
            // Increment
            await page.click('.btn-increment');
            await page.waitForTimeout(300);
            let value = await page.$eval('.counter-value', el => el.textContent);
            if (value === '43') {
                console.log('✅ Sync increment works');
            }
            
            // Decrement twice
            await page.click('.btn-decrement');
            await page.click('.btn-decrement');
            await page.waitForTimeout(300);
            value = await page.$eval('.counter-value', el => el.textContent);
            if (value === '41') {
                console.log('✅ Sync decrement works');
                testsPassed++;
                await takeScreenshot(page, '03-after-sync-operations');
            } else {
                console.log(`❌ Expected 41, got ${value}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test sync buttons:', error.message);
            testsFailed++;
        }

        // Test 6: Test message updates based on count
        console.log('\n📋 Test 6: Testing dynamic message updates...');
        try {
            // Get to zero
            for (let i = 0; i < 41; i++) {
                await page.click('.btn-decrement');
            }
            await page.waitForTimeout(500);
            
            let message = await page.$eval('.message', el => el.textContent);
            if (message.includes('Zero')) {
                console.log('✅ Zero message displayed correctly');
            }
            
            // Go negative
            await page.click('.btn-decrement');
            await page.waitForTimeout(300);
            message = await page.$eval('.message', el => el.textContent);
            if (message.includes('negative')) {
                console.log('✅ Negative message displayed correctly');
            }
            
            // Get to a round number
            for (let i = 0; i < 11; i++) {
                await page.click('.btn-increment');
            }
            await page.waitForTimeout(300);
            message = await page.$eval('.message', el => el.textContent);
            if (message.includes('Perfect ten') || message.includes('round number')) {
                console.log('✅ Round number message displayed correctly');
                testsPassed++;
                await takeScreenshot(page, '04-message-updates');
            } else {
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test message updates:', error.message);
            testsFailed++;
        }

        // Test 7: Test async reset functionality
        console.log('\n📋 Test 7: Testing async reset button...');
        try {
            await page.click('.btn-async:nth-child(1)'); // Async Reset
            
            // Check loading state
            const hasLoadingClass = await page.$eval('.counter-display', el => el.classList.contains('loading'));
            if (hasLoadingClass) {
                console.log('✅ Loading state activated during async reset');
            }
            
            // Check loading message
            const loadingMessage = await page.$eval('.message', el => el.textContent);
            if (loadingMessage.includes('Resetting')) {
                console.log('✅ Loading message displayed during reset');
            }
            
            // Wait for reset to complete
            await page.waitForFunction(
                () => {
                    const msg = document.querySelector('.message')?.textContent || '';
                    return msg.includes('Reset complete') || msg.includes('Fresh start');
                },
                { timeout: 5000 }
            );
            
            const resetValue = await page.$eval('.counter-value', el => el.textContent);
            if (resetValue === '0') {
                console.log('✅ Async reset completed successfully');
                testsPassed++;
                await takeScreenshot(page, '05-after-async-reset');
            } else {
                console.log(`❌ Expected 0 after reset, got ${resetValue}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test async reset:', error.message);
            testsFailed++;
        }

        // Test 8: Test random number functionality
        console.log('\n📋 Test 8: Testing random number button...');
        try {
            await page.click('.btn-async:nth-child(2)'); // Random button
            
            // Check loading message
            const loadingMessage = await page.$eval('.message', el => el.textContent);
            if (loadingMessage.includes('Fetching random')) {
                console.log('✅ Loading message displayed during random fetch');
            }
            
            // Wait for random number to load
            await page.waitForFunction(
                () => {
                    const msg = document.querySelector('.message')?.textContent || '';
                    return msg.includes('Random number') && msg.includes('loaded');
                },
                { timeout: 5000 }
            );
            
            const randomValue = parseInt(await page.$eval('.counter-value', el => el.textContent));
            if (randomValue >= 0 && randomValue < 100) {
                console.log(`✅ Random number loaded: ${randomValue}`);
                testsPassed++;
                await takeScreenshot(page, '06-after-random');
            } else {
                console.log(`❌ Random value out of expected range: ${randomValue}`);
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test random number:', error.message);
            testsFailed++;
        }

        // Test 9: Test button disabled state during async operations
        console.log('\n📋 Test 9: Testing button disabled states...');
        try {
            // Click async reset and immediately check button states
            await page.click('.btn-async:nth-child(1)');
            
            const resetBtnDisabled = await page.$eval('.btn-async:nth-child(1)', el => el.classList.contains('disabled'));
            const randomBtnDisabled = await page.$eval('.btn-async:nth-child(2)', el => el.classList.contains('disabled'));
            
            if (resetBtnDisabled && randomBtnDisabled) {
                console.log('✅ Async buttons disabled during operation');
                testsPassed++;
            } else {
                console.log('❌ Async buttons not properly disabled');
                testsFailed++;
            }
            
            // Wait for operation to complete
            await page.waitForTimeout(1500);
        } catch (error) {
            console.error('❌ Failed to test disabled states:', error.message);
            testsFailed++;
        }

        // Test 10: Test animations and transitions
        console.log('\n📋 Test 10: Testing animations...');
        try {
            // The counter value should have fadeIn animation
            const hasAnimation = await page.evaluate(() => {
                const counterValue = document.querySelector('.counter-value');
                const styles = window.getComputedStyle(counterValue);
                return styles.animation.includes('fadeIn');
            });
            
            if (hasAnimation) {
                console.log('✅ Counter animations are applied');
                testsPassed++;
            } else {
                console.log('❌ Counter animations not found');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test animations:', error.message);
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
        console.log('📊 ASYNC COUNTER TEST SUMMARY');
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
        const response = await fetch(ASYNC_COUNTER_URL);
        return response.ok;
    } catch (error) {
        return false;
    }
}

// Run tests
console.log('🔧 Layer9 Async Counter - Comprehensive Puppeteer Test Suite');
console.log('===========================================================\n');

checkServer().then(isRunning => {
    if (!isRunning) {
        console.error('❌ HTTP server is not running at', ASYNC_COUNTER_URL);
        console.error('Please start the server first from the project root:');
        console.error('  cd .. && python3 -m http.server 8000');
        process.exit(1);
    }
    return runAsyncCounterTests();
}).catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});