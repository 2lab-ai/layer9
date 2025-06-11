//! Server Functions - L4

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Server function trait
pub trait ServerFunction {
    type Request: Serialize;
    type Response: for<'de> Deserialize<'de>;

    fn endpoint(&self) -> &'static str;

    async fn call(&self, req: Self::Request) -> Result<Self::Response, ServerError>;
}

#[derive(Debug)]
pub struct ServerError {
    pub message: String,
    pub status: u16,
}

/// Response type
pub struct Response<T> {
    pub data: T,
    pub status: u16,
    pub headers: Vec<(String, String)>,
}

impl<T> Response<T> {
    pub fn ok(data: T) -> Self {
        Response {
            data,
            status: 200,
            headers: vec![],
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}

/// Call server function from client
pub async fn call_server<F: ServerFunction>(
    func: &F,
    request: F::Request,
) -> Result<F::Response, ServerError> {
    let endpoint = func.endpoint();
    let body = serde_json::to_string(&request).map_err(|e| ServerError {
        message: e.to_string(),
        status: 400,
    })?;

    // Use fetch API
    let window = web_sys::window().unwrap();

    let init = web_sys::RequestInit::new();
    init.set_method("POST");
    init.set_body(&JsValue::from_str(&body));
    let headers = web_sys::Headers::new().unwrap();
    headers.set("Content-Type", "application/json").unwrap();
    init.set_headers(&headers.into());

    let response = JsFuture::from(window.fetch_with_str_and_init(endpoint, &init))
        .await
        .map_err(|_| ServerError {
            message: "Fetch failed".to_string(),
            status: 500,
        })?;

    let response: web_sys::Response = response.dyn_into().unwrap();
    let json = JsFuture::from(response.json().unwrap())
        .await
        .map_err(|_| ServerError {
            message: "Failed to parse response".to_string(),
            status: 500,
        })?;

    serde_wasm_bindgen::from_value(json).map_err(|e| ServerError {
        message: e.to_string(),
        status: 500,
    })
}

/// Macro to define server functions
#[macro_export]
macro_rules! server_function {
    (
        async fn $name:ident($($arg:ident: $arg_ty:ty),*) -> Result<$ret:ty> {
            $($body:tt)*
        }
    ) => {
        pub async fn $name($($arg: $arg_ty),*) -> Result<$ret, $crate::server::ServerError> {
            $($body)*
        }
    };
}

// Re-export for wasm-bindgen
use wasm_bindgen_futures::JsFuture;
