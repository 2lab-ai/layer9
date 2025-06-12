#!/usr/bin/env node

const puppeteer = require('puppeteer');

async function quickDomCheck() {
    console.log('🔍 Quick DOM Check\n');
    
    const browser = await puppeteer.launch({ 
        headless: true,
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });
    
    try {
        const page = await browser.newPage();
        
        // Log console messages
        page.on('console', msg => {
            console.log(`[Console] ${msg.text()}`);
        });
        
        // Navigate
        console.log('Loading page...');
        await page.goto('http://localhost:8080', {
            waitUntil: 'networkidle0',
            timeout: 10000
        });
        
        // Wait a bit
        await new Promise(r => setTimeout(r, 2000));
        
        // Get full page HTML
        console.log('\n📄 Full page HTML:');
        const html = await page.content();
        console.log(html);
        
        // Get body content specifically
        console.log('\n📦 Body content:');
        const bodyContent = await page.evaluate(() => document.body.innerHTML);
        console.log(bodyContent);
        
        // Check specific elements
        console.log('\n🔍 Element checks:');
        
        // Check root
        const hasRoot = await page.$('#root') !== null;
        console.log(`#root exists: ${hasRoot}`);
        
        if (hasRoot) {
            const rootContent = await page.$eval('#root', el => ({
                innerHTML: el.innerHTML,
                childCount: el.children.length,
                firstChildTag: el.firstElementChild ? el.firstElementChild.tagName : 'none'
            }));
            console.log(`#root children: ${rootContent.childCount}`);
            console.log(`First child tag: ${rootContent.firstChildTag}`);
            console.log(`#root innerHTML:\n${rootContent.innerHTML}`);
        }
        
        // Check for layer9-app
        const hasLayer9App = await page.$('.layer9-app') !== null;
        console.log(`\n.layer9-app exists: ${hasLayer9App}`);
        
        // Check for counter-value
        const hasCounterValue = await page.$('.counter-value') !== null;
        console.log(`.counter-value exists: ${hasCounterValue}`);
        
        // Check all divs
        const divInfo = await page.$$eval('div', divs => 
            divs.map(div => ({
                id: div.id || 'no-id',
                class: div.className || 'no-class',
                tag: div.tagName,
                text: div.textContent.substring(0, 50)
            }))
        );
        console.log('\nAll DIVs:');
        divInfo.forEach(div => {
            console.log(`  <${div.tag} id="${div.id}" class="${div.class}">${div.text}...`);
        });
        
        // Check all buttons
        const buttonInfo = await page.$$eval('button', buttons => 
            buttons.map(btn => ({
                class: btn.className,
                text: btn.textContent,
                onclick: btn.onclick ? 'has-handler' : 'no-handler'
            }))
        );
        console.log('\nAll buttons:');
        buttonInfo.forEach(btn => {
            console.log(`  <button class="${btn.class}">${btn.text}</button> [${btn.onclick}]`);
        });
        
        // Take screenshot
        await page.screenshot({ path: 'quick-check.png' });
        console.log('\nScreenshot saved: quick-check.png');
        
    } finally {
        await browser.close();
    }
}

quickDomCheck().catch(console.error);