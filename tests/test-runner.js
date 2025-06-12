const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// ANSI color codes
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    cyan: '\x1b[36m',
    magenta: '\x1b[35m'
};

// Test configurations
const tests = [
    {
        name: 'Counter Example',
        script: 'e2e/counter.test.js',
        url: 'http://localhost:8000/examples/counter/'
    },
    {
        name: 'Async Counter Example',
        script: 'e2e/async-counter.test.js',
        url: 'http://localhost:8000/examples/async-counter/'
    },
    {
        name: 'Todo App Example',
        script: 'e2e/todo-app.test.js',
        url: 'http://localhost:8000/examples/todo-app/'
    },
    {
        name: 'Memory Game Example',
        script: 'e2e/memory-game.test.js',
        url: 'http://localhost:8000/examples/memory-game/'
    }
];

// Helper function to check if server is running
async function checkServer() {
    try {
        const response = await fetch('http://localhost:8000');
        return response.ok;
    } catch (error) {
        return false;
    }
}

// Helper function to run a single test
function runTest(test) {
    return new Promise((resolve, reject) => {
        console.log(`\n${colors.cyan}▶ Running: ${test.name}${colors.reset}`);
        console.log(`${colors.bright}${colors.blue}${'─'.repeat(50)}${colors.reset}\n`);

        const startTime = Date.now();
        const testProcess = spawn('node', [test.script], {
            cwd: __dirname,
            stdio: 'inherit'
        });

        testProcess.on('close', (code) => {
            const duration = ((Date.now() - startTime) / 1000).toFixed(2);
            
            if (code === 0) {
                console.log(`\n${colors.green}✅ ${test.name} passed in ${duration}s${colors.reset}`);
                resolve({ test: test.name, passed: true, duration });
            } else {
                console.log(`\n${colors.red}❌ ${test.name} failed in ${duration}s${colors.reset}`);
                resolve({ test: test.name, passed: false, duration });
            }
        });

        testProcess.on('error', (error) => {
            console.error(`${colors.red}Error running ${test.name}:${colors.reset}`, error);
            resolve({ test: test.name, passed: false, error: error.message });
        });
    });
}

// Main function to run all tests
async function runAllTests() {
    console.log(`${colors.bright}${colors.magenta}
╔══════════════════════════════════════════════════════════╗
║     Layer9 Comprehensive Test Suite                      ║
║     Testing all working examples with Puppeteer          ║
╚══════════════════════════════════════════════════════════╝${colors.reset}
`);

    // Check if server is running
    console.log(`${colors.yellow}🔍 Checking if HTTP server is running...${colors.reset}`);
    const serverRunning = await checkServer();
    
    if (!serverRunning) {
        console.error(`\n${colors.red}❌ HTTP server is not running!${colors.reset}`);
        console.error(`\n${colors.yellow}Please start the server first:${colors.reset}`);
        console.error(`  ${colors.cyan}cd ..${colors.reset}`);
        console.error(`  ${colors.cyan}python3 -m http.server 8000${colors.reset}\n`);
        process.exit(1);
    }
    
    console.log(`${colors.green}✅ Server is running${colors.reset}\n`);

    // Create screenshots directory
    const screenshotsDir = path.join(__dirname, 'screenshots');
    if (!fs.existsSync(screenshotsDir)) {
        fs.mkdirSync(screenshotsDir, { recursive: true });
    }

    // Run all tests sequentially
    const results = [];
    const startTime = Date.now();

    for (const test of tests) {
        const result = await runTest(test);
        results.push(result);
    }

    const totalDuration = ((Date.now() - startTime) / 1000).toFixed(2);

    // Generate summary
    console.log(`\n${colors.bright}${colors.blue}${'═'.repeat(60)}${colors.reset}`);
    console.log(`${colors.bright}${colors.cyan}📊 TEST SUMMARY${colors.reset}`);
    console.log(`${colors.bright}${colors.blue}${'═'.repeat(60)}${colors.reset}\n`);

    const passed = results.filter(r => r.passed).length;
    const failed = results.filter(r => !r.passed).length;

    results.forEach(result => {
        const status = result.passed ? 
            `${colors.green}✅ PASSED${colors.reset}` : 
            `${colors.red}❌ FAILED${colors.reset}`;
        const duration = result.duration ? ` (${result.duration}s)` : '';
        console.log(`  ${status} ${result.test}${duration}`);
    });

    console.log(`\n${colors.bright}Total Tests: ${tests.length}${colors.reset}`);
    console.log(`${colors.green}Passed: ${passed}${colors.reset}`);
    console.log(`${colors.red}Failed: ${failed}${colors.reset}`);
    console.log(`${colors.yellow}Duration: ${totalDuration}s${colors.reset}`);

    // Generate test report
    const reportPath = path.join(__dirname, 'test-report.json');
    const report = {
        timestamp: new Date().toISOString(),
        totalTests: tests.length,
        passed,
        failed,
        duration: totalDuration,
        results: results.map(r => ({
            ...r,
            screenshotsPath: path.join('screenshots', r.test.toLowerCase().replace(/\s+/g, '-'))
        }))
    };

    fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
    console.log(`\n${colors.cyan}📄 Test report saved to: ${reportPath}${colors.reset}`);
    console.log(`${colors.cyan}📸 Screenshots saved in: ${screenshotsDir}${colors.reset}`);

    // Exit with appropriate code
    if (failed > 0) {
        console.log(`\n${colors.red}${colors.bright}Some tests failed!${colors.reset}\n`);
        process.exit(1);
    } else {
        console.log(`\n${colors.green}${colors.bright}All tests passed!${colors.reset}\n`);
        process.exit(0);
    }
}

// Handle uncaught errors
process.on('unhandledRejection', (error) => {
    console.error(`\n${colors.red}Unhandled error:${colors.reset}`, error);
    process.exit(1);
});

// Run the tests
runAllTests().catch(error => {
    console.error(`\n${colors.red}Fatal error:${colors.reset}`, error);
    process.exit(1);
});