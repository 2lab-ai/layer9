use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use clap::Parser;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Layer9 Development Server - Pure Rust, No Python"
)]
struct Args {
    /// Directory to serve
    #[arg(short, long, default_value = ".")]
    dir: PathBuf,

    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Enable hot reload via WebSocket
    #[arg(short = 'r', long)]
    hot_reload: bool,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "layer9_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    // Create broadcast channel for hot reload
    let (reload_tx, _) = broadcast::channel::<()>(100);
    let reload_tx_clone = reload_tx.clone();

    // Setup file watcher for hot reload
    if args.hot_reload {
        let dir = args.dir.clone();
        tokio::spawn(async move {
            watch_files(dir, reload_tx_clone).await;
        });
    }

    // Create the app
    let app = create_app(args.dir.clone(), reload_tx, args.hot_reload);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    info!("🚀 Layer9 Dev Server (Rust Edition)");
    info!("📁 Serving: {}", args.dir.display());
    info!("🌐 Listening on: http://localhost:{}", args.port);
    if args.hot_reload {
        info!("🔥 Hot reload: enabled");
    }
    info!("⚡ Pure Rust - No Python Required!");

    axum::serve(listener, app).await.unwrap();
}

fn create_app(dir: PathBuf, reload_tx: broadcast::Sender<()>, hot_reload: bool) -> Router {
    // Create service for serving files with proper MIME types
    let serve_dir = ServeDir::new(dir)
        .append_index_html_on_directories(true)
        .precompressed_gzip()
        .precompressed_br();

    // Build the router
    if hot_reload {
        Router::new()
            .route("/ws", get(websocket_handler))
            .nest_service("/", serve_dir)
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive().allow_credentials(false)),
            )
            .with_state(reload_tx)
    } else {
        Router::new().nest_service("/", serve_dir).layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive().allow_credentials(false)),
        )
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(reload_tx): axum::extract::State<broadcast::Sender<()>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, reload_tx))
}

async fn handle_socket(mut socket: WebSocket, reload_tx: broadcast::Sender<()>) {
    let mut rx = reload_tx.subscribe();

    // Send initial connection message
    if socket
        .send(axum::extract::ws::Message::Text("connected".to_string()))
        .await
        .is_err()
    {
        return;
    }

    // Listen for reload signals
    while let Ok(_) = rx.recv().await {
        if socket
            .send(axum::extract::ws::Message::Text("reload".to_string()))
            .await
            .is_err()
        {
            break;
        }
    }
}

async fn watch_files(dir: PathBuf, reload_tx: broadcast::Sender<()>) {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() || event.kind.is_create() {
                    let _ = tx.send(());
                }
            }
        },
        Config::default(),
    )
    .unwrap();

    watcher.watch(&dir, RecursiveMode::Recursive).unwrap();

    info!("👁️  Watching for file changes in: {}", dir.display());

    // Debounce file changes
    let mut last_reload = std::time::Instant::now();

    loop {
        if rx.recv().is_ok() {
            let now = std::time::Instant::now();
            if now.duration_since(last_reload).as_millis() > 100 {
                last_reload = now;
                info!("🔄 File change detected, triggering reload");
                let _ = reload_tx.send(());
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use axum::http::StatusCode;
//     use axum_test::TestServer;

//     #[tokio::test]
//     async fn test_server_starts() {
//         let (tx, _) = broadcast::channel(100);
//         let app = create_app(PathBuf::from("."), tx, false);
//         let server = TestServer::new(app).unwrap();

//         let response = server.get("/").await;
//         assert_eq!(response.status_code(), StatusCode::OK);
//     }
// }
