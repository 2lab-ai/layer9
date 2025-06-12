const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const GIFS_DIR = path.join(__dirname, '../../assets/gifs');

// Optimization profiles
const PROFILES = {
    readme: {
        // For README headers - very small
        maxWidth: 600,
        maxHeight: 400,
        colors: 64,
        lossy: 80,
        maxSizeMB: 2
    },
    documentation: {
        // For detailed documentation
        maxWidth: 800,
        maxHeight: 600,
        colors: 128,
        lossy: 30,
        maxSizeMB: 5
    },
    showcase: {
        // For main showcase - balance quality/size
        maxWidth: 800,
        maxHeight: 600,
        colors: 96,
        lossy: 50,
        maxSizeMB: 3
    }
};

// Check if gifsicle is installed
function checkGifsicle() {
    try {
        execSync('gifsicle --version', { stdio: 'pipe' });
        return true;
    } catch (error) {
        console.error('❌ gifsicle is not installed');
        console.error('\nInstall with:');
        console.error('  npm: npm install -g gifsicle');
        console.error('  brew: brew install gifsicle');
        console.error('  apt: sudo apt-get install gifsicle\n');
        return false;
    }
}

// Get file size in MB
function getFileSizeMB(filePath) {
    const stats = fs.statSync(filePath);
    return stats.size / 1024 / 1024;
}

// Optimize a single GIF
function optimizeGif(inputPath, outputPath, profile) {
    const originalSize = getFileSizeMB(inputPath);
    console.log(`\n📄 ${path.basename(inputPath)}`);
    console.log(`   Original size: ${originalSize.toFixed(2)} MB`);
    
    // Build gifsicle command
    let command = 'gifsicle';
    
    // Optimization level
    command += ' -O3';
    
    // Color reduction
    command += ` --colors ${profile.colors}`;
    
    // Lossy compression (if supported)
    command += ` --lossy=${profile.lossy}`;
    
    // Resize if needed
    const dimensions = execSync(`gifsicle --info "${inputPath}" | grep "logical screen" | head -1`, { shell: true })
        .toString()
        .match(/(\d+)x(\d+)/);
    
    if (dimensions) {
        const [, width, height] = dimensions;
        const currentWidth = parseInt(width);
        const currentHeight = parseInt(height);
        
        if (currentWidth > profile.maxWidth || currentHeight > profile.maxHeight) {
            const scale = Math.min(
                profile.maxWidth / currentWidth,
                profile.maxHeight / currentHeight
            );
            const newWidth = Math.floor(currentWidth * scale);
            const newHeight = Math.floor(currentHeight * scale);
            command += ` --resize ${newWidth}x${newHeight}`;
            console.log(`   Resizing: ${currentWidth}x${currentHeight} → ${newWidth}x${newHeight}`);
        }
    }
    
    // Input and output
    command += ` "${inputPath}" > "${outputPath}"`;
    
    try {
        execSync(command, { shell: true, stdio: 'pipe' });
        
        const newSize = getFileSizeMB(outputPath);
        const reduction = ((originalSize - newSize) / originalSize * 100).toFixed(1);
        
        console.log(`   Optimized size: ${newSize.toFixed(2)} MB`);
        console.log(`   ✅ Reduced by ${reduction}%`);
        
        // Check if still too large
        if (newSize > profile.maxSizeMB) {
            console.log(`   ⚠️  Still larger than target ${profile.maxSizeMB} MB`);
            
            // Try more aggressive optimization
            if (profile.colors > 32) {
                console.log('   Trying more aggressive optimization...');
                const aggressiveProfile = { ...profile, colors: Math.floor(profile.colors / 2) };
                optimizeGif(inputPath, outputPath, aggressiveProfile);
            }
        }
        
        return true;
    } catch (error) {
        console.error(`   ❌ Optimization failed: ${error.message}`);
        return false;
    }
}

// Main optimization function
function optimizeAllGifs() {
    if (!checkGifsicle()) {
        process.exit(1);
    }
    
    console.log('🎨 Layer9 GIF Optimization Tool');
    console.log('===============================\n');
    
    // Create optimized directory
    const optimizedDir = path.join(GIFS_DIR, 'optimized');
    if (!fs.existsSync(optimizedDir)) {
        fs.mkdirSync(optimizedDir, { recursive: true });
    }
    
    // Get all GIF files
    const gifFiles = fs.readdirSync(GIFS_DIR)
        .filter(f => f.endsWith('.gif') && !f.includes('optimized'));
    
    if (gifFiles.length === 0) {
        console.log('No GIF files found to optimize.');
        console.log('Run the capture pipeline first: ./capture-and-create-gifs.sh');
        return;
    }
    
    console.log(`Found ${gifFiles.length} GIF files to optimize.\n`);
    
    // Process each GIF
    let totalOriginal = 0;
    let totalOptimized = 0;
    
    gifFiles.forEach(file => {
        const inputPath = path.join(GIFS_DIR, file);
        
        // Determine profile based on filename
        let profile = PROFILES.documentation;
        if (file.includes('overview') || file.includes('showcase')) {
            profile = PROFILES.showcase;
        } else if (file.includes('header') || file.includes('small')) {
            profile = PROFILES.readme;
        }
        
        // Create optimized versions
        const optimizedPath = path.join(optimizedDir, file);
        
        const originalSize = getFileSizeMB(inputPath);
        totalOriginal += originalSize;
        
        if (optimizeGif(inputPath, optimizedPath, profile)) {
            totalOptimized += getFileSizeMB(optimizedPath);
        }
    });
    
    // Summary
    console.log('\n📊 Optimization Summary');
    console.log('======================');
    console.log(`Total original size:  ${totalOriginal.toFixed(2)} MB`);
    console.log(`Total optimized size: ${totalOptimized.toFixed(2)} MB`);
    console.log(`Total reduction:      ${(totalOriginal - totalOptimized).toFixed(2)} MB`);
    console.log(`Reduction percentage: ${((totalOriginal - totalOptimized) / totalOriginal * 100).toFixed(1)}%`);
    
    console.log('\n✅ Optimization complete!');
    console.log(`\n📁 Optimized GIFs saved to: ${optimizedDir}`);
    
    // Create comparison script
    const compareScript = path.join(GIFS_DIR, 'compare-sizes.sh');
    const scriptContent = `#!/bin/bash
echo "GIF Size Comparison"
echo "=================="
echo ""
echo "Original GIFs:"
ls -lh *.gif | grep -v optimized
echo ""
echo "Optimized GIFs:"
ls -lh optimized/*.gif
`;
    
    fs.writeFileSync(compareScript, scriptContent);
    fs.chmodSync(compareScript, '755');
    console.log(`\nCreated comparison script: ${compareScript}`);
}

// Custom optimization for specific use cases
function createReadmeGifs() {
    console.log('\n🎯 Creating README-optimized GIFs...\n');
    
    const readmeGifs = {
        'layer9-examples-showcase.gif': 'layer9-readme-header.gif',
        'counter-overview.gif': 'counter-readme.gif',
        'todo-app-overview.gif': 'todo-readme.gif',
        'async-counter-overview.gif': 'async-counter-readme.gif',
        'memory-game-overview.gif': 'memory-game-readme.gif'
    };
    
    const readmeDir = path.join(GIFS_DIR, 'readme');
    if (!fs.existsSync(readmeDir)) {
        fs.mkdirSync(readmeDir, { recursive: true });
    }
    
    Object.entries(readmeGifs).forEach(([source, target]) => {
        const inputPath = path.join(GIFS_DIR, source);
        const outputPath = path.join(readmeDir, target);
        
        if (fs.existsSync(inputPath)) {
            optimizeGif(inputPath, outputPath, PROFILES.readme);
        }
    });
    
    console.log(`\n📁 README GIFs saved to: ${readmeDir}`);
}

// Run based on command line arguments
const args = process.argv.slice(2);

if (args.includes('--readme')) {
    createReadmeGifs();
} else {
    optimizeAllGifs();
    
    if (args.includes('--with-readme')) {
        createReadmeGifs();
    }
}

// Usage instructions
if (args.includes('--help')) {
    console.log('Usage: node optimize-gifs.js [options]');
    console.log('');
    console.log('Options:');
    console.log('  --readme        Create README-optimized versions only');
    console.log('  --with-readme   Optimize all GIFs and create README versions');
    console.log('  --help          Show this help message');
}