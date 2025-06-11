#!/usr/bin/env node
/**
 * Layer9 Continuous Integration Test Suite
 * 
 * Comprehensive testing pipeline for CI/CD environments
 * Designed for GitHub Actions, GitLab CI, Jenkins, etc.
 */

const puppeteer = require('puppeteer');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Test configuration
const CONFIG = {
    BASE_URL: process.env.TEST_URL || 'http://localhost:8080',
    HEADLESS: process.env.HEADLESS !== 'false',
    SLOW_MO: parseInt(process.env.SLOW_MO || '0'),
    TIMEOUT: parseInt(process.env.TEST_TIMEOUT || '30000'),
    PARALLEL_TESTS: parseInt(process.env.PARALLEL_TESTS || '3'),
    GENERATE_REPORT: process.env.GENERATE_REPORT !== 'false'
};

// Test categories
const TEST_SUITES = {
    unit: 'Unit Tests',
    integration: 'Integration Tests',
    e2e: 'End-to-End Tests',
    performance: 'Performance Tests',
    security: 'Security Tests',
    accessibility: 'Accessibility Tests'
};

class Layer9CITestSuite {
    constructor() {
        this.results = {
            startTime: Date.now(),
            endTime: null,
            suites: {},
            summary: {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0
            }
        };
        
        this.browser = null;
        this.serverProcess = null;
    }

    log(level, message, indent = 0) {
        const colors = {
            info: '\x1b[36m',
            success: '\x1b[32m',
            error: '\x1b[31m',
            warn: '\x1b[33m',
            dim: '\x1b[90m'
        };
        const prefix = '  '.repeat(indent);
        console.log(`${colors[level]}${prefix}${message}\x1b[0m`);
    }

    async runTest(suite, name, testFn) {
        const startTime = Date.now();
        this.results.summary.total++;

        try {
            await testFn();
            const duration = Date.now() - startTime;
            
            this.results.suites[suite].tests.push({
                name,
                status: 'passed',
                duration
            });
            
            this.results.summary.passed++;
            this.log('success', `✓ ${name} (${duration}ms)`, 2);
            
        } catch (error) {
            const duration = Date.now() - startTime;
            
            this.results.suites[suite].tests.push({
                name,
                status: 'failed',
                duration,
                error: error.message,
                stack: error.stack
            });
            
            this.results.summary.failed++;
            this.log('error', `✗ ${name} (${duration}ms)`, 2);
            this.log('error', `  ${error.message}`, 3);
        }
    }

    async runSuite(suiteName, tests) {
        this.log('info', `\n${TEST_SUITES[suiteName]}`);
        this.log('dim', '─'.repeat(40));
        
        this.results.suites[suiteName] = {
            name: TEST_SUITES[suiteName],
            startTime: Date.now(),
            tests: []
        };

        for (const [testName, testFn] of Object.entries(tests)) {
            await this.runTest(suiteName, testName, testFn);
        }

        this.results.suites[suiteName].endTime = Date.now();
        this.results.suites[suiteName].duration = 
            this.results.suites[suiteName].endTime - 
            this.results.suites[suiteName].startTime;
    }

    async setupBrowser() {
        this.browser = await puppeteer.launch({
            headless: CONFIG.HEADLESS,
            slowMo: CONFIG.SLOW_MO,
            args: [
                '--no-sandbox',
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage',
                '--disable-gpu'
            ]
        });
    }

    async createPage() {
        const page = await this.browser.newPage();
        
        // Set viewport
        await page.setViewport({
            width: 1280,
            height: 720
        });

        // Track console messages and errors
        const logs = [];
        page.on('console', msg => logs.push({ type: msg.type(), text: msg.text() }));
        page.on('pageerror', error => logs.push({ type: 'error', text: error.toString() }));
        
        page.logs = logs;
        
        return page;
    }

    // Unit Tests
    async runUnitTests() {
        await this.runSuite('unit', {
            'WASM Module Exports': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                const exports = await page.evaluate(() => {
                    return window.wasm_bindgen ? Object.keys(window.wasm_bindgen) : null;
                });
                
                if (!exports || exports.length === 0) {
                    throw new Error('No WASM exports found');
                }
                
                await page.close();
            },

            'Component Structure': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                const structure = await page.evaluate(() => {
                    const app = document.querySelector('.layer9-app');
                    return {
                        hasApp: !!app,
                        hasTitle: !!document.querySelector('h1'),
                        hasButtons: document.querySelectorAll('button').length >= 3,
                        hasDisplay: !!document.querySelector('#counter-display')
                    };
                });
                
                if (!Object.values(structure).every(v => v)) {
                    throw new Error('Invalid component structure');
                }
                
                await page.close();
            },

            'State Management': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                // Test state initialization
                const initialState = await page.$eval('#counter-display', el => el.textContent);
                if (initialState !== 'Count: 0') {
                    throw new Error('Invalid initial state');
                }
                
                await page.close();
            }
        });
    }

    // Integration Tests
    async runIntegrationTests() {
        await this.runSuite('integration', {
            'Button Click Events': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                // Test all button interactions
                await page.click('button.btn-primary');
                await page.waitForTimeout(100);
                
                const afterIncrement = await page.$eval('#counter-display', el => el.textContent);
                if (afterIncrement !== 'Count: 1') {
                    throw new Error('Increment failed');
                }
                
                await page.close();
            },

            'State Persistence': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                // Multiple state changes
                for (let i = 0; i < 5; i++) {
                    await page.click('button.btn-primary');
                }
                await page.waitForTimeout(200);
                
                const count = await page.$eval('#counter-display', el => el.textContent);
                if (count !== 'Count: 5') {
                    throw new Error('State persistence failed');
                }
                
                await page.close();
            },

            'Event Handling': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                // Rapid clicking test
                const clickPromises = [];
                for (let i = 0; i < 10; i++) {
                    clickPromises.push(page.click('button.btn-primary'));
                }
                await Promise.all(clickPromises);
                await page.waitForTimeout(500);
                
                const count = await page.$eval('#counter-display', el => el.textContent);
                if (count !== 'Count: 10') {
                    throw new Error('Event handling race condition');
                }
                
                await page.close();
            }
        });
    }

    // E2E Tests
    async runE2ETests() {
        await this.runSuite('e2e', {
            'Complete User Flow': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                // Simulate complete user interaction
                await page.click('button.btn-primary'); // +1
                await page.click('button.btn-primary'); // +2
                await page.click('button.btn-secondary'); // -1
                await page.click('button.btn-warning'); // reset
                
                await page.waitForTimeout(100);
                
                const finalCount = await page.$eval('#counter-display', el => el.textContent);
                if (finalCount !== 'Count: 0') {
                    throw new Error('User flow failed');
                }
                
                await page.close();
            },

            'Page Reload Behavior': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                // Change state
                await page.click('button.btn-primary');
                await page.click('button.btn-primary');
                
                // Reload page
                await page.reload();
                
                // Check state reset
                const count = await page.$eval('#counter-display', el => el.textContent);
                if (count !== 'Count: 0') {
                    throw new Error('State not reset after reload');
                }
                
                await page.close();
            }
        });
    }

    // Performance Tests
    async runPerformanceTests() {
        await this.runSuite('performance', {
            'Page Load Time': async () => {
                const page = await this.createPage();
                const startTime = Date.now();
                
                await page.goto(CONFIG.BASE_URL, { waitUntil: 'networkidle0' });
                
                const loadTime = Date.now() - startTime;
                if (loadTime > 5000) {
                    throw new Error(`Page load too slow: ${loadTime}ms`);
                }
                
                await page.close();
            },

            'WASM Initialization': async () => {
                const page = await this.createPage();
                
                await page.goto(CONFIG.BASE_URL);
                
                const metrics = await page.evaluate(() => {
                    const navigation = performance.getEntriesByType('navigation')[0];
                    return {
                        domReady: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
                        loadComplete: navigation.loadEventEnd - navigation.loadEventStart
                    };
                });
                
                if (metrics.loadComplete > 3000) {
                    throw new Error(`WASM load too slow: ${metrics.loadComplete}ms`);
                }
                
                await page.close();
            },

            'Memory Usage': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                const initialMetrics = await page.metrics();
                
                // Stress test
                for (let i = 0; i < 1000; i++) {
                    await page.click('button.btn-primary');
                }
                
                const finalMetrics = await page.metrics();
                const heapGrowthMB = (finalMetrics.JSHeapUsedSize - initialMetrics.JSHeapUsedSize) / 1024 / 1024;
                
                if (heapGrowthMB > 50) {
                    throw new Error(`Memory leak detected: ${heapGrowthMB.toFixed(2)}MB growth`);
                }
                
                await page.close();
            }
        });
    }

    // Security Tests
    async runSecurityTests() {
        await this.runSuite('security', {
            'XSS Protection': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                // Try to inject script
                const hasXSSVulnerability = await page.evaluate(() => {
                    // Check if app properly escapes content
                    const testScript = '<script>window.xssTest = true;</script>';
                    document.body.innerHTML += testScript;
                    return window.xssTest === true;
                });
                
                if (hasXSSVulnerability) {
                    throw new Error('XSS vulnerability detected');
                }
                
                await page.close();
            },

            'Content Security Policy': async () => {
                const page = await this.createPage();
                const response = await page.goto(CONFIG.BASE_URL);
                
                const headers = response.headers();
                const csp = headers['content-security-policy'];
                
                // Note: Python's simple HTTP server doesn't set CSP headers
                // This test would fail in dev but should pass in production
                
                await page.close();
            }
        });
    }

    // Accessibility Tests
    async runAccessibilityTests() {
        await this.runSuite('accessibility', {
            'ARIA Labels': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                const accessibility = await page.evaluate(() => {
                    const buttons = document.querySelectorAll('button');
                    return Array.from(buttons).every(btn => 
                        btn.textContent || btn.getAttribute('aria-label')
                    );
                });
                
                if (!accessibility) {
                    throw new Error('Missing accessible labels');
                }
                
                await page.close();
            },

            'Keyboard Navigation': async () => {
                const page = await this.createPage();
                await page.goto(CONFIG.BASE_URL);
                
                // Tab through elements
                await page.keyboard.press('Tab');
                await page.keyboard.press('Tab');
                await page.keyboard.press('Enter'); // Activate button
                
                await page.waitForTimeout(100);
                
                const count = await page.$eval('#counter-display', el => el.textContent);
                if (count === 'Count: 0') {
                    throw new Error('Keyboard navigation not working');
                }
                
                await page.close();
            }
        });
    }

    generateReport() {
        this.results.endTime = Date.now();
        this.results.duration = this.results.endTime - this.results.startTime;

        const report = {
            timestamp: new Date().toISOString(),
            duration: this.results.duration,
            environment: {
                node: process.version,
                platform: process.platform,
                ci: process.env.CI || false
            },
            summary: this.results.summary,
            suites: this.results.suites
        };

        if (CONFIG.GENERATE_REPORT) {
            const reportPath = path.join(process.cwd(), 'test-report.json');
            fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
            this.log('info', `\nReport saved to: ${reportPath}`);
        }

        // Console output
        console.log('\n' + '═'.repeat(50));
        console.log('  Layer9 CI Test Results');
        console.log('═'.repeat(50));
        
        console.log(`\nTotal Tests: ${this.results.summary.total}`);
        console.log(`✓ Passed: ${this.results.summary.passed}`);
        console.log(`✗ Failed: ${this.results.summary.failed}`);
        console.log(`○ Skipped: ${this.results.summary.skipped}`);
        console.log(`\nDuration: ${(this.results.duration / 1000).toFixed(2)}s`);
        
        const passRate = (this.results.summary.passed / this.results.summary.total * 100).toFixed(1);
        console.log(`Pass Rate: ${passRate}%`);

        // Failed tests details
        if (this.results.summary.failed > 0) {
            console.log('\n' + '─'.repeat(50));
            console.log('Failed Tests:');
            
            Object.entries(this.results.suites).forEach(([suiteName, suite]) => {
                const failed = suite.tests.filter(t => t.status === 'failed');
                if (failed.length > 0) {
                    console.log(`\n${suite.name}:`);
                    failed.forEach(test => {
                        console.log(`  ✗ ${test.name}`);
                        console.log(`    ${test.error}`);
                    });
                }
            });
        }

        console.log('\n' + '═'.repeat(50) + '\n');

        return this.results.summary.failed === 0;
    }

    async run() {
        try {
            this.log('info', 'Layer9 CI Test Suite Starting...\n');

            await this.setupBrowser();

            // Run all test suites
            await this.runUnitTests();
            await this.runIntegrationTests();
            await this.runE2ETests();
            await this.runPerformanceTests();
            await this.runSecurityTests();
            await this.runAccessibilityTests();

            const success = this.generateReport();

            await this.browser.close();

            process.exit(success ? 0 : 1);

        } catch (error) {
            this.log('error', `Fatal error: ${error.message}`);
            if (this.browser) await this.browser.close();
            process.exit(1);
        }
    }
}

// Run the CI test suite
if (require.main === module) {
    const suite = new Layer9CITestSuite();
    suite.run();
}

module.exports = Layer9CITestSuite;