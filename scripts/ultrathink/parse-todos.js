#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

class TodoParser {
    constructor() {
        this.todos = [];
        this.categories = {
            critical: [],
            high: [],
            medium: [],
            low: [],
            completed: []
        };
    }

    parseREADME() {
        const readmePath = path.join(process.cwd(), 'README.md');
        const content = fs.readFileSync(readmePath, 'utf8');
        
        // Parse different TODO sections
        this.parseCriticalSection(content);
        this.parseHighPrioritySection(content);
        this.parseMediumPrioritySection(content);
        this.parseNiceToHaveSection(content);
        this.parseInlineMarkers(content);
    }

    parseCriticalSection(content) {
        const criticalMatch = content.match(/### 🚨 URGENT: Fix Our Lies\n([\s\S]*?)(?=\n###|\n##|$)/);
        if (criticalMatch) {
            const items = this.extractListItems(criticalMatch[1]);
            items.forEach(item => {
                const cleanText = this.cleanTodoText(item);
                if (!this.isCompleted(cleanText, content)) {
                    this.categories.critical.push({
                        text: cleanText,
                        priority: 'critical',
                        type: 'fix'
                    });
                }
            });
        }
    }

    parseHighPrioritySection(content) {
        const highMatch = content.match(/### (?:High Priority|Critical \(Blocking.*?\))\n([\s\S]*?)(?=\n###|\n##|$)/g);
        if (highMatch) {
            highMatch.forEach(section => {
                const items = this.extractListItems(section);
                items.forEach(item => {
                    const cleanText = this.cleanTodoText(item);
                    if (!this.isCompleted(cleanText, content)) {
                        this.categories.high.push({
                            text: cleanText,
                            priority: 'high',
                            type: 'feature'
                        });
                    }
                });
            });
        }
    }

    parseMediumPrioritySection(content) {
        const mediumMatch = content.match(/### Medium Priority\n([\s\S]*?)(?=\n###|\n##|$)/);
        if (mediumMatch) {
            const items = this.extractListItems(mediumMatch[1]);
            items.forEach(item => {
                const cleanText = this.cleanTodoText(item);
                if (!this.isCompleted(cleanText, content)) {
                    this.categories.medium.push({
                        text: cleanText,
                        priority: 'medium',
                        type: 'feature'
                    });
                }
            });
        }
    }

    parseNiceToHaveSection(content) {
        const niceMatch = content.match(/### Nice to Have\n([\s\S]*?)(?=\n###|\n##|$)/);
        if (niceMatch) {
            const items = this.extractListItems(niceMatch[1]);
            items.forEach(item => {
                const cleanText = this.cleanTodoText(item);
                if (!this.isCompleted(cleanText, content)) {
                    this.categories.low.push({
                        text: cleanText,
                        priority: 'low',
                        type: 'feature'
                    });
                }
            });
        }
    }

    parseInlineMarkers(content) {
        // Look for ❌ markers indicating not implemented
        const notImplementedMatch = content.matchAll(/- ❌ ([^\n]+)/g);
        for (const match of notImplementedMatch) {
            this.categories.high.push({
                text: this.cleanTodoText(match[1]),
                priority: 'high',
                type: 'missing'
            });
        }

        // Look for ⭕ markers
        const todoMatch = content.matchAll(/- ⭕ ([^\n]+)/g);
        for (const match of todoMatch) {
            this.categories.medium.push({
                text: this.cleanTodoText(match[1]),
                priority: 'medium',
                type: 'missing'
            });
        }
    }

    extractListItems(text) {
        const items = [];
        const lines = text.split('\n');
        
        for (const line of lines) {
            if (line.match(/^- \[[x ]\]/)) {
                items.push(line);
            }
        }
        
        return items;
    }

    cleanTodoText(text) {
        return text
            .replace(/^- \[[x ]\]\s*/, '')
            .replace(/^- /, '')
            .replace(/\*\*/g, '')
            .replace(/❌|⭕|✅/g, '')
            .trim();
    }

    isCompleted(itemText, content) {
        // Check if it's mentioned as completed elsewhere
        const lowerText = itemText.toLowerCase();
        
        // Special case: Python server elimination (be specific)
        if ((lowerText.includes('replace python server') || 
             lowerText.includes('python server with rust') ||
             (lowerText.includes('python') && lowerText.includes('server') && !lowerText.includes('hot reload')))) {
            return content.includes('✅ UPDATE: Python Web Server ELIMINATED!') ||
                   content.includes('✅ UPDATE: Python has been ELIMINATED!');
        }
        
        // Check for specific completed markers
        if (content.includes(`- [x] ${itemText}`) || 
            content.includes(`✅ ${itemText}`)) {
            return true;
        }
        
        // Check HONEST_STATUS.md for completion
        try {
            const honestStatus = fs.readFileSync(
                path.join(process.cwd(), 'HONEST_STATUS.md'), 
                'utf8'
            );
            if (honestStatus.includes('Python Web Server ELIMINATED') && 
                lowerText.includes('python server')) {
                return true;
            }
        } catch (e) {
            // File doesn't exist
        }
        
        return false;
    }

    getAllTodos() {
        return [
            ...this.categories.critical,
            ...this.categories.high,
            ...this.categories.medium,
            ...this.categories.low
        ];
    }

    displayList() {
        console.log('\n🚨 CRITICAL (Stop the lies!)');
        console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
        this.categories.critical.forEach((todo, i) => {
            console.log(`${i + 1}. ${todo.text}`);
        });

        console.log('\n🔴 HIGH PRIORITY');
        console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
        this.categories.high.forEach((todo, i) => {
            console.log(`${i + 1}. ${todo.text}`);
        });

        console.log('\n🟡 MEDIUM PRIORITY');
        console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
        this.categories.medium.forEach((todo, i) => {
            console.log(`${i + 1}. ${todo.text}`);
        });

        console.log('\n🟢 NICE TO HAVE');
        console.log('━━━━━━━━━━━━━━━━━━━━━━━━━━━━');
        this.categories.low.forEach((todo, i) => {
            console.log(`${i + 1}. ${todo.text}`);
        });
    }

    displayStatus() {
        const total = this.getAllTodos().length;
        const critical = this.categories.critical.length;
        const high = this.categories.high.length;
        const medium = this.categories.medium.length;
        const low = this.categories.low.length;
        
        console.log('\n📊 Implementation Status');
        console.log('━━━━━━━━━━━━━━━━━━━━━━━━');
        console.log(`Total TODOs: ${total}`);
        console.log(`  🚨 Critical: ${critical}`);
        console.log(`  🔴 High: ${high}`);
        console.log(`  🟡 Medium: ${medium}`);
        console.log(`  🟢 Low: ${low}`);
        console.log('');
        console.log('Progress bars:');
        console.log(`Critical: ${this.makeProgressBar(0, critical)}`);
        console.log(`High:     ${this.makeProgressBar(0, high)}`);
        console.log(`Medium:   ${this.makeProgressBar(0, medium)}`);
        console.log(`Low:      ${this.makeProgressBar(0, low)}`);
        
        // Estimate completion
        const implementedWeight = 30; // 30% complete as stated
        console.log(`\nOverall Progress: ${this.makeProgressBar(implementedWeight, 100)} ${implementedWeight}%`);
    }

    makeProgressBar(complete, total) {
        const width = 20;
        const filled = Math.round((complete / total) * width);
        const empty = width - filled;
        return `[${'█'.repeat(filled)}${' '.repeat(empty)}]`;
    }

    saveToJSON() {
        const output = {
            timestamp: new Date().toISOString(),
            categories: this.categories,
            total: this.getAllTodos().length,
            summary: {
                critical: this.categories.critical.length,
                high: this.categories.high.length,
                medium: this.categories.medium.length,
                low: this.categories.low.length
            }
        };
        
        fs.writeFileSync(
            path.join(__dirname, 'todos.json'),
            JSON.stringify(output, null, 2)
        );
    }
}

// Main execution
const parser = new TodoParser();
parser.parseREADME();

const args = process.argv.slice(2);
if (args.includes('--list')) {
    parser.displayList();
} else if (args.includes('--status')) {
    parser.displayStatus();
} else {
    parser.saveToJSON();
    console.log(`✅ Parsed ${parser.getAllTodos().length} TODOs from README.md`);
}