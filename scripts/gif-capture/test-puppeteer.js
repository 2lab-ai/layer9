const puppeteer = require('puppeteer');

async function test() {
    console.log('Testing Puppeteer...');
    
    try {
        const browser = await puppeteer.launch({
            headless: true,
            args: [
                '--no-sandbox',
                '--disable-setuid-sandbox',
                '--disable-dev-shm-usage'
            ]
        });
        
        console.log('✅ Browser launched successfully');
        
        const page = await browser.newPage();
        console.log('✅ New page created');
        
        await page.goto('http://localhost:8080');
        console.log('✅ Navigated to page');
        
        const title = await page.title();
        console.log('✅ Page title:', title);
        
        await browser.close();
        console.log('✅ Browser closed');
        
    } catch (error) {
        console.error('❌ Error:', error.message);
        console.error('Stack:', error.stack);
    }
}

test();