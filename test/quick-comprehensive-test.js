#!/usr/bin/env node

/**
 * Layer9 Quick Comprehensive Test Runner
 * Runs all tests quickly by reusing existing server if available
 */

const { spawn, execSync } = require('child_process');
const path = require('path');
const http = require('http');

class QuickTestRunner {
    constructor() {
        this.serverProcess = null;
        this.testResults = {
            total: 0,
            passed: 0,
            failed: 0,
            suites: []
        };
        this.serverStarted = false;
    }

    async checkServerRunning() {
        return new Promise((resolve) => {
            const req = http.get('http://localhost:8080', (res) => {
                resolve(res.statusCode === 200 || res.statusCode === 404);
            });
            req.on('error', () => resolve(false));
            req.setTimeout(1000, () => {
                req.destroy();
                resolve(false);
            });
        });
    }

    async ensureServer() {
        console.log('🔵 Checking for existing Layer9 server...');
        
        const isRunning = await this.checkServerRunning();
        if (isRunning) {
            console.log('✅ Server already running');
            return;
        }

        console.log('🔵 Starting Layer9 server for tests...');
        
        // First ensure server is built
        try {
            console.log('📦 Building server (if needed)...');
            execSync('cargo build --manifest-path crates/layer9-server/Cargo.toml', {
                stdio: 'inherit'
            });
        } catch (error) {
            console.log('⚠️ Build failed, trying to proceed anyway');
        }
        
        return new Promise((resolve, reject) => {
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

            this.serverStarted = true;
            let serverReady = false;
            
            const checkServerOutput = (data) => {
                const output = data.toString();
                if (!serverReady && (output.includes('Serving') || output.includes('Started') || output.includes('8080'))) {
                    serverReady = true;
                    setTimeout(() => {
                        console.log('✅ Server started successfully');
                        resolve();
                    }, 1000);
                }
            };

            this.serverProcess.stdout.on('data', checkServerOutput);
            this.serverProcess.stderr.on('data', checkServerOutput);

            this.serverProcess.on('error', (err) => {
                console.error('❌ Failed to start server:', err);
                reject(err);
            });

            // Shorter timeout and periodic checks
            let checkCount = 0;
            const checkInterval = setInterval(async () => {
                checkCount++;
                const running = await this.checkServerRunning();
                if (running || serverReady) {
                    clearInterval(checkInterval);
                    if (!serverReady) {
                        console.log('✅ Server detected on port 8080');
                        resolve();
                    }
                } else if (checkCount > 10) {
                    clearInterval(checkInterval);
                    console.log('⚠️ Server startup timeout - proceeding anyway');
                    resolve();
                }
            }, 1000);
        });
    }

    stopServer() {
        if (this.serverProcess && this.serverStarted) {
            console.log('🔵 Stopping server...');
            this.serverProcess.kill();
            this.serverProcess = null;
        }
    }

    async runTestSuite(name, command) {
        console.log(`\n📋 Running ${name}...`);
        console.log('━'.repeat(60));
        
        const startTime = Date.now();
        
        return new Promise((resolve) => {
            const proc = spawn('node', [command], {
                cwd: process.cwd(),
                stdio: 'inherit'
            });

            proc.on('close', (code) => {
                const duration = Date.now() - startTime;
                const passed = code === 0;
                
                this.testResults.total++;
                if (passed) {
                    this.testResults.passed++;
                    console.log(`\n✅ ${name} PASSED (${this.formatTime(duration)})`);
                } else {
                    this.testResults.failed++;
                    console.log(`\n❌ ${name} FAILED (${this.formatTime(duration)})`);
                }
                
                this.testResults.suites.push({
                    name,
                    passed,
                    duration
                });
                
                resolve();
            });

            proc.on('error', (err) => {
                console.error(`\n❌ Failed to run ${name}:`, err);
                this.testResults.total++;
                this.testResults.failed++;
                this.testResults.suites.push({
                    name,
                    passed: false,
                    duration: Date.now() - startTime
                });
                resolve();
            });
        });
    }

    async runRustTests() {
        console.log(`\n📋 Running Rust Tests...`);
        console.log('━'.repeat(60));
        
        const startTime = Date.now();
        
        return new Promise((resolve) => {
            const proc = spawn('cargo', ['test', '--all'], {
                cwd: process.cwd(),
                stdio: 'inherit'
            });

            proc.on('close', (code) => {
                const duration = Date.now() - startTime;
                const passed = code === 0;
                
                this.testResults.total++;
                if (passed) {
                    this.testResults.passed++;
                    console.log(`\n✅ Rust Tests PASSED`);
                } else {
                    this.testResults.failed++;
                    console.log(`\n❌ Rust Tests FAILED`);
                }
                
                this.testResults.suites.push({
                    name: 'Rust Tests',
                    passed,
                    duration
                });
                
                resolve();
            });

            proc.on('error', (err) => {
                console.error(`\n❌ Failed to run Rust tests:`, err);
                this.testResults.total++;
                this.testResults.failed++;
                this.testResults.suites.push({
                    name: 'Rust Tests',
                    passed: false,
                    duration: Date.now() - startTime
                });
                resolve();
            });
        });
    }

    formatTime(ms) {
        return (ms / 1000).toFixed(2) + 's';
    }

    printSummary() {
        console.log('\n' + '═'.repeat(60));
        console.log('📊 COMPREHENSIVE TEST RESULTS');
        console.log('═'.repeat(60));
        console.log(`\nTotal Test Suites: ${this.testResults.total}`);
        console.log(`Passed: \x1b[32m${this.testResults.passed}\x1b[0m`);
        console.log(`Failed: \x1b[31m${this.testResults.failed}\x1b[0m`);
        console.log(`Success Rate: ${((this.testResults.passed / this.testResults.total) * 100).toFixed(1)}%`);
        
        console.log('\nDetailed Results:');
        console.log('─'.repeat(60));
        this.testResults.suites.forEach(suite => {
            const status = suite.passed ? '\x1b[32m✓\x1b[0m' : '\x1b[31m✗\x1b[0m';
            const time = suite.duration ? ` (${this.formatTime(suite.duration)})` : '';
            console.log(`${status} ${suite.name}${time}`);
        });
        
        console.log('\n' + '═'.repeat(60));
        
        if (this.testResults.failed === 0) {
            console.log('\n✅ ALL TESTS PASSED! Layer9 is production ready! 🚀');
        } else {
            console.log('\n❌ SOME TESTS FAILED! Please check the output above.');
        }
    }

    async run() {
        console.log('\n🚀 LAYER9 COMPREHENSIVE TEST SUITE');
        console.log('═'.repeat(60));
        
        try {
            await this.ensureServer();
            
            // Run test suites in sequence
            await this.runTestSuite('Basic Validation', 'test/layer9-validation-fixed.js');
            await this.runTestSuite('Final Validation', 'test/final-validation.js');
            await this.runTestSuite('E2E Tests', 'test/e2e/layer9-counter.test.js');
            await this.runRustTests();
            
        } catch (error) {
            console.error('❌ Test runner error:', error);
        } finally {
            this.stopServer();
            this.printSummary();
            process.exit(this.testResults.failed > 0 ? 1 : 0);
        }
    }
}

// Run tests
const runner = new QuickTestRunner();
runner.run();