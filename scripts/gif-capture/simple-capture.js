const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

const EXAMPLES = {
    counter: {
        port: 8080,
        name: 'Counter'
    },
    'async-counter': {
        port: 8081,
        name: 'Async Counter'
    },
    'todo-app': {
        port: 8082,
        name: 'Todo App'
    },
    'memory-game': {
        port: 8083,
        name: 'Memory Game'
    }
};

const SCREENSHOT_DIR = path.join(__dirname, '../../assets/screenshots');

function ensureDirectory(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

async function captureExample(page, example, config) {
    console.log(`\n📸 Capturing ${config.name}...`);
    
    await page.goto(`http://localhost:${config.port}`, { waitUntil: 'networkidle2' });
    
    // Wait for loading to complete
    try {
        await page.waitForFunction(
            () => {
                const loading = document.querySelector('.loading, #loading');
                return !loading || loading.style.display === 'none' || loading.offsetParent === null;
            },
            { timeout: 10000 }
        );
    } catch (e) {
        console.log('  ⚠️  Loading check timed out, continuing...');
    }
    
    // Give it a moment to stabilize
    await page.waitForTimeout(2000);
    
    // Take initial screenshot
    const screenshotPath = path.join(SCREENSHOT_DIR, `${example}-01-initial.png`);
    await page.screenshot({ path: screenshotPath });
    console.log(`  ✅ Initial screenshot: ${screenshotPath}`);
    
    // Interact based on example type
    if (example === 'counter') {
        // Find and click increment button
        const buttons = await page.$$('button');
        for (const button of buttons) {
            const text = await button.evaluate(el => el.textContent);
            if (text.includes('Increment')) {
                await button.click();
                await page.waitForTimeout(500);
                const path2 = path.join(SCREENSHOT_DIR, `${example}-02-after-increment.png`);
                await page.screenshot({ path: path2 });
                console.log(`  ✅ After increment: ${path2}`);
                break;
            }
        }
    } else if (example === 'todo-app') {
        // Try to add a todo
        const input = await page.$('input[type="text"], input#todo-input, input.todo-input');
        if (input) {
            await input.type('Test todo item');
            await page.waitForTimeout(500);
            
            // Find add button
            const buttons = await page.$$('button');
            for (const button of buttons) {
                const text = await button.evaluate(el => el.textContent);
                if (text.includes('Add') || text === '+') {
                    await button.click();
                    await page.waitForTimeout(500);
                    const path2 = path.join(SCREENSHOT_DIR, `${example}-02-with-todo.png`);
                    await page.screenshot({ path: path2 });
                    console.log(`  ✅ With todo: ${path2}`);
                    break;
                }
            }
        }
    } else if (example === 'memory-game') {
        // Click some cards
        const cards = await page.$$('.card, .memory-card, .game-card');
        if (cards.length > 1) {
            await cards[0].click();
            await page.waitForTimeout(500);
            const path2 = path.join(SCREENSHOT_DIR, `${example}-02-card-flipped.png`);
            await page.screenshot({ path: path2 });
            console.log(`  ✅ Card flipped: ${path2}`);
        }
    } else if (example === 'async-counter') {
        // Find async increment button
        const buttons = await page.$$('button');
        for (const button of buttons) {
            const text = await button.evaluate(el => el.textContent);
            if (text.includes('Async') && text.includes('Increment')) {
                await button.click();
                await page.waitForTimeout(100);
                const path2 = path.join(SCREENSHOT_DIR, `${example}-02-loading.png`);
                await page.screenshot({ path: path2 });
                console.log(`  ✅ Loading state: ${path2}`);
                
                await page.waitForTimeout(2000);
                const path3 = path.join(SCREENSHOT_DIR, `${example}-03-completed.png`);
                await page.screenshot({ path: path3 });
                console.log(`  ✅ Completed: ${path3}`);
                break;
            }
        }
    }
}

async function captureAll() {
    console.log('🎬 Layer9 Simple Screenshot Capture');
    console.log('==================================\n');
    
    ensureDirectory(SCREENSHOT_DIR);
    
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
        
        // Enable console logging for debugging
        page.on('console', msg => {
            if (msg.type() === 'error') {
                console.log('  ❌ Page error:', msg.text());
            }
        });
        
        for (const [example, config] of Object.entries(EXAMPLES)) {
            try {
                await captureExample(page, example, config);
            } catch (error) {
                console.error(`  ❌ Error capturing ${config.name}:`, error.message);
            }
        }
        
        console.log('\n✅ Capture complete!');
        console.log(`📁 Screenshots saved to: ${SCREENSHOT_DIR}`);
        
    } catch (error) {
        console.error('Fatal error:', error);
    } finally {
        await browser.close();
    }
}

captureAll();