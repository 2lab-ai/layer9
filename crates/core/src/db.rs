//! Database/ORM Layer - L2/L3
//! Layer9 Database abstraction with support for multiple backends

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::marker::PhantomData;

/// Database connection trait
#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
pub trait DatabaseConnection: 'static {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, DbError>;
    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Value, DbError>;
    async fn query_many(&self, query: &str, params: Vec<Value>) -> Result<Vec<Value>, DbError>;
    async fn begin_transaction(&self) -> Result<String, DbError>;
    async fn commit_transaction(&self, tx_id: &str) -> Result<(), DbError>;
    async fn rollback_transaction(&self, tx_id: &str) -> Result<(), DbError>;
}

/// Database connection trait (server-side with Send)
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait DatabaseConnection: Send + Sync + 'static {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, DbError>;
    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Value, DbError>;
    async fn query_many(&self, query: &str, params: Vec<Value>) -> Result<Vec<Value>, DbError>;
    async fn begin_transaction(&self) -> Result<String, DbError>;
    async fn commit_transaction(&self, tx_id: &str) -> Result<(), DbError>;
    async fn rollback_transaction(&self, tx_id: &str) -> Result<(), DbError>;
}

/// Implement DatabaseConnection for Box<dyn DatabaseConnection> (WASM)
#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl DatabaseConnection for Box<dyn DatabaseConnection> {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, DbError> {
        self.as_ref().execute(query, params).await
    }
    
    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Value, DbError> {
        self.as_ref().query_one(query, params).await
    }
    
    async fn query_many(&self, query: &str, params: Vec<Value>) -> Result<Vec<Value>, DbError> {
        self.as_ref().query_many(query, params).await
    }
    
    async fn begin_transaction(&self) -> Result<String, DbError> {
        self.as_ref().begin_transaction().await
    }
    
    async fn commit_transaction(&self, tx_id: &str) -> Result<(), DbError> {
        self.as_ref().commit_transaction(tx_id).await
    }
    
    async fn rollback_transaction(&self, tx_id: &str) -> Result<(), DbError> {
        self.as_ref().rollback_transaction(tx_id).await
    }
}

/// Implement DatabaseConnection for Box<dyn DatabaseConnection> (Server)
#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl DatabaseConnection for Box<dyn DatabaseConnection> {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, DbError> {
        self.as_ref().execute(query, params).await
    }
    
    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Value, DbError> {
        self.as_ref().query_one(query, params).await
    }
    
    async fn query_many(&self, query: &str, params: Vec<Value>) -> Result<Vec<Value>, DbError> {
        self.as_ref().query_many(query, params).await
    }
    
    async fn begin_transaction(&self) -> Result<String, DbError> {
        self.as_ref().begin_transaction().await
    }
    
    async fn commit_transaction(&self, tx_id: &str) -> Result<(), DbError> {
        self.as_ref().commit_transaction(tx_id).await
    }
    
    async fn rollback_transaction(&self, tx_id: &str) -> Result<(), DbError> {
        self.as_ref().rollback_transaction(tx_id).await
    }
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub rows_affected: u64,
    pub last_insert_id: Option<i64>,
}

/// Database error
#[derive(Debug, Clone)]
pub struct DbError {
    pub kind: DbErrorKind,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum DbErrorKind {
    Connection,
    Query,
    Transaction,
    NotFound,
    UniqueViolation,
    ForeignKeyViolation,
    Timeout,
}

/// Transaction handle
pub struct Transaction {
    conn: Box<dyn DatabaseConnection>,
    _id: String,
}

impl Transaction {
    pub async fn execute(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, DbError> {
        self.conn.execute(query, params).await
    }

    pub async fn commit(self) -> Result<(), DbError> {
        self.conn.execute("COMMIT", vec![]).await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), DbError> {
        self.conn.execute("ROLLBACK", vec![]).await?;
        Ok(())
    }
}

/// ORM Model trait
pub trait Model: Sized + Serialize + DeserializeOwned {
    const TABLE_NAME: &'static str;
    const PRIMARY_KEY: &'static str = "id";

    fn table_name() -> &'static str {
        Self::TABLE_NAME
    }

    fn primary_key() -> &'static str {
        Self::PRIMARY_KEY
    }
}

/// Query builder
pub struct QueryBuilder<M: Model> {
    table: String,
    select: Vec<String>,
    joins: Vec<String>,
    where_clause: Vec<String>,
    order_by: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    params: Vec<Value>,
    _phantom: PhantomData<M>,
}

impl<M: Model> Default for QueryBuilder<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Model> QueryBuilder<M> {
    pub fn new() -> Self {
        QueryBuilder {
            table: M::table_name().to_string(),
            select: vec!["*".to_string()],
            joins: vec![],
            where_clause: vec![],
            order_by: vec![],
            limit: None,
            offset: None,
            params: vec![],
            _phantom: PhantomData,
        }
    }

    pub fn select(mut self, columns: &[&str]) -> Self {
        self.select = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn join(mut self, join_type: &str, table: &str, on: &str) -> Self {
        self.joins
            .push(format!("{} JOIN {} ON {}", join_type, table, on));
        self
    }

    pub fn where_eq(mut self, column: &str, value: impl Into<Value>) -> Self {
        self.where_clause
            .push(format!("{} = ${}", column, self.params.len() + 1));
        self.params.push(value.into());
        self
    }

    pub fn where_in(mut self, column: &str, values: Vec<impl Into<Value>>) -> Self {
        let placeholders: Vec<String> = values
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", self.params.len() + i + 1))
            .collect();

        self.where_clause
            .push(format!("{} IN ({})", column, placeholders.join(", ")));
        self.params.extend(values.into_iter().map(|v| v.into()));
        self
    }

    pub fn where_like(mut self, column: &str, pattern: &str) -> Self {
        self.where_clause
            .push(format!("{} LIKE ${}", column, self.params.len() + 1));
        self.params.push(Value::String(pattern.to_string()));
        self
    }

    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        self.order_by.push(format!("{} {}", column, direction));
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build(&self) -> (String, Vec<Value>) {
        let mut query = format!("SELECT {} FROM {}", self.select.join(", "), self.table);

        // Add joins
        for join in &self.joins {
            query.push_str(&format!(" {}", join));
        }

        // Add where clause
        if !self.where_clause.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_clause.join(" AND ")));
        }

        // Add order by
        if !self.order_by.is_empty() {
            query.push_str(&format!(" ORDER BY {}", self.order_by.join(", ")));
        }

        // Add limit/offset
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        (query, self.params.clone())
    }

    pub async fn execute<C: DatabaseConnection>(&self, conn: &C) -> Result<Vec<M>, DbError> {
        let (query, params) = self.build();
        let values = conn.query_many(&query, params).await?;
        values
            .into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| DbError {
                    kind: DbErrorKind::Query,
                    message: e.to_string(),
                })
            })
            .collect()
    }

    pub async fn first<C: DatabaseConnection>(&self, conn: &C) -> Result<M, DbError> {
        let (query, params) = self.build();
        let value = conn.query_one(&query, params).await?;
        serde_json::from_value(value).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }
}

/// Repository pattern
pub struct Repository<M: Model, C: DatabaseConnection> {
    conn: C,
    _phantom: PhantomData<M>,
}

impl<M: Model, C: DatabaseConnection> Repository<M, C> {
    pub fn new(conn: C) -> Self {
        Repository {
            conn,
            _phantom: PhantomData,
        }
    }

    pub async fn find_by_id(&self, id: impl Into<Value>) -> Result<M, DbError> {
        let query = format!(
            "SELECT * FROM {} WHERE {} = $1",
            M::table_name(),
            M::primary_key()
        );
        let value = self.conn.query_one(&query, vec![id.into()]).await?;
        serde_json::from_value(value).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }

    pub async fn find_all(&self) -> Result<Vec<M>, DbError> {
        let query = format!("SELECT * FROM {}", M::table_name());
        let values = self.conn.query_many(&query, vec![]).await?;
        values
            .into_iter()
            .map(|v| {
                serde_json::from_value(v).map_err(|e| DbError {
                    kind: DbErrorKind::Query,
                    message: e.to_string(),
                })
            })
            .collect()
    }

    pub async fn insert(&self, model: &M) -> Result<M, DbError> {
        let json = serde_json::to_value(model).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })?;

        if let Value::Object(map) = json {
            let columns: Vec<String> = map.keys().cloned().collect();
            let values: Vec<Value> = map.values().cloned().collect();
            let placeholders: Vec<String> = (1..=values.len()).map(|i| format!("${}", i)).collect();

            let query = format!(
                "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                M::table_name(),
                columns.join(", "),
                placeholders.join(", ")
            );

            let value = self.conn.query_one(&query, values).await?;
            serde_json::from_value(value).map_err(|e| DbError {
                kind: DbErrorKind::Query,
                message: e.to_string(),
            })
        } else {
            Err(DbError {
                kind: DbErrorKind::Query,
                message: "Model must serialize to object".to_string(),
            })
        }
    }

    pub async fn update(
        &self,
        id: impl Into<Value>,
        updates: HashMap<String, Value>,
    ) -> Result<M, DbError> {
        let mut set_clauses = vec![];
        let mut params = vec![];

        for (i, (column, value)) in updates.iter().enumerate() {
            set_clauses.push(format!("{} = ${}", column, i + 1));
            params.push(value.clone());
        }

        params.push(id.into());

        let query = format!(
            "UPDATE {} SET {} WHERE {} = ${} RETURNING *",
            M::table_name(),
            set_clauses.join(", "),
            M::primary_key(),
            params.len()
        );

        let value = self.conn.query_one(&query, params).await?;
        serde_json::from_value(value).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }

    pub async fn delete(&self, id: impl Into<Value>) -> Result<(), DbError> {
        let query = format!(
            "DELETE FROM {} WHERE {} = $1",
            M::table_name(),
            M::primary_key()
        );
        self.conn.execute(&query, vec![id.into()]).await?;
        Ok(())
    }

    pub fn query(&self) -> QueryBuilder<M> {
        QueryBuilder::new()
    }
}

/// Database migrations
pub struct Migration {
    pub version: i32,
    pub name: String,
    pub up: String,
    pub down: String,
}

pub struct Migrator<C: DatabaseConnection> {
    conn: C,
    migrations: Vec<Migration>,
}

impl<C: DatabaseConnection> Migrator<C> {
    pub fn new(conn: C) -> Self {
        Migrator {
            conn,
            migrations: vec![],
        }
    }

    pub fn add_migration(mut self, migration: Migration) -> Self {
        self.migrations.push(migration);
        self.migrations.sort_by_key(|m| m.version);
        self
    }

    pub async fn migrate(&self) -> Result<(), DbError> {
        // Create migrations table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
                vec![],
            )
            .await?;

        // Get applied migrations
        let applied_values = self
            .conn
            .query_many("SELECT version FROM _migrations ORDER BY version", vec![])
            .await?;

        let applied: Vec<i32> = applied_values
            .into_iter()
            .filter_map(|v| {
                if let Value::Object(map) = v {
                    map.get("version")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32)
                } else {
                    None
                }
            })
            .collect();

        // Apply pending migrations
        for migration in &self.migrations {
            if !applied.contains(&migration.version) {
                // Run migration
                self.conn.execute(&migration.up, vec![]).await?;

                // Record migration
                self.conn
                    .execute(
                        "INSERT INTO _migrations (version, name) VALUES ($1, $2)",
                        vec![
                            Value::Number(migration.version.into()),
                            Value::String(migration.name.clone()),
                        ],
                    )
                    .await?;

                web_sys::console::log_1(&format!("Applied migration: {}", migration.name).into());
            }
        }

        Ok(())
    }
}

/// Connection pool
pub struct ConnectionPool<C: DatabaseConnection> {
    connections: Vec<C>,
    _max_size: usize,
}

impl<C: DatabaseConnection> ConnectionPool<C> {
    pub fn new(max_size: usize) -> Self {
        ConnectionPool {
            connections: vec![],
            _max_size: max_size,
        }
    }

    pub async fn get(&self) -> Result<&C, DbError> {
        if let Some(conn) = self.connections.first() {
            Ok(conn)
        } else {
            Err(DbError {
                kind: DbErrorKind::Connection,
                message: "No connections available".to_string(),
            })
        }
    }
}

/// PostgreSQL connection implementation (client-side via HTTP)
#[derive(Clone)]
pub struct PostgresConnection {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    api_url: String,
    auth_token: Option<String>,
}

impl PostgresConnection {
    pub fn new(api_url: impl Into<String>) -> Self {
        PostgresConnection {
            api_url: api_url.into(),
            auth_token: None,
        }
    }

    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl DatabaseConnection for PostgresConnection {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, DbError> {
        let body = serde_json::json!({
            "query": query,
            "params": params,
        });

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        if let Some(token) = &self.auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let mut fetch_builder =
            crate::fetch::FetchBuilder::new(format!("{}/execute", self.api_url))
                .method(crate::fetch::Method::POST);

        for (key, value) in headers {
            fetch_builder = fetch_builder.header(key, value);
        }

        fetch_builder = fetch_builder.json(&body).map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: format!("{:?}", e),
        })?;

        let response = fetch_builder.send().await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;

        let text = response.text().await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;

        serde_json::from_str(&text).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }

    async fn query_one(&self, query: &str, params: Vec<Value>) -> Result<Value, DbError> {
        let body = serde_json::json!({
            "query": query,
            "params": params,
        });

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        if let Some(token) = &self.auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let mut fetch_builder =
            crate::fetch::FetchBuilder::new(format!("{}/query_one", self.api_url))
                .method(crate::fetch::Method::POST);

        for (key, value) in headers {
            fetch_builder = fetch_builder.header(key, value);
        }

        fetch_builder = fetch_builder.json(&body).map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: format!("{:?}", e),
        })?;

        let response = fetch_builder.send().await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;

        let text = response.text().await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;

        serde_json::from_str(&text).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }

    async fn query_many(&self, query: &str, params: Vec<Value>) -> Result<Vec<Value>, DbError> {
        let body = serde_json::json!({
            "query": query,
            "params": params,
        });

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        if let Some(token) = &self.auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        let mut fetch_builder =
            crate::fetch::FetchBuilder::new(format!("{}/query_many", self.api_url))
                .method(crate::fetch::Method::POST);

        for (key, value) in headers {
            fetch_builder = fetch_builder.header(key, value);
        }

        fetch_builder = fetch_builder.json(&body).map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: format!("{:?}", e),
        })?;

        let response = fetch_builder.send().await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;

        let text = response.text().await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;

        serde_json::from_str(&text).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }

    async fn begin_transaction(&self) -> Result<String, DbError> {
        self.execute("BEGIN", vec![]).await?;
        Ok(format!("tx_{}", js_sys::Math::random()))
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

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl DatabaseConnection for PostgresConnection {
    async fn execute(&self, _query: &str, _params: Vec<Value>) -> Result<QueryResult, DbError> {
        Err(DbError {
            kind: DbErrorKind::Connection,
            message: "PostgresConnection not implemented for server-side. Use SqlxConnection instead.".to_string(),
        })
    }

    async fn query_one(&self, _query: &str, _params: Vec<Value>) -> Result<Value, DbError> {
        Err(DbError {
            kind: DbErrorKind::Connection,
            message: "PostgresConnection not implemented for server-side. Use SqlxConnection instead.".to_string(),
        })
    }

    async fn query_many(&self, _query: &str, _params: Vec<Value>) -> Result<Vec<Value>, DbError> {
        Err(DbError {
            kind: DbErrorKind::Connection,
            message: "PostgresConnection not implemented for server-side. Use SqlxConnection instead.".to_string(),
        })
    }

    async fn begin_transaction(&self) -> Result<String, DbError> {
        Err(DbError {
            kind: DbErrorKind::Connection,
            message: "PostgresConnection not implemented for server-side. Use SqlxConnection instead.".to_string(),
        })
    }

    async fn commit_transaction(&self, _tx_id: &str) -> Result<(), DbError> {
        Err(DbError {
            kind: DbErrorKind::Connection,
            message: "PostgresConnection not implemented for server-side. Use SqlxConnection instead.".to_string(),
        })
    }

    async fn rollback_transaction(&self, _tx_id: &str) -> Result<(), DbError> {
        Err(DbError {
            kind: DbErrorKind::Connection,
            message: "PostgresConnection not implemented for server-side. Use SqlxConnection instead.".to_string(),
        })
    }
}

/// Hook for database operations
#[cfg(not(target_arch = "wasm32"))]
pub fn use_db() -> Box<dyn DatabaseConnection> {
    #[cfg(feature = "ssr")]
    {
        match crate::db_sqlx::use_db_server() {
            Ok(conn) => Box::new(conn),
            Err(_) => {
                // Fallback to HTTP connection if SQLx is not available
                let api_url = crate::env::env_or("DATABASE_API_URL", "http://localhost:3001/db");
                Box::new(PostgresConnection::new(api_url).with_auth("dummy-token"))
            }
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        let api_url = crate::env::env_or("DATABASE_API_URL", "http://localhost:3001/db");
        Box::new(PostgresConnection::new(api_url).with_auth("dummy-token"))
    }
}

/// Hook for database operations (WASM/client-side)
#[cfg(target_arch = "wasm32")]
pub fn use_db() -> PostgresConnection {
    let api_url = crate::env::env_or("DATABASE_API_URL", "/api/db");
    PostgresConnection::new(api_url).with_auth("dummy-token")
}

/// Hook for repository (client-side WASM)
#[cfg(target_arch = "wasm32")]
pub fn use_repository<M: Model>() -> Repository<M, PostgresConnection> {
    let conn = use_db();
    Repository::new(conn)
}

/// Hook for repository (server-side)
#[cfg(not(target_arch = "wasm32"))]
pub fn use_repository<M: Model>() -> Repository<M, Box<dyn DatabaseConnection>> {
    let conn = use_db();
    Repository::new(conn)
}
