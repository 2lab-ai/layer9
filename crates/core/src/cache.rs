//! Advanced Caching Strategies - L3/L4
//! Multi-layer caching with various invalidation strategies

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

/// Cache entry with metadata
#[derive(Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub access_count: u32,
    pub last_accessed: u64,
    pub tags: Vec<String>,
    pub etag: Option<String>,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Option<Duration>) -> Self {
        let now = current_timestamp();
        CacheEntry {
            value,
            created_at: now,
            expires_at: ttl.map(|d| now + d.as_secs()),
            access_count: 0,
            last_accessed: now,
            tags: vec![],
            etag: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            current_timestamp() > expires_at
        } else {
            false
        }
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_etag(mut self, etag: String) -> Self {
        self.etag = Some(etag);
        self
    }
}

/// Cache storage trait
pub trait CacheStorage<T: Clone>: 'static {
    fn get(&self, key: &str) -> Option<CacheEntry<T>>;
    fn set(&self, key: String, entry: CacheEntry<T>);
    fn remove(&self, key: &str) -> Option<CacheEntry<T>>;
    fn clear(&self);
    fn keys(&self) -> Vec<String>;
    fn size(&self) -> usize;
}

/// In-memory cache storage
#[derive(Clone)]
pub struct MemoryStorage<T: Clone> {
    data: Rc<RefCell<HashMap<String, CacheEntry<T>>>>,
    max_size: Option<usize>,
}

impl<T: Clone + 'static> MemoryStorage<T> {
    pub fn new(max_size: Option<usize>) -> Self {
        MemoryStorage {
            data: Rc::new(RefCell::new(HashMap::new())),
            max_size,
        }
    }

    fn evict_if_needed(&self) {
        if let Some(max_size) = self.max_size {
            let mut data = self.data.borrow_mut();
            while data.len() >= max_size {
                // LRU eviction
                if let Some(key) = data
                    .iter()
                    .min_by_key(|(_, entry)| entry.last_accessed)
                    .map(|(k, _)| k.clone())
                {
                    data.remove(&key);
                }
            }
        }
    }
}

impl<T: Clone + 'static> CacheStorage<T> for MemoryStorage<T> {
    fn get(&self, key: &str) -> Option<CacheEntry<T>> {
        let mut data = self.data.borrow_mut();
        if let Some(entry) = data.get_mut(key) {
            if !entry.is_expired() {
                entry.access_count += 1;
                entry.last_accessed = current_timestamp();
                Some(entry.clone())
            } else {
                data.remove(key);
                None
            }
        } else {
            None
        }
    }

    fn set(&self, key: String, entry: CacheEntry<T>) {
        self.evict_if_needed();
        self.data.borrow_mut().insert(key, entry);
    }

    fn remove(&self, key: &str) -> Option<CacheEntry<T>> {
        self.data.borrow_mut().remove(key)
    }

    fn clear(&self) {
        self.data.borrow_mut().clear();
    }

    fn keys(&self) -> Vec<String> {
        self.data.borrow().keys().cloned().collect()
    }

    fn size(&self) -> usize {
        self.data.borrow().len()
    }
}

/// LocalStorage cache storage
#[derive(Clone)]
pub struct LocalStorageCache {
    prefix: String,
}

impl LocalStorageCache {
    pub fn new(prefix: impl Into<String>) -> Self {
        LocalStorageCache {
            prefix: prefix.into(),
        }
    }

    fn full_key(&self, key: &str) -> String {
        format!("{}:{}", self.prefix, key)
    }

    fn get_storage(&self) -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }
}

impl CacheStorage<String> for LocalStorageCache {
    fn get(&self, key: &str) -> Option<CacheEntry<String>> {
        let storage = self.get_storage()?;
        let full_key = self.full_key(key);
        let json = storage.get_item(&full_key).ok()??;

        serde_json::from_str(&json).ok()
    }

    fn set(&self, key: String, entry: CacheEntry<String>) {
        if let Some(storage) = self.get_storage() {
            let full_key = self.full_key(&key);
            if let Ok(json) = serde_json::to_string(&entry) {
                let _ = storage.set_item(&full_key, &json);
            }
        }
    }

    fn remove(&self, key: &str) -> Option<CacheEntry<String>> {
        let storage = self.get_storage()?;
        let full_key = self.full_key(key);
        let entry = self.get(key);
        let _ = storage.remove_item(&full_key);
        entry
    }

    fn clear(&self) {
        if let Some(storage) = self.get_storage() {
            // Clear only our prefixed keys
            let mut keys_to_remove = vec![];
            for i in 0..storage.length().unwrap_or(0) {
                if let Ok(Some(key)) = storage.key(i) {
                    if key.starts_with(&self.prefix) {
                        keys_to_remove.push(key);
                    }
                }
            }

            for key in keys_to_remove {
                let _ = storage.remove_item(&key);
            }
        }
    }

    fn keys(&self) -> Vec<String> {
        let mut keys = vec![];
        if let Some(storage) = self.get_storage() {
            for i in 0..storage.length().unwrap_or(0) {
                if let Ok(Some(key)) = storage.key(i) {
                    if key.starts_with(&self.prefix) {
                        keys.push(
                            key.trim_start_matches(&format!("{}:", self.prefix))
                                .to_string(),
                        );
                    }
                }
            }
        }
        keys
    }

    fn size(&self) -> usize {
        self.keys().len()
    }
}

/// Cache invalidation strategies
#[derive(Debug, Clone)]
pub enum InvalidationStrategy {
    TTL(Duration),
    TagBased(Vec<String>),
    Manual,
    Stale(Duration), // Serve stale while revalidating
}

/// Cache warming strategy
pub trait CacheWarmer<T: Clone>: 'static {
    fn warm(&self, cache: &Cache<T>);
}

/// Main cache implementation
pub struct Cache<T: Clone> {
    storage: Box<dyn CacheStorage<T>>,
    invalidation: InvalidationStrategy,
    warmers: Vec<Box<dyn CacheWarmer<T>>>,
}

impl<T: Clone + 'static> Cache<T> {
    pub fn new(
        storage: impl CacheStorage<T> + 'static,
        invalidation: InvalidationStrategy,
    ) -> Self {
        Cache {
            storage: Box::new(storage),
            invalidation,
            warmers: vec![],
        }
    }

    pub fn get(&self, key: &str) -> Option<T> {
        self.storage.get(key).map(|entry| entry.value)
    }

    pub fn get_or_compute<F>(&self, key: &str, compute: F) -> T
    where
        F: FnOnce() -> T,
    {
        if let Some(value) = self.get(key) {
            value
        } else {
            let value = compute();
            self.set(key.to_string(), value.clone());
            value
        }
    }

    pub async fn get_or_compute_async<F, Fut>(&self, key: &str, compute: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        if let Some(value) = self.get(key) {
            value
        } else {
            let value = compute().await;
            self.set(key.to_string(), value.clone());
            value
        }
    }

    pub fn set(&self, key: String, value: T) {
        let ttl = match &self.invalidation {
            InvalidationStrategy::TTL(duration) => Some(*duration),
            InvalidationStrategy::Stale(duration) => Some(*duration),
            _ => None,
        };

        let entry = CacheEntry::new(value, ttl);
        self.storage.set(key, entry);
    }

    pub fn set_with_tags(&self, key: String, value: T, tags: Vec<String>) {
        let ttl = match &self.invalidation {
            InvalidationStrategy::TTL(duration) => Some(*duration),
            InvalidationStrategy::Stale(duration) => Some(*duration),
            _ => None,
        };

        let entry = CacheEntry::new(value, ttl).with_tags(tags);
        self.storage.set(key, entry);
    }

    pub fn invalidate(&self, key: &str) {
        self.storage.remove(key);
    }

    pub fn invalidate_by_tags(&self, tags: &[String]) {
        let keys = self.storage.keys();
        for key in keys {
            if let Some(entry) = self.storage.get(&key) {
                if tags.iter().any(|tag| entry.tags.contains(tag)) {
                    self.storage.remove(&key);
                }
            }
        }
    }

    pub fn clear(&self) {
        self.storage.clear();
    }

    pub fn warm(&self) {
        for warmer in &self.warmers {
            warmer.warm(self);
        }
    }

    pub fn add_warmer(mut self, warmer: impl CacheWarmer<T> + 'static) -> Self {
        self.warmers.push(Box::new(warmer));
        self
    }
}

/// Multi-layer cache
pub struct MultiLayerCache<T: Clone> {
    layers: Vec<Cache<T>>,
}

impl<T: Clone + 'static> MultiLayerCache<T> {
    pub fn new() -> Self {
        MultiLayerCache { layers: vec![] }
    }

    pub fn add_layer(mut self, cache: Cache<T>) -> Self {
        self.layers.push(cache);
        self
    }

    pub fn get(&self, key: &str) -> Option<T> {
        for (i, layer) in self.layers.iter().enumerate() {
            if let Some(value) = layer.get(key) {
                // Promote to higher layers
                for j in 0..i {
                    self.layers[j].set(key.to_string(), value.clone());
                }
                return Some(value);
            }
        }
        None
    }

    pub fn set(&self, key: String, value: T) {
        // Set in all layers
        for layer in &self.layers {
            layer.set(key.clone(), value.clone());
        }
    }

    pub fn invalidate(&self, key: &str) {
        for layer in &self.layers {
            layer.invalidate(key);
        }
    }
}

/// HTTP cache with ETag support
pub struct HttpCache {
    storage: MemoryStorage<HttpCacheEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct HttpCacheEntry {
    pub data: String,
    pub headers: HashMap<String, String>,
    pub status: u16,
}

/// Cached response wrapper
pub struct CachedResponse {
    pub data: String,
    pub headers: Vec<(String, String)>,
    pub status: u16,
}

impl HttpCache {
    pub fn new() -> Self {
        HttpCache {
            storage: MemoryStorage::new(Some(100)),
        }
    }

    pub async fn fetch(
        &self,
        url: &str,
        options: FetchOptions,
    ) -> Result<CachedResponse, FetchError> {
        let key = format!("{}:{:?}", url, options);

        // Check cache
        if let Some(entry) = self.storage.get(&key) {
            // Check if we have an ETag
            if let Some(etag) = &entry.etag {
                // Make conditional request
                let mut headers = options.headers.clone();
                headers.insert("If-None-Match".to_string(), etag.clone());

                let mut conditional_options = options.clone();
                conditional_options.headers = headers;

                match crate::fetch::fetch(url, Some(conditional_options)).await {
                    Ok(response) if response.status() == 304 => {
                        // Not modified, use cached version
                        Ok(CachedResponse {
                            data: entry.value.data.clone(),
                            headers: entry.value.headers.clone().into_iter().collect(),
                            status: 200,
                        })
                    }
                    Ok(response) => {
                        // Updated, cache new version
                        let status = response.status();
                        let text = response.text().await?;
                        let cached = CachedResponse {
                            data: text,
                            headers: vec![], // TODO: extract headers from response
                            status,
                        };
                        self.cache_response_data(&key, &cached);
                        Ok(cached)
                    }
                    Err(_e) => {
                        // Error, use stale cache if available
                        Ok(CachedResponse {
                            data: entry.value.data.clone(),
                            headers: entry.value.headers.clone().into_iter().collect(),
                            status: 200,
                        })
                    }
                }
            } else {
                // No ETag, check if expired
                if !entry.is_expired() {
                    Ok(CachedResponse {
                        data: entry.value.data.clone(),
                        headers: entry.value.headers.clone().into_iter().collect(),
                        status: 200,
                    })
                } else {
                    // Expired, fetch fresh
                    let response = crate::fetch::fetch(url, Some(options)).await?;
                    let status = response.status();
                    let text = response.text().await?;
                    let cached = CachedResponse {
                        data: text,
                        headers: vec![], // TODO: extract headers from response
                        status,
                    };
                    self.cache_response_data(&key, &cached);
                    Ok(cached)
                }
            }
        } else {
            // Not in cache, fetch
            let response = crate::fetch::fetch(url, Some(options)).await?;
            let status = response.status();
            let text = response.text().await?;
            let cached = CachedResponse {
                data: text,
                headers: vec![], // TODO: extract headers from response
                status,
            };
            self.cache_response_data(&key, &cached);
            Ok(cached)
        }
    }

    fn cache_response_data(&self, key: &str, response: &CachedResponse) {
        let cache_entry = HttpCacheEntry {
            data: response.data.clone(),
            headers: response.headers.iter().cloned().collect(),
            status: response.status,
        };

        // Determine TTL from cache headers
        let ttl = if let Some((_, cache_control)) =
            response.headers.iter().find(|(k, _)| k == "cache-control")
        {
            parse_cache_control_ttl(cache_control)
        } else {
            Some(Duration::from_secs(300)) // Default 5 minutes
        };

        let mut entry = CacheEntry::new(cache_entry, ttl);

        // Store ETag if present
        if let Some((_, etag)) = response.headers.iter().find(|(k, _)| k == "etag") {
            entry = entry.with_etag(etag.clone());
        }

        self.storage.set(key.to_string(), entry);
    }
}

/// Parse cache control header for TTL
fn parse_cache_control_ttl(header: &str) -> Option<Duration> {
    for directive in header.split(',') {
        let directive = directive.trim();
        if let Some(max_age) = directive.strip_prefix("max-age=") {
            if let Ok(seconds) = max_age.parse::<u64>() {
                return Some(Duration::from_secs(seconds));
            }
        }
    }
    None
}

/// Get current timestamp in seconds
fn current_timestamp() -> u64 {
    js_sys::Date::now() as u64 / 1000
}

/// Cache hooks
pub fn use_cache<T: Clone + 'static>() -> Cache<T> {
    Cache::new(
        MemoryStorage::new(Some(100)),
        InvalidationStrategy::TTL(Duration::from_secs(300)),
    )
}

pub fn use_local_cache() -> Cache<String> {
    Cache::new(
        LocalStorageCache::new("layer9-cache"),
        InvalidationStrategy::Manual,
    )
}

pub fn use_http_cache() -> HttpCache {
    HttpCache::new()
}

// Re-exports
use crate::fetch::{FetchError, FetchOptions};
use std::future::Future;
