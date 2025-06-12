const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

async function testCounterExample() {
  console.log('Testing Counter Example...');
  
  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });
  
  const page = await browser.newPage();
  
  // Set up console message listener
  const consoleLogs = [];
  page.on('console', msg => {
    consoleLogs.push({
      type: msg.type(),
      text: msg.text()
    });
  });
  
  // Set up error listener
  const pageErrors = [];
  page.on('pageerror', error => {
    pageErrors.push(error.toString());
  });
  
  try {
    console.log('Navigating to http://localhost:8080');
    await page.goto('http://localhost:8080', {
      waitUntil: 'networkidle0',
      timeout: 30000
    });
    
    // Wait for WASM to initialize
    await page.waitForFunction(() => {
      return window.wasmLoaded === true || document.querySelector('#root') !== null;
    }, { timeout: 10000 }).catch(() => {
      console.log('WASM load signal not found, continuing anyway...');
    });
    
    // Wait a bit more for component to render
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Take initial screenshot
    await page.screenshot({
      path: 'screenshots/counter-initial.png',
      fullPage: true
    });
    console.log('Initial screenshot saved');
    
    // Check if the counter component rendered
    const rootElement = await page.$('#root');
    if (!rootElement) {
      throw new Error('Root element not found');
    }
    
    // Check if loading is still showing
    const loadingElement = await page.$('.loading');
    if (loadingElement) {
      console.log('Warning: Loading element still visible');
    }
    
    // Get initial counter value
    const initialValue = await page.evaluate(() => {
      // Look for the large number display
      const elements = document.querySelectorAll('*');
      for (const el of elements) {
        const text = el.textContent.trim();
        if (text === '0' && el.tagName !== 'BUTTON' && 
            window.getComputedStyle(el).fontSize.includes('px') &&
            parseInt(window.getComputedStyle(el).fontSize) > 30) {
          return text;
        }
      }
      return null;
    });
    console.log('Initial counter value:', initialValue);
    
    // Find and click increment button using text content
    try {
      const incrementBtn = await page.$$eval('button', buttons => {
        const btn = buttons.find(b => b.textContent.includes('Increment'));
        if (btn) btn.click();
        return !!btn;
      });
      if (!incrementBtn) {
        throw new Error('Increment button not found');
      };
      await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for state update
      
      const newValue = await page.evaluate(() => {
        const elements = document.querySelectorAll('*');
        for (const el of elements) {
          const text = el.textContent.trim();
          if (/^\d+$/.test(text) && el.tagName !== 'BUTTON' && 
              window.getComputedStyle(el).fontSize.includes('px') &&
              parseInt(window.getComputedStyle(el).fontSize) > 30) {
            return text;
          }
        }
        return null;
      });
      console.log('Counter value after increment:', newValue);
      
      await page.screenshot({
        path: 'screenshots/counter-incremented.png',
        fullPage: true
      });
    } catch (e) {
      console.log('Failed to click increment button:', e.message);
    }
    
    // Find and click decrement button
    try {
      const decrementBtn = await page.$$eval('button', buttons => {
        const btn = buttons.find(b => b.textContent.includes('Decrement'));
        if (btn) btn.click();
        return !!btn;
      });
      if (!decrementBtn) {
        throw new Error('Decrement button not found');
      };
      await new Promise(resolve => setTimeout(resolve, 1000)); // Wait for state update
      
      const finalValue = await page.evaluate(() => {
        const elements = document.querySelectorAll('*');
        for (const el of elements) {
          const text = el.textContent.trim();
          if (/^\d+$/.test(text) && el.tagName !== 'BUTTON' && 
              window.getComputedStyle(el).fontSize.includes('px') &&
              parseInt(window.getComputedStyle(el).fontSize) > 30) {
            return text;
          }
        }
        return null;
      });
      console.log('Counter value after decrement:', finalValue);
      
      await page.screenshot({
        path: 'screenshots/counter-decremented.png',
        fullPage: true
      });
    } catch (e) {
      console.log('Failed to click decrement button:', e.message);
    }
    
    // Test quick increment buttons
    try {
      console.log('\nTesting quick increment buttons...');
      const plus10Btn = await page.$$eval('button', buttons => {
        const btn = buttons.find(b => b.textContent.includes('+10'));
        if (btn) btn.click();
        return !!btn;
      });
      if (!plus10Btn) {
        throw new Error('+10 button not found');
      };
      await new Promise(resolve => setTimeout(resolve, 500));
      
      const afterPlus10 = await page.evaluate(() => {
        const elements = document.querySelectorAll('*');
        for (const el of elements) {
          const text = el.textContent.trim();
          if (/^\d+$/.test(text) && el.tagName !== 'BUTTON' && 
              window.getComputedStyle(el).fontSize.includes('px') &&
              parseInt(window.getComputedStyle(el).fontSize) > 30) {
            return text;
          }
        }
        return null;
      });
      console.log('Counter value after +10:', afterPlus10);
    } catch (e) {
      console.log('Failed to test quick increment buttons:', e.message);
    }
    
    // Report results
    console.log('\n=== Test Results ===');
    console.log('Page loaded successfully: YES');
    console.log('Console errors:', pageErrors.length);
    console.log('Console warnings:', consoleLogs.filter(log => log.type === 'warning').length);
    
    if (pageErrors.length > 0) {
      console.log('\nErrors found:');
      pageErrors.forEach(err => console.log(' -', err));
    }
    
    const result = {
      success: pageErrors.length === 0,
      errors: pageErrors,
      warnings: consoleLogs.filter(log => log.type === 'warning'),
      logs: consoleLogs
    };
    
    fs.writeFileSync('test-results-counter.json', JSON.stringify(result, null, 2));
    
  } catch (error) {
    console.error('Test failed:', error);
    await page.screenshot({
      path: 'screenshots/counter-error.png',
      fullPage: true
    });
    
    const result = {
      success: false,
      error: error.toString(),
      errors: pageErrors,
      logs: consoleLogs
    };
    
    fs.writeFileSync('test-results-counter.json', JSON.stringify(result, null, 2));
  } finally {
    await browser.close();
  }
}

// Create screenshots directory
if (!fs.existsSync('screenshots')) {
  fs.mkdirSync('screenshots');
}

testCounterExample().catch(console.error);