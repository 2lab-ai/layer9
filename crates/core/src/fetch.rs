//! Fetch API - L4 (Real HTTP calls)

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// HTTP methods
#[derive(Clone, Copy)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl Method {
    fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
        }
    }
}

/// Fetch builder
pub struct FetchBuilder {
    url: String,
    method: Method,
    headers: HashMap<String, String>,
    body: Option<String>,
    credentials: Option<web_sys::RequestCredentials>,
}

impl FetchBuilder {
    pub fn new(url: impl Into<String>) -> Self {
        FetchBuilder {
            url: url.into(),
            method: Method::GET,
            headers: HashMap::new(),
            body: None,
            credentials: None,
        }
    }
    
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    pub fn json<T: Serialize>(mut self, data: &T) -> Result<Self, JsValue> {
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = Some(serde_json::to_string(data).map_err(|e| JsValue::from_str(&e.to_string()))?);
        Ok(self)
    }
    
    pub fn bearer_token(mut self, token: impl Into<String>) -> Self {
        self.headers.insert("Authorization".to_string(), format!("Bearer {}", token.into()));
        self
    }
    
    pub fn credentials(mut self, creds: web_sys::RequestCredentials) -> Self {
        self.credentials = Some(creds);
        self
    }
    
    pub async fn send(self) -> Result<FetchResponse, FetchError> {
        let mut opts = RequestInit::new();
        opts.method(self.method.as_str());
        
        // Set headers
        let headers = web_sys::Headers::new()?;
        for (key, value) in self.headers {
            headers.set(&key, &value)?;
        }
        opts.headers(&headers.into());
        
        // Set body
        if let Some(body) = self.body {
            opts.body(Some(&JsValue::from_str(&body)));
        }
        
        // Set credentials
        if let Some(creds) = self.credentials {
            opts.credentials(creds);
        }
        
        // Create request
        let request = Request::new_with_str_and_init(&self.url, &opts)?;
        
        // Perform fetch
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let response_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response_value.dyn_into()?;
        
        Ok(FetchResponse::new(response))
    }
}

/// Fetch response wrapper
pub struct FetchResponse {
    response: Response,
}

impl FetchResponse {
    fn new(response: Response) -> Self {
        FetchResponse { response }
    }
    
    pub fn ok(&self) -> bool {
        self.response.ok()
    }
    
    pub fn status(&self) -> u16 {
        self.response.status()
    }
    
    pub fn status_text(&self) -> String {
        self.response.status_text()
    }
    
    pub async fn text(self) -> Result<String, FetchError> {
        let text = JsFuture::from(self.response.text()?)
            .await?
            .as_string()
            .ok_or_else(|| JsValue::from_str("Failed to get text"))?;
        Ok(text)
    }
    
    pub async fn json<T: for<'de> Deserialize<'de>>(self) -> Result<T, FetchError> {
        let json = JsFuture::from(self.response.json()?).await?;
        let data = serde_wasm_bindgen::from_value(json)?;
        Ok(data)
    }
    
    pub async fn blob(self) -> Result<web_sys::Blob, FetchError> {
        let blob = JsFuture::from(self.response.blob()?)
            .await?
            .dyn_into()?;
        Ok(blob)
    }
}

/// Fetch error type
#[derive(Debug)]
pub struct FetchError {
    message: String,
}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        FetchError {
            message: format!("{:?}", value),
        }
    }
}

impl From<serde_wasm_bindgen::Error> for FetchError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        FetchError {
            message: err.to_string(),
        }
    }
}

/// Convenience function for GET requests
pub async fn get(url: impl Into<String>) -> Result<FetchResponse, FetchError> {
    FetchBuilder::new(url).send().await
}

/// Convenience function for POST requests
pub async fn post<T: Serialize>(url: impl Into<String>, data: &T) -> Result<FetchResponse, FetchError> {
    FetchBuilder::new(url)
        .method(Method::POST)
        .json(data)?
        .send()
        .await
}

/// SWR-like data fetching hook
pub struct SWR<T> {
    data: Option<T>,
    error: Option<String>,
    loading: bool,
    mutate: Box<dyn Fn()>,
}

impl<T: Clone + for<'de> Deserialize<'de> + 'static> SWR<T> {
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        let data = use_state(|| None::<T>);
        let error = use_state(|| None::<String>);
        let loading = use_state(|| true);
        
        // Fetch on mount
        let url_clone = url.clone();
        let data_clone = data.clone();
        let error_clone = error.clone();
        let loading_clone = loading.clone();
        
        spawn_local(async move {
            loading_clone.set(true);
            match get(&url_clone).await {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<T>().await {
                            Ok(json_data) => {
                                data_clone.set(Some(json_data));
                                error_clone.set(None);
                            }
                            Err(e) => {
                                error_clone.set(Some(e.message));
                            }
                        }
                    } else {
                        error_clone.set(Some(format!("HTTP {}", response.status())));
                    }
                }
                Err(e) => {
                    error_clone.set(Some(e.message));
                }
            }
            loading_clone.set(false);
        });
        
        // Mutate function
        let mutate = {
            let url = url.clone();
            let data = data.clone();
            let error = error.clone();
            let loading = loading.clone();
            
            Box::new(move || {
                let url = url.clone();
                let data = data.clone();
                let error = error.clone();
                let loading = loading.clone();
                
                spawn_local(async move {
                    loading.set(true);
                    match get(&url).await {
                        Ok(response) => {
                            if response.ok() {
                                match response.json::<T>().await {
                                    Ok(json_data) => {
                                        data.set(Some(json_data));
                                        error.set(None);
                                    }
                                    Err(e) => {
                                        error.set(Some(e.message));
                                    }
                                }
                            } else {
                                error.set(Some(format!("HTTP {}", response.status())));
                            }
                        }
                        Err(e) => {
                            error.set(Some(e.message));
                        }
                    }
                    loading.set(false);
                });
            })
        };
        
        SWR {
            data: data.get(),
            error: error.get(),
            loading: loading.get(),
            mutate,
        }
    }
    
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }
    
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    
    pub fn is_loading(&self) -> bool {
        self.loading
    }
    
    pub fn mutate(&self) {
        (self.mutate)();
    }
}

// Re-exports
use crate::component::{use_state, State};
use wasm_bindgen_futures::spawn_local;