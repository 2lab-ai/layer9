//! Server Functions - L4

use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use js_sys::Promise;

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
    let body = serde_json::to_string(&request)
        .map_err(|e| ServerError {
            message: e.to_string(),
            status: 400,
        })?;
    
    // Use fetch API
    let window = web_sys::window().unwrap();
    let response = JsFuture::from(
        window.fetch_with_str_and_init(
            endpoint,
            web_sys::RequestInit::new()
                .method("POST")
                .body(&JsValue::from_str(&body))
                .headers(&{
                    let headers = web_sys::Headers::new().unwrap();
                    headers.set("Content-Type", "application/json").unwrap();
                    headers
                }.into()),
        )
    ).await
    .map_err(|_| ServerError {
        message: "Fetch failed".to_string(),
        status: 500,
    })?;
    
    let response: web_sys::Response = response.dyn_into().unwrap();
    let json = JsFuture::from(response.json().unwrap()).await
        .map_err(|_| ServerError {
            message: "Failed to parse response".to_string(),
            status: 500,
        })?;
    
    serde_wasm_bindgen::from_value(json)
        .map_err(|e| ServerError {
            message: e.to_string(),
            status: 500,
        })
}

/// Macro to define server functions
#[macro_export]
macro_rules! server_function {
    (
        fn $name:ident($($arg:ident: $arg_ty:ty),*) -> $ret:ty {
            $($body:tt)*
        }
    ) => {
        #[wasm_bindgen]
        pub struct $name;
        
        #[derive(Serialize, Deserialize)]
        pub struct [<$name Request>] {
            $($arg: $arg_ty,)*
        }
        
        impl ServerFunction for $name {
            type Request = [<$name Request>];
            type Response = $ret;
            
            fn endpoint(&self) -> &'static str {
                concat!("/api/", stringify!($name))
            }
            
            async fn call(&self, req: Self::Request) -> Result<Self::Response, ServerError> {
                let [<$name Request>] { $($arg,)* } = req;
                $($body)*
            }
        }
    };
}

// Re-export for wasm-bindgen
use wasm_bindgen_futures::JsFuture;