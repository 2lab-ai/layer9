//! Server for Database CRUD Example

use axum::Router;
use layer9_core::{
    db_api::create_db_api_router,
    db::use_db,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Initialize environment
    if std::env::var("DATABASE_TYPE").is_err() {
        std::env::set_var("DATABASE_TYPE", "sqlite");
    }
    if std::env::var("DATABASE_URL").is_err() {
        std::env::set_var("DATABASE_URL", "sqlite:crud_example.db");
    }
    
    // Get database connection
    let db_conn = use_db();
    
    // Create database API router
    let db_api = create_db_api_router(db_conn);
    
    // Build the main application
    let app = Router::new()
        // Database API endpoints
        .nest("/api/db", db_api)
        // Serve static files
        .fallback_service(ServeDir::new("dist"))
        // Add CORS support
        .layer(CorsLayer::permissive());
    
    // Start server
    let addr = "127.0.0.1:3000";
    println!("Database CRUD server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
        
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}