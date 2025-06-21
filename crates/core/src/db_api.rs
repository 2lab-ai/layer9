//! Database API endpoints for client-side database access
//! This module provides HTTP endpoints that proxy database operations for WASM clients

#[cfg(feature = "ssr")]
pub use api_impl::*;

#[cfg(feature = "ssr")]
mod api_impl {
    use crate::db::{DatabaseConnection, DbError, DbErrorKind, QueryResult};
    use axum::{
        extract::{Json, State},
        http::StatusCode,
        response::{IntoResponse, Response},
        routing::post,
        Router,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::sync::Arc;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct QueryRequest {
        pub query: String,
        pub params: Vec<Value>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct QueryResponse {
        pub data: Value,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ErrorResponse {
        pub error: String,
        pub kind: String,
    }

    impl IntoResponse for DbError {
        fn into_response(self) -> Response {
            let error_response = ErrorResponse {
                error: self.message,
                kind: format!("{:?}", self.kind),
            };

            let status = match self.kind {
                DbErrorKind::NotFound => StatusCode::NOT_FOUND,
                DbErrorKind::UniqueViolation => StatusCode::CONFLICT,
                DbErrorKind::ForeignKeyViolation => StatusCode::BAD_REQUEST,
                DbErrorKind::Timeout => StatusCode::REQUEST_TIMEOUT,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            (status, Json(error_response)).into_response()
        }
    }

    /// Create database API router
    pub fn create_db_api_router<C: DatabaseConnection + Send + Sync + 'static>(
        connection: C,
    ) -> Router {
        let shared_conn = Arc::new(connection);

        Router::new()
            .route("/execute", post(execute_handler::<C>))
            .route("/query_one", post(query_one_handler::<C>))
            .route("/query_many", post(query_many_handler::<C>))
            .with_state(shared_conn)
    }

    async fn execute_handler<C: DatabaseConnection>(
        State(conn): State<Arc<C>>,
        Json(req): Json<QueryRequest>,
    ) -> Result<Json<QueryResult>, DbError> {
        let result = conn.execute(&req.query, req.params).await?;
        Ok(Json(result))
    }

    async fn query_one_handler<C: DatabaseConnection>(
        State(conn): State<Arc<C>>,
        Json(req): Json<QueryRequest>,
    ) -> Result<Json<Value>, DbError> {
        let result = conn.query_one(&req.query, req.params).await?;
        Ok(Json(result))
    }

    async fn query_many_handler<C: DatabaseConnection>(
        State(conn): State<Arc<C>>,
        Json(req): Json<QueryRequest>,
    ) -> Result<Json<Vec<Value>>, DbError> {
        let result = conn.query_many(&req.query, req.params).await?;
        Ok(Json(result))
    }

    use axum::extract::FromRequestParts;
    use axum::http::header::AUTHORIZATION;
    use axum::http::request::Parts;
    use async_trait::async_trait;
    
    /// Database API authentication token
    #[derive(Debug, Clone)]
    pub struct DbAuthToken(pub String);
    
    #[async_trait]
    impl<S> FromRequestParts<S> for DbAuthToken
    where
        S: Send + Sync,
    {
        type Rejection = StatusCode;
        
        async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
            let auth_header = parts
                .headers
                .get(AUTHORIZATION)
                .and_then(|value| value.to_str().ok());
                
            match auth_header {
                Some(auth) if auth.starts_with("Bearer ") => {
                    let token = auth.trim_start_matches("Bearer ");
                    Ok(DbAuthToken(token.to_string()))
                }
                _ => Err(StatusCode::UNAUTHORIZED),
            }
        }
    }
    
    /// Middleware for database API authentication
    pub async fn auth_middleware(
        req: axum::http::Request<axum::body::Body>,
        next: axum::middleware::Next,
    ) -> Result<Response, StatusCode> {
        let headers = req.headers();
        let auth_header = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok());
            
        match auth_header {
            Some(auth) if auth.starts_with("Bearer ") => {
                let token = auth.trim_start_matches("Bearer ");
                
                // Validate token
                if validate_db_token(token) {
                    Ok(next.run(req).await)
                } else {
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
            _ => {
                // Allow requests from localhost without auth in development
                let is_localhost = req
                    .headers()
                    .get("host")
                    .and_then(|h| h.to_str().ok())
                    .map(|h| h.starts_with("localhost") || h.starts_with("127.0.0.1"))
                    .unwrap_or(false);
                    
                if is_localhost && cfg!(debug_assertions) {
                    Ok(next.run(req).await)
                } else {
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        }
    }
    
    /// Validate database access token
    fn validate_db_token(token: &str) -> bool {
        // Get expected token from environment
        let expected_token = std::env::var("DATABASE_API_TOKEN")
            .unwrap_or_else(|_| "dummy-token".to_string());
            
        // In production, you would validate against a real auth service
        // For now, just check against environment variable
        token == expected_token
    }
    
    /// Create database API router with authentication
    pub fn create_db_api_router_with_auth<C: DatabaseConnection + Send + Sync + 'static>(
        connection: C,
    ) -> Router {
        let shared_conn = Arc::new(connection);
        
        Router::new()
            .route("/execute", post(execute_handler::<C>))
            .route("/query_one", post(query_one_handler::<C>))
            .route("/query_many", post(query_many_handler::<C>))
            .layer(axum::middleware::from_fn(auth_middleware))
            .with_state(shared_conn)
    }
}