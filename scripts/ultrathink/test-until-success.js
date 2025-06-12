#!/usr/bin/env node

const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

class TestRunner {
    constructor() {
        this.maxAttempts = 10;
        this.attempt = 0;
        this.testLog = path.join(__dirname, 'test-results.log');
        this.planPath = path.join(__dirname, 'implementation-plan.json');
        this.plan = this.loadPlan();
    }

    loadPlan() {
        try {
            const data = fs.readFileSync(this.planPath, 'utf8');
            return JSON.parse(data).plan;
        } catch (error) {
            console.error('No implementation plan found');
            return { testCommand: 'npm test' };
        }
    }

    log(message, level = 'info') {
        const colors = {
            info: '\x1b[36m',
            success: '\x1b[32m',
            warn: '\x1b[33m',
            error: '\x1b[31m'
        };
        
        const timestamp = new Date().toISOString();
        const logMessage = `[${timestamp}] ${message}\n`;
        fs.appendFileSync(this.testLog, logMessage);
        console.log(`${colors[level]}${message}\x1b[0m`);
    }

    async runTests() {
        this.log('🧪 Starting test loop...');
        
        while (this.attempt < this.maxAttempts) {
            this.attempt++;
            this.log(`\n📍 Test attempt ${this.attempt}/${this.maxAttempts}`);
            
            const success = await this.runTestCommand();
            
            if (success) {
                this.log('✅ All tests passed!', 'success');
                this.saveResults(true);
                return true;
            }
            
            this.log('❌ Tests failed', 'error');
            
            // Try to fix common issues
            await this.attemptAutoFix();
            
            // Wait before retry
            if (this.attempt < this.maxAttempts) {
                this.log('⏳ Retrying in 3 seconds...', 'warn');
                await this.sleep(3000);
            }
        }
        
        this.log('💀 Max attempts reached. Tests still failing.', 'error');
        this.saveResults(false);
        return false;
    }

    runTestCommand() {
        const command = this.plan.testCommand || 'npm test';
        const [cmd, ...args] = command.split(' ');
        
        this.log(`🏃 Running: ${command}`);
        
        return new Promise((resolve) => {
            const proc = spawn(cmd, args, { 
                stdio: 'pipe',
                shell: true 
            });
            
            let output = '';
            let errorOutput = '';
            
            proc.stdout.on('data', (data) => {
                output += data.toString();
                process.stdout.write(data);
            });
            
            proc.stderr.on('data', (data) => {
                errorOutput += data.toString();
                process.stderr.write(data);
            });
            
            proc.on('close', (code) => {
                this.lastOutput = output;
                this.lastError = errorOutput;
                resolve(code === 0);
            });
            
            proc.on('error', (error) => {
                this.log(`Process error: ${error.message}`, 'error');
                resolve(false);
            });
        });
    }

    async attemptAutoFix() {
        this.log('🔧 Attempting automatic fixes...', 'warn');
        
        // Analyze error output
        const errors = this.analyzeErrors();
        
        for (const error of errors) {
            switch (error.type) {
                case 'missing-import':
                    await this.fixMissingImport(error);
                    break;
                case 'type-error':
                    await this.fixTypeError(error);
                    break;
                case 'compilation-error':
                    await this.fixCompilationError(error);
                    break;
                case 'test-assertion':
                    await this.fixTestAssertion(error);
                    break;
                case 'missing-file':
                    await this.fixMissingFile(error);
                    break;
                default:
                    this.log(`  ⚠️  Cannot auto-fix: ${error.message}`, 'warn');
            }
        }
    }

    analyzeErrors() {
        const errors = [];
        const output = this.lastOutput + this.lastError;
        
        // Check for missing imports
        if (output.includes('unresolved import') || output.includes('module not found')) {
            errors.push({
                type: 'missing-import',
                message: 'Missing import detected'
            });
        }
        
        // Check for type errors
        if (output.includes('expected') && output.includes('found')) {
            errors.push({
                type: 'type-error',
                message: 'Type mismatch detected'
            });
        }
        
        // Check for compilation errors
        if (output.includes('error[E')) {
            errors.push({
                type: 'compilation-error',
                message: 'Rust compilation error'
            });
        }
        
        // Check for test failures
        if (output.includes('assertion failed') || output.includes('test failed')) {
            errors.push({
                type: 'test-assertion',
                message: 'Test assertion failed'
            });
        }
        
        // Check for missing files
        if (output.includes('No such file or directory')) {
            errors.push({
                type: 'missing-file',
                message: 'Missing file'
            });
        }
        
        return errors;
    }

    async fixMissingImport(error) {
        this.log('  🔧 Adding missing imports...', 'warn');
        // In a real implementation, this would analyze the error and add imports
        await this.runCommand('cargo', ['fix', '--allow-dirty']);
    }

    async fixTypeError(error) {
        this.log('  🔧 Attempting type fixes...', 'warn');
        await this.runCommand('cargo', ['clippy', '--fix', '--allow-dirty']);
    }

    async fixCompilationError(error) {
        this.log('  🔧 Running cargo check...', 'warn');
        await this.runCommand('cargo', ['check']);
    }

    async fixTestAssertion(error) {
        this.log('  🔧 Test assertions need manual review', 'warn');
        // This would need manual intervention
    }

    async fixMissingFile(error) {
        this.log('  🔧 Creating missing files...', 'warn');
        // Would create stub files based on error
    }

    runCommand(command, args) {
        return new Promise((resolve) => {
            const proc = spawn(command, args, { stdio: 'inherit' });
            proc.on('close', resolve);
        });
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    saveResults(success) {
        const results = {
            timestamp: new Date().toISOString(),
            success: success,
            attempts: this.attempt,
            command: this.plan.testCommand,
            duration: Date.now() - this.startTime
        };
        
        fs.writeFileSync(
            path.join(__dirname, 'test-results.json'),
            JSON.stringify(results, null, 2)
        );
    }
}

// Main execution
const runner = new TestRunner();
runner.startTime = Date.now();

runner.runTests().then(success => {
    process.exit(success ? 0 : 1);
}).catch(error => {
    console.error('Test runner failed:', error);
    process.exit(1);
});