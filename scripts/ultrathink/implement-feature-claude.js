#!/usr/bin/env node

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

class ClaudeFeatureImplementer {
    constructor(todoText) {
        this.todo = todoText;
        this.logFile = path.join(__dirname, 'implementation.log');
        this.startTime = Date.now();
        this.projectRoot = path.resolve(__dirname, '../..');
    }

    log(message) {
        const timestamp = new Date().toISOString();
        const logMessage = `[${timestamp}] ${message}\n`;
        fs.appendFileSync(this.logFile, logMessage);
        console.log(message);
    }

    async implement() {
        this.log(`🚀 Starting Claude-powered implementation: "${this.todo}"`);
        
        // Analyze what needs to be done
        const analysis = this.analyzeTodo();
        this.log(`📋 Analysis: ${analysis.type} implementation needed`);
        
        // Generate implementation prompt
        const prompt = this.generatePrompt(analysis);
        
        // Use Claude to implement
        this.log('🤖 Invoking Claude to implement feature...');
        await this.invokeClaudeImplementation(prompt);
        
        const duration = Math.round((Date.now() - this.startTime) / 1000);
        this.log(`✅ Implementation phase complete (${duration}s)`);
    }

    analyzeTodo() {
        const lower = this.todo.toLowerCase();
        
        if (lower.includes('counter example') && lower.includes('layer9')) {
            return { type: 'counter-example', priority: 'critical' };
        }
        if (lower.includes('ssr') || lower.includes('server-side')) {
            return { type: 'ssr', priority: 'high' };
        }
        if (lower.includes('state management') || lower.includes('hooks')) {
            return { type: 'state-management', priority: 'high' };
        }
        if (lower.includes('router') || lower.includes('routing')) {
            return { type: 'router', priority: 'high' };
        }
        if (lower.includes('component') || lower.includes('lifecycle')) {
            return { type: 'component', priority: 'medium' };
        }
        if (lower.includes('test')) {
            return { type: 'test', priority: 'medium' };
        }
        
        return { type: 'generic', priority: 'low' };
    }

    generatePrompt(analysis) {
        let basePrompt = `I need you to implement the following TODO from the Layer9 framework project: "${this.todo}"\n\n`;
        
        switch (analysis.type) {
            case 'counter-example':
                return basePrompt + `
The counter example currently uses raw DOM manipulation. You need to:
1. Read the current counter example at examples/counter/src/lib.rs
2. Understand the Layer9 component system by examining crates/core/src/component.rs and crates/core/src/state.rs
3. Rewrite the counter example to use Layer9 components, state hooks, and event handling
4. Ensure it still compiles to WASM and works correctly
5. The counter should increment/decrement when buttons are clicked
6. Use Layer9's html! macro and component system instead of raw DOM manipulation

Make sure to maintain the same functionality while demonstrating proper Layer9 framework usage.`;

            case 'ssr':
                return basePrompt + `
Server-Side Rendering is currently not implemented. You need to:
1. Examine crates/core/src/ssr.rs and identify placeholder code
2. Implement actual HTML generation from components
3. Add hydration markers for client-side rehydration
4. Update crates/layer9-server/src/main.rs to use the SSR functionality
5. Ensure components can render to HTML strings on the server
6. Handle async data fetching during SSR

Focus on getting basic SSR working first, then optimize.`;

            case 'state-management':
                return basePrompt + `
The state management system needs implementation. You need to:
1. Read crates/core/src/state.rs to understand the current structure
2. Implement reactive state updates that trigger re-renders
3. Add proper hook lifecycle (useState, useEffect, etc.)
4. Create a state provider system for global state
5. Ensure efficient re-rendering with minimal overhead
6. Add examples demonstrating state management

Make it similar to React hooks but adapted for Rust/WASM.`;

            case 'router':
                return basePrompt + `
The client-side router needs implementation. You need to:
1. Examine crates/core/src/router.rs and router_v2.rs
2. Implement history API integration for browser navigation
3. Add route matching with dynamic parameters (e.g., /user/:id)
4. Create navigation hooks (useNavigate, useParams)
5. Add route guards and middleware support
6. Integrate with the component system

Make it work seamlessly with the Layer9 component system.`;

            case 'test':
                return basePrompt + `
Implement comprehensive test infrastructure:
1. Create unit test utilities in crates/core/src/test.rs
2. Add integration tests for core functionality
3. Implement test helpers for component testing
4. Add proper test coverage for existing features
5. Create example tests that others can follow

Focus on making tests easy to write and maintain.`;

            default:
                return basePrompt + `
Analyze the codebase and implement this feature properly. Make sure to:
1. Follow existing code patterns and conventions
2. Add proper error handling
3. Include tests if applicable
4. Update any relevant documentation
5. Ensure the implementation is production-ready

Be thorough and implement the feature completely.`;
        }
    }

    async invokeClaudeImplementation(prompt) {
        try {
            // Save prompt to file for debugging
            const promptFile = path.join(__dirname, 'claude-prompt.txt');
            fs.writeFileSync(promptFile, prompt);
            
            // Change to project root directory
            process.chdir(this.projectRoot);
            
            // Create the claude command
            const claudeCommand = `claude "${prompt}"`;
            
            this.log('📝 Executing Claude implementation...');
            
            // Use spawn for better output handling
            return new Promise((resolve, reject) => {
                const claude = spawn('claude', [prompt], {
                    stdio: 'inherit',
                    shell: true,
                    cwd: this.projectRoot
                });
                
                claude.on('close', (code) => {
                    if (code === 0) {
                        this.log('✅ Claude implementation completed successfully');
                        
                        // Save implementation plan for test phase
                        this.saveImplementationPlan(this.getTestPlan());
                        
                        resolve();
                    } else {
                        this.log(`❌ Claude implementation failed with code ${code}`);
                        reject(new Error(`Claude exited with code ${code}`));
                    }
                });
                
                claude.on('error', (err) => {
                    this.log(`❌ Error executing Claude: ${err.message}`);
                    reject(err);
                });
            });
            
        } catch (error) {
            this.log(`❌ Failed to invoke Claude: ${error.message}`);
            throw error;
        }
    }

    getTestPlan() {
        const analysis = this.analyzeTodo();
        
        switch (analysis.type) {
            case 'counter-example':
                return {
                    type: 'counter-example',
                    files: [
                        'examples/counter/src/lib.rs',
                        'crates/core/src/component.rs',
                        'crates/core/src/state.rs'
                    ],
                    testCommand: 'npm run validate'
                };
            case 'ssr':
                return {
                    type: 'ssr',
                    files: [
                        'crates/core/src/ssr.rs',
                        'crates/core/src/vdom.rs',
                        'crates/layer9-server/src/main.rs'
                    ],
                    testCommand: 'cargo test ssr'
                };
            case 'state-management':
                return {
                    type: 'state-management',
                    files: [
                        'crates/core/src/state.rs',
                        'crates/core/src/component.rs'
                    ],
                    testCommand: 'cargo test state'
                };
            case 'router':
                return {
                    type: 'router',
                    files: [
                        'crates/core/src/router.rs',
                        'crates/core/src/router_v2.rs'
                    ],
                    testCommand: 'cargo test router'
                };
            case 'test':
                return {
                    type: 'test',
                    files: [
                        'crates/core/src/test.rs'
                    ],
                    testCommand: 'cargo test'
                };
            default:
                return {
                    type: 'generic',
                    testCommand: 'npm test'
                };
        }
    }

    saveImplementationPlan(plan) {
        // Still save the plan for the test phase
        fs.writeFileSync(
            path.join(__dirname, 'implementation-plan.json'),
            JSON.stringify({
                timestamp: new Date().toISOString(),
                todo: this.todo,
                plan: plan
            }, null, 2)
        );
    }
}

// Main execution
const todo = process.argv[2];
if (!todo) {
    console.error('No TODO provided');
    process.exit(1);
}

const implementer = new ClaudeFeatureImplementer(todo);
implementer.implement().catch(error => {
    console.error('Implementation failed:', error);
    process.exit(1);
});