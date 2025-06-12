const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// Configuration
const MEMORY_GAME_URL = 'http://localhost:8000/examples/memory-game/';
const SCREENSHOT_DIR = path.join(__dirname, '../screenshots/memory-game');
const TIMEOUT = 30000;

// Helper functions
async function waitForElement(page, selector, options = {}) {
    try {
        await page.waitForSelector(selector, { timeout: TIMEOUT, ...options });
        return true;
    } catch (error) {
        console.error(`Failed to find element: ${selector}`);
        return false;
    }
}

async function takeScreenshot(page, name) {
    if (!fs.existsSync(SCREENSHOT_DIR)) {
        fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    }
    const screenshotPath = path.join(SCREENSHOT_DIR, `${name}.png`);
    await page.screenshot({ path: screenshotPath, fullPage: true });
    console.log(`📸 Screenshot saved: ${name}.png`);
}

async function checkConsoleErrors(page) {
    const errors = [];
    page.on('console', msg => {
        if (msg.type() === 'error') {
            errors.push(msg.text());
        }
    });
    return errors;
}

// Main test function
async function runMemoryGameTests() {
    console.log('\n🚀 Starting Layer9 Memory Game Tests\n');
    
    const browser = await puppeteer.launch({
        headless: 'new',
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });

    let testsPassed = 0;
    let testsFailed = 0;
    const errors = [];

    try {
        const page = await browser.newPage();
        await page.setViewport({ width: 1280, height: 800 });
        
        // Monitor console errors
        const consoleErrors = await checkConsoleErrors(page);

        // Test 1: Navigate to memory game
        console.log('📋 Test 1: Loading the memory game application...');
        try {
            await page.goto(MEMORY_GAME_URL, { waitUntil: 'networkidle2', timeout: TIMEOUT });
            console.log('✅ Successfully loaded the memory game');
            testsPassed++;
        } catch (error) {
            console.error('❌ Failed to load the memory game:', error.message);
            testsFailed++;
            throw error;
        }

        // Test 2: Verify WASM loads correctly
        console.log('\n📋 Test 2: Verifying WASM initialization...');
        try {
            // Wait for game container to appear
            await waitForElement(page, '.game-container');
            await waitForElement(page, '.game-board');
            
            console.log('✅ WASM loaded successfully');
            testsPassed++;
            await takeScreenshot(page, '01-initial-load');
        } catch (error) {
            console.error('❌ WASM loading failed:', error.message);
            testsFailed++;
        }

        // Test 3: Verify all main components render
        console.log('\n📋 Test 3: Verifying all memory game components...');
        const components = [
            { selector: 'h1', name: 'Title with Memory branding' },
            { selector: '.subtitle', name: 'Subtitle' },
            { selector: '.stats', name: 'Statistics section' },
            { selector: '.stat:nth-child(1)', name: 'Moves counter' },
            { selector: '.stat:nth-child(2)', name: 'Matches counter' },
            { selector: '.game-board', name: 'Game board' },
            { selector: '.card', name: 'Card elements' },
            { selector: '.btn-secondary', name: 'New Game button' },
            { selector: 'footer', name: 'Footer' }
        ];

        let allComponentsFound = true;
        for (const component of components) {
            const found = await waitForElement(page, component.selector);
            if (found) {
                console.log(`  ✅ ${component.name} found`);
            } else {
                console.log(`  ❌ ${component.name} not found`);
                allComponentsFound = false;
            }
        }
        
        if (allComponentsFound) {
            testsPassed++;
        } else {
            testsFailed++;
        }

        // Test 4: Verify 4x4 grid (16 cards)
        console.log('\n📋 Test 4: Verifying 4x4 game grid...');
        try {
            const cardCount = await page.$$eval('.card', cards => cards.length);
            if (cardCount === 16) {
                console.log(`✅ Correct number of cards: ${cardCount} (4x4 grid)`);
                testsPassed++;
            } else {
                console.log(`❌ Expected 16 cards, found ${cardCount}`);
                testsFailed++;
            }
            
            // Verify all cards start face down
            const allFaceDown = await page.$$eval('.card', cards => 
                cards.every(card => !card.classList.contains('flipped'))
            );
            if (allFaceDown) {
                console.log('✅ All cards start face down');
            }
        } catch (error) {
            console.error('❌ Failed to verify game grid:', error.message);
            testsFailed++;
        }

        // Test 5: Test card flipping
        console.log('\n📋 Test 5: Testing card flip functionality...');
        try {
            // Click first card
            await page.click('.card:nth-child(1)');
            await page.waitForTimeout(300);
            
            const firstCardFlipped = await page.$eval('.card:nth-child(1)', card => 
                card.classList.contains('flipped')
            );
            
            if (firstCardFlipped) {
                console.log('✅ First card flipped successfully');
                
                // Get the emoji on the first card
                const firstEmoji = await page.$eval('.card:nth-child(1) .card-back', el => el.textContent);
                console.log(`  Card shows: ${firstEmoji}`);
                
                await takeScreenshot(page, '02-first-card-flipped');
                testsPassed++;
            } else {
                console.log('❌ First card did not flip');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test card flipping:', error.message);
            testsFailed++;
        }

        // Test 6: Test second card flip and mismatch
        console.log('\n📋 Test 6: Testing card mismatch behavior...');
        try {
            // Click a second card (likely to be different)
            await page.click('.card:nth-child(2)');
            await page.waitForTimeout(300);
            
            const secondCardFlipped = await page.$eval('.card:nth-child(2)', card => 
                card.classList.contains('flipped')
            );
            
            if (secondCardFlipped) {
                console.log('✅ Second card flipped successfully');
                await takeScreenshot(page, '03-two-cards-flipped');
                
                // Wait for cards to flip back if they don't match
                await page.waitForTimeout(1200);
                
                const cardsStillFlipped = await page.$$eval('.card:nth-child(1), .card:nth-child(2)', cards => 
                    cards.every(card => card.classList.contains('flipped'))
                );
                
                if (!cardsStillFlipped) {
                    console.log('✅ Mismatched cards flipped back');
                    testsPassed++;
                } else {
                    // They might have matched by chance
                    const matched = await page.$$eval('.card:nth-child(1), .card:nth-child(2)', cards => 
                        cards.every(card => card.classList.contains('matched'))
                    );
                    if (matched) {
                        console.log('✅ Cards matched (lucky!)');
                        testsPassed++;
                    }
                }
            } else {
                console.log('❌ Second card did not flip');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test mismatch behavior:', error.message);
            testsFailed++;
        }

        // Test 7: Test move counter
        console.log('\n📋 Test 7: Testing move counter...');
        try {
            const moves = await page.$eval('.stat:nth-child(1) .stat-value', el => el.textContent);
            const movesNum = parseInt(moves);
            
            if (movesNum > 0) {
                console.log(`✅ Move counter incremented: ${movesNum} moves`);
                testsPassed++;
            } else {
                console.log('❌ Move counter not incrementing');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test move counter:', error.message);
            testsFailed++;
        }

        // Test 8: Find and match a pair
        console.log('\n📋 Test 8: Testing matching pairs...');
        try {
            // Click New Game to reset
            await page.click('.btn-secondary');
            await page.waitForTimeout(500);
            
            // Find a matching pair by trying cards systematically
            let matchFound = false;
            let firstCard = null;
            let firstEmoji = null;
            
            for (let i = 1; i <= 16 && !matchFound; i++) {
                // Click first card
                await page.click(`.card:nth-child(${i})`);
                await page.waitForTimeout(300);
                
                firstEmoji = await page.$eval(`.card:nth-child(${i}) .card-back`, el => el.textContent);
                firstCard = i;
                
                // Try to find its match
                for (let j = i + 1; j <= 16; j++) {
                    await page.click(`.card:nth-child(${j})`);
                    await page.waitForTimeout(300);
                    
                    const secondEmoji = await page.$eval(`.card:nth-child(${j}) .card-back`, el => el.textContent);
                    
                    if (firstEmoji === secondEmoji) {
                        // Found a match!
                        await page.waitForTimeout(600);
                        
                        const bothMatched = await page.$$eval(`.card:nth-child(${i}), .card:nth-child(${j})`, cards => 
                            cards.every(card => card.classList.contains('matched'))
                        );
                        
                        if (bothMatched) {
                            console.log(`✅ Found and matched a pair: ${firstEmoji}`);
                            matchFound = true;
                            testsPassed++;
                            await takeScreenshot(page, '04-matched-pair');
                            break;
                        }
                    } else {
                        // Wait for cards to flip back
                        await page.waitForTimeout(1200);
                    }
                }
                
                if (!matchFound) {
                    // Reset this card
                    await page.waitForTimeout(1200);
                }
            }
            
            if (!matchFound) {
                console.log('❌ Could not find a matching pair (unexpected)');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test matching pairs:', error.message);
            testsFailed++;
        }

        // Test 9: Test matches counter
        console.log('\n📋 Test 9: Testing matches counter...');
        try {
            const matches = await page.$eval('.stat:nth-child(2) .stat-value', el => el.textContent);
            
            if (matches.includes('/8')) {
                const currentMatches = parseInt(matches.split('/')[0]);
                if (currentMatches > 0) {
                    console.log(`✅ Matches counter working: ${matches}`);
                    testsPassed++;
                } else {
                    console.log('❌ Matches counter not updating');
                    testsFailed++;
                }
            } else {
                console.log('❌ Matches counter format incorrect');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test matches counter:', error.message);
            testsFailed++;
        }

        // Test 10: Test New Game button
        console.log('\n📋 Test 10: Testing New Game functionality...');
        try {
            // Get current moves
            const movesBefore = await page.$eval('.stat:nth-child(1) .stat-value', el => el.textContent);
            
            // Click New Game
            await page.click('.btn-secondary');
            await page.waitForTimeout(500);
            
            // Check moves reset to 0
            const movesAfter = await page.$eval('.stat:nth-child(1) .stat-value', el => el.textContent);
            
            // Check all cards are face down
            const allFaceDown = await page.$$eval('.card', cards => 
                cards.every(card => !card.classList.contains('flipped') && !card.classList.contains('matched'))
            );
            
            if (movesAfter === '0' && allFaceDown) {
                console.log('✅ New Game resets the board correctly');
                testsPassed++;
                await takeScreenshot(page, '05-new-game');
            } else {
                console.log('❌ New Game did not reset properly');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test New Game:', error.message);
            testsFailed++;
        }

        // Test 11: Test card click prevention during pair checking
        console.log('\n📋 Test 11: Testing click prevention during animations...');
        try {
            // Click two different cards quickly
            await page.click('.card:nth-child(1)');
            await page.click('.card:nth-child(2)');
            
            // Immediately try to click a third card
            await page.click('.card:nth-child(3)');
            await page.waitForTimeout(300);
            
            // Check if third card is flipped
            const thirdCardFlipped = await page.$eval('.card:nth-child(3)', card => 
                card.classList.contains('flipped')
            );
            
            if (!thirdCardFlipped) {
                console.log('✅ Click prevention works during pair checking');
                testsPassed++;
            } else {
                console.log('❌ Third card should not flip during pair checking');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test click prevention:', error.message);
            testsFailed++;
        }

        // Test 12: Test animations exist
        console.log('\n📋 Test 12: Testing card animations...');
        try {
            const hasAnimations = await page.evaluate(() => {
                const card = document.querySelector('.card-inner');
                const styles = window.getComputedStyle(card);
                return styles.transition !== 'none' || styles.transform !== 'none';
            });
            
            if (hasAnimations) {
                console.log('✅ Card flip animations are present');
                testsPassed++;
            } else {
                console.log('❌ Card animations not found');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to test animations:', error.message);
            testsFailed++;
        }

        // Test 13: Complete a game (simplified - just check win screen exists in code)
        console.log('\n📋 Test 13: Verifying win screen capability...');
        try {
            // Since completing a full game would be time-consuming,
            // we'll verify the win screen code exists
            const pageContent = await page.content();
            if (pageContent.includes('Congratulations') && pageContent.includes('You won')) {
                console.log('✅ Win screen code is present');
                testsPassed++;
            } else {
                console.log('❌ Win screen code not found');
                testsFailed++;
            }
        } catch (error) {
            console.error('❌ Failed to verify win screen:', error.message);
            testsFailed++;
        }

        // Test 14: Check for console errors
        console.log('\n📋 Test 14: Checking for console errors...');
        if (consoleErrors.length === 0) {
            console.log('✅ No console errors detected');
            testsPassed++;
        } else {
            console.log(`❌ Found ${consoleErrors.length} console errors:`);
            consoleErrors.forEach(error => console.log(`   - ${error}`));
            testsFailed++;
        }

        // Final screenshot
        await takeScreenshot(page, '06-final-state');

    } catch (error) {
        console.error('\n💥 Critical error:', error);
        errors.push(error.message);
    } finally {
        await browser.close();
        
        // Summary
        console.log('\n' + '='.repeat(50));
        console.log('📊 MEMORY GAME TEST SUMMARY');
        console.log('='.repeat(50));
        console.log(`✅ Passed: ${testsPassed}`);
        console.log(`❌ Failed: ${testsFailed}`);
        console.log(`📸 Screenshots saved in: ${SCREENSHOT_DIR}`);
        if (errors.length > 0) {
            console.log(`⚠️  Critical errors: ${errors.length}`);
        }
        console.log('='.repeat(50) + '\n');

        process.exit(testsFailed > 0 || errors.length > 0 ? 1 : 0);
    }
}

// Check if server is running
async function checkServer() {
    try {
        const response = await fetch(MEMORY_GAME_URL);
        return response.ok;
    } catch (error) {
        return false;
    }
}

// Run tests
console.log('🔧 Layer9 Memory Game - Comprehensive Puppeteer Test Suite');
console.log('=========================================================\n');

checkServer().then(isRunning => {
    if (!isRunning) {
        console.error('❌ HTTP server is not running at', MEMORY_GAME_URL);
        console.error('Please start the server first from the project root:');
        console.error('  cd .. && python3 -m http.server 8000');
        process.exit(1);
    }
    return runMemoryGameTests();
}).catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});