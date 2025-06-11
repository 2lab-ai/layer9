//! Database/ORM Layer - L2/L3
//! WARP Database abstraction with support for multiple backends

use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use serde_json::Value;

/// Database connection trait
#[async_trait(?Send)]
pub trait DatabaseConnection: Clone + 'static {
    async fn execute(&self, query: &str, params: Vec<Value>) -> Result<QueryResult, DbError>;
    async fn query_one<T: DeserializeOwned>(&self, query: &str, params: Vec<Value>) -> Result<T, DbError>;
    async fn query_many<T: DeserializeOwned>(&self, query: &str, params: Vec<Value>) -> Result<Vec<T>, DbError>;
    async fn transaction<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&Transaction) -> Pin<Box<dyn Future<Output = Result<R, DbError>>>>;
}

/// Query result
#[derive(Debug, Clone)]
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
    id: String,
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
        self.joins.push(format!("{} JOIN {} ON {}", join_type, table, on));
        self
    }
    
    pub fn where_eq(mut self, column: &str, value: impl Into<Value>) -> Self {
        self.where_clause.push(format!("{} = ${}", column, self.params.len() + 1));
        self.params.push(value.into());
        self
    }
    
    pub fn where_in(mut self, column: &str, values: Vec<impl Into<Value>>) -> Self {
        let placeholders: Vec<String> = values.iter().enumerate()
            .map(|(i, _)| format!("${}", self.params.len() + i + 1))
            .collect();
        
        self.where_clause.push(format!("{} IN ({})", column, placeholders.join(", ")));
        self.params.extend(values.into_iter().map(|v| v.into()));
        self
    }
    
    pub fn where_like(mut self, column: &str, pattern: &str) -> Self {
        self.where_clause.push(format!("{} LIKE ${}", column, self.params.len() + 1));
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
        conn.query_many(&query, params).await
    }
    
    pub async fn first<C: DatabaseConnection>(&self, conn: &C) -> Result<M, DbError> {
        let (query, params) = self.build();
        conn.query_one(&query, params).await
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
        let query = format!("SELECT * FROM {} WHERE {} = $1", M::table_name(), M::primary_key());
        self.conn.query_one(&query, vec![id.into()]).await
    }
    
    pub async fn find_all(&self) -> Result<Vec<M>, DbError> {
        let query = format!("SELECT * FROM {}", M::table_name());
        self.conn.query_many(&query, vec![]).await
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
            
            self.conn.query_one(&query, values).await
        } else {
            Err(DbError {
                kind: DbErrorKind::Query,
                message: "Model must serialize to object".to_string(),
            })
        }
    }
    
    pub async fn update(&self, id: impl Into<Value>, updates: HashMap<String, Value>) -> Result<M, DbError> {
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
        
        self.conn.query_one(&query, params).await
    }
    
    pub async fn delete(&self, id: impl Into<Value>) -> Result<(), DbError> {
        let query = format!("DELETE FROM {} WHERE {} = $1", M::table_name(), M::primary_key());
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
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            vec![]
        ).await?;
        
        // Get applied migrations
        let applied: Vec<i32> = self.conn.query_many(
            "SELECT version FROM _migrations ORDER BY version",
            vec![]
        ).await?;
        
        // Apply pending migrations
        for migration in &self.migrations {
            if !applied.contains(&migration.version) {
                // Run migration
                self.conn.execute(&migration.up, vec![]).await?;
                
                // Record migration
                self.conn.execute(
                    "INSERT INTO _migrations (version, name) VALUES ($1, $2)",
                    vec![
                        Value::Number(migration.version.into()),
                        Value::String(migration.name.clone())
                    ]
                ).await?;
                
                web_sys::console::log_1(&format!("Applied migration: {}", migration.name).into());
            }
        }
        
        Ok(())
    }
}

/// Connection pool
pub struct ConnectionPool<C: DatabaseConnection> {
    connections: Vec<C>,
    max_size: usize,
}

impl<C: DatabaseConnection> ConnectionPool<C> {
    pub fn new(max_size: usize) -> Self {
        ConnectionPool {
            connections: vec![],
            max_size,
        }
    }
    
    pub async fn get(&self) -> Result<C, DbError> {
        if let Some(conn) = self.connections.first() {
            Ok(conn.clone())
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
        
        let response = crate::fetch::fetch(
            &format!("{}/execute", self.api_url),
            crate::fetch::FetchOptions {
                method: crate::fetch::Method::POST,
                headers,
                body: Some(body.to_string()),
                ..Default::default()
            }
        ).await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;
        
        serde_json::from_str(&response.text()).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }
    
    async fn query_one<T: DeserializeOwned>(&self, query: &str, params: Vec<Value>) -> Result<T, DbError> {
        let body = serde_json::json!({
            "query": query,
            "params": params,
        });
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        if let Some(token) = &self.auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
        
        let response = crate::fetch::fetch(
            &format!("{}/query_one", self.api_url),
            crate::fetch::FetchOptions {
                method: crate::fetch::Method::POST,
                headers,
                body: Some(body.to_string()),
                ..Default::default()
            }
        ).await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;
        
        serde_json::from_str(&response.text()).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }
    
    async fn query_many<T: DeserializeOwned>(&self, query: &str, params: Vec<Value>) -> Result<Vec<T>, DbError> {
        let body = serde_json::json!({
            "query": query,
            "params": params,
        });
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        if let Some(token) = &self.auth_token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }
        
        let response = crate::fetch::fetch(
            &format!("{}/query_many", self.api_url),
            crate::fetch::FetchOptions {
                method: crate::fetch::Method::POST,
                headers,
                body: Some(body.to_string()),
                ..Default::default()
            }
        ).await.map_err(|e| DbError {
            kind: DbErrorKind::Connection,
            message: e.to_string(),
        })?;
        
        serde_json::from_str(&response.text()).map_err(|e| DbError {
            kind: DbErrorKind::Query,
            message: e.to_string(),
        })
    }
    
    async fn transaction<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&Transaction) -> Pin<Box<dyn Future<Output = Result<R, DbError>>>>,
    {
        // Begin transaction
        self.execute("BEGIN", vec![]).await?;
        
        let tx = Transaction {
            conn: Box::new(self.clone()),
            id: format!("tx_{}", js_sys::Math::random()),
        };
        
        match f(&tx).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}

/// Hook for database operations
pub fn use_db<C: DatabaseConnection>() -> C {
    // In real app, this would get from context
    // For now, create a new connection
    let api_url = crate::env::env_or("DATABASE_API_URL", "http://localhost:3001/db");
    PostgresConnection::new(api_url).with_auth("dummy-token") as C
}

/// Hook for repository
pub fn use_repository<M: Model, C: DatabaseConnection>() -> Repository<M, C> {
    let conn = use_db();
    Repository::new(conn)
}

// Re-exports
pub use serde::de::DeserializeOwned;
use std::pin::Pin;
use std::future::Future;