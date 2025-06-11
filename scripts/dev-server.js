#!/usr/bin/env node
/**
 * Layer9 Development Server with Automatic Port Management
 * 
 * Features:
 * - Automatic port conflict resolution
 * - Process cleanup on exit
 * - Live reload capability
 * - Health monitoring
 */

const { spawn, exec } = require('child_process');
const http = require('http');
const path = require('path');
const fs = require('fs');

const CONFIG = {
    DEFAULT_PORT: 8080,
    MAX_PORT_ATTEMPTS: 10,
    HEALTH_CHECK_INTERVAL: 5000,
    SERVER_STARTUP_TIMEOUT: 10000
};

class Layer9DevServer {
    constructor() {
        this.port = CONFIG.DEFAULT_PORT;
        this.serverProcess = null;
        this.healthCheckInterval = null;
        this.isHealthy = false;
    }

    log(level, message) {
        const colors = {
            info: '\x1b[36m',
            success: '\x1b[32m', 
            warn: '\x1b[33m',
            error: '\x1b[31m'
        };
        const icons = {
            info: 'ℹ',
            success: '✓',
            warn: '⚠',
            error: '✗'
        };
        console.log(`${colors[level]}${icons[level]} ${message}\x1b[0m`);
    }

    async findAvailablePort() {
        for (let i = 0; i < CONFIG.MAX_PORT_ATTEMPTS; i++) {
            const testPort = CONFIG.DEFAULT_PORT + i;
            const isAvailable = await this.isPortAvailable(testPort);
            
            if (isAvailable) {
                return testPort;
            } else {
                this.log('warn', `Port ${testPort} is in use`);
                
                // Try to identify what's using it
                const processInfo = await this.getPortProcess(testPort);
                if (processInfo) {
                    this.log('info', `Port ${testPort} used by: ${processInfo}`);
                    
                    // Ask user if they want to kill it
                    if (i === 0) { // Only ask for default port
                        const shouldKill = await this.promptUser(
                            `Kill process on port ${testPort}? (y/n): `
                        );
                        if (shouldKill) {
                            await this.killPortProcess(testPort);
                            this.log('success', `Killed process on port ${testPort}`);
                            return testPort;
                        }
                    }
                }
            }
        }
        throw new Error('No available ports found');
    }

    isPortAvailable(port) {
        return new Promise((resolve) => {
            const server = http.createServer();
            server.listen(port, () => {
                server.close(() => resolve(true));
            });
            server.on('error', () => resolve(false));
        });
    }

    getPortProcess(port) {
        return new Promise((resolve) => {
            exec(`lsof -i :${port} | grep LISTEN | awk '{print $1, $2}'`, (error, stdout) => {
                resolve(error ? null : stdout.trim());
            });
        });
    }

    killPortProcess(port) {
        return new Promise((resolve, reject) => {
            exec(`lsof -ti:${port} | xargs kill -9`, (error) => {
                if (error && error.code !== 1) {
                    reject(error);
                } else {
                    resolve();
                }
            });
        });
    }

    promptUser(question) {
        return new Promise((resolve) => {
            const readline = require('readline').createInterface({
                input: process.stdin,
                output: process.stdout
            });
            
            readline.question(question, (answer) => {
                readline.close();
                resolve(answer.toLowerCase() === 'y');
            });
        });
    }

    async buildWasm() {
        this.log('info', 'Building WASM package...');
        
        return new Promise((resolve, reject) => {
            const build = spawn('npm', ['run', 'build:example'], {
                stdio: 'inherit'
            });

            build.on('close', (code) => {
                if (code === 0) {
                    this.log('success', 'WASM build completed');
                    resolve();
                } else {
                    reject(new Error(`Build failed with code ${code}`));
                }
            });
        });
    }

    async startServer() {
        this.port = await this.findAvailablePort();
        this.log('info', `Starting server on port ${this.port}...`);

        return new Promise((resolve, reject) => {
            const startTime = Date.now();
            
            this.serverProcess = spawn('python3', ['-m', 'http.server', this.port.toString()], {
                cwd: path.join(process.cwd(), 'examples', 'counter'),
                stdio: 'pipe'
            });

            this.serverProcess.stdout.on('data', (data) => {
                console.log(`\x1b[90m[server] ${data.toString().trim()}\x1b[0m`);
            });

            this.serverProcess.stderr.on('data', (data) => {
                console.error(`\x1b[91m[server] ${data.toString().trim()}\x1b[0m`);
            });

            this.serverProcess.on('error', reject);

            // Check if server is ready
            const checkServer = setInterval(async () => {
                try {
                    const response = await fetch(`http://localhost:${this.port}`);
                    if (response.ok) {
                        clearInterval(checkServer);
                        const startupTime = Date.now() - startTime;
                        this.log('success', `Server ready in ${startupTime}ms`);
                        this.log('success', `Layer9 running at http://localhost:${this.port}`);
                        this.isHealthy = true;
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

    startHealthMonitoring() {
        this.healthCheckInterval = setInterval(async () => {
            try {
                const response = await fetch(`http://localhost:${this.port}`);
                if (!response.ok) {
                    this.isHealthy = false;
                    this.log('warn', 'Server health check failed');
                } else if (!this.isHealthy) {
                    this.isHealthy = true;
                    this.log('success', 'Server health restored');
                }
            } catch (error) {
                if (this.isHealthy) {
                    this.isHealthy = false;
                    this.log('error', 'Server is down');
                }
            }
        }, CONFIG.HEALTH_CHECK_INTERVAL);
    }

    setupCleanup() {
        const cleanup = () => {
            this.log('info', 'Shutting down...');
            
            if (this.healthCheckInterval) {
                clearInterval(this.healthCheckInterval);
            }
            
            if (this.serverProcess) {
                this.serverProcess.kill();
                this.log('success', 'Server stopped');
            }
            
            process.exit(0);
        };

        process.on('SIGINT', cleanup);
        process.on('SIGTERM', cleanup);
        process.on('exit', cleanup);
    }

    async run() {
        try {
            console.log('\x1b[36m╔══════════════════════════════════════╗\x1b[0m');
            console.log('\x1b[36m║      Layer9 Development Server       ║\x1b[0m');
            console.log('\x1b[36m╚══════════════════════════════════════╝\x1b[0m\n');

            this.setupCleanup();

            // Build WASM
            await this.buildWasm();

            // Start server
            await this.startServer();

            // Start health monitoring
            this.startHealthMonitoring();

            console.log('\n\x1b[90mPress Ctrl+C to stop the server\x1b[0m\n');

            // Keep process alive
            await new Promise(() => {});

        } catch (error) {
            this.log('error', `Fatal error: ${error.message}`);
            process.exit(1);
        }
    }
}

// Run the dev server
if (require.main === module) {
    const server = new Layer9DevServer();
    server.run();
}

module.exports = Layer9DevServer;