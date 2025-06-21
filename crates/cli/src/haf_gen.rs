//! HAF Project Structure Generator
//! 
//! Generates proper HAF-compliant project structure with layer separation

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::Result;
use colored::*;

/// HAF project structure
pub struct HafProject {
    pub name: String,
    pub root: PathBuf,
}

impl HafProject {
    pub fn new(name: String, root: PathBuf) -> Self {
        Self { name, root }
    }
    
    /// Generate HAF project structure
    pub fn generate(&self) -> Result<()> {
        println!("{}", "🏗️  Generating HAF project structure...".cyan());
        
        // Create directory structure
        self.create_directories()?;
        
        // Generate layer modules
        self.generate_l1_core()?;
        self.generate_l2_runtime()?;
        self.generate_l3_framework()?;
        
        // Generate main lib.rs
        self.generate_lib_rs()?;
        
        // Generate contracts
        self.generate_contracts()?;
        
        // Generate HAF config
        self.generate_haf_config()?;
        
        // Generate example components
        self.generate_examples()?;
        
        println!("{}", "✅ HAF project structure created!".green());
        Ok(())
    }
    
    fn create_directories(&self) -> Result<()> {
        let dirs = vec![
            "src/l1_core",
            "src/l1_core/components",
            "src/l1_core/models",
            "src/l1_core/logic",
            "src/l2_runtime",
            "src/l2_runtime/services",
            "src/l2_runtime/effects",
            "src/l3_framework",
            "src/l3_framework/api",
            "src/l3_framework/dom",
            "src/contracts",
            "tests/haf",
        ];
        
        for dir in dirs {
            fs::create_dir_all(self.root.join(dir))?;
        }
        
        Ok(())
    }
    
    fn generate_l1_core(&self) -> Result<()> {
        // L1 module
        let l1_mod = r#"//! L1: Core - Pure Business Logic
//! 
//! This layer contains only pure functions and data structures.
//! No side effects, no I/O, no dependencies on external systems.

pub mod components;
pub mod models;
pub mod logic;

use layer9::haf::{layers::L1, Layer};

/// Example of L1 pure function
pub fn pure_computation(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pure_computation() {
        assert_eq!(pure_computation(2, 3), 5);
    }
}
"#;
        fs::write(self.root.join("src/l1_core/mod.rs"), l1_mod)?;
        
        // Models
        let models = r#"//! Pure data models

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppState {
    pub users: Vec<User>,
    pub selected_user: Option<u32>,
}
"#;
        fs::write(self.root.join("src/l1_core/models.rs"), models)?;
        
        // Logic
        let logic = r#"//! Pure business logic

use super::models::*;

/// Add a new user (pure function)
pub fn add_user(state: &AppState, user: User) -> AppState {
    let mut new_state = state.clone();
    new_state.users.push(user);
    new_state
}

/// Select a user (pure function)
pub fn select_user(state: &AppState, user_id: u32) -> AppState {
    let mut new_state = state.clone();
    new_state.selected_user = Some(user_id);
    new_state
}
"#;
        fs::write(self.root.join("src/l1_core/logic.rs"), logic)?;
        
        // Components module
        let components_mod = r#"//! Pure components

pub mod counter;

pub use counter::Counter;
"#;
        fs::write(self.root.join("src/l1_core/components/mod.rs"), components_mod)?;
        
        Ok(())
    }
    
    fn generate_l2_runtime(&self) -> Result<()> {
        // L2 module
        let l2_mod = r#"//! L2: Runtime - Execution Environment
//! 
//! This layer manages side effects, lifecycle, and orchestration.
//! Can depend on L1 but not L3.

pub mod services;
pub mod effects;

use layer9::haf::{layers::L2, Layer};
use crate::l1_core;

/// Runtime state manager
pub struct RuntimeState {
    // Runtime-specific state
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Execute an effect
    pub fn execute_effect<F>(&mut self, effect: F) 
    where
        F: FnOnce() -> ()
    {
        // Schedule and execute effect
        effect();
    }
}
"#;
        fs::write(self.root.join("src/l2_runtime/mod.rs"), l2_mod)?;
        
        // Services
        let services = r#"//! Runtime services

use layer9::haf::{layers::L2, Service};
use crate::l1_core::models::*;

// State management service
pub mod state_service {
    use super::*;
    
    pub fn get_current_state() -> AppState {
        // In real app, would manage state persistence
        AppState {
            users: vec![],
            selected_user: None,
        }
    }
    
    pub fn update_state(new_state: AppState) {
        // Update state in runtime
    }
}

// Event handling service
pub mod event_service {
    pub fn dispatch_event(event_id: u64) {
        // Handle event dispatching
    }
}
"#;
        fs::write(self.root.join("src/l2_runtime/services.rs"), services)?;
        
        // Effects
        let effects = r#"//! Effect management

use std::future::Future;

/// Effect handle
pub struct EffectHandle {
    id: u64,
}

/// Schedule an effect
pub fn schedule_effect<F>(f: F) -> EffectHandle
where
    F: FnOnce() + 'static
{
    // In real implementation, would queue effect
    EffectHandle { id: 0 }
}

/// Cancel an effect
pub fn cancel_effect(handle: EffectHandle) {
    // Cancel queued effect
}
"#;
        fs::write(self.root.join("src/l2_runtime/effects.rs"), effects)?;
        
        Ok(())
    }
    
    fn generate_l3_framework(&self) -> Result<()> {
        // L3 module
        let l3_mod = r#"//! L3: Framework - External Interfaces
//! 
//! This layer handles all I/O, external APIs, and user-facing interfaces.
//! Can depend on both L1 and L2.

pub mod api;
pub mod dom;

use layer9::haf::{layers::L3, Layer};
use crate::{l1_core, l2_runtime};

/// Application entry point
pub fn mount_app(selector: &str) {
    // Initialize app and mount to DOM
    println!("Mounting HAF app to {}", selector);
}
"#;
        fs::write(self.root.join("src/l3_framework/mod.rs"), l3_mod)?;
        
        // API
        let api = r#"//! External API interactions

use layer9::haf::{layers::L3, Service};

// API service module
pub mod api_service {
    pub async fn fetch_users() -> Result<String, String> {
        // Make HTTP request
        Ok("[]".to_string())
    }
    
    pub async fn save_user(user_json: String) -> Result<(), String> {
        // POST to API
        Ok(())
    }
}
"#;
        fs::write(self.root.join("src/l3_framework/api.rs"), api)?;
        
        // DOM
        let dom = r#"//! DOM manipulation

use web_sys::{window, Document, Element};

/// Get document
pub fn document() -> Document {
    window().unwrap().document().unwrap()
}

/// Query selector
pub fn query_selector(selector: &str) -> Option<Element> {
    document().query_selector(selector).ok().flatten()
}

/// Create element
pub fn create_element(tag: &str) -> Element {
    document().create_element(tag).unwrap()
}
"#;
        fs::write(self.root.join("src/l3_framework/dom.rs"), dom)?;
        
        Ok(())
    }
    
    fn generate_lib_rs(&self) -> Result<()> {
        let lib_rs = format!(r#"//! {} - HAF Layer9 Application
//! 
//! This application follows the Hierarchical Architecture First (HAF) principles
//! with strict layer separation and compile-time enforcement.

#![warn(clippy::all)]

// Layer modules
pub mod l1_core;
pub mod l2_runtime;
pub mod l3_framework;
pub mod contracts;

// Re-exports
pub use l1_core::models::*;
pub use l3_framework::mount_app;

use wasm_bindgen::prelude::*;

/// Application entry point
#[wasm_bindgen(start)]
pub fn main() {{
    // Set panic hook
    console_error_panic_hook::set_once();
    
    // Mount application
    mount_app("#app");
}}
"#, self.name);
        
        fs::write(self.root.join("src/lib.rs"), lib_rs)?;
        Ok(())
    }
    
    fn generate_contracts(&self) -> Result<()> {
        let contracts_mod = r#"//! Layer translation contracts

use layer9::haf::*;
use crate::l1_core::models::*;

/// Contract for state updates (L1 → L2)
pub struct StateUpdateContract;

impl L1ToL2Contract for StateUpdateContract {
    type L1Type = AppState;
    type L2Type = StateUpdateCommand;
    
    fn translate(state: Self::L1Type) -> Self::L2Type {
        StateUpdateCommand {
            new_state: state,
            timestamp: 0, // Would use actual timestamp
        }
    }
}

pub struct StateUpdateCommand {
    pub new_state: AppState,
    pub timestamp: u64,
}

/// Contract for API calls (L2 → L3)
pub struct ApiCallContract;

impl L2ToL3Contract for ApiCallContract {
    type L2Type = ApiRequest;
    type L3Type = HttpRequest;
    
    fn translate(req: Self::L2Type) -> Self::L3Type {
        HttpRequest {
            url: req.endpoint,
            method: "POST".to_string(),
            body: req.payload,
        }
    }
}

pub struct ApiRequest {
    pub endpoint: String,
    pub payload: String,
}

pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub body: String,
}
"#;
        
        fs::write(self.root.join("src/contracts/mod.rs"), contracts_mod)?;
        Ok(())
    }
    
    fn generate_haf_config(&self) -> Result<()> {
        let haf_toml = r#"# HAF Configuration

[haf]
# Enable strict mode - fail compilation on violations
strict = true

# Layer annotations are required
require_annotations = true

[layers.l1]
# L1 can only depend on itself
allowed_dependencies = []
forbidden_imports = ["std::io", "tokio", "web_sys", "reqwest"]

[layers.l2]
# L2 can depend on L1
allowed_dependencies = ["l1"]
forbidden_imports = ["web_sys", "reqwest"]

[layers.l3]
# L3 can depend on L1 and L2
allowed_dependencies = ["l1", "l2"]

[linting]
# Run HAF linter on build
lint_on_build = true

# Treat warnings as errors
warnings_as_errors = true
"#;
        
        fs::write(self.root.join("haf.toml"), haf_toml)?;
        Ok(())
    }
    
    fn generate_examples(&self) -> Result<()> {
        let example = r#"//! Example HAF component

use layer9::haf::{layers::L1, component::*};

pub struct Counter;

impl Default for Counter {
    fn default() -> Self {
        Self
    }
}

impl PureComponent<L1> for Counter {
    type Props = CounterProps;
    
    fn render(&self, props: &Self::Props) -> VNode<L1> {
        VNode::Element {
            tag: "div".to_string(),
            props: VProps::default(),
            children: vec![
                VNode::Text(format!("Count: {}", props.count))
            ],
        }
    }
}

#[derive(Clone)]
pub struct CounterProps {
    pub count: i32,
}
"#;
        
        fs::write(self.root.join("src/l1_core/components/counter.rs"), example)?;
        Ok(())
    }
}

/// Generate HAF project from CLI
pub fn generate_haf_project(name: &str) -> Result<()> {
    let root = PathBuf::from(name);
    
    if root.exists() {
        return Err(anyhow::anyhow!("Directory {} already exists", name));
    }
    
    fs::create_dir_all(&root)?;
    
    let project = HafProject::new(name.to_string(), root.clone());
    project.generate()?;
    
    // Generate Cargo.toml
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
layer9 = {{ version = "0.1", features = ["haf"] }}
wasm-bindgen = "0.2"
web-sys = "0.3"
console_error_panic_hook = "0.1"

[features]
default = ["haf-strict"]
haf-strict = [] # Enable strict HAF enforcement

[profile.release]
opt-level = "z"
lto = true
"#, name.replace('-', "_"));
    
    fs::write(root.join("Cargo.toml"), cargo_toml)?;
    
    // Generate README
    let readme = format!(r#"# {}

A HAF-compliant Layer9 application with strict architectural enforcement.

## Architecture

This project follows the Hierarchical Architecture First (HAF) principles:

- **L1 (Core)**: Pure business logic, no side effects
- **L2 (Runtime)**: Effect management and orchestration  
- **L3 (Framework)**: External interfaces and I/O

## Development

```bash
# Run HAF linter
layer9 haf-lint

# Start development server
layer9 dev

# Build for production
layer9 build --mode production
```

## Layer Guidelines

### L1: Core
- Pure functions only
- No I/O or side effects
- Immutable data structures
- Business logic and algorithms

### L2: Runtime
- Effect scheduling
- State management
- Service orchestration
- Can use L1, but not L3

### L3: Framework
- DOM manipulation
- HTTP requests
- Browser APIs
- Can use both L1 and L2
"#, name);
    
    fs::write(root.join("README.md"), readme)?;
    
    println!("\n{}", "📁 Generated files:".green().bold());
    println!("  {} Cargo.toml", "✓".green());
    println!("  {} src/lib.rs", "✓".green());
    println!("  {} src/l1_core/", "✓".green());
    println!("  {} src/l2_runtime/", "✓".green());
    println!("  {} src/l3_framework/", "✓".green());
    println!("  {} src/contracts/", "✓".green());
    println!("  {} haf.toml", "✓".green());
    println!("  {} README.md", "✓".green());
    
    Ok(())
}