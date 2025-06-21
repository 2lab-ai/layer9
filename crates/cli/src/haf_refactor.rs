//! HAF Automated Refactoring Tool
//! 
//! Automatically refactors existing Layer9 code to follow HAF principles

use std::path::{Path, PathBuf};
use std::fs;
use syn::{parse_file, File, Item, ItemFn, ItemStruct, ItemImpl, ItemUse, visit_mut::{self, VisitMut}};
use quote::quote;
use anyhow::{Result, Context};
use colored::*;

/// Refactoring operation
#[derive(Debug)]
pub enum RefactorOp {
    /// Move function to appropriate layer
    MoveFunction {
        name: String,
        from: PathBuf,
        to_layer: Layer,
        reason: String,
    },
    /// Extract side effects from function
    ExtractSideEffects {
        function: String,
        file: PathBuf,
        effects: Vec<String>,
    },
    /// Split mixed-concern struct
    SplitStruct {
        name: String,
        file: PathBuf,
        l1_fields: Vec<String>,
        l2_fields: Vec<String>,
        l3_fields: Vec<String>,
    },
    /// Add layer annotation
    AddLayerAnnotation {
        file: PathBuf,
        layer: Layer,
    },
    /// Create contract for cross-layer communication
    CreateContract {
        from_type: String,
        to_type: String,
        from_layer: Layer,
        to_layer: Layer,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layer {
    L1,
    L2,
    L3,
}

/// HAF Refactorer
pub struct HafRefactorer {
    operations: Vec<RefactorOp>,
    dry_run: bool,
}

impl HafRefactorer {
    pub fn new(dry_run: bool) -> Self {
        Self {
            operations: Vec::new(),
            dry_run,
        }
    }
    
    /// Analyze and refactor a directory
    pub fn refactor_directory(&mut self, dir: &Path) -> Result<()> {
        println!("{}", "🔧 Analyzing code for HAF refactoring...".cyan());
        
        self.analyze_directory(dir)?;
        
        if self.operations.is_empty() {
            println!("{}", "✨ No refactoring needed - code is HAF compliant!".green());
            return Ok(());
        }
        
        self.print_refactor_plan();
        
        if !self.dry_run {
            if self.confirm_refactor()? {
                self.execute_refactoring()?;
            }
        }
        
        Ok(())
    }
    
    fn analyze_directory(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() && !path.ends_with("target") {
                self.analyze_directory(&path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.analyze_file(&path)?;
            }
        }
        Ok(())
    }
    
    fn analyze_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let syntax = parse_file(&content)?;
        
        // Check for layer annotation
        if !self.has_layer_annotation(&content) {
            self.operations.push(RefactorOp::AddLayerAnnotation {
                file: path.to_owned(),
                layer: self.infer_layer(path, &syntax),
            });
        }
        
        // Analyze items
        for item in &syntax.items {
            match item {
                Item::Fn(func) => self.analyze_function(func, path)?,
                Item::Struct(s) => self.analyze_struct(s, path)?,
                Item::Impl(imp) => self.analyze_impl(imp, path)?,
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn has_layer_annotation(&self, content: &str) -> bool {
        content.lines().take(5).any(|line| 
            line.contains("//! L1") || 
            line.contains("//! L2") || 
            line.contains("//! L3")
        )
    }
    
    fn infer_layer(&self, path: &Path, syntax: &File) -> Layer {
        let path_str = path.to_string_lossy();
        
        // Infer from path
        if path_str.contains("core") || path_str.contains("model") || path_str.contains("logic") {
            return Layer::L1;
        }
        if path_str.contains("runtime") || path_str.contains("service") {
            return Layer::L2;
        }
        if path_str.contains("api") || path_str.contains("web") || path_str.contains("handler") {
            return Layer::L3;
        }
        
        // Infer from imports
        let mut has_io = false;
        let mut has_web = false;
        
        for item in &syntax.items {
            if let Item::Use(u) = item {
                let use_str = quote!(#u).to_string();
                if use_str.contains("std::io") || use_str.contains("tokio") {
                    has_io = true;
                }
                if use_str.contains("web_sys") || use_str.contains("reqwest") {
                    has_web = true;
                }
            }
        }
        
        if has_web {
            Layer::L3
        } else if has_io {
            Layer::L2
        } else {
            Layer::L1
        }
    }
    
    fn analyze_function(&mut self, func: &ItemFn, path: &Path) -> Result<()> {
        let func_str = quote!(#func).to_string();
        let mut side_effects = Vec::new();
        
        // Check for I/O operations
        if func_str.contains("println!") || func_str.contains("eprintln!") {
            side_effects.push("Console I/O".to_string());
        }
        if func_str.contains("fs::") || func_str.contains("File::") {
            side_effects.push("File I/O".to_string());
        }
        if func_str.contains(".await") {
            side_effects.push("Async operation".to_string());
        }
        if func_str.contains("window()") || func_str.contains("document()") {
            side_effects.push("DOM manipulation".to_string());
        }
        
        // Check if function is in wrong layer
        let current_layer = self.infer_layer(path, &parse_file(&fs::read_to_string(path)?)?);
        let required_layer = if !side_effects.is_empty() {
            if side_effects.iter().any(|s| s.contains("DOM")) {
                Layer::L3
            } else {
                Layer::L2
            }
        } else {
            Layer::L1
        };
        
        if current_layer != required_layer && current_layer == Layer::L1 {
            self.operations.push(RefactorOp::MoveFunction {
                name: func.sig.ident.to_string(),
                from: path.to_owned(),
                to_layer: required_layer,
                reason: format!("Contains side effects: {:?}", side_effects),
            });
        } else if !side_effects.is_empty() && current_layer == Layer::L1 {
            self.operations.push(RefactorOp::ExtractSideEffects {
                function: func.sig.ident.to_string(),
                file: path.to_owned(),
                effects: side_effects,
            });
        }
        
        Ok(())
    }
    
    fn analyze_struct(&mut self, s: &ItemStruct, path: &Path) -> Result<()> {
        let mut l1_fields = Vec::new();
        let mut l2_fields = Vec::new();
        let mut l3_fields = Vec::new();
        
        if let syn::Fields::Named(fields) = &s.fields {
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_type = quote!(#field.ty).to_string();
                
                // Categorize fields by type
                if field_type.contains("Rc<RefCell") || field_type.contains("Arc<Mutex") {
                    l2_fields.push(field_name);
                } else if field_type.contains("web_sys::") || field_type.contains("JsValue") {
                    l3_fields.push(field_name);
                } else {
                    l1_fields.push(field_name);
                }
            }
        }
        
        // If struct has mixed concerns, suggest splitting
        let concerns = [!l1_fields.is_empty(), !l2_fields.is_empty(), !l3_fields.is_empty()]
            .iter()
            .filter(|&&x| x)
            .count();
            
        if concerns > 1 {
            self.operations.push(RefactorOp::SplitStruct {
                name: s.ident.to_string(),
                file: path.to_owned(),
                l1_fields,
                l2_fields,
                l3_fields,
            });
        }
        
        Ok(())
    }
    
    fn analyze_impl(&mut self, imp: &ItemImpl, path: &Path) -> Result<()> {
        // Analyze methods in impl block
        for item in &imp.items {
            if let syn::ImplItem::Method(method) = item {
                // Create a standalone function for analysis
                let func = syn::ItemFn {
                    attrs: method.attrs.clone(),
                    vis: method.vis.clone(),
                    sig: method.sig.clone(),
                    block: Box::new(method.block.clone()),
                };
                self.analyze_function(&func, path)?;
            }
        }
        Ok(())
    }
    
    fn print_refactor_plan(&self) {
        println!("\n{}", "📋 Refactoring Plan:".yellow().bold());
        println!("{}", "=".repeat(50).dimmed());
        
        for (i, op) in self.operations.iter().enumerate() {
            println!("\n{}. {}", i + 1, self.format_operation(op));
        }
        
        println!("\n{}", "=".repeat(50).dimmed());
        println!("{}", format!("Total operations: {}", self.operations.len()).cyan());
    }
    
    fn format_operation(&self, op: &RefactorOp) -> String {
        match op {
            RefactorOp::MoveFunction { name, from, to_layer, reason } => {
                format!(
                    "{} Move function '{}'\n   {} {} → {:?}\n   {} {}",
                    "📦".cyan(),
                    name.yellow(),
                    "from:".dimmed(),
                    from.display(),
                    to_layer,
                    "reason:".dimmed(),
                    reason
                )
            }
            RefactorOp::ExtractSideEffects { function, file, effects } => {
                format!(
                    "{} Extract side effects from '{}'\n   {} {}\n   {} {:?}",
                    "🔧".cyan(),
                    function.yellow(),
                    "in:".dimmed(),
                    file.display(),
                    "effects:".dimmed(),
                    effects
                )
            }
            RefactorOp::SplitStruct { name, file, l1_fields, l2_fields, l3_fields } => {
                format!(
                    "{} Split struct '{}'\n   {} {}\n   {} L1: {:?}\n   {} L2: {:?}\n   {} L3: {:?}",
                    "✂️".cyan(),
                    name.yellow(),
                    "in:".dimmed(),
                    file.display(),
                    " ".dimmed(),
                    l1_fields,
                    " ".dimmed(),
                    l2_fields,
                    " ".dimmed(),
                    l3_fields
                )
            }
            RefactorOp::AddLayerAnnotation { file, layer } => {
                format!(
                    "{} Add layer annotation\n   {} {}\n   {} {:?}",
                    "🏷️".cyan(),
                    "to:".dimmed(),
                    file.display(),
                    "layer:".dimmed(),
                    layer
                )
            }
            RefactorOp::CreateContract { from_type, to_type, from_layer, to_layer } => {
                format!(
                    "{} Create contract\n   {} {} ({:?}) → {} ({:?})",
                    "📄".cyan(),
                    "for:".dimmed(),
                    from_type,
                    from_layer,
                    to_type,
                    to_layer
                )
            }
        }
    }
    
    fn confirm_refactor(&self) -> Result<bool> {
        use dialoguer::Confirm;
        
        Ok(Confirm::new()
            .with_prompt("Apply these refactoring operations?")
            .default(false)
            .interact()?)
    }
    
    fn execute_refactoring(&mut self) -> Result<()> {
        use indicatif::{ProgressBar, ProgressStyle};
        
        let pb = ProgressBar::new(self.operations.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        
        for op in &self.operations {
            pb.set_message(format!("Applying: {}", self.short_op_description(op)));
            
            match op {
                RefactorOp::AddLayerAnnotation { file, layer } => {
                    self.add_layer_annotation(file, *layer)?;
                }
                RefactorOp::ExtractSideEffects { function, file, effects } => {
                    self.extract_side_effects(function, file, effects)?;
                }
                // Other operations would be implemented similarly
                _ => {
                    // For now, just log what would be done
                    println!("\n{} {}", "Would:".yellow(), self.format_operation(op));
                }
            }
            
            pb.inc(1);
        }
        
        pb.finish_with_message("Refactoring complete!");
        Ok(())
    }
    
    fn short_op_description(&self, op: &RefactorOp) -> String {
        match op {
            RefactorOp::MoveFunction { name, .. } => format!("Moving {}", name),
            RefactorOp::ExtractSideEffects { function, .. } => format!("Extracting from {}", function),
            RefactorOp::SplitStruct { name, .. } => format!("Splitting {}", name),
            RefactorOp::AddLayerAnnotation { .. } => "Adding annotation".to_string(),
            RefactorOp::CreateContract { .. } => "Creating contract".to_string(),
        }
    }
    
    fn add_layer_annotation(&self, file: &Path, layer: Layer) -> Result<()> {
        let content = fs::read_to_string(file)?;
        let annotation = match layer {
            Layer::L1 => "//! L1: Core - Pure business logic\n",
            Layer::L2 => "//! L2: Runtime - Execution environment\n",
            Layer::L3 => "//! L3: Framework - External interfaces\n",
        };
        
        let new_content = format!("{}\n{}", annotation, content);
        fs::write(file, new_content)?;
        
        Ok(())
    }
    
    fn extract_side_effects(&self, function: &str, file: &Path, effects: &[String]) -> Result<()> {
        let content = fs::read_to_string(file)?;
        let mut syntax = parse_file(&content)?;
        
        let mut extractor = SideEffectExtractor {
            function_name: function.to_string(),
            effects: effects.to_vec(),
        };
        
        extractor.visit_file_mut(&mut syntax);
        
        let new_content = quote!(#syntax).to_string();
        fs::write(file, new_content)?;
        
        Ok(())
    }
}

/// AST visitor for extracting side effects
struct SideEffectExtractor {
    function_name: String,
    effects: Vec<String>,
}

impl VisitMut for SideEffectExtractor {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        if func.sig.ident == self.function_name {
            // Create pure version and effect version
            let pure_name = format!("{}_pure", self.function_name);
            let effect_name = format!("{}_effects", self.function_name);
            
            // This is a simplified version - real implementation would
            // properly extract side effects into separate functions
            
            // Add comment explaining the refactoring
            func.attrs.push(syn::parse_quote! {
                #[doc = "TODO: This function has been marked for side effect extraction"]
            });
        }
        
        visit_mut::visit_item_fn_mut(self, func);
    }
}

/// Run the HAF refactorer
pub fn run_refactorer(path: &Path, dry_run: bool) -> Result<()> {
    let mut refactorer = HafRefactorer::new(dry_run);
    refactorer.refactor_directory(path)?;
    Ok(())
}

/// Generate refactoring suggestions for a specific file
pub fn suggest_refactoring(file: &Path) -> Result<Vec<String>> {
    let mut refactorer = HafRefactorer::new(true);
    refactorer.analyze_file(file)?;
    
    let suggestions = refactorer.operations.iter()
        .map(|op| refactorer.format_operation(op))
        .collect();
    
    Ok(suggestions)
}