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

    /// Middleware for database API authentication
    pub async fn auth_middleware(
        // You can add auth headers extraction here
        req: axum::http::Request<axum::body::Body>,
        next: axum::middleware::Next,
    ) -> Result<Response, StatusCode> {
        // TODO: Implement proper authentication
        // For now, just pass through
        Ok(next.run(req).await)
    }
}