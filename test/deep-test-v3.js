#!/usr/bin/env node

/**
 * Deep Component Testing for Layer9 - Fixed Version
 * Tests the exact DOM structure and functionality
 */

const puppeteer = require('puppeteer');

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function deepTest() {
    console.log('🔬 DEEP COMPONENT TESTING FOR LAYER9\n');
    
    const browser = await puppeteer.launch({ 
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    let passed = 0;
    let failed = 0;
    
    function assert(condition, message) {
        if (condition) {
            console.log(`   ✅ ${message}`);
            passed++;
        } else {
            console.log(`   ❌ ${message}`);
            failed++;
        }
    }
    
    try {
        const page = await browser.newPage();
        
        // Enable console logging
        page.on('console', msg => {
            const type = msg.type();
            if (type === 'log' || type === 'info') {
                console.log(`[Browser] ${msg.text()}`);
            }
        });
        
        page.on('pageerror', error => {
            console.log(`[Error] ${error.toString()}`);
        });
        
        // Navigate
        console.log('1. Loading page...');
        const response = await page.goto('http://localhost:8080', {
            waitUntil: 'networkidle0',
            timeout: 30000
        });
        
        assert(response.status() === 200, `Page loaded with status ${response.status()}`);
        
        // Wait for initialization
        await sleep(2000);
        
        // Take screenshot
        await page.screenshot({ path: 'test-initial.png' });
        console.log('   Screenshot: test-initial.png\n');
        
        // Check root element
        console.log('2. Checking DOM structure...');
        const hasRoot = await page.$('#root') !== null;
        assert(hasRoot, '#root element exists');
        
        // Check app container
        const hasApp = await page.$('.layer9-app') !== null;
        assert(hasApp, '.layer9-app container exists');
        
        // Check title
        const title = await page.$eval('h1', el => el.textContent).catch(() => null);
        assert(title === 'Layer9 Counter', `Title is "${title}"`);
        
        // Check counter display
        const counterText = await page.$eval('.counter-value', el => el.textContent).catch(() => null);
        assert(counterText === 'Count: 0', `Initial counter is "${counterText}"`);
        
        // Check buttons
        const buttons = await page.$$eval('button', btns => btns.map(b => b.textContent));
        assert(buttons.includes('Increment'), 'Increment button exists');
        assert(buttons.includes('Decrement'), 'Decrement button exists');
        assert(buttons.includes('Reset'), 'Reset button exists');
        
        // Functional tests
        console.log('\n3. Testing functionality...');
        
        // Find buttons by class
        const incrementBtn = await page.$('.btn.btn-primary');
        const decrementBtn = await page.$('.btn.btn-secondary');
        const resetBtn = await page.$('.btn.btn-warning');
        
        assert(incrementBtn !== null, 'Found increment button by class');
        assert(decrementBtn !== null, 'Found decrement button by class');
        assert(resetBtn !== null, 'Found reset button by class');
        
        // Test increment
        await incrementBtn.click();
        await sleep(500);
        const afterInc = await page.$eval('.counter-value', el => el.textContent);
        assert(afterInc === 'Count: 1', `After increment: "${afterInc}"`);
        
        // Test multiple increments
        await incrementBtn.click();
        await incrementBtn.click();
        await sleep(500);
        const afterMultiInc = await page.$eval('.counter-value', el => el.textContent);
        assert(afterMultiInc === 'Count: 3', `After 3 increments: "${afterMultiInc}"`);
        
        // Test decrement
        await decrementBtn.click();
        await sleep(500);
        const afterDec = await page.$eval('.counter-value', el => el.textContent);
        assert(afterDec === 'Count: 2', `After decrement: "${afterDec}"`);
        
        // Test reset
        await resetBtn.click();
        await sleep(500);
        const afterReset = await page.$eval('.counter-value', el => el.textContent);
        assert(afterReset === 'Count: 0', `After reset: "${afterReset}"`);
        
        // Stress test
        console.log('\n4. Stress testing...');
        for (let i = 0; i < 10; i++) {
            await incrementBtn.click();
        }
        await sleep(500);
        const afterStress = await page.$eval('.counter-value', el => el.textContent);
        assert(afterStress === 'Count: 10', `After 10 increments: "${afterStress}"`);
        
        // Get full DOM structure
        console.log('\n5. DOM Structure:');
        const domStructure = await page.evaluate(() => {
            function getStructure(element, depth = 0) {
                const indent = '  '.repeat(depth);
                let result = `${indent}<${element.tagName}`;
                
                if (element.id) result += ` id="${element.id}"`;
                if (element.className) result += ` class="${element.className}"`;
                
                result += '>';
                
                if (element.children.length === 0 && element.textContent) {
                    result += element.textContent.trim();
                }
                
                result += `</${element.tagName}>`;
                
                return result;
            }
            
            function traverseDOM(element, depth = 0) {
                let structure = getStructure(element, depth) + '\n';
                
                for (const child of element.children) {
                    structure += traverseDOM(child, depth + 1);
                }
                
                return structure;
            }
            
            const root = document.getElementById('root');
            return root ? traverseDOM(root) : 'Root not found';
        });
        console.log(domStructure);
        
        // Final screenshot
        await page.screenshot({ path: 'test-final.png' });
        console.log('   Screenshot: test-final.png\n');
        
    } catch (error) {
        console.error('\n❌ Test error:', error.message);
        failed++;
    } finally {
        console.log('='.repeat(50));
        console.log('📊 TEST RESULTS\n');
        console.log(`Total checks: ${passed + failed}`);
        console.log(`Passed: ${passed}`);
        console.log(`Failed: ${failed}`);
        console.log(`Success rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);
        console.log('='.repeat(50));
        
        if (failed === 0) {
            console.log('\n✅ ALL TESTS PASSED!\n');
        } else {
            console.log('\n❌ SOME TESTS FAILED\n');
        }
        
        await browser.close();
        process.exit(failed === 0 ? 0 : 1);
    }
}

deepTest().catch(console.error);