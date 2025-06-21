//! HAF Linting Tool
//! 
//! Static analysis tool to detect HAF architectural violations in Layer9 code.
//! This helps enforce HAF principles even before compilation.

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::{HashMap, HashSet};
use syn::{parse_file, Item, ItemMod, ItemUse, UseTree, Type, Expr, visit::Visit};
use colored::*;

/// HAF violation types
#[derive(Debug, Clone)]
pub enum Violation {
    /// L1 code importing L2/L3 modules
    LayerDependencyViolation {
        from_layer: String,
        to_layer: String,
        file: PathBuf,
        line: usize,
    },
    /// Side effects in L1 code
    SideEffectInPureLayer {
        file: PathBuf,
        line: usize,
        description: String,
    },
    /// Direct I/O operations outside L3
    IoOutsideFrameworkLayer {
        file: PathBuf,
        line: usize,
        operation: String,
    },
    /// Missing layer annotation
    MissingLayerAnnotation {
        file: PathBuf,
        module: String,
    },
    /// Mixed layer concerns in single module
    MixedLayerConcerns {
        file: PathBuf,
        layers: Vec<String>,
    },
}

/// Layer detection from module path or annotations
#[derive(Debug, Clone, PartialEq)]
enum Layer {
    L1,
    L2,
    L3,
    Unknown,
}

impl Layer {
    fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy();
        
        if path_str.contains("/l1_") || path_str.contains("/core/") || path_str.contains("_pure") {
            Layer::L1
        } else if path_str.contains("/l2_") || path_str.contains("/runtime/") {
            Layer::L2
        } else if path_str.contains("/l3_") || path_str.contains("/framework/") || path_str.contains("/api/") {
            Layer::L3
        } else {
            Layer::Unknown
        }
    }
    
    fn from_annotation(content: &str) -> Option<Self> {
        if content.contains("//! L1") || content.contains("/// L1") {
            Some(Layer::L1)
        } else if content.contains("//! L2") || content.contains("/// L2") {
            Some(Layer::L2)
        } else if content.contains("//! L3") || content.contains("/// L3") {
            Some(Layer::L3)
        } else {
            None
        }
    }
    
    fn can_depend_on(&self, other: &Layer) -> bool {
        match (self, other) {
            // L1 can only depend on L1
            (Layer::L1, Layer::L1) => true,
            (Layer::L1, _) => false,
            
            // L2 can depend on L1 and L2
            (Layer::L2, Layer::L1) => true,
            (Layer::L2, Layer::L2) => true,
            (Layer::L2, _) => false,
            
            // L3 can depend on anything
            (Layer::L3, _) => true,
            
            // Unknown layers are ignored for now
            (Layer::Unknown, _) => true,
            (_, Layer::Unknown) => true,
        }
    }
}

/// HAF Linter
pub struct HafLinter {
    violations: Vec<Violation>,
    layer_map: HashMap<PathBuf, Layer>,
}

impl HafLinter {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            layer_map: HashMap::new(),
        }
    }
    
    /// Lint a directory recursively
    pub fn lint_directory(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.scan_directory(dir)?;
        self.analyze_files()?;
        Ok(())
    }
    
    /// Scan directory to build layer map
    fn scan_directory(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.scan_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = fs::read_to_string(&path)?;
                
                // Determine layer from annotation or path
                let layer = Layer::from_annotation(&content)
                    .unwrap_or_else(|| Layer::from_path(&path));
                
                self.layer_map.insert(path, layer);
            }
        }
        Ok(())
    }
    
    /// Analyze all files for violations
    fn analyze_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let files: Vec<_> = self.layer_map.keys().cloned().collect();
        
        for file_path in files {
            let content = fs::read_to_string(&file_path)?;
            let file_layer = self.layer_map[&file_path].clone();
            
            // Parse the file
            if let Ok(syntax_tree) = parse_file(&content) {
                let mut visitor = HafVisitor {
                    file_path: file_path.clone(),
                    file_layer,
                    violations: Vec::new(),
                    layer_map: &self.layer_map,
                };
                
                visitor.visit_file(&syntax_tree);
                self.violations.extend(visitor.violations);
            }
            
            // Check for missing layer annotation
            if file_layer == Layer::Unknown {
                self.violations.push(Violation::MissingLayerAnnotation {
                    file: file_path,
                    module: String::from("unknown"),
                });
            }
        }
        
        Ok(())
    }
    
    /// Print violations report
    pub fn report(&self) {
        if self.violations.is_empty() {
            println!("{}", "✓ No HAF violations found!".green());
            return;
        }
        
        println!("{}", format!("Found {} HAF violations:", self.violations.len()).red());
        println!();
        
        for violation in &self.violations {
            match violation {
                Violation::LayerDependencyViolation { from_layer, to_layer, file, line } => {
                    println!("{} {} → {} dependency violation", "✗".red(), from_layer, to_layer);
                    println!("  {} {}:{}", "at".dimmed(), file.display(), line);
                    println!("  {} {} cannot depend on {}", "fix:".yellow(), from_layer, to_layer);
                    println!();
                }
                
                Violation::SideEffectInPureLayer { file, line, description } => {
                    println!("{} Side effect in L1 (pure) layer", "✗".red());
                    println!("  {} {}:{}", "at".dimmed(), file.display(), line);
                    println!("  {} {}", "found:".dimmed(), description);
                    println!("  {} Move side effects to L2 or L3", "fix:".yellow());
                    println!();
                }
                
                Violation::IoOutsideFrameworkLayer { file, line, operation } => {
                    println!("{} I/O operation outside L3 layer", "✗".red());
                    println!("  {} {}:{}", "at".dimmed(), file.display(), line);
                    println!("  {} {}", "operation:".dimmed(), operation);
                    println!("  {} Move I/O operations to L3", "fix:".yellow());
                    println!();
                }
                
                Violation::MissingLayerAnnotation { file, module } => {
                    println!("{} Missing layer annotation", "✗".red());
                    println!("  {} {}", "at".dimmed(), file.display());
                    println!("  {} Add //! L1, //! L2, or //! L3 comment", "fix:".yellow());
                    println!();
                }
                
                Violation::MixedLayerConcerns { file, layers } => {
                    println!("{} Mixed layer concerns in single module", "✗".red());
                    println!("  {} {}", "at".dimmed(), file.display());
                    println!("  {} {:?}", "found layers:".dimmed(), layers);
                    println!("  {} Separate concerns into different modules", "fix:".yellow());
                    println!();
                }
            }
        }
        
        println!("{}", format!("Total violations: {}", self.violations.len()).red().bold());
    }
    
    /// Check if any violations were found
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }
}

/// AST visitor to detect violations
struct HafVisitor<'a> {
    file_path: PathBuf,
    file_layer: Layer,
    violations: Vec<Violation>,
    layer_map: &'a HashMap<PathBuf, Layer>,
}

impl<'ast> Visit<'ast> for HafVisitor<'_> {
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        // Check import violations
        self.check_use_tree(&node.tree, 0);
    }
    
    fn visit_expr(&mut self, node: &'ast Expr) {
        // Check for side effects in L1
        if self.file_layer == Layer::L1 {
            match node {
                // I/O operations
                Expr::Call(call) => {
                    if let Expr::Path(path) = &*call.func {
                        let path_str = quote::quote!(#path).to_string();
                        
                        // Check for known I/O operations
                        if path_str.contains("println") || 
                           path_str.contains("print") ||
                           path_str.contains("eprintln") ||
                           path_str.contains("fs::") ||
                           path_str.contains("std::io") ||
                           path_str.contains("tokio::") {
                            self.violations.push(Violation::SideEffectInPureLayer {
                                file: self.file_path.clone(),
                                line: 0, // Would need span info for accurate line
                                description: format!("I/O operation: {}", path_str),
                            });
                        }
                    }
                }
                
                // Mutable state
                Expr::AssignOp(assign) => {
                    self.violations.push(Violation::SideEffectInPureLayer {
                        file: self.file_path.clone(),
                        line: 0,
                        description: "Mutable state modification".to_string(),
                    });
                }
                
                _ => {}
            }
        }
        
        // Check for I/O outside L3
        if self.file_layer != Layer::L3 && self.file_layer != Layer::Unknown {
            if let Expr::Call(call) = node {
                if let Expr::Path(path) = &*call.func {
                    let path_str = quote::quote!(#path).to_string();
                    
                    if path_str.contains("web_sys::") ||
                       path_str.contains("window()") ||
                       path_str.contains("document()") ||
                       path_str.contains("fetch") ||
                       path_str.contains("WebSocket") {
                        self.violations.push(Violation::IoOutsideFrameworkLayer {
                            file: self.file_path.clone(),
                            line: 0,
                            operation: path_str,
                        });
                    }
                }
            }
        }
        
        // Continue visiting
        syn::visit::visit_expr(self, node);
    }
}

impl HafVisitor<'_> {
    fn check_use_tree(&mut self, tree: &UseTree, depth: usize) {
        match tree {
            UseTree::Path(path) => {
                // Check if this import violates layer dependencies
                let import_path = path.ident.to_string();
                
                // Map common imports to layers
                let import_layer = if import_path.contains("l1_") || import_path == "core" {
                    Layer::L1
                } else if import_path.contains("l2_") || import_path == "runtime" {
                    Layer::L2
                } else if import_path.contains("l3_") || import_path == "framework" || import_path == "web_sys" {
                    Layer::L3
                } else {
                    Layer::Unknown
                };
                
                if !self.file_layer.can_depend_on(&import_layer) {
                    self.violations.push(Violation::LayerDependencyViolation {
                        from_layer: format!("{:?}", self.file_layer),
                        to_layer: format!("{:?}", import_layer),
                        file: self.file_path.clone(),
                        line: 0,
                    });
                }
                
                self.check_use_tree(&path.tree, depth + 1);
            }
            UseTree::Group(group) => {
                for item in &group.items {
                    self.check_use_tree(item, depth);
                }
            }
            _ => {}
        }
    }
}

/// CLI integration
pub fn run_linter(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let mut linter = HafLinter::new();
    
    println!("{}", "Running HAF architectural linter...".cyan());
    println!();
    
    linter.lint_directory(path)?;
    linter.report();
    
    Ok(!linter.has_violations())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::write;
    
    #[test]
    fn test_layer_detection() {
        assert_eq!(Layer::from_path(Path::new("src/l1_core.rs")), Layer::L1);
        assert_eq!(Layer::from_path(Path::new("src/l2_runtime.rs")), Layer::L2);
        assert_eq!(Layer::from_path(Path::new("src/l3_framework.rs")), Layer::L3);
        assert_eq!(Layer::from_path(Path::new("src/utils.rs")), Layer::Unknown);
    }
    
    #[test]
    fn test_layer_dependencies() {
        assert!(Layer::L1.can_depend_on(&Layer::L1));
        assert!(!Layer::L1.can_depend_on(&Layer::L2));
        assert!(!Layer::L1.can_depend_on(&Layer::L3));
        
        assert!(Layer::L2.can_depend_on(&Layer::L1));
        assert!(Layer::L2.can_depend_on(&Layer::L2));
        assert!(!Layer::L2.can_depend_on(&Layer::L3));
        
        assert!(Layer::L3.can_depend_on(&Layer::L1));
        assert!(Layer::L3.can_depend_on(&Layer::L2));
        assert!(Layer::L3.can_depend_on(&Layer::L3));
    }
    
    #[test]
    fn test_violation_detection() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create test file with violations
        let l1_file = temp_dir.path().join("l1_core.rs");
        write(&l1_file, r#"
//! L1
use crate::l2_runtime::Effect; // Violation!
use std::io::println; // Side effect!

fn pure_function() {
    println!("This is a side effect!"); // Violation!
}
        "#)?;
        
        let mut linter = HafLinter::new();
        linter.lint_directory(temp_dir.path())?;
        
        assert!(linter.has_violations());
        assert!(linter.violations.len() >= 2);
        
        Ok(())
    }
}