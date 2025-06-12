#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

class PerformanceAnalyzer {
    constructor() {
        this.results = {
            timestamp: new Date().toISOString(),
            metrics: {}
        };
    }

    async analyze() {
        console.log('⚡ Layer9 Performance Analysis');
        console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
        
        // Analyze WASM bundle size
        await this.analyzeWasmSize();
        
        // Analyze Rust compilation time
        await this.analyzeCompilationTime();
        
        // Analyze server performance
        await this.analyzeServerPerformance();
        
        // Analyze memory usage
        await this.analyzeMemoryUsage();
        
        // Generate report
        this.generateReport();
    }

    async analyzeWasmSize() {
        console.log('\n📦 WASM Bundle Analysis');
        
        const wasmPath = path.join(process.cwd(), 'examples/counter/pkg');
        if (fs.existsSync(wasmPath)) {
            const files = fs.readdirSync(wasmPath);
            const wasmFile = files.find(f => f.endsWith('.wasm'));
            
            if (wasmFile) {
                const stats = fs.statSync(path.join(wasmPath, wasmFile));
                const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
                
                console.log(`  WASM size: ${sizeMB}MB`);
                console.log(`  Status: ${sizeMB > 1 ? '⚠️  Needs optimization' : '✅ Good'}`);
                
                this.results.metrics.wasmSize = {
                    bytes: stats.size,
                    mb: parseFloat(sizeMB),
                    status: sizeMB > 1 ? 'needs-optimization' : 'good'
                };
            }
        }
        
        // Check JS bundle
        const jsFiles = fs.readdirSync(wasmPath).filter(f => f.endsWith('.js'));
        let totalJsSize = 0;
        jsFiles.forEach(file => {
            const stats = fs.statSync(path.join(wasmPath, file));
            totalJsSize += stats.size;
        });
        
        console.log(`  JS size: ${(totalJsSize / 1024).toFixed(2)}KB`);
        this.results.metrics.jsSize = totalJsSize;
    }

    async analyzeCompilationTime() {
        console.log('\n🔨 Compilation Performance');
        
        try {
            const start = Date.now();
            execSync('cargo build --release --target wasm32-unknown-unknown', {
                cwd: path.join(process.cwd(), 'examples/counter'),
                stdio: 'pipe'
            });
            const duration = Date.now() - start;
            
            console.log(`  Release build: ${(duration / 1000).toFixed(2)}s`);
            this.results.metrics.compilationTime = duration;
        } catch (error) {
            console.log('  ⚠️  Could not measure compilation time');
        }
    }

    async analyzeServerPerformance() {
        console.log('\n🚀 Server Performance');
        
        // Check if server is running
        try {
            const response = await fetch('http://localhost:8080');
            if (response.ok) {
                // Measure response times
                const times = [];
                for (let i = 0; i < 10; i++) {
                    const start = Date.now();
                    await fetch('http://localhost:8080');
                    times.push(Date.now() - start);
                }
                
                const avg = times.reduce((a, b) => a + b, 0) / times.length;
                const min = Math.min(...times);
                const max = Math.max(...times);
                
                console.log(`  Average response: ${avg.toFixed(2)}ms`);
                console.log(`  Min/Max: ${min}ms / ${max}ms`);
                
                this.results.metrics.serverResponse = { avg, min, max };
            } else {
                console.log('  ⚠️  Server not running');
            }
        } catch (error) {
            console.log('  ⚠️  Server not accessible');
        }
    }

    async analyzeMemoryUsage() {
        console.log('\n💾 Memory Analysis');
        
        try {
            // Get Rust server process
            const processes = execSync("ps aux | grep layer9-server | grep -v grep", { encoding: 'utf8' });
            const lines = processes.trim().split('\n');
            
            if (lines.length > 0) {
                const parts = lines[0].split(/\s+/);
                const memory = parseFloat(parts[5]) / 1024; // Convert to MB
                
                console.log(`  Server memory: ${memory.toFixed(2)}MB`);
                this.results.metrics.serverMemory = memory;
            }
        } catch (error) {
            console.log('  ⚠️  Could not analyze memory');
        }
    }

    generateReport() {
        console.log('\n📊 Performance Summary');
        console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
        
        const metrics = this.results.metrics;
        
        // Overall score
        let score = 100;
        if (metrics.wasmSize && metrics.wasmSize.mb > 1) score -= 20;
        if (metrics.serverResponse && metrics.serverResponse.avg > 10) score -= 10;
        if (metrics.serverMemory && metrics.serverMemory > 50) score -= 10;
        
        console.log(`\n🏆 Performance Score: ${score}/100`);
        
        // Recommendations
        console.log('\n💡 Recommendations:');
        if (metrics.wasmSize && metrics.wasmSize.mb > 1) {
            console.log('  - Optimize WASM bundle size (currently ' + metrics.wasmSize.mb + 'MB)');
            console.log('    • Enable wasm-opt level 3');
            console.log('    • Use wee_alloc for smaller memory footprint');
            console.log('    • Remove debug symbols in release builds');
        }
        
        if (metrics.serverResponse && metrics.serverResponse.avg > 10) {
            console.log('  - Improve server response time');
            console.log('    • Enable compression');
            console.log('    • Add caching headers');
        }
        
        // Save report
        const reportPath = path.join(__dirname, 'performance-report.json');
        fs.writeFileSync(reportPath, JSON.stringify(this.results, null, 2));
        console.log(`\n📁 Full report saved: ${reportPath}`);
    }
}

// Main execution
const analyzer = new PerformanceAnalyzer();
analyzer.analyze().catch(error => {
    console.error('Analysis failed:', error);
    process.exit(1);
});