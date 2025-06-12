const puppeteer = require('puppeteer');
const path = require('path');
const fs = require('fs');

// Configuration
const TODO_APP_URL = 'http://localhost:8082';
const SCREENSHOT_DIR = path.join(__dirname, 'screenshots', 'realistic-demo');

// Realistic todo items organized by category
const TODO_DATA = {
    work: [
        { text: 'Review Q4 budget proposals', completed: true },
        { text: 'Prepare slides for team meeting', completed: false },
        { text: 'Update project roadmap', completed: false },
        { text: 'Send weekly status report', completed: true },
        { text: 'Schedule 1:1 with new team member', completed: false },
        { text: 'Review pull requests', completed: true },
        { text: 'Deploy hotfix to production', completed: true },
        { text: 'Document API endpoints', completed: false }
    ],
    personal: [
        { text: 'Buy groceries for the week', completed: false },
        { text: 'Call mom for her birthday', completed: true },
        { text: 'Book dentist appointment', completed: false },
        { text: 'Renew gym membership', completed: false },
        { text: 'Pick up dry cleaning', completed: true },
        { text: 'Plan weekend trip', completed: false },
        { text: 'Pay utility bills', completed: true },
        { text: 'Replace car air filter', completed: false }
    ],
    learning: [
        { text: 'Complete Rust tutorial chapter 5', completed: true },
        { text: 'Watch WebAssembly conference talks', completed: false },
        { text: 'Read "Clean Architecture" book', completed: false },
        { text: 'Practice algorithm problems', completed: true },
        { text: 'Build side project with Layer9', completed: false },
        { text: 'Write blog post about WASM', completed: false }
    ]
};

// Helper functions
function ensureDirectory(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

async function takeScreenshot(page, name, description) {
    const screenshotPath = path.join(SCREENSHOT_DIR, `${name}.png`);
    await page.screenshot({ 
        path: screenshotPath, 
        fullPage: false,
        clip: {
            x: 0,
            y: 0,
            width: 1200,
            height: 800
        }
    });
    console.log(`📸 ${name}.png - ${description}`);
}

async function addTodo(page, text) {
    await page.type('#todo-input', text);
    await page.click('.add-btn');
    await page.waitForTimeout(100);
}

async function toggleTodo(page, index) {
    await page.click(`.todo-item:nth-child(${index}) .todo-checkbox`);
    await page.waitForTimeout(100);
}

// Main demo scenarios
async function generateRealisticDemos() {
    ensureDirectory(SCREENSHOT_DIR);
    
    const browser = await puppeteer.launch({
        headless: false,
        args: ['--window-size=1200,800']
    });

    try {
        const page = await browser.newPage();
        await page.setViewport({ width: 1200, height: 800 });
        
        console.log('\n🎯 Generating Realistic Todo App Demos\n');

        // Scenario 1: Daily Work Tasks
        console.log('📋 Scenario 1: Daily Work Tasks');
        await page.goto(TODO_APP_URL, { waitUntil: 'networkidle2' });
        await page.waitForFunction(
            () => document.getElementById('loading').style.display === 'none'
        );
        
        // Add work todos
        for (const todo of TODO_DATA.work.slice(0, 5)) {
            await addTodo(page, todo.text);
        }
        
        // Mark some as completed
        await toggleTodo(page, 1); // Review Q4 budget
        await toggleTodo(page, 4); // Send weekly status
        
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'work-tasks', 'Daily work tasks with some completed');

        // Scenario 2: Mixed Personal & Work
        console.log('\n📋 Scenario 2: Mixed Personal & Work Tasks');
        
        // Add personal todos
        await addTodo(page, TODO_DATA.personal[0].text); // Buy groceries
        await addTodo(page, TODO_DATA.personal[2].text); // Book dentist
        await addTodo(page, TODO_DATA.personal[5].text); // Plan weekend trip
        
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'mixed-tasks', 'Mix of work and personal tasks');

        // Show filters in action
        await page.click('.filter-btn:nth-child(2)'); // Active
        await page.waitForTimeout(300);
        await takeScreenshot(page, 'active-tasks-only', 'Filtered view showing only active tasks');

        await page.click('.filter-btn:nth-child(3)'); // Completed
        await page.waitForTimeout(300);
        await takeScreenshot(page, 'completed-tasks-only', 'Filtered view showing only completed tasks');

        await page.click('.filter-btn:nth-child(1)'); // All
        await page.waitForTimeout(300);

        // Scenario 3: Learning & Development
        console.log('\n📋 Scenario 3: Learning & Development Focus');
        
        // Clear current todos
        await page.evaluate(() => {
            document.querySelectorAll('.todo-item').forEach(item => {
                const deleteBtn = item.querySelector('.delete-btn');
                if (deleteBtn) deleteBtn.click();
            });
        });
        await page.waitForTimeout(500);

        // Add learning todos
        for (const todo of TODO_DATA.learning) {
            await addTodo(page, todo.text);
        }
        
        // Mark completed items
        await toggleTodo(page, 1); // Rust tutorial
        await toggleTodo(page, 4); // Algorithm problems
        
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'learning-goals', 'Learning and development goals tracking');

        // Scenario 4: End of Day Review
        console.log('\n📋 Scenario 4: End of Day Review');
        
        // Clear and create end-of-day scenario
        await page.evaluate(() => {
            document.querySelectorAll('.todo-item').forEach(item => {
                const deleteBtn = item.querySelector('.delete-btn');
                if (deleteBtn) deleteBtn.click();
            });
        });
        await page.waitForTimeout(500);

        // Add a full day's worth of todos
        const fullDayTodos = [
            { text: 'Morning standup meeting', completed: true },
            { text: 'Code review for feature branch', completed: true },
            { text: 'Fix critical bug in production', completed: true },
            { text: 'Lunch with product team', completed: true },
            { text: 'Write unit tests for new feature', completed: true },
            { text: 'Update project documentation', completed: false },
            { text: 'Respond to client emails', completed: true },
            { text: 'Prepare tomorrow\'s presentation', completed: false },
            { text: 'Review candidate resumes', completed: false },
            { text: 'Update Jira tickets', completed: true }
        ];

        for (const todo of fullDayTodos) {
            await addTodo(page, todo.text);
            if (todo.completed) {
                const currentCount = await page.$$eval('.todo-item', items => items.length);
                await toggleTodo(page, currentCount);
            }
        }

        await page.waitForTimeout(500);
        await takeScreenshot(page, 'end-of-day', 'End of day review - productive day!');

        // Show clear completed in action
        await page.click('.clear-btn');
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'tomorrow-tasks', 'Ready for tomorrow - cleared completed tasks');

        // Scenario 5: Project Planning
        console.log('\n📋 Scenario 5: Project Planning');
        
        const projectTasks = [
            'Define project requirements',
            'Create wireframes and mockups',
            'Set up development environment',
            'Implement authentication system',
            'Build user dashboard',
            'Add payment integration',
            'Write test cases',
            'Perform security audit',
            'Deploy to staging',
            'User acceptance testing',
            'Prepare launch announcement',
            'Deploy to production'
        ];

        for (const task of projectTasks) {
            await addTodo(page, task);
        }

        // Mark first few as completed
        await toggleTodo(page, 1);
        await toggleTodo(page, 2);
        await toggleTodo(page, 3);
        await toggleTodo(page, 4); // Currently working on this

        await page.waitForTimeout(500);
        await takeScreenshot(page, 'project-planning', 'Project roadmap tracking');

        // Scenario 6: Weekend Planning
        console.log('\n📋 Scenario 6: Weekend Planning');
        
        await page.evaluate(() => {
            document.querySelectorAll('.todo-item').forEach(item => {
                const deleteBtn = item.querySelector('.delete-btn');
                if (deleteBtn) deleteBtn.click();
            });
        });
        await page.waitForTimeout(500);

        const weekendTodos = [
            'Saturday morning run',
            'Farmers market shopping',
            'Brunch with friends',
            'Clean the apartment',
            'Meal prep for next week',
            'Movie night',
            'Sunday yoga class',
            'Read for book club',
            'Plan next week\'s schedule',
            'Early bed for Monday'
        ];

        for (const todo of weekendTodos) {
            await addTodo(page, todo);
        }

        await page.waitForTimeout(500);
        await takeScreenshot(page, 'weekend-planning', 'Weekend activities and planning');

        console.log('\n✅ All realistic demos captured successfully!');
        console.log(`📁 Screenshots saved in: ${SCREENSHOT_DIR}`);
        
    } catch (error) {
        console.error('❌ Error during capture:', error);
    } finally {
        await browser.close();
    }
}

// Check server and run
async function checkServer() {
    try {
        const response = await fetch(TODO_APP_URL);
        return response.ok;
    } catch (error) {
        return false;
    }
}

console.log('🎯 Todo App Realistic Demo Generator');
console.log('==================================\n');

checkServer().then(isRunning => {
    if (!isRunning) {
        console.error(`❌ Server is not running at ${TODO_APP_URL}`);
        console.error('Please start the server first with the correct port.');
        process.exit(1);
    }
    return generateRealisticDemos();
}).catch(error => {
    console.error('Fatal error:', error);
    process.exit(1);
});