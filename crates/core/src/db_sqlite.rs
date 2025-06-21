//! SQLite Database Implementation
//! Provides a lightweight database option for development and small-scale deployments

#[cfg(feature = "ssr")]
pub use sqlite_impl::*;

#[cfg(feature = "ssr")]
mod sqlite_impl {
    use crate::db::{DatabaseConnection, DbError, DbErrorKind, QueryResult};
    use async_trait::async_trait;
    use serde_json::Value;
    use sqlx::{sqlite::SqlitePoolOptions, Column, Pool, Row, Sqlite};
    use std::sync::Arc;

    /// SQLite-based connection for lightweight database needs
    #[derive(Clone)]
    pub struct SqliteConnection {
        pool: Arc<Pool<Sqlite>>,
    }

    impl SqliteConnection {
        /// Create a new SQLite connection
        /// Use ":memory:" for in-memory database
        pub async fn new(database_url: &str) -> Result<Self, DbError> {
            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await
                .map_err(|e| DbError {
                    kind: DbErrorKind::Connection,
                    message: format!("Failed to connect to SQLite database: {}", e),
                })?;

            Ok(SqliteConnection {
                pool: Arc::new(pool),
            })
        }

        /// Create an in-memory SQLite database (useful for testing)
        pub async fn in_memory() -> Result<Self, DbError> {
            Self::new(":memory:").await
        }

        /// Get the underlying SQLx pool
        pub fn pool(&self) -> &Pool<Sqlite> {
            &self.pool
        }

        /// Convert SQLx Row to JSON Value
        fn row_to_json(row: &sqlx::sqlite::SqliteRow) -> Result<Value, DbError> {
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
                } else if let Ok(val) = row.try_get::<Option<Vec<u8>>, _>(name) {
                    // Handle BLOB data as base64
                    match val {
                        Some(bytes) => {
                            use base64::{Engine as _, engine::general_purpose};
                            Value::String(general_purpose::STANDARD.encode(bytes))
                        }
                        None => Value::Null,
                    }
                } else {
                    Value::Null
                };
                
                map.insert(name.to_string(), value);
            }
            
            Ok(Value::Object(map))
        }

        /// Convert query parameters from JSON to SQLite format
        fn convert_params(params: Vec<Value>) -> Vec<Option<String>> {
            params
                .into_iter()
                .map(|v| match v {
                    Value::String(s) => Some(s),
                    Value::Number(n) => Some(n.to_string()),
                    Value::Bool(b) => Some(if b { "1" } else { "0" }.to_string()),
                    Value::Null => None,
                    _ => Some(v.to_string()),
                })
                .collect()
        }
    }

    #[async_trait]
    impl DatabaseConnection for SqliteConnection {
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
                    kind: match &e {
                        sqlx::Error::Database(db_err) if db_err.message().contains("UNIQUE") => {
                            DbErrorKind::UniqueViolation
                        }
                        sqlx::Error::Database(db_err) if db_err.message().contains("FOREIGN KEY") => {
                            DbErrorKind::ForeignKeyViolation
                        }
                        _ => DbErrorKind::Query,
                    },
                    message: format!("Query execution failed: {}", e),
                })?;

            Ok(QueryResult {
                rows_affected: result.rows_affected(),
                last_insert_id: Some(result.last_insert_rowid()),
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
            // SQLite uses a different transaction syntax
            self.execute("BEGIN TRANSACTION", vec![]).await?;
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

    /// Create default tables for a new SQLite database
    pub async fn create_default_schema(conn: &SqliteConnection) -> Result<(), DbError> {
        // Enable foreign keys (disabled by default in SQLite)
        conn.execute("PRAGMA foreign_keys = ON", vec![]).await?;
        
        // Create users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            vec![],
        ).await?;

        // Create posts table (example)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS posts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                content TEXT,
                published BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            vec![],
        ).await?;

        // Create sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id INTEGER NOT NULL,
                expires_at DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )",
            vec![],
        ).await?;

        Ok(())
    }
}