#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function generateCommitMessage() {
    try {
        // Get git status
        const status = execSync('git status --porcelain', { encoding: 'utf8' });
        const diff = execSync('git diff --cached --stat', { encoding: 'utf8' });
        
        // Analyze changes
        const files = status.split('\n').filter(line => line.trim());
        const fileCount = files.length;
        
        // Categorize changes
        const categories = {
            rust: files.filter(f => f.endsWith('.rs')).length,
            js: files.filter(f => f.endsWith('.js')).length,
            docs: files.filter(f => f.endsWith('.md')).length,
            config: files.filter(f => f.includes('Cargo.toml') || f.includes('package.json')).length,
            test: files.filter(f => f.includes('test')).length
        };
        
        // Check for completed TODOs
        let completedTodo = null;
        try {
            const commitMsgPath = path.join(__dirname, 'commit-message.txt');
            if (fs.existsSync(commitMsgPath)) {
                completedTodo = fs.readFileSync(commitMsgPath, 'utf8');
                fs.unlinkSync(commitMsgPath); // Clean up
                console.log(completedTodo);
                return;
            }
        } catch (e) {
            // No automated commit message
        }
        
        // Generate smart commit message
        let message = '';
        let description = [];
        
        // Determine primary change type
        if (categories.rust > categories.js) {
            message = '🦀 Update Rust implementation';
            if (categories.test > 0) description.push('Add tests');
        } else if (categories.js > 0) {
            message = '🔧 Update JavaScript tooling';
        } else if (categories.docs > 0) {
            message = '📚 Update documentation';
        } else if (categories.config > 0) {
            message = '⚙️ Update configuration';
        } else {
            message = '🚀 Update Layer9 framework';
        }
        
        // Add specific changes
        if (files.some(f => f.includes('layer9-server'))) {
            description.push('Improve Rust server');
        }
        if (files.some(f => f.includes('ultrathink'))) {
            description.push('Enhance automation tooling');
        }
        if (files.some(f => f.includes('Makefile'))) {
            description.push('Update build system');
        }
        
        // Build final message
        if (description.length > 0) {
            message += '\n\n' + description.map(d => `- ${d}`).join('\n');
        }
        
        message += '\n\n🤖 Generated with [Claude Code](https://claude.ai/code)\n\nCo-Authored-By: Claude <noreply@anthropic.com>';
        
        console.log(message);
        
    } catch (error) {
        // Fallback message
        console.log(`🚀 Update Layer9 framework

- Implement improvements
- Update tooling
- Enhance automation

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>`);
    }
}

generateCommitMessage();