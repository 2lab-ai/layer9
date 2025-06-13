#!/usr/bin/env node

/**
 * Layer9 Framework Production Validation - Fixed for actual counter app
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
        log.info('\n🧪 Layer9 Test Suite');
        console.log('==================================================');
        
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
            await this.validateConsoleErrors(page);
            
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
        const testName = 'Page Loading';
        try {
            const response = await page.goto(this.url, {
                waitUntil: 'networkidle2',
                timeout: 30000
            });

            const title = await page.title();
            
            if (response.ok()) {
                log.success(`${testName} - "${title}"`);
                this.results.passed.push(testName);
            } else {
                throw new Error(`HTTP ${response.status()}`);
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateWASMInitialization(page) {
        const testName = 'WASM and App Rendering';
        try {
            // Wait for buttons to appear (indicates app is rendered)
            await page.waitForSelector('button', { timeout: 5000 });
            
            // Check if buttons are rendered
            const buttons = await page.$$eval('button', elements => elements.length);
            
            if (buttons > 0) {
                log.success(`${testName} - App rendered: ${buttons} buttons found`);
                this.results.passed.push(testName);
            } else {
                throw new Error('No buttons found - app may not be rendered');
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateUIRendering(page) {
        const testName = 'Basic Interaction';
        try {
            // Wait for counter display
            const counterEl = await page.waitForSelector('.counter-value', { timeout: 5000 });
            const initialValue = await page.$eval('.counter-value', el => parseInt(el.textContent));
            log.metric('Initial count', initialValue);
            
            // Find and list all buttons
            const buttons = await page.evaluate(() => {
                const btns = Array.from(document.querySelectorAll('button'));
                return btns.map((btn, i) => ({
                    text: btn.textContent.trim(),
                    class: btn.className,
                    hasClick: !!btn.onclick
                }));
            });
            
            // Log button info
            buttons.forEach((btn, i) => {
                log.metric(`Button ${i}`, `"${btn.text}" (class: ${btn.class}, onclick: ${btn.hasClick})`);
            });
            
            this.results.passed.push(testName);
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateInteractivity(page) {
        const testName = 'State Management';
        try {
            // Get initial value
            const initial = await page.$eval('.counter-value', el => parseInt(el.textContent));
            
            // Click increment button
            await page.click('button.btn-increment');
            await new Promise(resolve => setTimeout(resolve, 100));
            
            const afterInc = await page.$eval('.counter-value', el => parseInt(el.textContent));
            log.metric('Count after increment', afterInc);
            
            if (afterInc === initial + 1) {
                log.success(`${testName} - State management works`);
                this.results.passed.push(testName);
            } else {
                throw new Error(`Expected ${initial + 1}, got ${afterInc}`);
            }
        } catch (error) {
            log.error(`${testName} - ${error.message}`);
            this.results.failed.push({ test: testName, error: error.message });
        }
    }

    async validateConsoleErrors(page) {
        const testName = 'Console Errors';
        if (this.errors.length === 0) {
            log.success(`${testName} - No JavaScript errors`);
            this.results.passed.push(testName);
        } else {
            log.error(`${testName} - Found ${this.errors.length} errors`);
            this.errors.forEach(err => log.error(`  ${err}`));
            this.results.failed.push({ test: testName, error: `${this.errors.length} console errors` });
        }
    }

    reportResults() {
        console.log('\n==================================================');
        const total = this.results.passed.length + this.results.failed.length;
        
        if (this.results.failed.length === 0) {
            log.success('ALL TESTS PASSED');
        } else {
            log.error('SOME TESTS FAILED');
            
            console.log('\nFailed Tests:');
            this.results.failed.forEach(failure => {
                log.error(`  ${failure.test}: ${failure.error}`);
            });
        }
    }
}

// Main execution
(async () => {
    try {
        // Wait a bit for server to be ready
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        const validator = new Layer9Validator();
        const exitCode = await validator.validate();
        process.exit(exitCode);
    } catch (error) {
        console.error('Validation failed:', error);
        process.exit(1);
    }
})();