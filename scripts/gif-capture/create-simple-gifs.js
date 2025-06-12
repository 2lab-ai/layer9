const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const SCREENSHOT_DIR = path.join(__dirname, '../../assets/screenshots');
const GIF_DIR = path.join(__dirname, '../../assets/gifs');

function ensureDirectory(dir) {
    if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
    }
}

function createGif(example, files) {
    console.log(`\n🎬 Creating GIF for ${example}...`);
    
    const outputPath = path.join(GIF_DIR, `${example}.gif`);
    const inputFiles = files.map(f => path.join(SCREENSHOT_DIR, f)).join(' ');
    
    // Create animated GIF with ImageMagick
    const command = `convert -delay 150 -loop 0 ${inputFiles} -resize 800x600 ${outputPath}`;
    
    try {
        execSync(command);
        const size = (fs.statSync(outputPath).size / 1024 / 1024).toFixed(2);
        console.log(`  ✅ Created: ${outputPath} (${size} MB)`);
        return true;
    } catch (error) {
        console.error(`  ❌ Failed: ${error.message}`);
        return false;
    }
}

function createGifs() {
    console.log('🎨 Creating GIFs from screenshots');
    console.log('================================\n');
    
    ensureDirectory(GIF_DIR);
    
    // Get all screenshots
    const screenshots = fs.readdirSync(SCREENSHOT_DIR)
        .filter(f => f.endsWith('.png'))
        .sort();
    
    // Group by example
    const examples = {};
    screenshots.forEach(file => {
        const example = file.split('-')[0];
        if (!examples[example]) examples[example] = [];
        examples[example].push(file);
    });
    
    // Create GIF for each example
    for (const [example, files] of Object.entries(examples)) {
        createGif(example, files);
    }
    
    // Create a combined showcase GIF
    console.log('\n🎯 Creating showcase GIF...');
    const showcaseFiles = [
        'counter-01-initial.png',
        'counter-02-after-increment.png',
        'todo-app-01-initial.png',
        'todo-app-02-with-todo.png',
        'memory-game-01-initial.png',
        'memory-game-02-card-flipped.png'
    ].filter(f => fs.existsSync(path.join(SCREENSHOT_DIR, f)));
    
    if (showcaseFiles.length > 0) {
        createGif('layer9-showcase', showcaseFiles);
    }
    
    console.log('\n✅ GIF creation complete!');
    console.log(`📁 GIFs saved to: ${GIF_DIR}`);
}

// Create optimized versions
function optimizeGifs() {
    console.log('\n🔧 Optimizing GIFs...');
    
    const gifs = fs.readdirSync(GIF_DIR)
        .filter(f => f.endsWith('.gif'));
    
    gifs.forEach(gif => {
        const inputPath = path.join(GIF_DIR, gif);
        const optimizedPath = path.join(GIF_DIR, 'optimized', gif);
        ensureDirectory(path.join(GIF_DIR, 'optimized'));
        
        // Optimize with reduced colors and size
        const command = `convert ${inputPath} -resize 600x450 -colors 64 -dither FloydSteinberg ${optimizedPath}`;
        
        try {
            execSync(command);
            const originalSize = (fs.statSync(inputPath).size / 1024 / 1024).toFixed(2);
            const optimizedSize = (fs.statSync(optimizedPath).size / 1024 / 1024).toFixed(2);
            console.log(`  ✅ ${gif}: ${originalSize} MB → ${optimizedSize} MB`);
        } catch (error) {
            console.error(`  ❌ Failed to optimize ${gif}`);
        }
    });
}

// Run
createGifs();
optimizeGifs();