//! Simplified HAF Linter for Layer9
//! 
//! Basic architectural violation detection

use std::path::{Path, PathBuf};
use std::fs;
use colored::*;
use anyhow::Result;

/// Simple HAF violation
#[derive(Debug)]
pub enum Violation {
    MissingLayerAnnotation { file: PathBuf },
    SuspectedLayerViolation { file: PathBuf, line: usize, reason: String },
}

/// Simple HAF Linter
pub struct SimpleLinter {
    violations: Vec<Violation>,
}

impl SimpleLinter {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }
    
    pub fn lint_directory(&mut self, dir: &Path) -> Result<()> {
        println!("Scanning directory: {}", dir.display());
        self.scan_directory(dir)?;
        Ok(())
    }
    
    fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() && !path.ends_with("target") {
                self.scan_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.lint_file(&path)?;
            }
        }
        Ok(())
    }
    
    fn lint_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let mut violations = Vec::new();
        
        // Check for layer annotation
        let has_annotation = content.lines().take(10).any(|line| 
            line.contains("//! L1") || 
            line.contains("//! L2") || 
            line.contains("//! L3")
        );
        
        if !has_annotation && path.to_string_lossy().contains("/haf/") {
            violations.push(Violation::MissingLayerAnnotation { 
                file: path.to_owned() 
            });
        }
        
        // Simple pattern matching for violations
        for (line_num, line) in content.lines().enumerate() {
            // Check for I/O in files that might be L1
            if (path.to_string_lossy().contains("core") || path.to_string_lossy().contains("pure")) && (line.contains("println!") || line.contains("fs::") || line.contains(".await")) {
                violations.push(Violation::SuspectedLayerViolation {
                    file: path.to_owned(),
                    line: line_num + 1,
                    reason: "Possible side effect in pure layer".to_string(),
                });
            }
        }
        
        // Add violations to the linter
        self.violations.extend(violations);
        
        Ok(())
    }
    
    pub fn report(&self) {
        if self.violations.is_empty() {
            println!("{}", "✓ No HAF violations found!".green());
            return;
        }
        
        println!("{}", format!("Found {} potential HAF violations:", self.violations.len()).yellow());
        println!();
        
        for violation in &self.violations {
            match violation {
                Violation::MissingLayerAnnotation { file } => {
                    println!("{} Missing layer annotation", "⚠".yellow());
                    println!("  {} {}", "file:".dimmed(), file.display());
                    println!("  {} Add //! L1, //! L2, or //! L3 annotation", "fix:".green());
                    println!();
                }
                
                Violation::SuspectedLayerViolation { file, line, reason } => {
                    println!("{} Suspected layer violation", "⚠".yellow());
                    println!("  {} {}:{}", "at:".dimmed(), file.display(), line);
                    println!("  {} {}", "reason:".dimmed(), reason);
                    println!();
                }
            }
        }
    }
    
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

/// Run the simple linter
pub fn run_simple_linter(path: &Path) -> Result<bool> {
    let mut linter = SimpleLinter::new();
    
    println!("{}", "Running simplified HAF linter...".cyan());
    println!();
    
    linter.lint_directory(path)?;
    linter.report();
    
    Ok(!linter.has_violations())
}