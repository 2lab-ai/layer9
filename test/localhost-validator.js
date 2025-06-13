#!/usr/bin/env node
/**
 * Layer9 Localhost:8080 Ultimate Validator
 * 
 * Elon Mode: This test doesn't fail. It fixes problems and succeeds.
 * 
 * Core Philosophy:
 * - If the port is blocked, kill the blocker
 * - If the server isn't running, start it
 * - If WASM isn't built, build it
 * - If the test fails, diagnose and retry
 * - Success is the only option
 */

const puppeteer = require('puppeteer');
const { spawn, exec } = require('child_process');
const http = require('http');
const path = require('path');
const fs = require('fs');

class LocalhostValidator {
    constructor() {
        this.attempt = 0;
        this.maxAttempts = 10;
        this.diagnostics = [];
        this.serverProcess = null;
        this.browser = null;
        
        // Success criteria
        this.criteria = {
            serverResponds: false,
            wasmLoads: false,
            uiRenders: false,
            stateWorks: false,
            noErrors: false
        };
    }

    log(message, type = 'info') {
        const icons = {
            info: '🔵',
            success: '✅',
            error: '❌',
            fix: '🔧',
            rocket: '🚀'
        };
        console.log(`${icons[type]} ${message}`);
        this.diagnostics.push({ time: Date.now(), message, type });
    }

    async executeCommand(command) {
        return new Promise((resolve, reject) => {
            exec(command, (error, stdout, stderr) => {
                resolve({ error, stdout, stderr });
            });
        });
    }

    async checkPort() {
        this.log('Checking port 8080 status...');
        
        const portCheck = await this.executeCommand('lsof -i :8080');
        
        if (portCheck.stdout) {
            this.log('Port 8080 is occupied', 'error');
            this.log('Killing process on port 8080', 'fix');
            
            await this.executeCommand('lsof -ti:8080 | xargs kill -9');
            await new Promise(resolve => setTimeout(resolve, 1000));
            
            this.log('Port 8080 cleared', 'success');
        } else {
            this.log('Port 8080 is available', 'success');
        }
    }

    async ensureWasmBuilt() {
        this.log('Checking WASM build...');
        
        const wasmPath = path.join(process.cwd(), 'examples', 'counter', 'pkg', 'layer9_example_counter_bg.wasm');
        
        if (!fs.existsSync(wasmPath)) {
            this.log('WASM not built', 'error');
            this.log('Building WASM...', 'fix');
            
            await new Promise((resolve, reject) => {
                const build = spawn('npm', ['run', 'build:example'], {
                    stdio: 'inherit'
                });
                
                build.on('close', (code) => {
                    if (code === 0) {
                        this.log('WASM built successfully', 'success');
                        resolve();
                    } else {
                        reject(new Error('WASM build failed'));
                    }
                });
            });
        } else {
            this.log('WASM already built', 'success');
        }
    }

    async startServer() {
        this.log('Starting HTTP server...');
        
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

            // Capture server output for debugging
            this.serverProcess.stdout.on('data', (data) => {
                console.log(`Server stdout: ${data}`);
            });

            this.serverProcess.stderr.on('data', (data) => {
                console.log(`Server stderr: ${data}`);
            });

            // Give server more time to start
            setTimeout(() => {
                this.log('Server started on port 8080', 'success');
                resolve();
            }, 5000);
        });
    }

    async validateWithPuppeteer() {
        this.log('Starting Puppeteer validation...', 'rocket');
        
        this.browser = await puppeteer.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });

        const page = await this.browser.newPage();
        
        // Capture all console messages
        const consoleLogs = [];
        page.on('console', msg => {
            consoleLogs.push({ type: msg.type(), text: msg.text() });
            if (msg.type() === 'error') {
                this.log(`Console error: ${msg.text()}`, 'error');
            }
        });

        // Test 1: Server Responds
        try {
            this.log('Test 1: Server Response');
            const response = await page.goto('http://localhost:8080', {
                waitUntil: 'networkidle0',
                timeout: 10000
            });
            
            if (response && response.status() === 200) {
                this.criteria.serverResponds = true;
                this.log('Server responds correctly', 'success');
            } else {
                throw new Error(`Server returned status ${response?.status()}`);
            }
        } catch (error) {
            this.log(`Server test failed: ${error.message}`, 'error');
            throw error;
        }

        // Test 2: WASM Loads
        try {
            this.log('Test 2: WASM Loading');
            
            // Wait for loading to disappear
            await page.waitForFunction(
                () => {
                    const loading = document.querySelector('.loading');
                    return !loading || loading.style.display === 'none';
                },
                { timeout: 10000 }
            );
            
            const wasmStatus = await page.evaluate(() => {
                return {
                    wasmSupported: typeof WebAssembly !== 'undefined',
                    bindgenLoaded: typeof window.wasm_bindgen !== 'undefined',
                    hasRoot: !!document.querySelector('#root'),
                    hasButtons: document.querySelectorAll('button').length > 0,
                    loadingHidden: !document.querySelector('.loading') || document.querySelector('.loading').style.display === 'none'
                };
            });
            
            if (wasmStatus.wasmSupported && wasmStatus.hasRoot && wasmStatus.hasButtons && wasmStatus.loadingHidden) {
                this.criteria.wasmLoads = true;
                this.log('WASM loaded successfully', 'success');
                this.log(`Found: Root element, ${wasmStatus.hasButtons ? 'buttons present' : 'no buttons'}, loading hidden`, 'info');
            } else {
                throw new Error(`WASM failed to load properly: ${JSON.stringify(wasmStatus)}`);
            }
        } catch (error) {
            this.log(`WASM test failed: ${error.message}`, 'error');
            throw error;
        }

        // Test 3: UI Renders
        try {
            this.log('Test 3: UI Rendering');
            
            const elements = await page.evaluate(() => {
                return {
                    title: !!document.querySelector('h1'),
                    counterText: document.body.textContent.includes('Count:') || document.body.textContent.match(/\d+/),
                    buttons: document.querySelectorAll('button').length,
                    hasRoot: !!document.querySelector('#root'),
                    bodyHTML: document.body.innerHTML.substring(0, 200)
                };
            });
            
            if (elements.hasRoot && elements.buttons >= 3) {
                this.criteria.uiRenders = true;
                this.log('UI rendered correctly', 'success');
                this.log(`Found: Root element, ${elements.buttons} buttons`, 'info');
            } else {
                throw new Error(`UI elements missing: ${JSON.stringify(elements)}`);
            }
        } catch (error) {
            this.log(`UI test failed: ${error.message}`, 'error');
            throw error;
        }

        // Test 4: State Management
        try {
            this.log('Test 4: State Management');
            
            // Get initial state by looking for number in the page
            const initialState = await page.evaluate(() => {
                const text = document.body.textContent;
                const match = text.match(/\b(\d+)\b/);
                return match ? parseInt(match[1]) : null;
            });
            this.log(`Initial state: ${initialState}`, 'info');
            
            // Find buttons by their text content
            const buttons = await page.$$('button');
            let incrementBtn = null;
            let decrementBtn = null;
            let resetBtn = null;
            
            for (const button of buttons) {
                const text = await button.evaluate(el => el.textContent);
                if (text.includes('Increment') || text.includes('+')) {
                    incrementBtn = button;
                } else if (text.includes('Decrement') || text.includes('-') || text.includes('−')) {
                    decrementBtn = button;
                } else if (text.includes('Reset')) {
                    resetBtn = button;
                }
            }
            
            if (!incrementBtn || !decrementBtn) {
                throw new Error('Could not find increment/decrement buttons');
            }
            
            // Click increment
            await incrementBtn.click();
            await new Promise(resolve => setTimeout(resolve, 500));
            
            const afterIncrement = await page.evaluate(() => {
                const text = document.body.textContent;
                const match = text.match(/\b(\d+)\b/);
                return match ? parseInt(match[1]) : null;
            });
            this.log(`After increment: ${afterIncrement}`, 'info');
            
            // Click decrement
            await decrementBtn.click();
            await new Promise(resolve => setTimeout(resolve, 500));
            
            const afterDecrement = await page.evaluate(() => {
                const text = document.body.textContent;
                const match = text.match(/\b(\d+)\b/);
                return match ? parseInt(match[1]) : null;
            });
            this.log(`After decrement: ${afterDecrement}`, 'info');
            
            // Basic state validation
            if (initialState !== null && afterIncrement === initialState + 1 && afterDecrement === initialState) {
                this.criteria.stateWorks = true;
                this.log('State management works correctly', 'success');
            } else {
                throw new Error(`State management failed: initial=${initialState}, increment=${afterIncrement}, decrement=${afterDecrement}`);
            }
        } catch (error) {
            this.log(`State test failed: ${error.message}`, 'error');
            throw error;
        }

        // Test 5: No Console Errors
        try {
            this.log('Test 5: Console Errors Check');
            
            const errors = consoleLogs.filter(log => log.type === 'error');
            
            if (errors.length === 0) {
                this.criteria.noErrors = true;
                this.log('No console errors detected', 'success');
            } else {
                throw new Error(`Found ${errors.length} console errors`);
            }
        } catch (error) {
            this.log(`Error check failed: ${error.message}`, 'error');
            throw error;
        }

        await this.browser.close();
        this.browser = null;
    }

    async diagnoseFailure() {
        this.log('Running diagnostics...', 'fix');
        
        // Check if server is still running
        const serverCheck = await this.executeCommand('lsof -i :8080');
        if (!serverCheck.stdout) {
            this.log('Server crashed - will restart', 'fix');
            return 'server_crash';
        }
        
        // Check if WASM files exist
        const wasmExists = fs.existsSync(
            path.join(process.cwd(), 'examples', 'counter', 'pkg', 'layer9_example_counter_bg.wasm')
        );
        if (!wasmExists) {
            this.log('WASM files missing - will rebuild', 'fix');
            return 'wasm_missing';
        }
        
        // Check network
        try {
            await fetch('http://localhost:8080');
        } catch (error) {
            this.log('Network issue detected', 'fix');
            return 'network_issue';
        }
        
        return 'unknown';
    }

    async cleanup() {
        if (this.browser) {
            await this.browser.close();
        }
        if (this.serverProcess) {
            this.serverProcess.kill();
        }
    }

    async run() {
        console.log('\n🚀 LAYER9 LOCALHOST:8080 VALIDATOR 🚀');
        console.log('━'.repeat(50));
        console.log('Mission: Validate localhost:8080 or fix trying\n');

        while (this.attempt < this.maxAttempts) {
            this.attempt++;
            this.log(`\nAttempt ${this.attempt}/${this.maxAttempts}`, 'rocket');
            
            try {
                // Phase 1: Environment Setup
                await this.checkPort();
                await this.ensureWasmBuilt();
                await this.startServer();
                
                // Phase 2: Validation
                await this.validateWithPuppeteer();
                
                // Check if all criteria passed
                const allPassed = Object.values(this.criteria).every(v => v);
                
                if (allPassed) {
                    this.log('\n🎉 ALL TESTS PASSED! 🎉', 'rocket');
                    this.log('Layer9 is running perfectly on localhost:8080', 'success');
                    
                    console.log('\n📊 Final Report:');
                    Object.entries(this.criteria).forEach(([key, value]) => {
                        console.log(`  ${value ? '✅' : '❌'} ${key}`);
                    });
                    
                    await this.cleanup();
                    process.exit(0);
                }
                
            } catch (error) {
                this.log(`Attempt ${this.attempt} failed: ${error.message}`, 'error');
                
                // Diagnose and fix
                const issue = await this.diagnoseFailure();
                
                switch (issue) {
                    case 'server_crash':
                        this.log('Restarting server...', 'fix');
                        if (this.serverProcess) this.serverProcess.kill();
                        this.serverProcess = null;
                        break;
                        
                    case 'wasm_missing':
                        this.log('Rebuilding WASM...', 'fix');
                        break;
                        
                    case 'network_issue':
                        this.log('Waiting for network...', 'fix');
                        await new Promise(resolve => setTimeout(resolve, 2000));
                        break;
                }
                
                // Clean up for retry
                await this.cleanup();
                
                if (this.attempt < this.maxAttempts) {
                    this.log('Retrying in 3 seconds...', 'fix');
                    await new Promise(resolve => setTimeout(resolve, 3000));
                }
            }
        }
        
        // If we get here, we've exhausted attempts
        this.log('\n❌ VALIDATION FAILED ❌', 'error');
        console.log('\n📊 Diagnostic Summary:');
        this.diagnostics.slice(-20).forEach(d => {
            console.log(`  ${d.message}`);
        });
        
        await this.cleanup();
        process.exit(1);
    }
}

// Execute with extreme prejudice
const validator = new LocalhostValidator();
validator.run().catch(error => {
    console.error('💥 CATASTROPHIC FAILURE:', error);
    process.exit(1);
});