#!/usr/bin/env node

/**
 * Layer9 Validation Test
 * Confirms that Layer9 reactive rendering is working correctly
 */

const puppeteer = require('puppeteer');

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function validateLayer9() {
    console.log('🎯 VALIDATING LAYER9 FRAMEWORK\n');
    
    const browser = await puppeteer.launch({ 
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    const results = {
        pageLoad: false,
        domStructure: false,
        increment: false,
        decrement: false,
        reset: false,
        reactiveUpdates: false
    };
    
    try {
        const page = await browser.newPage();
        
        // Track console logs
        const logs = [];
        page.on('console', msg => {
            logs.push(msg.text());
        });
        
        // Load page
        console.log('1. Loading Layer9 app...');
        const response = await page.goto('http://localhost:8080', {
            waitUntil: 'networkidle0',
            timeout: 10000
        });
        
        results.pageLoad = response.status() === 200;
        console.log(`   Page loaded: ${results.pageLoad ? '✅' : '❌'}`);
        
        await sleep(1000);
        
        // Check DOM structure
        console.log('\n2. Checking DOM structure...');
        const structure = await page.evaluate(() => {
            return {
                root: document.querySelector('#root') !== null,
                app: document.querySelector('.layer9-app') !== null,
                title: document.querySelector('h1')?.textContent || '',
                counter: document.querySelector('.counter-value')?.textContent || '',
                buttons: Array.from(document.querySelectorAll('button')).map(b => b.textContent)
            };
        });
        
        results.domStructure = structure.root && 
                              structure.app && 
                              structure.title === 'Layer9 Counter' &&
                              structure.counter === 'Count: 0' &&
                              structure.buttons.length === 3;
        
        console.log(`   DOM structure: ${results.domStructure ? '✅' : '❌'}`);
        console.log(`   - Root: ${structure.root ? '✓' : '✗'}`);
        console.log(`   - App: ${structure.app ? '✓' : '✗'}`);
        console.log(`   - Title: "${structure.title}"`);
        console.log(`   - Counter: "${structure.counter}"`);
        console.log(`   - Buttons: ${structure.buttons.join(', ')}`);
        
        // Test increment
        console.log('\n3. Testing increment...');
        await page.evaluate(() => {
            document.querySelector('.btn.btn-primary').click();
        });
        await sleep(500);
        
        const afterInc = await page.$eval('.counter-value', el => el.textContent);
        results.increment = afterInc === 'Count: 1';
        console.log(`   Increment: ${results.increment ? '✅' : '❌'} (${afterInc})`);
        
        // Test decrement
        console.log('\n4. Testing decrement...');
        await page.evaluate(() => {
            document.querySelector('.btn.btn-secondary').click();
        });
        await sleep(500);
        
        const afterDec = await page.$eval('.counter-value', el => el.textContent);
        results.decrement = afterDec === 'Count: 0';
        console.log(`   Decrement: ${results.decrement ? '✅' : '❌'} (${afterDec})`);
        
        // Test reset
        console.log('\n5. Testing reset...');
        // First increment a few times
        for (let i = 0; i < 5; i++) {
            await page.evaluate(() => {
                document.querySelector('.btn.btn-primary').click();
            });
            await sleep(100);
        }
        
        // Then reset
        await page.evaluate(() => {
            document.querySelector('.btn.btn-warning').click();
        });
        await sleep(500);
        
        const afterReset = await page.$eval('.counter-value', el => el.textContent);
        results.reset = afterReset === 'Count: 0';
        console.log(`   Reset: ${results.reset ? '✅' : '❌'} (${afterReset})`);
        
        // Check reactive updates
        console.log('\n6. Checking reactive updates...');
        const changeCount = logs.filter(log => log.startsWith('Count changed to:')).length;
        results.reactiveUpdates = changeCount >= 7; // Initial + 1 inc + 1 dec + 5 inc + 1 reset
        console.log(`   Reactive updates: ${results.reactiveUpdates ? '✅' : '❌'} (${changeCount} changes logged)`);
        
        // Take screenshot
        await page.screenshot({ path: 'layer9-validated.png' });
        console.log('\n   Screenshot saved: layer9-validated.png');
        
    } catch (error) {
        console.error('\n❌ Validation error:', error.message);
    } finally {
        await browser.close();
        
        // Summary
        console.log('\n' + '='.repeat(50));
        console.log('📊 VALIDATION SUMMARY\n');
        
        const allPassed = Object.values(results).every(v => v);
        let passed = 0;
        
        for (const [test, result] of Object.entries(results)) {
            console.log(`   ${test.padEnd(20)} ${result ? '✅ PASS' : '❌ FAIL'}`);
            if (result) passed++;
        }
        
        console.log(`\n   Overall: ${passed}/${Object.keys(results).length} tests passed`);
        console.log('='.repeat(50));
        
        if (allPassed) {
            console.log('\n🎉 LAYER9 IS WORKING PERFECTLY!\n');
            console.log('The reactive rendering system is functioning correctly.');
            console.log('All components update properly when state changes.');
            console.log('Event handlers are correctly bound and re-bound on updates.\n');
        } else {
            console.log('\n⚠️  Some tests failed, but check the details above.\n');
        }
        
        process.exit(allPassed ? 0 : 1);
    }
}

validateLayer9().catch(console.error);