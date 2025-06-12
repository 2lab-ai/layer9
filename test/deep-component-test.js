#!/usr/bin/env node

/**
 * Deep Component Testing for Layer9
 * 
 * This test verifies the exact DOM structure that should be rendered
 * by the counter example and checks each component systematically.
 */

const puppeteer = require('puppeteer');

// Expected DOM structure based on the counter example code
const EXPECTED_STRUCTURE = {
    root: {
        selector: '#root',
        children: {
            app: {
                selector: '.layer9-app',
                tag: 'DIV',
                children: {
                    style: {
                        tag: 'STYLE',
                        hasContent: true
                    },
                    title: {
                        tag: 'H1',
                        text: 'Layer9 Counter'
                    },
                    counterValue: {
                        selector: '.counter-value',
                        tag: 'P',
                        textPattern: /^Count: \d+$/
                    },
                    buttonContainer: {
                        selector: '.button-container',
                        tag: 'DIV',
                        children: {
                            incrementBtn: {
                                selector: '.btn.btn-primary',
                                tag: 'BUTTON',
                                text: 'Increment'
                            },
                            decrementBtn: {
                                selector: '.btn.btn-secondary',
                                tag: 'BUTTON',
                                text: 'Decrement'
                            },
                            resetBtn: {
                                selector: '.btn.btn-warning',
                                tag: 'BUTTON',
                                text: 'Reset'
                            }
                        }
                    },
                    info: {
                        selector: '.info',
                        tag: 'P',
                        containsText: ['Built with', 'Layer9', 'Reactive Rust Web Framework']
                    }
                }
            }
        }
    }
};

async function deepComponentTest() {
    console.log('🔬 DEEP COMPONENT TESTING FOR LAYER9\n');
    
    const browser = await puppeteer.launch({ 
        headless: false, // Show browser for debugging
        devtools: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    let passed = 0;
    let failed = 0;
    
    try {
        const page = await browser.newPage();
        
        // Enable detailed console logging
        page.on('console', msg => {
            console.log(`[Browser Console] ${msg.type()}: ${msg.text()}`);
        });
        
        page.on('pageerror', error => {
            console.log(`[Page Error] ${error.toString()}`);
        });
        
        // Navigate to the app
        console.log('1. Loading page...');
        const response = await page.goto('http://localhost:8080', {
            waitUntil: 'networkidle0',
            timeout: 30000
        });
        
        console.log(`   Response status: ${response.status()}`);
        console.log(`   Response URL: ${response.url()}`);
        
        // Wait for any async initialization
        await page.waitForTimeout(2000);
        
        // Take initial screenshot
        await page.screenshot({ path: 'deep-test-initial.png' });
        console.log('   Screenshot saved: deep-test-initial.png\n');
        
        // 2. Verify root element
        console.log('2. Checking root element...');
        const hasRoot = await page.$('#root') !== null;
        if (hasRoot) {
            console.log('   ✅ #root exists');
            passed++;
        } else {
            console.log('   ❌ #root NOT FOUND');
            failed++;
        }
        
        // 3. Check if WASM initialized
        console.log('\n3. Checking WASM initialization...');
        const wasmCheck = await page.evaluate(() => {
            return {
                hasWebAssembly: typeof WebAssembly !== 'undefined',
                hasWasmModule: typeof window.__wasm_module !== 'undefined'
            };
        });
        console.log(`   WebAssembly available: ${wasmCheck.hasWebAssembly}`);
        console.log(`   WASM module loaded: ${wasmCheck.hasWasmModule}`);
        
        // 4. Deep DOM structure check
        console.log('\n4. Deep DOM structure verification...');
        
        async function checkElement(path, expected, parentSelector = '') {
            const fullSelector = parentSelector + (expected.selector || expected.tag || '');
            const indent = '   ' + '  '.repeat(path.length);
            
            console.log(`${indent}Checking: ${path.join(' > ')}`);
            
            if (expected.selector) {
                const element = await page.$(fullSelector);
                if (!element) {
                    console.log(`${indent}❌ Element not found: ${fullSelector}`);
                    failed++;
                    return false;
                }
                
                // Check tag name
                if (expected.tag) {
                    const tagName = await page.evaluate(el => el.tagName, element);
                    if (tagName === expected.tag) {
                        console.log(`${indent}✅ Tag: ${tagName}`);
                        passed++;
                    } else {
                        console.log(`${indent}❌ Tag mismatch: expected ${expected.tag}, got ${tagName}`);
                        failed++;
                    }
                }
                
                // Check text content
                if (expected.text) {
                    const text = await page.evaluate(el => el.textContent.trim(), element);
                    if (text === expected.text) {
                        console.log(`${indent}✅ Text: "${text}"`);
                        passed++;
                    } else {
                        console.log(`${indent}❌ Text mismatch: expected "${expected.text}", got "${text}"`);
                        failed++;
                    }
                }
                
                // Check text pattern
                if (expected.textPattern) {
                    const text = await page.evaluate(el => el.textContent.trim(), element);
                    if (expected.textPattern.test(text)) {
                        console.log(`${indent}✅ Text matches pattern: "${text}"`);
                        passed++;
                    } else {
                        console.log(`${indent}❌ Text doesn't match pattern: "${text}"`);
                        failed++;
                    }
                }
                
                // Check contains text
                if (expected.containsText) {
                    const text = await page.evaluate(el => el.textContent, element);
                    const hasAll = expected.containsText.every(t => text.includes(t));
                    if (hasAll) {
                        console.log(`${indent}✅ Contains all required text`);
                        passed++;
                    } else {
                        console.log(`${indent}❌ Missing required text`);
                        failed++;
                    }
                }
                
                // Check children
                if (expected.children) {
                    for (const [key, child] of Object.entries(expected.children)) {
                        await checkElement([...path, key], child, fullSelector + ' ');
                    }
                }
                
                return true;
            }
            
            return false;
        }
        
        // Start recursive check
        await checkElement(['root'], EXPECTED_STRUCTURE.root);
        
        // 5. Functional testing
        console.log('\n5. Functional component testing...');
        
        // Get initial count
        const initialCount = await page.$eval('.counter-value', el => el.textContent);
        console.log(`   Initial count: "${initialCount}"`);
        
        // Test increment
        console.log('   Testing increment...');
        await page.click('.btn.btn-primary');
        await page.waitForTimeout(500);
        const afterInc = await page.$eval('.counter-value', el => el.textContent);
        console.log(`   After increment: "${afterInc}"`);
        if (afterInc === 'Count: 1') {
            console.log('   ✅ Increment works');
            passed++;
        } else {
            console.log('   ❌ Increment failed');
            failed++;
        }
        
        // Test decrement  
        console.log('   Testing decrement...');
        await page.click('.btn.btn-secondary');
        await page.waitForTimeout(500);
        const afterDec = await page.$eval('.counter-value', el => el.textContent);
        console.log(`   After decrement: "${afterDec}"`);
        if (afterDec === 'Count: 0') {
            console.log('   ✅ Decrement works');
            passed++;
        } else {
            console.log('   ❌ Decrement failed');
            failed++;
        }
        
        // Test reset
        console.log('   Testing reset...');
        await page.click('.btn.btn-primary');
        await page.click('.btn.btn-primary');
        await page.waitForTimeout(500);
        await page.click('.btn.btn-warning');
        await page.waitForTimeout(500);
        const afterReset = await page.$eval('.counter-value', el => el.textContent);
        console.log(`   After reset: "${afterReset}"`);
        if (afterReset === 'Count: 0') {
            console.log('   ✅ Reset works');
            passed++;
        } else {
            console.log('   ❌ Reset failed');
            failed++;
        }
        
        // 6. Get full DOM structure for debugging
        console.log('\n6. Current DOM structure:');
        const domStructure = await page.evaluate(() => {
            function getStructure(element, depth = 0) {
                const indent = '  '.repeat(depth);
                let result = `${indent}<${element.tagName}`;
                
                if (element.id) result += ` id="${element.id}"`;
                if (element.className) result += ` class="${element.className}"`;
                
                result += '>';
                
                if (element.children.length === 0 && element.textContent) {
                    result += element.textContent.trim().substring(0, 50);
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
        
        // Take final screenshot
        await page.screenshot({ path: 'deep-test-final.png' });
        console.log('   Screenshot saved: deep-test-final.png\n');
        
    } catch (error) {
        console.error('\n❌ Test error:', error.message);
        failed++;
    } finally {
        console.log('\n' + '='.repeat(50));
        console.log('📊 DEEP TEST RESULTS\n');
        console.log(`Total checks: ${passed + failed}`);
        console.log(`Passed: ${passed}`);
        console.log(`Failed: ${failed}`);
        console.log(`Success rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);
        console.log('='.repeat(50));
        
        if (failed === 0) {
            console.log('\n✅ ALL COMPONENTS VERIFIED!\n');
        } else {
            console.log('\n❌ COMPONENT VERIFICATION FAILED\n');
        }
        
        // Keep browser open for inspection
        console.log('Browser kept open for inspection. Press Ctrl+C to exit.');
        await new Promise(() => {}); // Keep process alive
    }
}

deepComponentTest().catch(console.error);