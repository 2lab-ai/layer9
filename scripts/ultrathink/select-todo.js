#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

class TodoSelector {
    constructor() {
        this.todosPath = path.join(__dirname, 'todos.json');
        this.completedPath = path.join(__dirname, 'completed-todos.json');
        this.todos = this.loadTodos();
        this.completed = this.loadCompleted();
    }

    loadTodos() {
        try {
            const data = fs.readFileSync(this.todosPath, 'utf8');
            return JSON.parse(data);
        } catch (error) {
            console.error('Error loading todos. Run parse-todos.js first.');
            process.exit(1);
        }
    }
    
    loadCompleted() {
        try {
            const data = fs.readFileSync(this.completedPath, 'utf8');
            return JSON.parse(data);
        } catch (error) {
            return [];
        }
    }

    selectTodo() {
        // Priority order: critical > high > medium > low
        const priorities = ['critical', 'high', 'medium', 'low'];
        
        for (const priority of priorities) {
            const todos = this.todos.categories[priority];
            if (todos && todos.length > 0) {
                // Filter out completed TODOs
                const remaining = todos.filter(todo => 
                    !this.completed.some(completed => 
                        completed.text === todo.text || 
                        this.isSimilar(completed.text, todo.text)
                    )
                );
                
                if (remaining.length === 0) continue;
                
                // Select based on implementation difficulty and impact
                const selected = this.selectBestTodo(remaining);
                
                // Save selection
                this.saveSelection(selected);
                
                // Mark as completed
                this.markCompleted(selected);
                
                // Return just the text for the Makefile
                console.log(selected.text);
                return;
            }
        }
        
        console.error('No TODOs found to implement!');
        process.exit(1);
    }
    
    isSimilar(text1, text2) {
        // Check if two TODO texts are similar (to avoid duplicates)
        const normalize = (str) => str.toLowerCase().replace(/[^a-z0-9]/g, '');
        return normalize(text1).includes(normalize(text2)) || 
               normalize(text2).includes(normalize(text1));
    }
    
    markCompleted(todo) {
        this.completed.push({
            text: todo.text,
            completedAt: new Date().toISOString()
        });
        fs.writeFileSync(this.completedPath, JSON.stringify(this.completed, null, 2));
    }

    selectBestTodo(todos) {
        // Score each TODO based on various factors
        const scored = todos.map(todo => {
            let score = 0;
            
            // Prefer smaller, focused tasks
            if (todo.text.length < 50) score += 10;
            
            // Prefer tasks we can verify
            if (todo.text.includes('test') || todo.text.includes('validate')) score += 5;
            
            // Prefer infrastructure over features initially
            if (todo.type === 'fix') score += 20;
            
            // Specific high-value targets
            if (todo.text.includes('Bundle size')) score += 50;
            if (todo.text.includes('Forms')) score += 45;
            if (todo.text.includes('auth')) score += 40;
            if (todo.text.includes('Upload')) score += 35;
            if (todo.text.includes('PWA')) score += 30;
            if (todo.text.includes('Code Splitting')) score += 30;
            if (todo.text.includes('Database')) score += 25;
            if (todo.text.includes('Production Deploy')) score += 20;
            
            // Avoid complex tasks for now
            if (todo.text.includes('GraphQL')) score -= 10;
            if (todo.text.includes('OAuth')) score -= 10;
            if (todo.text.includes('production')) score -= 5;
            
            return { ...todo, score };
        });
        
        // Sort by score and select the highest
        scored.sort((a, b) => b.score - a.score);
        return scored[0];
    }

    saveSelection(todo) {
        const selection = {
            timestamp: new Date().toISOString(),
            selected: todo,
            reason: this.explainSelection(todo)
        };
        
        fs.writeFileSync(
            path.join(__dirname, 'current-todo.json'),
            JSON.stringify(selection, null, 2)
        );
    }

    explainSelection(todo) {
        const reasons = [];
        
        if (todo.priority === 'critical') {
            reasons.push('Critical priority - must fix lies in documentation');
        }
        
        if (todo.text.includes('counter example')) {
            reasons.push('Counter example should demonstrate the framework');
        }
        
        if (todo.text.includes('SSR')) {
            reasons.push('SSR is a core feature claim that needs implementation');
        }
        
        if (todo.text.includes('state')) {
            reasons.push('State management is fundamental to the framework');
        }
        
        if (reasons.length === 0) {
            reasons.push(`${todo.priority} priority task for framework completion`);
        }
        
        return reasons.join('; ');
    }
}

// Main execution
const selector = new TodoSelector();
selector.selectTodo();