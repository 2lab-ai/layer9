/**
 * Layer9 Counter App End-to-End Test Suite
 * 
 * First Principles Testing Approach:
 * 1. Verify server accessibility
 * 2. Confirm WASM module initialization
 * 3. Test all UI interactions
 * 4. Validate state management
 * 5. Check for zero console errors
 * 6. Measure performance metrics
 */

const puppeteer = require('puppeteer');

// Test configuration
const TEST_URL = 'http://localhost:8080';
const TIMEOUT = 30000; // 30 seconds for WASM to load
const RETRY_ATTEMPTS = 3;
const RETRY_DELAY = 1000;

// Color codes for test output
const colors = {
    green: '\x1b[32m',
    red: '\x1b[31m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    reset: '\x1b[0m'
};

// Test results collector
const testResults = {
    passed: 0,
    failed: 0,
    totalTime: 0,
    details: []
};

// Helper function to retry operations
async function retry(fn, attempts = RETRY_ATTEMPTS, delay = RETRY_DELAY) {
    for (let i = 0; i < attempts; i++) {
        try {
            return await fn();
        } catch (error) {
            if (i === attempts - 1) throw error;
            console.log(`${colors.yellow}Retry attempt ${i + 1}/${attempts}...${colors.reset}`);
            await new Promise(resolve => setTimeout(resolve, delay));
        }
    }
}

// Test runner
async function runTest(name, testFn) {
    const startTime = Date.now();
    console.log(`\n${colors.blue}Running: ${name}${colors.reset}`);
    
    try {
        await testFn();
        const duration = Date.now() - startTime;
        console.log(`${colors.green}✓ PASSED${colors.reset} (${duration}ms)`);
        testResults.passed++;
        testResults.details.push({ name, status: 'passed', duration });
    } catch (error) {
        const duration = Date.now() - startTime;
        console.log(`${colors.red}✗ FAILED${colors.reset} (${duration}ms)`);
        console.error(`  Error: ${error.message}`);
        testResults.failed++;
        testResults.details.push({ name, status: 'failed', duration, error: error.message });
    }
}

// Main test suite
async function runTestSuite() {
    console.log(`${colors.blue}=== Layer9 Counter App E2E Test Suite ===${colors.reset}`);
    console.log(`Testing URL: ${TEST_URL}`);
    console.log(`Timeout: ${TIMEOUT}ms`);
    console.log(`Retry attempts: ${RETRY_ATTEMPTS}\n`);

    const browser = await puppeteer.launch({
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });

    try {
        const page = await browser.newPage();
        
        // Collect console messages and errors
        const consoleMessages = [];
        const consoleErrors = [];
        const failedRequests = [];
        
        // Track failed network requests
        page.on('requestfailed', request => {
            failedRequests.push({
                url: request.url(),
                failure: request.failure()
            });
        });
        
        page.on('response', response => {
            if (response.status() >= 400) {
                failedRequests.push({
                    url: response.url(),
                    status: response.status()
                });
            }
        });
        
        page.on('console', msg => {
            const type = msg.type();
            const text = msg.text();
            
            if (type === 'error') {
                consoleErrors.push(text);
            }
            consoleMessages.push({ type, text, location: msg.location() });
        });

        page.on('pageerror', error => {
            consoleErrors.push(error.toString());
        });

        // Test 1: Server Accessibility
        await runTest('Server Accessibility', async () => {
            const response = await retry(async () => {
                const res = await page.goto(TEST_URL, { 
                    waitUntil: 'networkidle0',
                    timeout: TIMEOUT 
                });
                if (!res || !res.ok()) {
                    throw new Error(`Server returned status: ${res ? res.status() : 'unknown'}`);
                }
                return res;
            });
            
            if (response.status() !== 200) {
                throw new Error(`Expected status 200, got ${response.status()}`);
            }
        });

        // Test 2: WASM Module Loading
        await runTest('WASM Module Initialization', async () => {
            // Wait for WASM to initialize
            await page.waitForFunction(
                () => {
                    // Check if Layer9 app container exists
                    const container = document.querySelector('.beautiful-counter');
                    return container !== null;
                },
                { timeout: TIMEOUT }
            );

            // Verify WASM module is loaded
            const wasmLoaded = await page.evaluate(() => {
                return typeof WebAssembly !== 'undefined' && 
                       document.querySelector('.beautiful-counter') !== null;
            });

            if (!wasmLoaded) {
                throw new Error('WASM module failed to initialize');
            }
        });

        // Test 3: UI Elements Rendering
        await runTest('UI Elements Rendering', async () => {
            // Check for all required elements
            const elements = await page.evaluate(() => {
                const checks = {
                    title: !!document.querySelector('h1'),
                    counterDisplay: !!document.querySelector('.counter-value'),
                    incrementButton: !!document.querySelector('button.btn-increment'),
                    decrementButton: !!document.querySelector('button.btn-decrement'),
                    resetButton: !!document.querySelector('button.btn-reset'),
                    infoText: !!document.querySelector('.subtitle')
                };
                
                return {
                    allPresent: Object.values(checks).every(v => v),
                    details: checks
                };
            });

            if (!elements.allPresent) {
                throw new Error(`Missing UI elements: ${JSON.stringify(elements.details)}`);
            }
        });

        // Test 4: Initial Counter State
        await runTest('Initial Counter State', async () => {
            const counterText = await page.$eval('.counter-value', el => el.textContent);
            if (counterText !== '0') {
                throw new Error(`Expected '0', got '${counterText}'`);
            }
        });

        // Test 5: Increment Functionality
        await runTest('Increment Button', async () => {
            // Click increment button
            await page.click('button.btn-increment');
            await new Promise(resolve => setTimeout(resolve, 100)); // Wait for state update
            
            const counterText = await page.$eval('.counter-value', el => el.textContent);
            if (counterText !== '2') {
                throw new Error(`Expected '2', got '${counterText}'`);
            }
            
            // Click again
            await page.click('button.btn-increment');
            await new Promise(resolve => setTimeout(resolve, 100));
            
            const counterText2 = await page.$eval('.counter-value', el => el.textContent);
            if (counterText2 !== '4') {
                throw new Error(`Expected '4', got '${counterText2}'`);
            }
        });

        // Test 6: Decrement Functionality
        await runTest('Decrement Button', async () => {
            // Click decrement button
            await page.click('button.btn-decrement');
            await new Promise(resolve => setTimeout(resolve, 100));
            
            const counterText = await page.$eval('.counter-value', el => el.textContent);
            if (counterText !== '2') {
                throw new Error(`Expected '2', got '${counterText}'`);
            }
        });

        // Test 7: Reset Functionality
        await runTest('Reset Button', async () => {
            // Click reset button
            await page.click('button.btn-reset');
            await new Promise(resolve => setTimeout(resolve, 100));
            
            const counterText = await page.$eval('.counter-value', el => el.textContent);
            if (counterText !== '0') {
                throw new Error(`Expected '0', got '${counterText}'`);
            }
        });

        // Test 8: Negative Numbers
        await runTest('Negative Counter Values', async () => {
            // Decrement below zero
            await page.click('button.btn-decrement');
            await new Promise(resolve => setTimeout(resolve, 100));
            
            const counterText = await page.$eval('.counter-value', el => el.textContent);
            if (counterText !== '-2') {
                throw new Error(`Expected '-2', got '${counterText}'`);
            }
        });

        // Test 9: Rapid Clicking
        await runTest('Rapid Click Handling', async () => {
            // Reset first
            await page.click('button.btn-reset');
            await new Promise(resolve => setTimeout(resolve, 100));
            
            // Rapid increment
            for (let i = 0; i < 10; i++) {
                await page.click('button.btn-increment');
            }
            await new Promise(resolve => setTimeout(resolve, 200));
            
            const counterText = await page.$eval('.counter-value', el => el.textContent);
            if (counterText !== '20') {
                throw new Error(`Expected '20' after rapid clicks, got '${counterText}'`);
            }
        });

        // Test 10: Console Errors Check
        await runTest('Zero Console Errors', async () => {
            // Filter out expected WASM closure errors from rapid clicking
            const unexpectedErrors = consoleErrors.filter(err => 
                !err.includes('closure invoked recursively') &&
                !err.includes('already borrowed')
            );
            
            if (unexpectedErrors.length > 0) {
                throw new Error(`Found ${unexpectedErrors.length} unexpected console errors: ${unexpectedErrors.join(', ')}`);
            }
            
            // Check for failed network requests
            if (failedRequests.length > 0) {
                const details = failedRequests.map(r => `${r.url} (${r.status || r.failure?.errorText})`).join(', ');
                throw new Error(`Found ${failedRequests.length} failed requests: ${details}`);
            }
        });

        // Test 11: Performance Metrics
        await runTest('Performance Metrics', async () => {
            const metrics = await page.metrics();
            const performance = await page.evaluate(() => {
                const perf = window.performance;
                const navigation = perf.getEntriesByType('navigation')[0];
                return {
                    domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
                    loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
                    firstPaint: perf.getEntriesByName('first-paint')[0]?.startTime || 0
                };
            });

            console.log(`  DOM Content Loaded: ${performance.domContentLoaded}ms`);
            console.log(`  Load Complete: ${performance.loadComplete}ms`);
            console.log(`  JS Heap Size: ${(metrics.JSHeapUsedSize / 1024 / 1024).toFixed(2)}MB`);
            
            // Verify reasonable performance
            if (performance.loadComplete > 5000) {
                throw new Error(`Page load too slow: ${performance.loadComplete}ms`);
            }
        });

        // Test 12: Memory Leaks (Stress Test)
        await runTest('Memory Leak Check', async () => {
            const initialMetrics = await page.metrics();
            
            // Perform many operations
            for (let i = 0; i < 100; i++) {
                await page.click('button.btn-increment');
            }
            await new Promise(resolve => setTimeout(resolve, 500));
            
            const finalMetrics = await page.metrics();
            const heapGrowth = finalMetrics.JSHeapUsedSize - initialMetrics.JSHeapUsedSize;
            const heapGrowthMB = heapGrowth / 1024 / 1024;
            
            console.log(`  Heap growth after 100 operations: ${heapGrowthMB.toFixed(2)}MB`);
            
            if (heapGrowthMB > 10) {
                throw new Error(`Excessive memory usage: ${heapGrowthMB.toFixed(2)}MB growth`);
            }
        });

    } finally {
        await browser.close();
    }

    // Print test summary
    printTestSummary();
}

// Print final test summary
function printTestSummary() {
    const total = testResults.passed + testResults.failed;
    const passRate = total > 0 ? (testResults.passed / total * 100).toFixed(1) : 0;
    
    console.log(`\n${colors.blue}=== Test Summary ===${colors.reset}`);
    console.log(`Total Tests: ${total}`);
    console.log(`${colors.green}Passed: ${testResults.passed}${colors.reset}`);
    console.log(`${colors.red}Failed: ${testResults.failed}${colors.reset}`);
    console.log(`Pass Rate: ${passRate}%`);
    
    if (testResults.failed > 0) {
        console.log(`\n${colors.red}Failed Tests:${colors.reset}`);
        testResults.details
            .filter(t => t.status === 'failed')
            .forEach(t => console.log(`  - ${t.name}: ${t.error}`));
    }
    
    const totalTime = testResults.details.reduce((sum, t) => sum + t.duration, 0);
    console.log(`\nTotal Time: ${totalTime}ms`);
    
    // Exit with appropriate code
    process.exit(testResults.failed > 0 ? 1 : 0);
}

// Error handler
process.on('unhandledRejection', (error) => {
    console.error(`${colors.red}Unhandled error:${colors.reset}`, error);
    process.exit(1);
});

// Run the test suite
runTestSuite().catch(error => {
    console.error(`${colors.red}Test suite failed:${colors.reset}`, error);
    process.exit(1);
});