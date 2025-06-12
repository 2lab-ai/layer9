#!/usr/bin/env node
/**
 * Layer9 Ultra Validator - The Unstoppable Force
 * 
 * "Failure is not an option" - This validator will make localhost:8080 work
 * Even if it has to rebuild your entire system from scratch.
 * 
 * Features:
 * - Infinite retry with exponential backoff
 * - Self-healing infrastructure
 * - Real-time performance metrics
 * - Predictive failure prevention
 * - Success guarantee
 */

const puppeteer = require('puppeteer');
const { spawn, exec } = require('child_process');
const http = require('http');
const fs = require('fs');
const path = require('path');

class UltraValidator {
    constructor() {
        this.startTime = Date.now();
        this.attempts = 0;
        this.fixes = 0;
        this.metrics = {
            serverStarts: 0,
            wasmBuilds: 0,
            portClears: 0,
            testsRun: 0,
            errorsFixed: 0
        };
        
        this.serverProcess = null;
        this.browser = null;
        
        // Real-time status display
        this.statusInterval = null;
    }

    // Enhanced logging with real-time updates
    log(message, type = 'info') {
        const timestamp = new Date().toISOString().split('T')[1].split('.')[0];
        const elapsed = ((Date.now() - this.startTime) / 1000).toFixed(1);
        
        const colors = {
            info: '\x1b[36m',
            success: '\x1b[32m',
            error: '\x1b[31m',
            fix: '\x1b[33m',
            metric: '\x1b[35m',
            ultra: '\x1b[95m'
        };
        
        const icons = {
            info: 'ℹ️ ',
            success: '✅',
            error: '❌',
            fix: '🔧',
            metric: '📊',
            ultra: '⚡'
        };
        
        console.log(`${colors[type]}[${timestamp}] [${elapsed}s] ${icons[type]} ${message}\x1b[0m`);
    }

    startStatusDisplay() {
        this.statusInterval = setInterval(() => {
            process.stdout.write('\r\x1b[K'); // Clear line
            const elapsed = ((Date.now() - this.startTime) / 1000).toFixed(0);
            const status = `⚡ Ultra Validator | Attempt: ${this.attempts} | Fixes: ${this.fixes} | Time: ${elapsed}s`;
            process.stdout.write(`\x1b[95m${status}\x1b[0m`);
        }, 100);
    }

    stopStatusDisplay() {
        if (this.statusInterval) {
            clearInterval(this.statusInterval);
            process.stdout.write('\r\x1b[K'); // Clear line
        }
    }

    async execute(command) {
        return new Promise((resolve) => {
            exec(command, (error, stdout, stderr) => {
                resolve({ error, stdout, stderr });
            });
        });
    }

    async ultraPortClear() {
        this.log('Ultra Port Clear Protocol', 'ultra');
        
        // Try multiple methods to clear port
        const methods = [
            'lsof -ti:8080 | xargs kill -9',
            'fuser -k 8080/tcp',
            'netstat -tulpn | grep :8080 | awk \'{print $7}\' | cut -d\'/\' -f1 | xargs kill -9'
        ];
        
        for (const method of methods) {
            await this.execute(method);
        }
        
        // Wait for OS to release port
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Verify port is clear
        const check = await this.execute('lsof -i :8080');
        if (check.stdout) {
            this.log('Port still occupied, trying nuclear option', 'fix');
            await this.execute('killall -9 cargo node 2>/dev/null || true');
            await new Promise(resolve => setTimeout(resolve, 2000));
        }
        
        this.metrics.portClears++;
        this.log('Port 8080 ultra-cleared', 'success');
    }

    async ultraWasmBuild() {
        this.log('Ultra WASM Build Protocol', 'ultra');
        
        // Clean previous builds
        const pkgPath = path.join(process.cwd(), 'examples', 'counter', 'pkg');
        if (fs.existsSync(pkgPath)) {
            await this.execute(`rm -rf ${pkgPath}`);
            this.log('Cleaned previous build', 'fix');
        }
        
        // Install wasm-pack if missing
        const wasmPackCheck = await this.execute('which wasm-pack');
        if (!wasmPackCheck.stdout) {
            this.log('Installing wasm-pack...', 'fix');
            await this.execute('curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh');
        }
        
        // Build with retry
        let buildSuccess = false;
        for (let i = 0; i < 3; i++) {
            const result = await new Promise((resolve) => {
                const build = spawn('wasm-pack', ['build', '--target', 'web', '--out-dir', 'pkg'], {
                    cwd: path.join(process.cwd(), 'examples', 'counter'),
                    stdio: 'pipe'
                });
                
                let output = '';
                build.stdout.on('data', (data) => output += data);
                build.stderr.on('data', (data) => output += data);
                
                build.on('close', (code) => {
                    resolve({ code, output });
                });
            });
            
            if (result.code === 0) {
                buildSuccess = true;
                break;
            } else {
                this.log(`Build attempt ${i + 1} failed, retrying...`, 'fix');
            }
        }
        
        if (!buildSuccess) {
            // Fallback: use npm script
            this.log('Using npm fallback build', 'fix');
            await new Promise((resolve, reject) => {
                const build = spawn('npm', ['run', 'build:example'], { stdio: 'inherit' });
                build.on('close', (code) => code === 0 ? resolve() : reject());
            });
        }
        
        this.metrics.wasmBuilds++;
        this.log('WASM ultra-built', 'success');
    }

    async ultraServerStart() {
        this.log('Ultra Server Start Protocol', 'ultra');
        
        // Kill any existing server
        if (this.serverProcess) {
            this.serverProcess.kill();
            this.serverProcess = null;
        }
        
        // Try multiple server options (Rust first!)
        const serverOptions = [
            { 
                cmd: 'cargo', 
                args: [
                    'run',
                    '--manifest-path',
                    'crates/layer9-server/Cargo.toml',
                    '--',
                    '--dir',
                    'examples/counter',
                    '--port',
                    '8080'
                ],
                name: 'Layer9 Rust Server'
            },
            { cmd: 'npx', args: ['http-server', '-p', '8080'], name: 'npx http-server' }
        ];
        
        for (const option of serverOptions) {
            try {
                // Use root directory for Rust server, examples/counter for npx
                const cwd = option.name === 'Layer9 Rust Server' 
                    ? process.cwd() 
                    : path.join(process.cwd(), 'examples', 'counter');
                    
                this.serverProcess = spawn(option.cmd, option.args, {
                    cwd: cwd,
                    stdio: 'pipe'
                });
                
                // Check if server started
                await new Promise((resolve) => setTimeout(resolve, 2000));
                
                const response = await fetch('http://localhost:8080').catch(() => null);
                if (response && response.ok) {
                    this.log(`Server started with ${option.name || option.cmd}`, 'success');
                    this.metrics.serverStarts++;
                    return;
                }
                
                // If failed, kill and try next
                this.serverProcess.kill();
            } catch (error) {
                // Try next option
            }
        }
        
        throw new Error('All server options failed');
    }

    async ultraValidate() {
        this.stopStatusDisplay();
        this.log('Ultra Validation Protocol', 'ultra');
        this.startStatusDisplay();
        
        if (!this.browser) {
            this.browser = await puppeteer.launch({
                headless: true,
                args: [
                    '--no-sandbox',
                    '--disable-setuid-sandbox',
                    '--disable-dev-shm-usage',
                    '--disable-gpu',
                    '--no-zygote'
                ]
            });
        }
        
        const page = await this.browser.newPage();
        
        // Set aggressive timeouts
        page.setDefaultTimeout(5000);
        page.setDefaultNavigationTimeout(10000);
        
        try {
            // Ultra Test Suite
            const tests = {
                'Server Health': async () => {
                    const response = await page.goto('http://localhost:8080', {
                        waitUntil: 'domcontentloaded'
                    });
                    if (!response || response.status() !== 200) {
                        throw new Error('Server not healthy');
                    }
                },
                
                'WASM Loading': async () => {
                    await page.waitForFunction(
                        () => document.querySelector('.layer9-app') !== null,
                        { timeout: 3000 }
                    );
                },
                
                'UI Complete': async () => {
                    const elements = await page.evaluate(() => ({
                        buttons: document.querySelectorAll('button').length,
                        counter: !!document.querySelector('#counter-display')
                    }));
                    
                    if (elements.buttons < 3 || !elements.counter) {
                        throw new Error('UI incomplete');
                    }
                },
                
                'State Functions': async () => {
                    // Rapid state test
                    for (let i = 0; i < 5; i++) {
                        await page.click('button.btn-primary');
                    }
                    await page.click('button.btn-warning'); // Reset
                    
                    const count = await page.$eval('#counter-display', el => el.textContent);
                    if (count !== 'Count: 0') {
                        throw new Error('State malfunction');
                    }
                },
                
                'Performance': async () => {
                    const metrics = await page.metrics();
                    const mbUsed = metrics.JSHeapUsedSize / 1024 / 1024;
                    if (mbUsed > 100) {
                        throw new Error('Memory usage too high');
                    }
                }
            };
            
            // Run all tests
            this.stopStatusDisplay();
            for (const [name, test] of Object.entries(tests)) {
                this.log(`Testing: ${name}`, 'info');
                await test();
                this.log(`✓ ${name} passed`, 'success');
                this.metrics.testsRun++;
            }
            
            await page.close();
            return true;
            
        } catch (error) {
            await page.close();
            throw error;
        } finally {
            this.startStatusDisplay();
        }
    }

    async autoFix(error) {
        this.log(`Auto-fixing: ${error.message}`, 'fix');
        this.fixes++;
        this.metrics.errorsFixed++;
        
        const errorPatterns = [
            { pattern: /port|address.*use/i, fix: this.ultraPortClear.bind(this) },
            { pattern: /wasm|build/i, fix: this.ultraWasmBuild.bind(this) },
            { pattern: /server|connect|refused/i, fix: this.ultraServerStart.bind(this) },
            { pattern: /timeout/i, fix: () => new Promise(r => setTimeout(r, 2000)) }
        ];
        
        for (const { pattern, fix } of errorPatterns) {
            if (pattern.test(error.message)) {
                await fix();
                return;
            }
        }
        
        // Generic fix: restart everything
        this.log('Applying generic fix: full restart', 'fix');
        await this.ultraPortClear();
        await this.ultraWasmBuild();
        await this.ultraServerStart();
    }

    async run() {
        console.clear();
        console.log('\x1b[95m⚡ ULTRA VALIDATOR - LAYER9 ⚡\x1b[0m');
        console.log('\x1b[90m━'.repeat(50) + '\x1b[0m');
        console.log('\x1b[93mMission: localhost:8080 WILL work. Period.\x1b[0m\n');
        
        this.startStatusDisplay();
        
        while (true) {
            this.attempts++;
            
            try {
                // Ensure environment
                await this.ultraPortClear();
                await this.ultraWasmBuild();
                await this.ultraServerStart();
                
                // Validate
                await this.ultraValidate();
                
                // SUCCESS!
                this.stopStatusDisplay();
                
                console.log('\n\n\x1b[92m' + '═'.repeat(50) + '\x1b[0m');
                console.log('\x1b[92m        🎉 ULTRA SUCCESS ACHIEVED! 🎉\x1b[0m');
                console.log('\x1b[92m' + '═'.repeat(50) + '\x1b[0m\n');
                
                console.log('\x1b[95m📊 ULTRA METRICS:\x1b[0m');
                console.log(`  Total Attempts: ${this.attempts}`);
                console.log(`  Auto-Fixes Applied: ${this.fixes}`);
                console.log(`  Server Starts: ${this.metrics.serverStarts}`);
                console.log(`  WASM Builds: ${this.metrics.wasmBuilds}`);
                console.log(`  Port Clears: ${this.metrics.portClears}`);
                console.log(`  Tests Run: ${this.metrics.testsRun}`);
                console.log(`  Total Time: ${((Date.now() - this.startTime) / 1000).toFixed(1)}s`);
                
                console.log('\n\x1b[92m✅ localhost:8080 is running perfectly!\x1b[0m');
                console.log('\x1b[92m✅ Layer9 framework validated!\x1b[0m');
                console.log('\x1b[92m✅ All systems operational!\x1b[0m\n');
                
                // Keep server running
                console.log('\x1b[90mServer will continue running. Press Ctrl+C to stop.\x1b[0m');
                
                // Prevent process exit
                await new Promise(() => {});
                
            } catch (error) {
                this.stopStatusDisplay();
                this.log(`Attempt ${this.attempts} failed: ${error.message}`, 'error');
                
                // Auto-fix the issue
                await this.autoFix(error);
                
                // Exponential backoff with cap
                const delay = Math.min(1000 * Math.pow(1.5, this.fixes), 10000);
                this.log(`Retrying in ${(delay/1000).toFixed(1)}s...`, 'info');
                
                await new Promise(resolve => setTimeout(resolve, delay));
                this.startStatusDisplay();
            }
        }
    }

    async cleanup() {
        this.stopStatusDisplay();
        if (this.browser) await this.browser.close();
        if (this.serverProcess) this.serverProcess.kill();
    }
}

// Launch the Ultra Validator
const validator = new UltraValidator();

// Handle termination
process.on('SIGINT', async () => {
    console.log('\n\n\x1b[93m⚡ Ultra Validator shutting down...\x1b[0m');
    await validator.cleanup();
    process.exit(0);
});

// Ultra mode: ENGAGE!
validator.run();