/**
 * Complete workflow E2E tests for Layer9
 * Tests realistic user journeys through the application
 */

const puppeteer = require('puppeteer');
const { expect } = require('chai');

describe('Layer9 Complete Workflow E2E Tests', () => {
    let browser;
    let page;
    
    before(async () => {
        browser = await puppeteer.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-setuid-sandbox']
        });
    });
    
    after(async () => {
        await browser.close();
    });
    
    beforeEach(async () => {
        page = await browser.newPage();
        await page.setViewport({ width: 1280, height: 720 });
        
        // Set up console message collection
        page.on('console', msg => {
            if (msg.type() === 'error') {
                console.error('Browser console error:', msg.text());
            }
        });
    });
    
    afterEach(async () => {
        await page.close();
    });

    describe('User Authentication Flow', () => {
        it('should complete full authentication workflow', async () => {
            await page.goto('http://localhost:8080/examples/auth-jwt-demo');
            
            // Wait for app to load
            await page.waitForSelector('[data-testid="auth-app"]', { timeout: 5000 });
            
            // Should start logged out
            const loginButton = await page.$('[data-testid="login-button"]');
            expect(loginButton).to.exist;
            
            // Fill login form
            await page.type('[data-testid="username-input"]', 'testuser');
            await page.type('[data-testid="password-input"]', 'testpass123');
            
            // Submit login
            await page.click('[data-testid="login-button"]');
            
            // Wait for login to complete
            await page.waitForSelector('[data-testid="user-profile"]', { timeout: 5000 });
            
            // Verify logged in state
            const username = await page.$eval('[data-testid="username-display"]', el => el.textContent);
            expect(username).to.equal('testuser');
            
            // Test authenticated action
            await page.click('[data-testid="protected-action"]');
            await page.waitForSelector('[data-testid="action-success"]');
            
            // Logout
            await page.click('[data-testid="logout-button"]');
            await page.waitForSelector('[data-testid="login-button"]');
            
            // Verify logged out
            const loginButtonAfterLogout = await page.$('[data-testid="login-button"]');
            expect(loginButtonAfterLogout).to.exist;
        });
        
        it('should handle token refresh', async () => {
            await page.goto('http://localhost:8080/examples/auth-jwt-demo');
            
            // Login
            await page.type('[data-testid="username-input"]', 'testuser');
            await page.type('[data-testid="password-input"]', 'testpass123');
            await page.click('[data-testid="login-button"]');
            await page.waitForSelector('[data-testid="user-profile"]');
            
            // Simulate token expiry by waiting
            await page.evaluate(() => {
                // Force token to be considered expired
                window.__forceTokenExpiry = true;
            });
            
            // Perform action that requires auth
            await page.click('[data-testid="protected-action"]');
            
            // Should auto-refresh and complete action
            await page.waitForSelector('[data-testid="action-success"]');
            
            // Verify still logged in
            const username = await page.$eval('[data-testid="username-display"]', el => el.textContent);
            expect(username).to.equal('testuser');
        });
    });

    describe('Form Submission Workflow', () => {
        it('should handle complex form with validation', async () => {
            await page.goto('http://localhost:8080/examples/forms-demo');
            
            // Wait for form
            await page.waitForSelector('[data-testid="user-form"]');
            
            // Try submitting empty form
            await page.click('[data-testid="submit-button"]');
            
            // Check for validation errors
            const nameError = await page.$('[data-testid="name-error"]');
            expect(nameError).to.exist;
            
            // Fill form with invalid data
            await page.type('[data-testid="email-input"]', 'invalid-email');
            await page.type('[data-testid="age-input"]', '-5');
            
            // Check real-time validation
            await page.waitForSelector('[data-testid="email-error"]');
            await page.waitForSelector('[data-testid="age-error"]');
            
            // Fix form data
            await page.evaluate(() => {
                document.querySelector('[data-testid="email-input"]').value = '';
                document.querySelector('[data-testid="age-input"]').value = '';
            });
            
            await page.type('[data-testid="name-input"]', 'John Doe');
            await page.type('[data-testid="email-input"]', 'john@example.com');
            await page.type('[data-testid="age-input"]', '25');
            await page.type('[data-testid="bio-textarea"]', 'Software developer');
            
            // Select options
            await page.select('[data-testid="country-select"]', 'US');
            await page.click('[data-testid="terms-checkbox"]');
            
            // Submit form
            await page.click('[data-testid="submit-button"]');
            
            // Wait for success
            await page.waitForSelector('[data-testid="form-success"]');
            
            // Verify submitted data display
            const submittedData = await page.$eval('[data-testid="submitted-data"]', el => el.textContent);
            expect(submittedData).to.include('John Doe');
            expect(submittedData).to.include('john@example.com');
        });
        
        it('should handle file upload', async () => {
            await page.goto('http://localhost:8080/examples/forms-demo');
            
            const fileInput = await page.$('[data-testid="file-input"]');
            const testFilePath = './test-fixtures/test-image.png';
            
            await fileInput.uploadFile(testFilePath);
            
            // Wait for preview
            await page.waitForSelector('[data-testid="file-preview"]');
            
            // Verify file info
            const fileName = await page.$eval('[data-testid="file-name"]', el => el.textContent);
            expect(fileName).to.equal('test-image.png');
            
            // Submit with file
            await page.click('[data-testid="upload-button"]');
            await page.waitForSelector('[data-testid="upload-success"]');
        });
    });

    describe('Real-time Features', () => {
        it('should handle WebSocket chat workflow', async () => {
            // Open two pages for chat
            const page1 = await browser.newPage();
            const page2 = await browser.newPage();
            
            await page1.goto('http://localhost:8080/examples/websocket-chat');
            await page2.goto('http://localhost:8080/examples/websocket-chat');
            
            // Login both users
            await page1.type('[data-testid="username-input"]', 'User1');
            await page1.click('[data-testid="join-chat"]');
            
            await page2.type('[data-testid="username-input"]', 'User2');
            await page2.click('[data-testid="join-chat"]');
            
            // Wait for connection
            await page1.waitForSelector('[data-testid="chat-connected"]');
            await page2.waitForSelector('[data-testid="chat-connected"]');
            
            // User1 sends message
            await page1.type('[data-testid="message-input"]', 'Hello from User1!');
            await page1.click('[data-testid="send-button"]');
            
            // Both should see the message
            await page1.waitForSelector('text=Hello from User1!');
            await page2.waitForSelector('text=Hello from User1!');
            
            // User2 responds
            await page2.type('[data-testid="message-input"]', 'Hi User1!');
            await page2.click('[data-testid="send-button"]');
            
            // Both should see the response
            await page1.waitForSelector('text=Hi User1!');
            await page2.waitForSelector('text=Hi User1!');
            
            // Test disconnect/reconnect
            await page1.evaluate(() => {
                window.__simulateDisconnect();
            });
            
            await page1.waitForSelector('[data-testid="chat-disconnected"]');
            
            // Should auto-reconnect
            await page1.waitForSelector('[data-testid="chat-connected"]', { timeout: 10000 });
            
            // Cleanup
            await page1.close();
            await page2.close();
        });
        
        it('should sync state across tabs', async () => {
            const page1 = await browser.newPage();
            const page2 = await browser.newPage();
            
            await page1.goto('http://localhost:8080/examples/state-sync-demo');
            await page2.goto('http://localhost:8080/examples/state-sync-demo');
            
            // Change theme in page1
            await page1.click('[data-testid="theme-toggle"]');
            
            // Should update in both pages
            await page1.waitForSelector('.dark-theme');
            await page2.waitForSelector('.dark-theme');
            
            // Update counter in page2
            await page2.click('[data-testid="increment-button"]');
            await page2.click('[data-testid="increment-button"]');
            
            // Both should show updated count
            const count1 = await page1.$eval('[data-testid="counter-value"]', el => el.textContent);
            const count2 = await page2.$eval('[data-testid="counter-value"]', el => el.textContent);
            
            expect(count1).to.equal('2');
            expect(count2).to.equal('2');
            
            await page1.close();
            await page2.close();
        });
    });

    describe('Performance and Loading', () => {
        it('should handle large list rendering', async () => {
            await page.goto('http://localhost:8080/examples/virtual-list-demo');
            
            // Wait for initial render
            await page.waitForSelector('[data-testid="virtual-list"]');
            
            // Check only visible items are rendered
            const visibleItems = await page.$$('[data-testid="list-item"]');
            expect(visibleItems.length).to.be.lessThan(50); // Should use virtualization
            
            // Scroll to bottom
            await page.evaluate(() => {
                const list = document.querySelector('[data-testid="virtual-list"]');
                list.scrollTop = list.scrollHeight;
            });
            
            // Wait for new items
            await page.waitForFunction(() => {
                const items = document.querySelectorAll('[data-testid="list-item"]');
                return items[items.length - 1].textContent.includes('Item 999');
            });
            
            // Search functionality
            await page.type('[data-testid="search-input"]', 'Item 500');
            await page.waitForSelector('text=Item 500');
            
            // Clear search
            await page.evaluate(() => {
                document.querySelector('[data-testid="search-input"]').value = '';
            });
            await page.keyboard.press('Enter');
        });
        
        it('should lazy load images', async () => {
            await page.goto('http://localhost:8080/examples/image-gallery');
            
            // Initially, only visible images should be loaded
            const initialImages = await page.evaluate(() => {
                return Array.from(document.querySelectorAll('img')).filter(img => img.complete).length;
            });
            
            expect(initialImages).to.be.lessThan(10);
            
            // Scroll down
            await page.evaluate(() => window.scrollBy(0, 1000));
            
            // Wait for more images to load
            await page.waitForFunction((initial) => {
                const loaded = Array.from(document.querySelectorAll('img')).filter(img => img.complete).length;
                return loaded > initial;
            }, {}, initialImages);
            
            // Test image optimization
            const imageSrc = await page.$eval('img', el => el.src);
            expect(imageSrc).to.include('w='); // Should have width parameter
            expect(imageSrc).to.include('q='); // Should have quality parameter
        });
    });

    describe('Error Handling and Recovery', () => {
        it('should handle and recover from errors gracefully', async () => {
            await page.goto('http://localhost:8080/examples/error-boundary-demo');
            
            // Trigger an error
            await page.click('[data-testid="trigger-error"]');
            
            // Should show error boundary
            await page.waitForSelector('[data-testid="error-boundary"]');
            
            const errorMessage = await page.$eval('[data-testid="error-message"]', el => el.textContent);
            expect(errorMessage).to.include('Something went wrong');
            
            // Should have retry button
            await page.click('[data-testid="retry-button"]');
            
            // Should recover
            await page.waitForSelector('[data-testid="app-content"]');
            
            // App should be functional again
            await page.click('[data-testid="normal-action"]');
            await page.waitForSelector('[data-testid="action-success"]');
        });
        
        it('should handle network failures', async () => {
            await page.goto('http://localhost:8080/examples/data-fetching');
            
            // Simulate offline
            await page.setOfflineMode(true);
            
            // Try to fetch data
            await page.click('[data-testid="fetch-data"]');
            
            // Should show offline message
            await page.waitForSelector('[data-testid="offline-message"]');
            
            // Go back online
            await page.setOfflineMode(false);
            
            // Should auto-retry or allow manual retry
            await page.click('[data-testid="retry-fetch"]');
            
            // Should load data
            await page.waitForSelector('[data-testid="data-list"]');
        });
    });

    describe('Accessibility', () => {
        it('should be keyboard navigable', async () => {
            await page.goto('http://localhost:8080/examples/accessible-form');
            
            // Tab through form
            await page.keyboard.press('Tab');
            const firstFocused = await page.evaluate(() => document.activeElement.dataset.testid);
            expect(firstFocused).to.equal('first-input');
            
            await page.keyboard.press('Tab');
            const secondFocused = await page.evaluate(() => document.activeElement.dataset.testid);
            expect(secondFocused).to.equal('second-input');
            
            // Submit with Enter
            await page.keyboard.type('test value');
            await page.keyboard.press('Enter');
            
            await page.waitForSelector('[data-testid="form-submitted"]');
        });
        
        it('should have proper ARIA labels', async () => {
            await page.goto('http://localhost:8080');
            
            // Check main navigation
            const navAriaLabel = await page.$eval('nav', el => el.getAttribute('aria-label'));
            expect(navAriaLabel).to.exist;
            
            // Check buttons
            const buttons = await page.$$('button');
            for (const button of buttons) {
                const hasLabel = await button.evaluate(el => 
                    el.getAttribute('aria-label') || el.textContent.trim()
                );
                expect(hasLabel).to.be.ok;
            }
        });
    });
});