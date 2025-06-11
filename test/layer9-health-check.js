/**
 * Layer9 Health Check & Server Validation System
 * 
 * Elon Mode: Move fast, break nothing, iterate rapidly
 * 
 * Core Objectives:
 * 1. Automatic port conflict resolution
 * 2. Server health monitoring
 * 3. WASM framework validation
 * 4. Performance benchmarking
 * 5. Continuous integration readiness
 */

const puppeteer = require('puppeteer');
const { spawn, exec } = require('child_process');
const http = require('http');
const fs = require('fs');
const path = require('path');

// Configuration
const CONFIG = {
    DEFAULT_PORT: 8080,
    MAX_PORT_ATTEMPTS: 10,
    SERVER_STARTUP_TIMEOUT: 10000,
    TEST_TIMEOUT: 30000,
    PERFORMANCE_THRESHOLDS: {
        firstPaint: 1000,      // 1s
        domReady: 2000,        // 2s
        wasmLoad: 3000,        // 3s
        totalLoad: 5000,       // 5s
        memoryGrowth: 10       // 10MB max
    }
};

// Color utilities
const log = {
    info: (msg) => console.log(`\x1b[36mℹ ${msg}\x1b[0m`),
    success: (msg) => console.log(`\x1b[32m✓ ${msg}\x1b[0m`),
    error: (msg) => console.log(`\x1b[31m✗ ${msg}\x1b[0m`),
    warn: (msg) => console.log(`\x1b[33m⚠ ${msg}\x1b[0m`),
    metric: (key, value) => console.log(`\x1b[34m  ${key}: ${value}\x1b[0m`)
};

class Layer9HealthCheck {
    constructor() {
        this.port = CONFIG.DEFAULT_PORT;
        this.serverProcess = null;
        this.browser = null;
        this.metrics = {
            serverStartTime: 0,
            testResults: [],
            performanceData: {}
        };
    }

    /**
     * Find available port starting from default
     */
    async findAvailablePort() {
        for (let i = 0; i < CONFIG.MAX_PORT_ATTEMPTS; i++) {
            const testPort = CONFIG.DEFAULT_PORT + i;
            if (await this.isPortAvailable(testPort)) {
                return testPort;
            }
        }
        throw new Error('No available ports found');
    }

    /**
     * Check if port is available
     */
    isPortAvailable(port) {
        return new Promise((resolve) => {
            const server = http.createServer();
            server.listen(port, () => {
                server.close(() => resolve(true));
            });
            server.on('error', () => resolve(false));
        });
    }

    /**
     * Kill process using specific port
     */
    async killPortProcess(port) {
        return new Promise((resolve) => {
            exec(`lsof -ti:${port} | xargs kill -9`, (error) => {
                resolve(!error);
            });
        });
    }

    /**
     * Build WASM package
     */
    async buildWasm() {
        log.info('Building WASM package...');
        return new Promise((resolve, reject) => {
            const build = spawn('npm', ['run', 'build:example'], {
                cwd: process.cwd(),
                stdio: 'inherit'
            });

            build.on('close', (code) => {
                if (code === 0) {
                    log.success('WASM build completed');
                    resolve();
                } else {
                    reject(new Error(`Build failed with code ${code}`));
                }
            });
        });
    }

    /**
     * Start development server
     */
    async startServer() {
        // First, ensure any existing process is killed
        await this.killPortProcess(this.port);
        
        // Find available port
        this.port = await this.findAvailablePort();
        log.info(`Starting server on port ${this.port}...`);

        return new Promise((resolve, reject) => {
            const startTime = Date.now();
            
            this.serverProcess = spawn('python3', ['-m', 'http.server', this.port.toString()], {
                cwd: path.join(process.cwd(), 'examples', 'counter'),
                stdio: 'pipe'
            });

            this.serverProcess.on('error', reject);

            // Check if server is ready
            const checkServer = setInterval(async () => {
                try {
                    const response = await fetch(`http://localhost:${this.port}`);
                    if (response.ok) {
                        clearInterval(checkServer);
                        this.metrics.serverStartTime = Date.now() - startTime;
                        log.success(`Server started in ${this.metrics.serverStartTime}ms`);
                        resolve();
                    }
                } catch (e) {
                    // Server not ready yet
                }

                if (Date.now() - startTime > CONFIG.SERVER_STARTUP_TIMEOUT) {
                    clearInterval(checkServer);
                    reject(new Error('Server startup timeout'));
                }
            }, 100);
        });
    }

    /**
     * Run comprehensive health checks
     */
    async runHealthChecks() {
        log.info('Starting Layer9 health checks...\n');

        this.browser = await puppeteer.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });

        const page = await this.browser.newPage();

        // Enable performance tracking
        await page.evaluateOnNewDocument(() => {
            window.layer9Metrics = {
                wasmInitTime: null,
                firstRenderTime: null,
                interactionTimes: []
            };
        });

        try {
            // 1. Server Connectivity Test
            await this.testServerConnectivity(page);

            // 2. WASM Loading Test
            await this.testWasmLoading(page);

            // 3. Framework Functionality Test
            await this.testFrameworkFunctionality(page);

            // 4. Performance Benchmarks
            await this.testPerformance(page);

            // 5. Error Handling Test
            await this.testErrorHandling(page);

            // 6. Memory Stability Test
            await this.testMemoryStability(page);

        } finally {
            await this.browser.close();
        }
    }

    async testServerConnectivity(page) {
        const test = 'Server Connectivity';
        try {
            const response = await page.goto(`http://localhost:${this.port}`, {
                waitUntil: 'networkidle0',
                timeout: CONFIG.TEST_TIMEOUT
            });

            if (response.status() !== 200) {
                throw new Error(`Status ${response.status()}`);
            }

            this.recordTest(test, true);
            log.success(`${test} - Server responding on port ${this.port}`);
        } catch (error) {
            this.recordTest(test, false, error.message);
            throw error;
        }
    }

    async testWasmLoading(page) {
        const test = 'WASM Module Loading';
        try {
            const startTime = Date.now();

            await page.waitForFunction(
                () => document.querySelector('.layer9-app') !== null,
                { timeout: CONFIG.TEST_TIMEOUT }
            );

            const wasmInfo = await page.evaluate(() => {
                return {
                    wasmSupported: typeof WebAssembly !== 'undefined',
                    moduleLoaded: !!window.wasm_bindgen,
                    appRendered: !!document.querySelector('.layer9-app')
                };
            });

            if (!wasmInfo.wasmSupported || !wasmInfo.appRendered) {
                throw new Error('WASM not fully loaded');
            }

            const loadTime = Date.now() - startTime;
            this.metrics.performanceData.wasmLoadTime = loadTime;

            this.recordTest(test, true);
            log.success(`${test} - Loaded in ${loadTime}ms`);
        } catch (error) {
            this.recordTest(test, false, error.message);
            throw error;
        }
    }

    async testFrameworkFunctionality(page) {
        const test = 'Framework Functionality';
        try {
            // Test counter functionality
            const initialCount = await page.$eval('#counter-display', el => el.textContent);
            
            // Test increment
            await page.click('button.btn-primary');
            await new Promise(resolve => setTimeout(resolve, 100));
            const afterIncrement = await page.$eval('#counter-display', el => el.textContent);

            // Test decrement
            await page.click('button.btn-secondary');
            await new Promise(resolve => setTimeout(resolve, 100));
            const afterDecrement = await page.$eval('#counter-display', el => el.textContent);

            // Test reset
            await page.click('button.btn-warning');
            await new Promise(resolve => setTimeout(resolve, 100));
            const afterReset = await page.$eval('#counter-display', el => el.textContent);

            if (initialCount !== 'Count: 0' || 
                afterIncrement !== 'Count: 1' || 
                afterDecrement !== 'Count: 0' || 
                afterReset !== 'Count: 0') {
                throw new Error('State management failed');
            }

            this.recordTest(test, true);
            log.success(`${test} - All interactions working`);
        } catch (error) {
            this.recordTest(test, false, error.message);
            throw error;
        }
    }

    async testPerformance(page) {
        const test = 'Performance Benchmarks';
        try {
            const metrics = await page.evaluate(() => {
                const perf = window.performance;
                const navigation = perf.getEntriesByType('navigation')[0];
                const paintMetrics = {};
                
                perf.getEntriesByType('paint').forEach(entry => {
                    paintMetrics[entry.name] = entry.startTime;
                });

                return {
                    domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
                    loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
                    ...paintMetrics
                };
            });

            Object.assign(this.metrics.performanceData, metrics);

            // Validate against thresholds
            const violations = [];
            if (metrics['first-paint'] > CONFIG.PERFORMANCE_THRESHOLDS.firstPaint) {
                violations.push(`First paint: ${metrics['first-paint']}ms`);
            }
            if (metrics.loadComplete > CONFIG.PERFORMANCE_THRESHOLDS.totalLoad) {
                violations.push(`Total load: ${metrics.loadComplete}ms`);
            }

            if (violations.length > 0) {
                throw new Error(`Performance violations: ${violations.join(', ')}`);
            }

            this.recordTest(test, true);
            log.success(`${test} - All metrics within thresholds`);
            log.metric('First Paint', `${metrics['first-paint']}ms`);
            log.metric('DOM Ready', `${metrics.domContentLoaded}ms`);
            log.metric('Total Load', `${metrics.loadComplete}ms`);
        } catch (error) {
            this.recordTest(test, false, error.message);
            throw error;
        }
    }

    async testErrorHandling(page) {
        const test = 'Error Handling';
        try {
            const errors = [];
            
            page.on('console', msg => {
                if (msg.type() === 'error') {
                    errors.push(msg.text());
                }
            });

            page.on('pageerror', error => {
                errors.push(error.toString());
            });

            // Wait a bit to catch any async errors
            await new Promise(resolve => setTimeout(resolve, 1000));

            if (errors.length > 0) {
                throw new Error(`Console errors detected: ${errors.join(', ')}`);
            }

            this.recordTest(test, true);
            log.success(`${test} - No console errors`);
        } catch (error) {
            this.recordTest(test, false, error.message);
            throw error;
        }
    }

    async testMemoryStability(page) {
        const test = 'Memory Stability';
        try {
            const initialMetrics = await page.metrics();
            
            // Stress test with rapid interactions
            for (let i = 0; i < 100; i++) {
                await page.click('button.btn-primary');
                if (i % 10 === 0) {
                    await page.click('button.btn-warning'); // Reset periodically
                }
            }

            await new Promise(resolve => setTimeout(resolve, 500));
            const finalMetrics = await page.metrics();

            const heapGrowthMB = (finalMetrics.JSHeapUsedSize - initialMetrics.JSHeapUsedSize) / 1024 / 1024;

            if (heapGrowthMB > CONFIG.PERFORMANCE_THRESHOLDS.memoryGrowth) {
                throw new Error(`Excessive memory growth: ${heapGrowthMB.toFixed(2)}MB`);
            }

            this.recordTest(test, true);
            log.success(`${test} - Memory growth: ${heapGrowthMB.toFixed(2)}MB`);
        } catch (error) {
            this.recordTest(test, false, error.message);
            throw error;
        }
    }

    recordTest(name, passed, error = null) {
        this.metrics.testResults.push({ name, passed, error });
    }

    /**
     * Generate health report
     */
    generateReport() {
        const totalTests = this.metrics.testResults.length;
        const passedTests = this.metrics.testResults.filter(t => t.passed).length;
        const failedTests = totalTests - passedTests;
        const passRate = (passedTests / totalTests * 100).toFixed(1);

        console.log('\n\x1b[36m══════════════════════════════════════\x1b[0m');
        console.log('\x1b[36m     Layer9 Health Check Report\x1b[0m');
        console.log('\x1b[36m══════════════════════════════════════\x1b[0m\n');

        console.log(`Server Port: ${this.port}`);
        console.log(`Server Start Time: ${this.metrics.serverStartTime}ms`);
        console.log(`Total Tests: ${totalTests}`);
        console.log(`\x1b[32mPassed: ${passedTests}\x1b[0m`);
        console.log(`\x1b[31mFailed: ${failedTests}\x1b[0m`);
        console.log(`Pass Rate: ${passRate}%\n`);

        if (failedTests > 0) {
            console.log('\x1b[31mFailed Tests:\x1b[0m');
            this.metrics.testResults
                .filter(t => !t.passed)
                .forEach(t => console.log(`  - ${t.name}: ${t.error}`));
            console.log('');
        }

        console.log('\x1b[34mPerformance Metrics:\x1b[0m');
        Object.entries(this.metrics.performanceData).forEach(([key, value]) => {
            console.log(`  ${key}: ${value}ms`);
        });

        const healthStatus = failedTests === 0 ? 'HEALTHY' : 'UNHEALTHY';
        const statusColor = failedTests === 0 ? '\x1b[32m' : '\x1b[31m';
        
        console.log(`\n${statusColor}System Status: ${healthStatus}\x1b[0m\n`);

        return failedTests === 0;
    }

    /**
     * Cleanup resources
     */
    async cleanup() {
        if (this.serverProcess) {
            this.serverProcess.kill();
            log.info('Server process terminated');
        }
        if (this.browser) {
            await this.browser.close();
        }
    }

    /**
     * Main execution flow
     */
    async run() {
        try {
            // Build WASM
            await this.buildWasm();

            // Start server
            await this.startServer();

            // Run health checks
            await this.runHealthChecks();

            // Generate report
            const isHealthy = this.generateReport();

            // Cleanup
            await this.cleanup();

            process.exit(isHealthy ? 0 : 1);

        } catch (error) {
            log.error(`Fatal error: ${error.message}`);
            await this.cleanup();
            process.exit(1);
        }
    }
}

// Execute health check
if (require.main === module) {
    const healthCheck = new Layer9HealthCheck();
    healthCheck.run();
}

module.exports = Layer9HealthCheck;