#!/usr/bin/env node

/**
 * Layer9 Production Stress Test
 * 
 * Simulates real-world production conditions:
 * - Slow network conditions
 * - Multiple concurrent users
 * - Edge cases and error scenarios
 * - Extended operation periods
 */

const puppeteer = require('puppeteer');

class ProductionStressTest {
    constructor() {
        this.url = 'http://localhost:8080';
        this.scenarios = [];
    }

    async run() {
        console.log('\n🔥 LAYER9 PRODUCTION STRESS TEST\n');
        
        // Run all scenarios
        await this.testSlowNetwork();
        await this.testConcurrentUsers();
        await this.testEdgeCases();
        await this.testLongRunningSession();
        await this.testErrorRecovery();
        
        // Report results
        this.report();
    }

    async testSlowNetwork() {
        const scenario = 'Slow 3G Network';
        console.log(`\n📡 Testing: ${scenario}`);
        
        const browser = await puppeteer.launch({ headless: true });
        try {
            const page = await browser.newPage();
            
            // Simulate slow 3G
            const client = await page.target().createCDPSession();
            await client.send('Network.emulateNetworkConditions', {
                offline: false,
                downloadThroughput: 50 * 1024 / 8, // 50kb/s
                uploadThroughput: 50 * 1024 / 8,
                latency: 2000 // 2 second latency
            });
            
            const start = Date.now();
            await page.goto(this.url, { waitUntil: 'networkidle0', timeout: 30000 });
            const loadTime = Date.now() - start;
            
            // Check if app still works
            await page.waitForSelector('.layer9-app', { timeout: 10000 });
            await page.click('button.btn-primary');
            await new Promise(resolve => setTimeout(resolve, 100));
            
            const counter = await page.$eval('#counter-display', el => el.textContent);
            
            if (counter === 'Count: 1' && loadTime < 30000) {
                this.scenarios.push({ name: scenario, status: 'PASSED', metrics: { loadTime: `${loadTime}ms` } });
                console.log(`✅ PASSED - Load time: ${loadTime}ms`);
            } else {
                throw new Error('Failed under slow network');
            }
        } catch (error) {
            this.scenarios.push({ name: scenario, status: 'FAILED', error: error.message });
            console.log(`❌ FAILED - ${error.message}`);
        } finally {
            await browser.close();
        }
    }

    async testConcurrentUsers() {
        const scenario = 'Concurrent Users';
        const userCount = 10;
        console.log(`\n👥 Testing: ${scenario} (${userCount} users)`);
        
        try {
            const browsers = await Promise.all(
                Array(userCount).fill(0).map(() => puppeteer.launch({ headless: true }))
            );
            
            const results = await Promise.all(
                browsers.map(async (browser, index) => {
                    const page = await browser.newPage();
                    await page.goto(this.url);
                    
                    // Each user performs different operations
                    for (let i = 0; i <= index; i++) {
                        await page.click('button.btn-primary');
                        await new Promise(resolve => setTimeout(resolve, 50));
                    }
                    
                    const value = await page.$eval('#counter-display', el => el.textContent);
                    return { user: index + 1, finalValue: value };
                })
            );
            
            // Cleanup
            await Promise.all(browsers.map(b => b.close()));
            
            const allCorrect = results.every(r => r.finalValue === `Count: ${r.user}`);
            
            if (allCorrect) {
                this.scenarios.push({ name: scenario, status: 'PASSED', metrics: { users: userCount } });
                console.log(`✅ PASSED - All ${userCount} users handled correctly`);
            } else {
                throw new Error('Concurrent user handling failed');
            }
        } catch (error) {
            this.scenarios.push({ name: scenario, status: 'FAILED', error: error.message });
            console.log(`❌ FAILED - ${error.message}`);
        }
    }

    async testEdgeCases() {
        const scenario = 'Edge Cases';
        console.log(`\n🔧 Testing: ${scenario}`);
        
        const browser = await puppeteer.launch({ headless: true });
        try {
            const page = await browser.newPage();
            await page.goto(this.url);
            
            const tests = [];
            
            // Test 1: Extreme values
            await page.click('button.btn-warning'); // Reset
            for (let i = 0; i < 1000; i++) {
                await page.click('button.btn-primary');
            }
            const largeValue = await page.$eval('#counter-display', el => el.textContent);
            tests.push({ test: 'Large values', passed: largeValue === 'Count: 1000' });
            
            // Test 2: Negative extreme
            await page.click('button.btn-warning'); // Reset
            for (let i = 0; i < 500; i++) {
                await page.click('button.btn-secondary');
            }
            const negativeValue = await page.$eval('#counter-display', el => el.textContent);
            tests.push({ test: 'Negative values', passed: negativeValue === 'Count: -500' });
            
            // Test 3: Rapid state changes
            await page.click('button.btn-warning');
            const clicks = [];
            for (let i = 0; i < 100; i++) {
                clicks.push(page.click(i % 3 === 0 ? 'button.btn-primary' : 
                          i % 3 === 1 ? 'button.btn-secondary' : 'button.btn-warning'));
            }
            await Promise.all(clicks);
            await new Promise(resolve => setTimeout(resolve, 500));
            
            const finalState = await page.$eval('#counter-display', el => el.textContent);
            tests.push({ test: 'Rapid state changes', passed: finalState === 'Count: 0' });
            
            const allPassed = tests.every(t => t.passed);
            
            if (allPassed) {
                this.scenarios.push({ name: scenario, status: 'PASSED', metrics: { tests: tests.length } });
                console.log(`✅ PASSED - All edge cases handled`);
            } else {
                throw new Error(`Failed tests: ${tests.filter(t => !t.passed).map(t => t.test).join(', ')}`);
            }
        } catch (error) {
            this.scenarios.push({ name: scenario, status: 'FAILED', error: error.message });
            console.log(`❌ FAILED - ${error.message}`);
        } finally {
            await browser.close();
        }
    }

    async testLongRunningSession() {
        const scenario = 'Long Running Session';
        const durationMinutes = 0.5; // 30 seconds for quick test
        console.log(`\n⏱️  Testing: ${scenario} (${durationMinutes} minutes)`);
        
        const browser = await puppeteer.launch({ headless: true });
        try {
            const page = await browser.newPage();
            await page.goto(this.url);
            
            const startHeap = (await page.metrics()).JSHeapUsedSize;
            const startTime = Date.now();
            let operations = 0;
            
            // Run for specified duration
            while ((Date.now() - startTime) < durationMinutes * 60 * 1000) {
                // Random operations
                const action = Math.floor(Math.random() * 3);
                if (action === 0) await page.click('button.btn-primary');
                else if (action === 1) await page.click('button.btn-secondary');
                else await page.click('button.btn-warning');
                
                operations++;
                await new Promise(resolve => setTimeout(resolve, 100));
            }
            
            const endHeap = (await page.metrics()).JSHeapUsedSize;
            const heapGrowthMB = (endHeap - startHeap) / 1024 / 1024;
            
            // Check if still responsive
            await page.click('button.btn-warning');
            await new Promise(resolve => setTimeout(resolve, 100));
            const finalValue = await page.$eval('#counter-display', el => el.textContent);
            
            if (finalValue === 'Count: 0' && heapGrowthMB < 20) {
                this.scenarios.push({ 
                    name: scenario, 
                    status: 'PASSED', 
                    metrics: { 
                        operations, 
                        heapGrowth: `${heapGrowthMB.toFixed(2)}MB`,
                        duration: `${durationMinutes}min`
                    } 
                });
                console.log(`✅ PASSED - ${operations} operations, heap growth: ${heapGrowthMB.toFixed(2)}MB`);
            } else {
                throw new Error('Performance degradation detected');
            }
        } catch (error) {
            this.scenarios.push({ name: scenario, status: 'FAILED', error: error.message });
            console.log(`❌ FAILED - ${error.message}`);
        } finally {
            await browser.close();
        }
    }

    async testErrorRecovery() {
        const scenario = 'Error Recovery';
        console.log(`\n🔄 Testing: ${scenario}`);
        
        const browser = await puppeteer.launch({ headless: true });
        try {
            const page = await browser.newPage();
            
            // Test recovery from various error conditions
            const tests = [];
            
            // Test 1: Page reload recovery
            await page.goto(this.url);
            await page.click('button.btn-primary');
            await page.reload();
            await page.waitForSelector('.layer9-app');
            const afterReload = await page.$eval('#counter-display', el => el.textContent);
            tests.push({ test: 'Reload recovery', passed: afterReload === 'Count: 0' });
            
            // Test 2: Navigation recovery
            await page.goto('about:blank');
            await page.goto(this.url);
            await page.waitForSelector('.layer9-app');
            const afterNav = await page.$eval('#counter-display', el => el.textContent);
            tests.push({ test: 'Navigation recovery', passed: afterNav === 'Count: 0' });
            
            // Test 3: Multiple tab handling
            const page2 = await browser.newPage();
            await page2.goto(this.url);
            await page2.click('button.btn-primary');
            await page.bringToFront();
            await page.click('button.btn-primary');
            const tab1Value = await page.$eval('#counter-display', el => el.textContent);
            const tab2Value = await page2.$eval('#counter-display', el => el.textContent);
            tests.push({ 
                test: 'Multiple tabs', 
                passed: tab1Value === 'Count: 1' && tab2Value === 'Count: 1' 
            });
            
            const allPassed = tests.every(t => t.passed);
            
            if (allPassed) {
                this.scenarios.push({ name: scenario, status: 'PASSED', metrics: { tests: tests.length } });
                console.log(`✅ PASSED - All recovery scenarios handled`);
            } else {
                throw new Error(`Failed: ${tests.filter(t => !t.passed).map(t => t.test).join(', ')}`);
            }
        } catch (error) {
            this.scenarios.push({ name: scenario, status: 'FAILED', error: error.message });
            console.log(`❌ FAILED - ${error.message}`);
        } finally {
            await browser.close();
        }
    }

    report() {
        console.log('\n' + '='.repeat(60));
        console.log('📊 PRODUCTION STRESS TEST RESULTS\n');
        
        const passed = this.scenarios.filter(s => s.status === 'PASSED').length;
        const failed = this.scenarios.filter(s => s.status === 'FAILED').length;
        const total = this.scenarios.length;
        
        console.log(`Total Scenarios: ${total}`);
        console.log(`Passed: \x1b[32m${passed}\x1b[0m`);
        console.log(`Failed: \x1b[31m${failed}\x1b[0m`);
        console.log(`Success Rate: ${((passed / total) * 100).toFixed(1)}%`);
        
        console.log('\nScenario Details:');
        this.scenarios.forEach(s => {
            const icon = s.status === 'PASSED' ? '✅' : '❌';
            console.log(`\n${icon} ${s.name}: ${s.status}`);
            if (s.metrics) {
                Object.entries(s.metrics).forEach(([key, value]) => {
                    console.log(`   ${key}: ${value}`);
                });
            }
            if (s.error) {
                console.log(`   Error: ${s.error}`);
            }
        });
        
        console.log('\n' + '='.repeat(60));
        
        if (failed === 0) {
            console.log('\n🎉 LAYER9 PASSED ALL PRODUCTION STRESS TESTS!\n');
            console.log('The framework is ready for production deployment.');
        } else {
            console.log('\n⚠️  Some scenarios failed. Review and fix before production.\n');
        }
        
        process.exit(failed > 0 ? 1 : 0);
    }
}

// Run stress test
(async () => {
    const test = new ProductionStressTest();
    await test.run();
})();