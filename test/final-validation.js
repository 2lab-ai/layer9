#!/usr/bin/env node

/**
 * Layer9 Final Production Validation
 * 
 * Essential checks for production readiness
 */

const puppeteer = require('puppeteer');

async function validateLayer9() {
    console.log('\n🚀 LAYER9 FINAL VALIDATION\n');
    
    const tests = {
        passed: 0,
        failed: 0,
        startTime: Date.now()
    };
    
    const browser = await puppeteer.launch({ 
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    try {
        const page = await browser.newPage();
        
        // Track errors
        let errors = [];
        page.on('console', msg => {
            if (msg.type() === 'error') errors.push(msg.text());
        });
        page.on('pageerror', error => errors.push(error.toString()));
        
        // 1. Load Test
        console.log('1️⃣  Testing: Page Load');
        const loadStart = Date.now();
        let response;
        try {
            response = await page.goto('http://localhost:8080', {
                waitUntil: 'networkidle0',
                timeout: 10000
            });
        } catch (e) {
            console.log('   ⚠️  Server not running, skipping browser tests');
            tests.failed += 5; // Mark all browser tests as failed
            return;
        }
        const loadTime = Date.now() - loadStart;
        
        if (response.status() === 200 && loadTime < 5000) {
            console.log(`   ✅ Loaded in ${loadTime}ms`);
            tests.passed++;
        } else {
            console.log(`   ❌ Load failed`);
            tests.failed++;
        }
        
        // 2. WASM Test
        console.log('2️⃣  Testing: WASM Module');
        try {
            // Debug: Take screenshot and log page content
            const content = await page.content();
            if (content.includes('404') || content.includes('Not Found')) {
                throw new Error('Page returned 404');
            }
            
            await page.waitForSelector('.layer9-app', { timeout: 5000 });
            
            // Debug: Check if WASM loaded
            const hasWasm = await page.evaluate(() => {
                return typeof WebAssembly !== 'undefined';
            });
            
            if (!hasWasm) {
                throw new Error('WebAssembly not supported');
            }
            
            if (errors.length === 0) {
                console.log('   ✅ WASM initialized without errors');
                tests.passed++;
            } else {
                console.log(`   ⚠️  Console errors: ${errors.join(', ')}`);
                throw new Error(`${errors.length} errors found`);
            }
        } catch (e) {
            console.log(`   ❌ WASM failed: ${e.message}`);
            tests.failed++;
        }
        
        // 3. Functionality Test
        console.log('3️⃣  Testing: Core Functionality');
        try {
            // Wait for counter to be ready
            await page.waitForSelector('.counter-value', { timeout: 3000 });
            
            // Test increment
            await page.click('button.btn-primary');
            await new Promise(r => setTimeout(r, 200));
            const afterInc = await page.$eval('.counter-value', el => el.textContent);
            
            // Test decrement
            await page.click('button.btn-secondary');
            await new Promise(r => setTimeout(r, 200));
            const afterDec = await page.$eval('.counter-value', el => el.textContent);
            
            // Test reset
            await page.click('button.btn-warning');
            await new Promise(r => setTimeout(r, 200));
            const afterReset = await page.$eval('.counter-value', el => el.textContent);
            
            if (afterInc === 'Count: 1' && afterDec === 'Count: 0' && afterReset === 'Count: 0') {
                console.log('   ✅ All functions work correctly');
                tests.passed++;
            } else {
                throw new Error('Function test failed');
            }
        } catch (e) {
            console.log(`   ❌ Functionality failed: ${e.message}`);
            tests.failed++;
        }
        
        // 4. Stress Test
        console.log('4️⃣  Testing: Stress Test (1000 operations)');
        try {
            const startHeap = (await page.metrics()).JSHeapUsedSize;
            
            // Perform 1000 random operations
            for (let i = 0; i < 1000; i++) {
                const buttons = ['button.btn-primary', 'button.btn-secondary', 'button.btn-warning'];
                await page.click(buttons[Math.floor(Math.random() * buttons.length)]);
            }
            
            await new Promise(r => setTimeout(r, 500));
            const endHeap = (await page.metrics()).JSHeapUsedSize;
            const heapGrowthMB = (endHeap - startHeap) / 1024 / 1024;
            
            if (heapGrowthMB < 10 && errors.length === 0) {
                console.log(`   ✅ No crashes, heap growth: ${heapGrowthMB.toFixed(2)}MB`);
                tests.passed++;
            } else {
                throw new Error(`Heap growth: ${heapGrowthMB.toFixed(2)}MB`);
            }
        } catch (e) {
            console.log(`   ❌ Stress test failed: ${e.message}`);
            tests.failed++;
        }
        
        // 5. Performance Metrics
        console.log('5️⃣  Testing: Performance Metrics');
        try {
            const metrics = await page.metrics();
            const performance = await page.evaluate(() => ({
                memory: performance.memory ? performance.memory.usedJSHeapSize / 1024 / 1024 : 0,
                documents: document.querySelectorAll('*').length,
                listeners: Array.from(document.querySelectorAll('*'))
                    .reduce((count, el) => count + (el.onclick ? 1 : 0), 0)
            }));
            
            console.log(`   📊 Memory: ${(metrics.JSHeapUsedSize / 1024 / 1024).toFixed(2)}MB`);
            console.log(`   📊 DOM Nodes: ${performance.documents}`);
            console.log(`   📊 Event Listeners: ${performance.listeners}`);
            
            if (metrics.JSHeapUsedSize / 1024 / 1024 < 20) {
                console.log('   ✅ Performance within limits');
                tests.passed++;
            } else {
                throw new Error('Memory usage too high');
            }
        } catch (e) {
            console.log(`   ❌ Performance test failed: ${e.message}`);
            tests.failed++;
        }
        
    } catch (error) {
        console.error('Fatal error:', error.message);
        tests.failed = 5;
    } finally {
        await browser.close();
    }
    
    // Final Report
    const totalTime = ((Date.now() - tests.startTime) / 1000).toFixed(2);
    const total = tests.passed + tests.failed;
    const passRate = ((tests.passed / total) * 100).toFixed(1);
    
    console.log('\n' + '='.repeat(50));
    console.log('📊 FINAL VALIDATION RESULTS\n');
    console.log(`Total Tests: ${total}`);
    console.log(`Passed: \x1b[32m${tests.passed}\x1b[0m`);
    console.log(`Failed: \x1b[31m${tests.failed}\x1b[0m`);
    console.log(`Pass Rate: ${passRate}%`);
    console.log(`Total Time: ${totalTime}s`);
    console.log('='.repeat(50));
    
    if (tests.failed === 0) {
        console.log('\n✅ LAYER9 IS PRODUCTION READY!\n');
        console.log('🎯 All systems operational');
        console.log('🚀 Ready for deployment');
        console.log('💪 Framework validated\n');
    } else {
        console.log('\n❌ VALIDATION FAILED\n');
        console.log('Please fix the issues and run again.\n');
    }
    
    process.exit(tests.failed > 0 ? 1 : 0);
}

// Run validation
validateLayer9().catch(error => {
    console.error('Validation error:', error);
    process.exit(1);
});