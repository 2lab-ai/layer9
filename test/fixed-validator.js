#!/usr/bin/env node
/**
 * Layer9 Fixed Validator - Handles reactive DOM updates properly
 */

const puppeteer = require('puppeteer');
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

class FixedValidator {
    constructor() {
        this.serverProcess = null;
        this.browser = null;
        this.passed = true;
    }

    log(message, type = 'info') {
        const icons = {
            info: '🔵',
            success: '✅',
            error: '❌',
            warning: '⚠️',
            test: '🧪'
        };
        console.log(`${icons[type] || ''} ${message}`);
    }

    async startServer() {
        this.log('Starting Layer9 server...');
        
        return new Promise((resolve) => {
            this.serverProcess = spawn('cargo', [
                'run',
                '--manifest-path',
                'crates/layer9-server/Cargo.toml',
                '--',
                '--dir',
                'examples/counter',
                '--port',
                '8080'
            ], {
                cwd: process.cwd(),
                stdio: 'pipe'
            });

            let serverReady = false;
            
            this.serverProcess.stdout.on('data', (data) => {
                const output = data.toString();
                if (output.includes('Listening on') && !serverReady) {
                    serverReady = true;
                    setTimeout(resolve, 1000); // Give it a moment to stabilize
                }
            });

            this.serverProcess.stderr.on('data', (data) => {
                const output = data.toString();
                if (output.includes('Listening on') && !serverReady) {
                    serverReady = true;
                    setTimeout(resolve, 1000);
                }
            });

            // Fallback timeout
            setTimeout(() => {
                if (!serverReady) {
                    this.log('Server startup timeout - proceeding anyway', 'warning');
                    resolve();
                }
            }, 10000);
        });
    }

    async runTests() {
        this.log('Layer9 Test Suite', 'test');
        console.log('='.repeat(50));
        
        // Start server
        await this.startServer();
        this.log('Server started', 'success');
        
        // Launch browser
        this.browser = await puppeteer.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
        
        const page = await this.browser.newPage();
        
        // Enable console logging
        page.on('console', msg => {
            if (msg.type() === 'error') {
                console.log('  [console error]', msg.text());
            }
        });
        
        try {
            // Test 1: Page loads
            this.log('Test 1: Page Loading', 'test');
            await page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
            const title = await page.title();
            if (title) {
                this.log(`Page loaded: "${title}"`, 'success');
            } else {
                throw new Error('Page failed to load');
            }
            
            // Test 2: WASM loads and app renders
            this.log('Test 2: WASM and App Rendering', 'test');
            await page.waitForFunction(
                () => {
                    const loading = document.querySelector('.loading');
                    return !loading || window.getComputedStyle(loading).display === 'none';
                },
                { timeout: 15000 }
            );
            
            const appStatus = await page.evaluate(() => {
                return {
                    hasButtons: document.querySelectorAll('button').length > 0,
                    hasRoot: !!document.querySelector('#root'),
                    bodyText: document.body.textContent.substring(0, 100)
                };
            });
            
            if (appStatus.hasButtons && appStatus.hasRoot) {
                this.log(`App rendered: ${appStatus.hasButtons ? 'buttons found' : 'no buttons'}`, 'success');
            } else {
                throw new Error('App failed to render properly');
            }
            
            // Test 3: State management (simplified)
            this.log('Test 3: Basic Interaction', 'test');
            
            // Get initial count from counter display
            const initialCount = await page.evaluate(() => {
                const counterValue = document.querySelector('.counter-value');
                if (counterValue) {
                    return parseInt(counterValue.textContent);
                }
                // Fallback to regex
                const text = document.body.textContent;
                const match = text.match(/\b(\d+)\b/);
                return match ? parseInt(match[1]) : null;
            });
            this.log(`Initial count: ${initialCount}`, 'info');
            
            // Debug: log all button info
            const buttonInfo = await page.evaluate(() => {
                const buttons = Array.from(document.querySelectorAll('button'));
                return buttons.map(b => ({
                    text: b.textContent.trim(),
                    className: b.className,
                    hasOnClick: !!b.onclick
                }));
            });
            this.log(`Found ${buttonInfo.length} buttons:`, 'info');
            buttonInfo.forEach((btn, i) => {
                this.log(`  Button ${i}: "${btn.text}" (class: ${btn.className}, onclick: ${btn.hasOnClick})`, 'info');
            });
            
            // Click increment button
            const clicked = await page.evaluate(() => {
                const buttons = Array.from(document.querySelectorAll('button'));
                const incButton = buttons.find(b => 
                    b.textContent.includes('Increment') || 
                    b.className.includes('btn-increment')
                );
                if (incButton) {
                    incButton.click();
                    return true;
                }
                return false;
            });
            
            if (!clicked) {
                this.log('Failed to find increment button!', 'error');
                this.passed = false;
            }
            
            // Wait for update
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            // Get new count from counter display
            const newCount = await page.evaluate(() => {
                const counterValue = document.querySelector('.counter-value');
                if (counterValue) {
                    return parseInt(counterValue.textContent);
                }
                // Fallback to regex
                const text = document.body.textContent;
                const match = text.match(/\b(\d+)\b/);
                return match ? parseInt(match[1]) : null;
            });
            this.log(`Count after increment: ${newCount}`, 'info');
            
            if (newCount !== null && newCount !== initialCount) {
                this.log('State management works', 'success');
            } else {
                this.log('State management issue detected', 'error');
                this.passed = false;
            }
            
            // Test 4: No console errors
            this.log('Test 4: Console Errors', 'test');
            const jsErrors = await page.evaluate(() => window.__errors || []);
            if (jsErrors.length === 0) {
                this.log('No JavaScript errors', 'success');
            } else {
                this.log(`Found ${jsErrors.length} JavaScript errors`, 'error');
                this.passed = false;
            }
            
        } catch (error) {
            this.log(`Test failed: ${error.message}`, 'error');
            this.passed = false;
        }
        
        // Cleanup
        await this.browser.close();
        this.serverProcess.kill();
        
        // Summary
        console.log('\n' + '='.repeat(50));
        if (this.passed) {
            this.log('ALL TESTS PASSED', 'success');
            process.exit(0);
        } else {
            this.log('SOME TESTS FAILED', 'error');
            process.exit(1);
        }
    }
}

// Run the tests
const validator = new FixedValidator();
validator.runTests().catch(err => {
    console.error('Fatal error:', err);
    process.exit(1);
});