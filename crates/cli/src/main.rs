//! Layer9 CLI - Development tools

mod deploy;
// mod haf_lint;
// mod haf_gen;
// mod haf_refactor;
mod haf_lint_simple;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "layer9")]
#[command(about = "Layer9 - Web Architecture Rust Platform", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Layer9 project
    New {
        /// Project name
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "default")]
        template: String,
    },

    /// Start development server
    Dev {
        /// Port to run on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Enable hot reload
        #[arg(short, long, default_value = "true")]
        hot: bool,

        /// Open browser automatically
        #[arg(short, long)]
        open: bool,
    },

    /// Build for production
    Build {
        /// Build mode
        #[arg(short, long, default_value = "production")]
        #[allow(dead_code)]
        mode: String,

        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,

        /// Enable SSG
        #[arg(long)]
        ssg: bool,
    },

    /// Deploy to production
    Deploy {
        /// Target platform (vercel, netlify, aws, cloudflare)
        #[arg(short, long, default_value = "vercel")]
        target: String,
        
        /// Environment to deploy to
        #[arg(short, long, default_value = "production")]
        env: String,
        
        /// Build directory
        #[arg(short, long, default_value = "dist")]
        build_dir: PathBuf,
        
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
        
        /// Force deployment without confirmation
        #[arg(short, long)]
        force: bool,
        
        /// Show deployment logs
        #[arg(short = 'v', long)]
        #[allow(dead_code)]
        verbose: bool,
    },
    
    /// Check deployment status
    DeployStatus {
        /// Deployment ID
        deployment_id: String,
        
        /// Platform
        #[arg(short, long, default_value = "vercel")]
        platform: String,
    },
    
    /// List recent deployments
    DeployList {
        /// Platform
        #[arg(short, long, default_value = "vercel")]
        platform: String,
        
        /// Number of deployments to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Rollback to previous deployment
    DeployRollback {
        /// Deployment ID to rollback to
        deployment_id: String,
        
        /// Platform
        #[arg(short, long, default_value = "vercel")]
        platform: String,
    },
    
    /// Generate deployment configuration
    DeployInit {
        /// Generate example environment file
        #[arg(long)]
        env_example: bool,
    },

    /// Run type checking
    Check,

    /// Format code
    Fmt,

    /// Run HAF architectural linter
    HafLint {
        /// Path to lint (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        
        /// Fail with exit code 1 if violations found
        #[arg(long)]
        strict: bool,
    },
    
    /// Generate HAF-compliant project structure
    HafNew {
        /// Project name
        name: String,
    },
    
    /// Automatically refactor code to follow HAF principles
    HafRefactor {
        /// Path to refactor (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
        
        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, template } => {
            new_project(&name, &template).await?;
        }
        Commands::Dev { port, hot, open } => {
            dev_server(port, hot, open).await?;
        }
        Commands::Build { mode, output, ssg } => {
            build_project(&mode, &output, ssg).await?;
        }
        Commands::Deploy { target, env, build_dir, config, force, verbose } => {
            let platform = target.parse()?;
            let options = deploy::DeployOptions {
                platform,
                build_dir,
                environment: env,
                config_path: config,
                force,
                verbose,
            };
            deploy::deploy(options).await?;
        }
        Commands::DeployStatus { deployment_id, platform } => {
            let platform = platform.parse()?;
            let status = deploy::get_status(platform, &deployment_id).await?;
            println!("Deployment Status: {}", status);
        }
        Commands::DeployList { platform, limit } => {
            let platform = platform.parse()?;
            let deployments = deploy::list_deployments(platform, limit).await?;
            
            println!("{}", "Recent Deployments:".bright_blue().bold());
            for (i, deploy) in deployments.iter().enumerate() {
                println!(
                    "{}. {} - {} - {}",
                    i + 1,
                    deploy.deployment_id.bright_black(),
                    deploy.status,
                    deploy.url.cyan()
                );
            }
        }
        Commands::DeployRollback { deployment_id, platform } => {
            let platform = platform.parse()?;
            deploy::rollback(platform, &deployment_id).await?;
        }
        Commands::DeployInit { env_example } => {
            if env_example {
                generate_env_example().await?;
            } else {
                generate_deploy_config().await?;
            }
        }
        Commands::Check => {
            check_project().await?;
        }
        Commands::Fmt => {
            format_project().await?;
        }
        Commands::HafLint { path, strict } => {
            // Use simple linter for now due to syn version issues
            let success = haf_lint_simple::run_simple_linter(&path)?;
            if strict && !success {
                std::process::exit(1);
            }
        }
        Commands::HafNew { name } => {
            // haf_gen::generate_haf_project(&name)?;
            
            println!("{}", "HAF project generator temporarily disabled".yellow());
            println!("Project name: {}", name);
        }
        Commands::HafRefactor { path, dry_run } => {
            // haf_refactor::run_refactorer(&path, dry_run)?;
            println!("{}", "HAF refactorer temporarily disabled".yellow());
            println!("Path: {}, dry-run: {}", path.display(), dry_run);
        }
    }

    Ok(())
}

/// Create new project
async fn new_project(name: &str, template: &str) -> Result<()> {
    use dialoguer::{theme::ColorfulTheme, Select};
    use indicatif::{ProgressBar, ProgressStyle};

    println!(
        "{}",
        "🚀 Creating new Layer9 project...".bright_blue().bold()
    );

    // Select template if not specified
    let template = if template == "default" {
        let templates = vec![
            "minimal", 
            "dashboard", 
            "full-stack", 
            "static-site",
            "haf-minimal",
            "haf-full"
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a template")
            .items(&templates)
            .default(0)
            .interact()?;
        templates[selection]
    } else {
        template
    };

    let pb = ProgressBar::new(5);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // Create project directory
    pb.set_message("Creating directory");
    std::fs::create_dir_all(name)?;
    pb.inc(1);

    // Generate Cargo.toml
    pb.set_message("Generating Cargo.toml");
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
layer9-framework = "0.1"
wasm-bindgen = "0.2"
web-sys = "0.3"
console_error_panic_hook = "0.1"

[profile.release]
opt-level = "z"
lto = true
"#,
        name.replace('-', "_")
    );
    std::fs::write(format!("{}/Cargo.toml", name), cargo_toml)?;
    pb.inc(1);

    // Create src directory
    pb.set_message("Creating source files");
    std::fs::create_dir_all(format!("{}/src", name))?;

    // Generate main.rs based on template
    let main_rs = match template {
        "minimal" => include_str!("../templates/minimal.rs"),
        "dashboard" => include_str!("../templates/dashboard.rs"),
        "full-stack" => include_str!("../templates/fullstack.rs"),
        "haf-minimal" => include_str!("../templates/haf-minimal.rs"),
        "haf-full" => include_str!("../templates/haf-full.rs"),
        _ => include_str!("../templates/minimal.rs"),
    };
    std::fs::write(format!("{}/src/lib.rs", name), main_rs)?;
    pb.inc(1);

    // Create layer9.toml config
    pb.set_message("Creating configuration");
    let layer9_toml = r#"[project]
name = "{name}"
version = "0.1.0"

[build]
target = "web"
output = "dist"

[dev]
port = 3000
hot_reload = true

[deploy]
platform = "vercel"
"#;
    std::fs::write(format!("{}/layer9.toml", name), layer9_toml)?;
    pb.inc(1);

    // Create .gitignore
    pb.set_message("Setting up git");
    let gitignore = r#"/target
/dist
/pkg
Cargo.lock
.DS_Store
*.wasm
"#;
    std::fs::write(format!("{}/.gitignore", name), gitignore)?;
    pb.inc(1);

    pb.finish_with_message("Done!");

    println!("\n{}", "✨ Project created successfully!".green().bold());
    println!("\nTo get started:");
    println!("  {}", format!("cd {}", name).bright_black());
    println!("  {}", "layer9 dev".bright_black());

    Ok(())
}

/// Start development server
async fn dev_server(port: u16, hot: bool, open: bool) -> Result<()> {
    use axum::{routing::get, Router};
    use notify::{Event, RecursiveMode, Watcher};
    use std::sync::mpsc;
    use tower_http::services::ServeDir;
    use tower_livereload::LiveReloadLayer;

    println!(
        "{}",
        "🔧 Starting Layer9 development server..."
            .bright_blue()
            .bold()
    );

    // Check if wasm-pack is installed
    if which::which("wasm-pack").is_err() {
        println!("{}", "⚠️  wasm-pack not found. Installing...".yellow());
        std::process::Command::new("curl")
            .args([
                "https://rustwasm.github.io/wasm-pack/installer/init.sh",
                "-sSf",
            ])
            .stdout(std::process::Stdio::piped())
            .spawn()?
            .wait()?;
    }

    // Initial build
    println!("{}", "📦 Building WASM bundle...".cyan());
    build_wasm()?;

    // Setup file watcher for hot reload
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res {
            if event.paths.iter().any(|p| {
                p.extension()
                    .is_some_and(|ext| ext == "rs" || ext == "toml")
            }) {
                let _ = tx.send(());
            }
        }
    })?;
    watcher.watch(std::path::Path::new("src"), RecursiveMode::Recursive)?;

    // Spawn rebuild task
    if hot {
        tokio::spawn(async move {
            while rx.recv().is_ok() {
                println!("{}", "🔄 Detected changes, rebuilding...".yellow());
                if let Err(e) = build_wasm() {
                    eprintln!("{}", format!("❌ Build error: {}", e).red());
                } else {
                    println!("{}", "✅ Rebuild complete!".green());
                }
            }
        });
    }

    // Create router
    let app = if hot {
        Router::new()
            .route("/", get(serve_index))
            .nest_service("/pkg", ServeDir::new("pkg"))
            .layer(LiveReloadLayer::new())
    } else {
        Router::new()
            .route("/", get(serve_index))
            .nest_service("/pkg", ServeDir::new("pkg"))
    };

    // Start server
    let addr = format!("0.0.0.0:{}", port);
    println!(
        "{}",
        format!("🌐 Server running at http://localhost:{}", port)
            .green()
            .bold()
    );

    if open {
        if let Err(e) = webbrowser::open(&format!("http://localhost:{}", port)) {
            eprintln!("Failed to open browser: {}", e);
        }
    }

    println!("\n{}", "Press Ctrl+C to stop".bright_black());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Build WASM bundle
fn build_wasm() -> Result<()> {
    let output = std::process::Command::new("wasm-pack")
        .args(["build", "--target", "web", "--out-dir", "pkg"])
        .output()
        .context("Failed to run wasm-pack")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Serve index.html
async fn serve_index() -> axum::response::Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Layer9 Dev Server</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #000;
            color: #fff;
            margin: 0;
            padding: 0;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
        }
        #layer9-root { width: 100%; }
        .loading {
            text-align: center;
            padding: 2rem;
        }
        .error {
            background: rgba(239, 68, 68, 0.1);
            border: 1px solid rgba(239, 68, 68, 0.3);
            padding: 2rem;
            border-radius: 0.5rem;
            margin: 2rem;
        }
    </style>
</head>
<body>
    <div id="layer9-root">
        <div class="loading">
            <h1>Loading Layer9...</h1>
            <p>Initializing application...</p>
        </div>
    </div>
    <script type="module">
        import init from './pkg/layer9_app.js';
        
        init().catch(err => {
            document.getElementById('layer9-root').innerHTML = `
                <div class="error">
                    <h2>Failed to load application</h2>
                    <pre>${err.message}</pre>
                </div>
            `;
        });
    </script>
</body>
</html>"#;

    axum::response::Html(html.to_string())
}

/// Build for production
async fn build_project(mode: &str, output: &PathBuf, ssg: bool) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};

    println!("{}", "📦 Building for production...".bright_blue().bold());

    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // Clean output directory
    pb.set_message("Cleaning output directory");
    if output.exists() {
        std::fs::remove_dir_all(output)?;
    }
    std::fs::create_dir_all(output)?;
    pb.inc(1);

    // Build WASM with optimizations
    pb.set_message("Building optimized WASM");
    let args = if mode == "production" {
        vec![
            "build",
            "--target",
            "web",
            "--out-dir",
            output.to_str().unwrap(),
            "--release",
        ]
    } else {
        vec![
            "build",
            "--target",
            "web",
            "--out-dir",
            output.to_str().unwrap(),
        ]
    };

    let output_cmd = std::process::Command::new("wasm-pack")
        .args(args)
        .output()?;

    if !output_cmd.status.success() {
        return Err(anyhow::anyhow!(
            "Build failed:\n{}",
            String::from_utf8_lossy(&output_cmd.stderr)
        ));
    }
    pb.inc(1);

    // Optimize WASM size
    pb.set_message("Optimizing bundle size");
    if mode == "production" && which::which("wasm-opt").is_ok() {
        let wasm_file = output.join("layer9_app_bg.wasm");
        std::process::Command::new("wasm-opt")
            .args([
                "-Oz",
                "-o",
                wasm_file.to_str().unwrap(),
                wasm_file.to_str().unwrap(),
            ])
            .output()?;
    }
    pb.inc(1);

    // Generate static pages if SSG enabled
    if ssg {
        pb.set_message("Generating static pages");
        // TODO: Implement SSG generation
        pb.inc(1);
    } else {
        pb.inc(1);
    }

    pb.finish_with_message("Build complete!");

    // Print bundle size
    let wasm_size = std::fs::metadata(output.join("layer9_app_bg.wasm"))?.len();
    let js_size = std::fs::metadata(output.join("layer9_app.js"))?.len();

    println!("\n{}", "📊 Bundle Analysis:".green().bold());
    println!("  WASM: {}", format_size(wasm_size));
    println!("  JS:   {}", format_size(js_size));
    println!("  Total: {}", format_size(wasm_size + js_size));

    Ok(())
}

/// Generate deployment configuration file
async fn generate_deploy_config() -> Result<()> {
    use dialoguer::Confirm;
    
    println!("{}", "🔧 Generating deployment configuration...".bright_blue().bold());
    
    let config_path = PathBuf::from("layer9.deploy.toml");
    
    if config_path.exists() && !Confirm::new()
        .with_prompt("layer9.deploy.toml already exists. Overwrite?")
        .default(false)
        .interact()? {
        println!("{}", "Cancelled".yellow());
        return Ok(());
    }
    
    std::fs::write(&config_path, deploy::config::EXAMPLE_CONFIG)?;
    
    println!("{}", "✅ Created layer9.deploy.toml".green().bold());
    println!("Edit this file to configure your deployment settings.");
    println!("Don't forget to set up your API tokens in .env!");
    
    Ok(())
}

/// Generate .env.example file
async fn generate_env_example() -> Result<()> {
    println!("{}", "📝 Generating .env.example...".bright_blue().bold());
    
    let config = deploy::config::DeployConfig::from_project_root()
        .unwrap_or_default();
    
    let example = deploy::environment::generate_env_example(&config)?;
    std::fs::write(".env.example", example)?;
    
    println!("{}", "✅ Created .env.example".green().bold());
    println!("Copy this file to .env and fill in your secrets.");
    
    Ok(())
}

/// Check project
async fn check_project() -> Result<()> {
    println!("{}", "🔍 Running type check...".bright_blue().bold());

    let output = std::process::Command::new("cargo")
        .args(["check"])
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Type check failed"));
    }

    println!("{}", "✅ All checks passed!".green().bold());
    Ok(())
}

/// Format project
async fn format_project() -> Result<()> {
    println!("{}", "🎨 Formatting code...".bright_blue().bold());

    let output = std::process::Command::new("cargo")
        .args(["fmt"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Format failed"));
    }

    println!("{}", "✅ Code formatted!".green().bold());
    Ok(())
}

/// Format file size
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
