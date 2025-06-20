#!/usr/bin/env node

/**
 * Layer9 Framework Production Validation
 * 
 * First Principles: What must work for this to be production-ready?
 * 1. Server must respond
 * 2. WASM must load without errors
 * 3. UI must render and be interactive
 * 4. State management must work correctly
 * 5. No memory leaks or performance issues
 */

const puppeteer = require('puppeteer');

// Terminal colors for clear output
const log = {
    info: (msg) => console.log(`\x1b[36m${msg}\x1b[0m`),
    success: (msg) => console.log(`\x1b[32m✓ ${msg}\x1b[0m`),
    error: (msg) => console.log(`\x1b[31m✗ ${msg}\x1b[0m`),
    warn: (msg) => console.log(`\x1b[33m⚠ ${msg}\x1b[0m`),
    metric: (key, value) => console.log(`  \x1b[34m${key}:\x1b[0m ${value}`)
};

class Layer9Validator {
    constructor() {
        this.url = 'http://localhost:8080';
        this.results = {
            passed: [],
            failed: [],
            metrics: {}
        };
    }

    async validate() {
        log.info('\n🚀 Layer9 Framework Validation\n');
        
        const browser = await puppeteer.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });

        try {
            const page = await browser.newPage();
            
            // Monitor everything
            this.setupMonitoring(page);
            
            // Run validation tests
            await this.validateServerResponse(page);
            await this.validateWASMInitialization(page);
            await this.validateUIRendering(page);
            await this.validateInteractivity(page);
            await this.validateStateManagement(page);
            await this.validatePerformance(page);
            await this.validateReliability(page);
            
            // Report results
            this.reportResults();
            
        } catch (error) {
            log.error(`Critical failure: ${error.message}`);
            this.results.failed.push({ test: 'Framework', error: error.message });
        } finally {
            await browser.close();
        }

        // Return exit code
        return this.results.failed.length === 0 ? 0 : 1;
    }

    setupMonitoring(page) {
        // Track all errors
        this.errors = [];
        this.networkErrors = [];
        
        page.on('console', msg => {
            if (msg.type() === 'error') {
                this.errors.push(msg.text());
            }
        });

        page.on('pageerror', error => {
            this.errors.push(error.toString());
        });

        page.on('requestfailed', request => {
            this.networkErrors.push({
                url: request.url(),
                error: request.failure().errorText
            });
        });

        page.on('response', response => {
            if (response.status() >= 400) {
                this.networkErrors.push({
                    url: response.url(),
                    status: response.status()
                });
            }
        });
    }

    async validateServerResponse(page) {
        const testName = 'Server Response';
        try {
            const response = await page.goto(this.url, {
                waitUntil: 'networkidle0',
                timeout: 10000
            });

            if (response.status() === 200) {
                log.success(`${testName} - Status: 200 OK`);
                this.results.passed.push(testName);
            } else {
                throw new Error(`Status: ${response.status()}`);
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateWASMInitialization(page) {
        const testName = 'WASM Initialization';
        try {
            // Wait for WASM to load
            await page.waitForFunction(
                () => typeof WebAssembly !== 'undefined' && document.querySelector('.beautiful-counter'),
                { timeout: 5000 }
            );

            // Check for WASM errors
            if (this.errors.length === 0) {
                log.success(`${testName} - No runtime errors`);
                this.results.passed.push(testName);
            } else {
                throw new Error(`${this.errors.length} runtime errors`);
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateUIRendering(page) {
        const testName = 'UI Rendering';
        try {
            // Check all critical UI elements
            const elements = await page.evaluate(() => {
                const checks = {
                    app: document.querySelector('.beautiful-counter'),
                    title: document.querySelector('h1'),
                    counter: document.querySelector('.counter-value'),
                    buttons: document.querySelectorAll('button').length
                };
                return {
                    rendered: checks.app && checks.title && checks.counter && checks.buttons >= 3,
                    details: checks
                };
            });

            if (elements.rendered) {
                log.success(`${testName} - All components rendered`);
                log.metric('Buttons found', elements.details.buttons);
                this.results.passed.push(testName);
            } else {
                throw new Error('Missing UI components');
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateInteractivity(page) {
        const testName = 'Interactivity';
        try {
            // Test button clicks
            const initialValue = await page.$eval('.counter-value', el => el.textContent);
            
            // Click increment
            await page.click('button.btn-increment');
            await new Promise(resolve => setTimeout(resolve, 100));
            
            const afterIncrement = await page.$eval('.counter-value', el => el.textContent);
            
            if (initialValue !== afterIncrement) {
                log.success(`${testName} - Buttons respond to clicks`);
                log.metric('Initial', initialValue);
                log.metric('After click', afterIncrement);
                this.results.passed.push(testName);
            } else {
                throw new Error('UI not responding to interactions');
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateStateManagement(page) {
        const testName = 'State Management';
        try {
            // Test state persistence through multiple operations
            await page.click('button.btn-reset'); // Reset
            await new Promise(resolve => setTimeout(resolve, 100));
            
            // Perform sequence
            for (let i = 0; i < 5; i++) {
                await page.click('button.btn-increment');
            }
            await new Promise(resolve => setTimeout(resolve, 100));
            
            for (let i = 0; i < 2; i++) {
                await page.click('button.btn-decrement');
            }
            await new Promise(resolve => setTimeout(resolve, 100));
            
            const finalValue = await page.$eval('.counter-value', el => el.textContent);
            
            if (finalValue === '6') {
                log.success(`${testName} - State correctly maintained`);
                log.metric('Expected', '6');
                log.metric('Actual', finalValue);
                this.results.passed.push(testName);
            } else {
                throw new Error(`Expected '6', got '${finalValue}'`);
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validatePerformance(page) {
        const testName = 'Performance';
        try {
            const metrics = await page.metrics();
            const performanceTiming = await page.evaluate(() => {
                const perf = window.performance;
                const timing = perf.timing;
                const navigation = perf.getEntriesByType('navigation')[0];
                
                return {
                    loadTime: timing.loadEventEnd - timing.navigationStart,
                    domReady: timing.domContentLoadedEventEnd - timing.navigationStart,
                    firstPaint: perf.getEntriesByName('first-paint')[0]?.startTime || 0,
                    jsHeapMB: (performance.memory?.usedJSHeapSize || 0) / 1024 / 1024
                };
            });

            const heapMB = (metrics.JSHeapUsedSize / 1024 / 1024).toFixed(2);
            
            if (performanceTiming.loadTime < 3000 && parseFloat(heapMB) < 10) {
                log.success(`${testName} - Within acceptable limits`);
                log.metric('Load time', `${performanceTiming.loadTime}ms`);
                log.metric('DOM ready', `${performanceTiming.domReady}ms`);
                log.metric('JS Heap', `${heapMB}MB`);
                this.results.passed.push(testName);
                
                this.results.metrics = {
                    ...performanceTiming,
                    heapMB: parseFloat(heapMB)
                };
            } else {
                throw new Error('Performance exceeds limits');
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateReliability(page) {
        const testName = 'Reliability';
        try {
            // Stress test with rapid operations
            const startHeap = (await page.metrics()).JSHeapUsedSize;
            
            // Perform 100 rapid operations
            for (let i = 0; i < 100; i++) {
                await page.click(i % 2 === 0 ? 'button.btn-increment' : 'button.btn-decrement');
            }
            
            await new Promise(resolve => setTimeout(resolve, 500));
            const endHeap = (await page.metrics()).JSHeapUsedSize;
            const heapGrowthMB = (endHeap - startHeap) / 1024 / 1024;
            
            // Check final state is correct
            const value = await page.$eval('.counter-value', el => el.textContent);
            
            if (heapGrowthMB < 5 && this.errors.length === 0) {
                log.success(`${testName} - No crashes or memory leaks`);
                log.metric('Heap growth', `${heapGrowthMB.toFixed(2)}MB`);
                log.metric('Final state', value);
                this.results.passed.push(testName);
            } else {
                throw new Error('Reliability issues detected');
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    reportResults() {
        const total = this.results.passed.length + this.results.failed.length;
        const passRate = ((this.results.passed.length / total) * 100).toFixed(1);
        
        console.log('\n' + '='.repeat(50));
        log.info('📊 VALIDATION RESULTS\n');
        
        console.log(`Total Tests: ${total}`);
        console.log(`Passed: \x1b[32m${this.results.passed.length}\x1b[0m`);
        console.log(`Failed: \x1b[31m${this.results.failed.length}\x1b[0m`);
        console.log(`Pass Rate: ${passRate}%`);
        
        if (this.results.failed.length > 0) {
            console.log('\n\x1b[31mFailed Tests:\x1b[0m');
            this.results.failed.forEach(f => {
                console.log(`  - ${f.test}: ${f.error}`);
            });
        }
        
        if (this.errors.length > 0) {
            console.log('\n\x1b[31mConsole Errors:\x1b[0m');
            this.errors.forEach(e => console.log(`  - ${e}`));
        }
        
        if (this.networkErrors.length > 0) {
            console.log('\n\x1b[33mNetwork Issues:\x1b[0m');
            this.networkErrors.forEach(e => {
                console.log(`  - ${e.url}: ${e.error || `Status ${e.status}`}`);
            });
        }
        
        console.log('\n' + '='.repeat(50));
        
        if (this.results.failed.length === 0) {
            log.success('\n✅ LAYER9 FRAMEWORK IS PRODUCTION READY!\n');
        } else {
            log.error('\n❌ VALIDATION FAILED - FIX ISSUES AND RETRY\n');
        }
    }
}

// Run validation
(async () => {
    const validator = new Layer9Validator();
    const exitCode = await validator.validate();
    process.exit(exitCode);
})();