# Getting Started with Layer9

Welcome to Layer9! This guide will help you get up and running with the framework in just a few minutes.

## Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Node.js 18+ (for development tools)
- wasm-pack (`curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`)

## Quick Start

### 1. Clone and Setup

```bash
git clone https://github.com/2lab-ai/layer9
cd layer9
npm install
```

### 2. Run the Counter Example

```bash
# Build and start the development server
npm run dev

# Visit http://localhost:8080
```

### 3. Create Your First App

```bash
# Use the Layer9 CLI
cargo run -p layer9-cli -- new my-app

# Navigate to your app
cd my-app

# Start developing
npm run dev
```

## Project Structure

```
my-app/
├── Cargo.toml          # Rust dependencies
├── src/
│   └── lib.rs          # Your app code
├── index.html          # HTML template
├── styles.css          # Styles
└── pkg/                # Built WASM output
```

## Basic Example

```rust
use layer9_core::prelude::*;

// Define a component
struct HelloWorld {
    name: State<String>,
}

impl Component for HelloWorld {
    fn render(&self) -> Element {
        view! {
            <div>
                <h1>"Hello, "{self.name.get()}"!"</h1>
                <input 
                    value={self.name.get()}
                    oninput={|e| self.name.set(e.target.value)}
                />
            </div>
        }
    }
}

// Initialize your app
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    let app = HelloWorld {
        name: State::new("Layer9".to_string()),
    };
    
    run_app(app);
}
```

## Key Concepts

### 1. Components
Components are the building blocks of Layer9 applications. They implement the `Component` trait and have a `render` method.

### 2. State Management
Layer9 provides reactive state management through `State<T>`:

```rust
let count = State::new(0);
count.set(count.get() + 1);
```

### 3. Routing
Define routes using the router:

```rust
let router = Router::new()
    .route("/", HomePage)
    .route("/about", AboutPage)
    .route("/user/:id", UserPage);
```

### 4. Styling
Layer9 supports CSS-in-Rust:

```rust
let styles = style! {
    background_color: "blue",
    padding: "20px",
    border_radius: "8px",
};
```

## Building for Production

### 1. Optimize Bundle Size

```bash
# Build with optimizations
wasm-pack build --release --target web
```

### 2. Serve with Layer9 Server

```bash
# Build the server
cargo build -p layer9-server --release

# Run in production
./target/release/layer9-server --dir dist --port 3000
```

### 3. Deploy

Layer9 apps can be deployed to any static hosting service:

- **Netlify**: Drop your `dist` folder
- **Vercel**: Use the static adapter
- **GitHub Pages**: Push to gh-pages branch
- **AWS S3**: Upload as static website

## Advanced Features

### Server-Side Rendering (SSR)

```rust
// Enable SSR in Cargo.toml
[features]
default = ["ssr"]

// Use SSR-compatible components
impl SSRComponent for MyApp {
    async fn get_server_props(ctx: &SSRContext) -> Result<Props, Error> {
        // Fetch data on server
        let data = fetch_data().await?;
        Ok(Props { data })
    }
}
```

### Database Integration

```rust
// Use the database hook
let db = use_db();
let users = db.query("SELECT * FROM users").await?;
```

### WebSocket Support

```rust
// Real-time features
let ws = use_websocket("ws://localhost:3001");
ws.on_message(|msg| {
    println!("Received: {}", msg);
});
```

## Troubleshooting

### Common Issues

1. **WASM not loading**
   - Ensure your server sets `Content-Type: application/wasm`
   - Check browser console for errors

2. **Build failures**
   - Update Rust: `rustup update`
   - Clear cache: `cargo clean`
   - Reinstall wasm-pack

3. **Hot reload not working**
   - Ensure WebSocket port is not blocked
   - Check server logs for errors

## Next Steps

- Read the [Architecture Guide](docs/architecture.md)
- Explore [Examples](examples/)
- Join our [Discord](https://discord.gg/layer9)
- Star us on [GitHub](https://github.com/2lab-ai/layer9)

## Need Help?

- 📚 [Documentation](https://layer9.dev/docs)
- 💬 [Discord Community](https://discord.gg/layer9)
- 🐛 [Report Issues](https://github.com/2lab-ai/layer9/issues)
- 📧 Email: support@2lab.ai

Welcome to the Layer9 community! 🚀