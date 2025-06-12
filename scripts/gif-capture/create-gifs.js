const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const EXAMPLES_DIR = path.join(__dirname, '../../examples');
const OUTPUT_DIR = path.join(__dirname, '../../assets/gifs');
const GIFSICLE_OPTS = '--optimize=3 --colors=128';

// GIF configurations for each example and scenario
const GIF_CONFIGS = {
    counter: {
        name: 'Counter',
        scenarios: {
            increment: {
                delay: 60,
                resize: '800x600',
                output: 'counter-increment.gif'
            },
            decrement: {
                delay: 60,
                resize: '800x600',
                output: 'counter-decrement.gif'
            },
            reset: {
                delay: 100,
                resize: '800x600',
                output: 'counter-reset.gif'
            },
            'quick-actions': {
                delay: 40,
                resize: '800x600',
                output: 'counter-quick-actions.gif'
            },
            overview: {
                // Combine key frames from all scenarios
                files: ['increment/001-*', 'increment/004-*', 'reset/001-*', 'reset/002-*'],
                delay: 120,
                resize: '600x450',
                output: 'counter-overview.gif'
            }
        }
    },
    'async-counter': {
        name: 'Async Counter',
        scenarios: {
            loading: {
                delay: 80,
                resize: '800x600',
                output: 'async-counter-loading.gif'
            },
            'async-increment': {
                delay: 100,
                resize: '800x600',
                output: 'async-counter-increment.gif'
            },
            'async-decrement': {
                delay: 100,
                resize: '800x600',
                output: 'async-counter-decrement.gif'
            },
            overview: {
                files: ['loading/001-*', 'loading/002-*', 'loading/003-*', 'async-increment/005-*'],
                delay: 120,
                resize: '600x450',
                output: 'async-counter-overview.gif'
            }
        }
    },
    'todo-app': {
        name: 'Todo App',
        scenarios: {
            'add-todos': {
                delay: 80,
                resize: '800x600',
                output: 'todo-add-items.gif'
            },
            'complete-todos': {
                delay: 100,
                resize: '800x600',
                output: 'todo-complete-items.gif'
            },
            filter: {
                delay: 100,
                resize: '800x600',
                output: 'todo-filter.gif'
            },
            'delete-clear': {
                delay: 100,
                resize: '800x600',
                output: 'todo-delete-clear.gif'
            },
            overview: {
                files: ['add-todos/001-*', 'add-todos/007-*', 'complete-todos/003-*', 'filter/002-*', 'delete-clear/004-*'],
                delay: 150,
                resize: '600x450',
                output: 'todo-app-overview.gif'
            }
        }
    },
    'memory-game': {
        name: 'Memory Game',
        scenarios: {
            start: {
                delay: 100,
                resize: '800x600',
                output: 'memory-game-start.gif'
            },
            'flip-cards': {
                delay: 80,
                resize: '800x600',
                output: 'memory-game-flip.gif'
            },
            match: {
                delay: 100,
                resize: '800x600',
                output: 'memory-game-match.gif'
            },
            win: {
                delay: 150,
                resize: '800x600',
                output: 'memory-game-win.gif'
            },
            overview: {
                files: ['start/001-*', 'flip-cards/002-*', 'flip-cards/003-*', 'match/003-*'],
                delay: 120,
                resize: '600x450',
                output: 'memory-game-overview.gif'
            }
        }
    }
};

// Helper functions
function ensureDirectory(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

function checkImageMagick() {
    try {
        execSync('convert -version', { stdio: 'pipe' });
        return true;
    } catch (error) {
        console.error('❌ ImageMagick is not installed.');
        console.error('\nTo install:');
        console.error('  macOS:    brew install imagemagick');
        console.error('  Ubuntu:   sudo apt-get install imagemagick');
        console.error('  Windows:  Download from https://imagemagick.org/script/download.php\n');
        return false;
    }
}

function checkGifsicle() {
    try {
        execSync('gifsicle --version', { stdio: 'pipe' });
        return true;
    } catch (error) {
        console.warn('⚠️  gifsicle is not installed (optional, for better optimization)');
        console.warn('  Install with: npm install -g gifsicle');
        return false;
    }
}

function createGif(inputPattern, outputPath, options) {
    const { delay = 100, resize = '800x600', optimize = true } = options;
    
    try {
        // Build ImageMagick command
        let command = `convert -delay ${delay} -loop 0 ${inputPattern}`;
        
        if (resize) {
            command += ` -resize ${resize}`;
        }
        
        // Add optimization flags
        command += ' -coalesce -layers OptimizePlus';
        
        // Create temporary GIF
        const tempPath = outputPath + '.tmp';
        command += ` "${tempPath}"`;
        
        console.log(`  Creating: ${path.basename(outputPath)}`);
        execSync(command, { stdio: 'pipe' });
        
        // Optimize with gifsicle if available
        if (optimize && checkGifsicle()) {
            try {
                execSync(`gifsicle ${GIFSICLE_OPTS} "${tempPath}" > "${outputPath}"`, { shell: true });
                fs.unlinkSync(tempPath);
                console.log(`  ✅ Optimized: ${path.basename(outputPath)}`);
            } catch (error) {
                // Fallback to unoptimized version
                fs.renameSync(tempPath, outputPath);
                console.log(`  ✅ Created: ${path.basename(outputPath)} (unoptimized)`);
            }
        } else {
            fs.renameSync(tempPath, outputPath);
            console.log(`  ✅ Created: ${path.basename(outputPath)}`);
        }
        
        // Report file size
        const stats = fs.statSync(outputPath);
        const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
        console.log(`     Size: ${sizeMB} MB`);
        
    } catch (error) {
        console.error(`  ❌ Failed to create ${path.basename(outputPath)}: ${error.message}`);
    }
}

function createGifsForExample(example, config) {
    console.log(`\n📦 Creating GIFs for ${config.name}`);
    
    const exampleDir = path.join(EXAMPLES_DIR, example);
    const screenshotsDir = path.join(exampleDir, 'screenshots');
    
    if (!fs.existsSync(screenshotsDir)) {
        console.log(`  ⚠️  No screenshots found for ${config.name}`);
        return;
    }
    
    for (const [scenario, scenarioConfig] of Object.entries(config.scenarios)) {
        const scenarioDir = path.join(screenshotsDir, scenario);
        
        if (scenario === 'overview') {
            // Special handling for overview GIFs that combine multiple scenarios
            const patterns = scenarioConfig.files.map(pattern => 
                path.join(screenshotsDir, pattern)
            );
            
            const inputPattern = patterns.join(' ');
            const outputPath = path.join(OUTPUT_DIR, scenarioConfig.output);
            
            createGif(inputPattern, outputPath, scenarioConfig);
        } else if (fs.existsSync(scenarioDir)) {
            const inputPattern = path.join(scenarioDir, '*.png');
            const outputPath = path.join(OUTPUT_DIR, scenarioConfig.output);
            
            createGif(inputPattern, outputPath, scenarioConfig);
        }
    }
}

function createMasterGif() {
    console.log('\n🎯 Creating master showcase GIF');
    
    // Combine overview GIFs from all examples
    const overviewGifs = [];
    for (const config of Object.values(GIF_CONFIGS)) {
        if (config.scenarios.overview) {
            const gifPath = path.join(OUTPUT_DIR, config.scenarios.overview.output);
            if (fs.existsSync(gifPath)) {
                overviewGifs.push(gifPath);
            }
        }
    }
    
    if (overviewGifs.length > 0) {
        const outputPath = path.join(OUTPUT_DIR, 'layer9-examples-showcase.gif');
        const inputPattern = overviewGifs.join(' ');
        
        createGif(inputPattern, outputPath, {
            delay: 200,
            resize: '600x450',
            optimize: true
        });
    }
}

function generateSizeReport() {
    console.log('\n📊 GIF Size Report');
    console.log('==================');
    
    const files = fs.readdirSync(OUTPUT_DIR).filter(f => f.endsWith('.gif'));
    let totalSize = 0;
    
    files.forEach(file => {
        const stats = fs.statSync(path.join(OUTPUT_DIR, file));
        const sizeMB = stats.size / 1024 / 1024;
        totalSize += sizeMB;
        console.log(`  ${file.padEnd(40)} ${sizeMB.toFixed(2)} MB`);
    });
    
    console.log('  ' + '-'.repeat(50));
    console.log(`  Total:${' '.repeat(34)} ${totalSize.toFixed(2)} MB`);
}

// Main function
async function main() {
    console.log('🎬 Layer9 GIF Creation Tool');
    console.log('===========================\n');
    
    // Check dependencies
    if (!checkImageMagick()) {
        process.exit(1);
    }
    
    // Ensure output directory exists
    ensureDirectory(OUTPUT_DIR);
    
    // Create GIFs for each example
    for (const [example, config] of Object.entries(GIF_CONFIGS)) {
        createGifsForExample(example, config);
    }
    
    // Create master showcase GIF
    createMasterGif();
    
    // Generate size report
    generateSizeReport();
    
    console.log('\n✅ GIF creation complete!');
    console.log(`\n📁 GIFs saved to: ${OUTPUT_DIR}`);
    console.log('\nTo view a GIF:');
    console.log(`  open ${path.join(OUTPUT_DIR, 'layer9-examples-showcase.gif')}`);
}

// Run
main().catch(console.error);