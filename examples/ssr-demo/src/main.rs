//! SSR Demo Server

use layer9_core::prelude::*;
use ssr_demo::SSRDemoApp;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize database if needed
    #[cfg(feature = "ssr")]
    {
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            if let Err(e) = layer9_core::db_sqlx::init_db_pool(&db_url).await {
                eprintln!("Warning: Failed to initialize database: {}", e);
            }
        }
    }

    // Create the SSR app
    let app = Arc::new(SSRDemoApp);
    
    // Create the server
    let router = create_ssr_server(app);
    
    // Run the server
    let addr = "0.0.0.0:3000";
    println!("SSR server running at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
        
    axum::serve(listener, router)
        .await
        .expect("Failed to start server");
}