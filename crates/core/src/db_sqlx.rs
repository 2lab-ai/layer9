//! SQLx Database Implementation for Server-Side Rendering
//! This module provides real database connectivity using SQLx for server-side operations

#[cfg(feature = "ssr")]
pub use sqlx_impl::*;

#[cfg(feature = "ssr")]
mod sqlx_impl {
    use crate::db::{DatabaseConnection, DbError, DbErrorKind, QueryResult};
    use async_trait::async_trait;
    use serde_json::Value;
    use sqlx::{postgres::PgPoolOptions, Column, Pool, Postgres, Row};
    use std::sync::Arc;

    /// SQLx-based PostgreSQL connection for server-side use
    #[derive(Clone)]
    pub struct SqlxConnection {
        pool: Arc<Pool<Postgres>>,
    }

    impl SqlxConnection {
        /// Create a new SQLx connection from a database URL
        pub async fn new(database_url: &str) -> Result<Self, DbError> {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await
                .map_err(|e| DbError {
                    kind: DbErrorKind::Connection,
                    message: format!("Failed to connect to database: {}", e),
                })?;

            Ok(SqlxConnection {
                pool: Arc::new(pool),
            })
        }

        /// Get the underlying SQLx pool
        pub fn pool(&self) -> &Pool<Postgres> {
            &self.pool
        }

        /// Convert SQLx Row to JSON Value
        fn row_to_json(row: &sqlx::postgres::PgRow) -> Result<Value, DbError> {
            let mut map = serde_json::Map::new();
            
            for column in row.columns() {
                let name = column.name();
                let value = if let Ok(val) = row.try_get::<Option<String>, _>(name) {
                    match val {
                        Some(s) => Value::String(s),
                        None => Value::Null,
                    }
                } else if let Ok(val) = row.try_get::<Option<i32>, _>(name) {
                    match val {
                        Some(i) => Value::Number(i.into()),
                        None => Value::Null,
                    }
                } else if let Ok(val) = row.try_get::<Option<i64>, _>(name) {
                    match val {
                        Some(i) => Value::Number(i.into()),
                        None => Value::Null,
                    }
                } else if let Ok(val) = row.try_get::<Option<f64>, _>(name) {
                    match val {
                        Some(f) => serde_json::Number::from_f64(f)
                            .map(Value::Number)
                            .unwrap_or(Value::Null),
                        None => Value::Null,
                    }
                } else if let Ok(val) = row.try_get::<Option<bool>, _>(name) {
                    match val {
                        Some(b) => Value::Bool(b),
                        None => Value::Null,
                    }
                } else if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(name) {
                    val.unwrap_or(Value::Null)
                } else {
                    Value::Null
                };
                
                map.insert(name.to_string(), value);
            }
            
            Ok(Value::Object(map))
        }

        /// Convert query parameters from JSON to SQLx format
        fn convert_params(params: Vec<Value>) -> Vec<Option<String>> {
            params
                .into_iter()
                .map(|v| match v {
                    Value::String(s) => Some(s),
                    Value::Number(n) => Some(n.to_string()),
                    Value::Bool(b) => Some(b.to_string()),
                    Value::Null => None,
                    _ => Some(v.to_string()),
                })
                .collect()
        }
    }

    #[async_trait]
    impl DatabaseConnection for SqlxConnection {
        async fn execute(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, DbError> {
            let mut sqlx_query = sqlx::query(query);
            
            // Bind parameters
            for param in Self::convert_params(params) {
                sqlx_query = sqlx_query.bind(param);
            }

            let result = sqlx_query
                .execute(&*self.pool)
                .await
                .map_err(|e| DbError {
                    kind: DbErrorKind::Query,
                    message: format!("Query execution failed: {}", e),
                })?;

            Ok(QueryResult {
                rows_affected: result.rows_affected(),
                last_insert_id: None, // PostgreSQL doesn't have LAST_INSERT_ID
            })
        }

        async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Value, DbError> {
            let mut sqlx_query = sqlx::query(query);
            
            // Bind parameters
            for param in Self::convert_params(params) {
                sqlx_query = sqlx_query.bind(param);
            }

            let row = sqlx_query
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => DbError {
                        kind: DbErrorKind::NotFound,
                        message: "No rows found".to_string(),
                    },
                    _ => DbError {
                        kind: DbErrorKind::Query,
                        message: format!("Query failed: {}", e),
                    },
                })?;

            Self::row_to_json(&row)
        }

        async fn query_many(&self, query: &str, params: Vec<Value>) -> Result<Vec<Value>, DbError> {
            let mut sqlx_query = sqlx::query(query);
            
            // Bind parameters
            for param in Self::convert_params(params) {
                sqlx_query = sqlx_query.bind(param);
            }

            let rows = sqlx_query
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| DbError {
                    kind: DbErrorKind::Query,
                    message: format!("Query failed: {}", e),
                })?;

            rows.iter()
                .map(Self::row_to_json)
                .collect::<Result<Vec<_>, _>>()
        }

        async fn begin_transaction(&self) -> Result<String, DbError> {
            // In a real implementation, we'd need to manage transaction state
            // For now, we'll use the connection directly
            self.execute("BEGIN", vec![]).await?;
            Ok(format!("tx_{}", uuid::Uuid::new_v4()))
        }

        async fn commit_transaction(&self, _tx_id: &str) -> Result<(), DbError> {
            self.execute("COMMIT", vec![]).await?;
            Ok(())
        }

        async fn rollback_transaction(&self, _tx_id: &str) -> Result<(), DbError> {
            self.execute("ROLLBACK", vec![]).await?;
            Ok(())
        }
    }

    use once_cell::sync::OnceCell;
    
    /// Database pool for server-side use
    static DB_POOL: OnceCell<SqlxConnection> = OnceCell::new();

    /// Initialize the database pool
    pub async fn init_db_pool(database_url: &str) -> Result<(), DbError> {
        let conn = SqlxConnection::new(database_url).await?;
        DB_POOL.set(conn).map_err(|_| DbError {
            kind: DbErrorKind::Connection,
            message: "Database pool already initialized".to_string(),
        })?;
        Ok(())
    }

    /// Get the database connection from the pool
    pub fn get_db_connection() -> Result<SqlxConnection, DbError> {
        DB_POOL.get().cloned().ok_or_else(|| DbError {
            kind: DbErrorKind::Connection,
            message: "Database pool not initialized. Call init_db_pool first.".to_string(),
        })
    }

    /// Server-side database hook
    pub fn use_db_server() -> Result<SqlxConnection, DbError> {
        get_db_connection()
    }
}

// Re-export for conditional compilation
#[cfg(not(feature = "ssr"))]
pub use crate::db::PostgresConnection as SqlxConnection;