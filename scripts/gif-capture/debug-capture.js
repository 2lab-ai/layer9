const puppeteer = require('puppeteer');

async function debugCapture() {
    const browser = await puppeteer.launch({
        headless: true,
        args: [
            '--no-sandbox',
            '--disable-setuid-sandbox',
            '--disable-dev-shm-usage'
        ]
    });
    
    try {
        const page = await browser.newPage();
        await page.setViewport({ width: 800, height: 600 });
        
        // Enable console logging
        page.on('console', msg => console.log('PAGE LOG:', msg.text()));
        page.on('error', err => console.log('PAGE ERROR:', err));
        page.on('pageerror', err => console.log('PAGE ERROR:', err));
        
        console.log('\n🔍 Debugging Counter (port 8080)...');
        await page.goto('http://localhost:8080', { waitUntil: 'networkidle2' });
        
        // Take a screenshot immediately
        await page.screenshot({ path: 'debug-counter-immediate.png' });
        console.log('✅ Screenshot saved: debug-counter-immediate.png');
        
        // Get page content
        const content = await page.content();
        console.log('\nPage HTML (first 500 chars):', content.substring(0, 500));
        
        // Check for common selectors
        const selectors = ['#app', '#root', '.app', 'body > div', '.counter'];
        for (const selector of selectors) {
            const element = await page.$(selector);
            if (element) {
                console.log(`✅ Found selector: ${selector}`);
                const text = await page.$eval(selector, el => el.textContent);
                console.log(`   Content: ${text.substring(0, 100)}...`);
            } else {
                console.log(`❌ Not found: ${selector}`);
            }
        }
        
        // Wait a bit for WASM to load
        console.log('\nWaiting 5 seconds for WASM to load...');
        await page.waitForTimeout(5000);
        
        // Take another screenshot
        await page.screenshot({ path: 'debug-counter-after-wait.png' });
        console.log('✅ Screenshot saved: debug-counter-after-wait.png');
        
        // Check content again
        const contentAfter = await page.content();
        console.log('\nPage HTML after wait (first 500 chars):', contentAfter.substring(0, 500));
        
        // Try to find any buttons
        const buttons = await page.$$('button');
        console.log(`\nFound ${buttons.length} buttons`);
        for (let i = 0; i < Math.min(buttons.length, 5); i++) {
            const text = await buttons[i].evaluate(el => el.textContent);
            console.log(`  Button ${i}: ${text}`);
        }
        
    } catch (error) {
        console.error('Error:', error);
    } finally {
        await browser.close();
    }
}

debugCapture();