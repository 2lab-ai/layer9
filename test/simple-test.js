#!/usr/bin/env node

const puppeteer = require('puppeteer');
const { spawn } = require('child_process');
const http = require('http');

async function waitForServer(url, timeout = 30000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
        try {
            await fetch(url);
            return true;
        } catch (e) {
            await new Promise(resolve => setTimeout(resolve, 100));
        }
    }
    return false;
}

async function runTest() {
    console.log('🧪 Layer9 Simple Test');
    console.log('===================\n');
    
    // Start the Rust server
    console.log('🚀 Starting Rust server...');
    const server = spawn('cargo', [
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
    
    server.stdout.on('data', (data) => {
        console.log(`[server] ${data.toString().trim()}`);
    });
    
    server.stderr.on('data', (data) => {
        console.log(`[server] ${data.toString().trim()}`);
    });
    
    // Wait for server to be ready
    console.log('⏳ Waiting for server to be ready...');
    const serverReady = await waitForServer('http://localhost:8080');
    
    if (!serverReady) {
        console.error('❌ Server failed to start');
        server.kill();
        process.exit(1);
    }
    
    console.log('✅ Server is ready\n');
    
    // Run Puppeteer tests
    console.log('🌐 Starting Puppeteer tests...');
    const browser = await puppeteer.launch({
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    try {
        const page = await browser.newPage();
        
        // Enable console logging
        page.on('console', msg => console.log('  [console]', msg.text()));
        page.on('error', err => console.log('  [error]', err.message));
        
        // Navigate to the page
        console.log('\n📍 Navigating to http://localhost:8080...');
        await page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
        
        // Check if page loaded
        const title = await page.title();
        console.log(`✅ Page loaded: "${title}"`);
        
        // Take a screenshot for debugging
        await page.screenshot({ path: 'test-screenshot.png' });
        console.log('📸 Screenshot saved: test-screenshot.png');
        
        // Check for WASM
        console.log('\n🔍 Checking WASM status...');
        const wasmStatus = await page.evaluate(() => {
            return {
                webAssemblySupported: typeof WebAssembly !== 'undefined',
                hasRoot: !!document.querySelector('#root'),
                loadingVisible: document.querySelector('.loading') && 
                               window.getComputedStyle(document.querySelector('.loading')).display !== 'none',
                bodyHTML: document.body.innerHTML.substring(0, 500)
            };
        });
        
        console.log('WASM Status:', JSON.stringify(wasmStatus, null, 2));
        
        // Wait for app to load
        console.log('\n⏳ Waiting for app to load...');
        try {
            await page.waitForFunction(
                () => {
                    const loading = document.querySelector('.loading');
                    return !loading || window.getComputedStyle(loading).display === 'none';
                },
                { timeout: 30000 }
            );
            console.log('✅ App loaded successfully');
        } catch (e) {
            console.error('❌ App failed to load:', e.message);
            
            // Get current state
            const currentState = await page.evaluate(() => {
                return {
                    loadingElement: document.querySelector('.loading'),
                    loadingDisplay: document.querySelector('.loading') ? 
                                   window.getComputedStyle(document.querySelector('.loading')).display : 'not found',
                    buttons: document.querySelectorAll('button').length,
                    bodyText: document.body.textContent.substring(0, 200)
                };
            });
            console.log('Current state:', JSON.stringify(currentState, null, 2));
        }
        
        // Check for buttons
        console.log('\n🔍 Checking for interactive elements...');
        const buttons = await page.$$('button');
        console.log(`Found ${buttons.length} buttons`);
        
        if (buttons.length > 0) {
            for (let i = 0; i < Math.min(3, buttons.length); i++) {
                const text = await buttons[i].evaluate(el => el.textContent);
                console.log(`  Button ${i + 1}: "${text}"`);
            }
        }
        
        // Test complete
        console.log('\n✅ Test completed');
        
    } catch (error) {
        console.error('\n❌ Test failed:', error.message);
        process.exit(1);
    } finally {
        await browser.close();
        server.kill();
    }
    
    process.exit(0);
}

runTest().catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});