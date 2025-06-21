const puppeteer = require('puppeteer');

async function testRapidClicks() {
  let browser;
  try {
    browser = await puppeteer.launch({
      headless: true,
      args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    const page = await browser.newPage();
    
    // Navigate to counter
    await page.goto('http://localhost:8085', { waitUntil: 'networkidle0' });
    
    // Wait for counter to initialize
    await page.waitForSelector('.counter-value', { timeout: 5000 });
    
    // Get initial value
    const initialValue = await page.$eval('.counter-value', el => parseInt(el.textContent));
    console.log('Initial counter value:', initialValue);
    
    // Get increment button
    const incrementButton = await page.$('.btn-increment');
    if (!incrementButton) {
      throw new Error('Increment button not found');
    }
    
    // Rapid click test - click 5 times quickly
    console.log('Performing 5 rapid clicks on increment button...');
    for (let i = 0; i < 5; i++) {
      await incrementButton.click();
      // Small delay to allow for event processing
      await new Promise(r => setTimeout(r, 50));
    }
    
    // Wait for final render
    await new Promise(r => setTimeout(r, 500));
    
    // Get final value
    const finalValue = await page.$eval('.counter-value', el => parseInt(el.textContent));
    console.log('Final counter value:', finalValue);
    
    // Calculate expected value (each click adds 2)
    const expectedValue = initialValue + (5 * 2);
    console.log('Expected value:', expectedValue);
    
    if (finalValue === expectedValue) {
      console.log('✅ SUCCESS: Counter correctly incremented by', finalValue - initialValue);
    } else {
      console.log('❌ FAIL: Counter shows', finalValue, 'but expected', expectedValue);
      console.log('   Difference:', finalValue - expectedValue);
    }
    
    // Test decrement as well
    const decrementButton = await page.$('.btn-decrement');
    if (!decrementButton) {
      throw new Error('Decrement button not found');
    }
    
    // Rapid decrement test
    console.log('\nPerforming 3 rapid clicks on decrement button...');
    const beforeDecrement = finalValue;
    for (let i = 0; i < 3; i++) {
      await decrementButton.click();
      await new Promise(r => setTimeout(r, 50));
    }
    
    await new Promise(r => setTimeout(r, 500));
    
    const afterDecrement = await page.$eval('.counter-value', el => parseInt(el.textContent));
    const expectedAfterDecrement = beforeDecrement - (3 * 2);
    
    console.log('After decrement:', afterDecrement);
    console.log('Expected after decrement:', expectedAfterDecrement);
    
    if (afterDecrement === expectedAfterDecrement) {
      console.log('✅ SUCCESS: Counter correctly decremented by', beforeDecrement - afterDecrement);
    } else {
      console.log('❌ FAIL: Counter shows', afterDecrement, 'but expected', expectedAfterDecrement);
    }
    
  } catch (error) {
    console.error('Test failed:', error);
  } finally {
    if (browser) {
      await browser.close();
    }
  }
}

// Run the test
testRapidClicks().then(() => {
  console.log('\nTest completed');
  process.exit(0);
}).catch(err => {
  console.error('Test error:', err);
  process.exit(1);
});