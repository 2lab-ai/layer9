//! WebSocket Support - L4

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

/// WebSocket connection state
#[derive(Clone, PartialEq)]
pub enum WsState {
    Connecting,
    Connected,
    Disconnected,
    Error(String),
}

/// WebSocket message
#[derive(Clone)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
}

/// WebSocket connection
pub struct WsConnection {
    ws: Rc<RefCell<Option<WebSocket>>>,
    state: Rc<RefCell<WsState>>,
    config: WsConfig,
    handlers: Rc<RefCell<WsHandlers>>,
    reconnect_timer: Rc<RefCell<Option<i32>>>,
    reconnect_attempts: Rc<RefCell<u32>>,
}

pub struct WsConfig {
    pub url: String,
    pub protocols: Vec<String>,
    pub reconnect: bool,
    pub reconnect_interval: u32,
    pub max_reconnect_attempts: u32,
    pub ping_interval: Option<u32>,
}

struct WsHandlers {
    on_open: Option<Box<dyn Fn()>>,
    on_message: Option<Box<dyn Fn(WsMessage)>>,
    on_error: Option<Box<dyn Fn(String)>>,
    on_close: Option<Box<dyn Fn(u16, String)>>,
}

impl WsConnection {
    pub fn new(config: WsConfig) -> Result<Self, JsValue> {
        let state = Rc::new(RefCell::new(WsState::Connecting));
        let handlers = Rc::new(RefCell::new(WsHandlers {
            on_open: None,
            on_message: None,
            on_error: None,
            on_close: None,
        }));

        let mut conn = WsConnection {
            ws: Rc::new(RefCell::new(None)),
            state: state.clone(),
            config,
            handlers: handlers.clone(),
            reconnect_timer: Rc::new(RefCell::new(None)),
            reconnect_attempts: Rc::new(RefCell::new(0)),
        };

        conn.connect()?;
        Ok(conn)
    }

    fn connect(&mut self) -> Result<(), JsValue> {
        // Create WebSocket
        let ws = if self.config.protocols.is_empty() {
            WebSocket::new(&self.config.url)?
        } else {
            let protocols = js_sys::Array::new();
            for protocol in &self.config.protocols {
                protocols.push(&JsValue::from_str(protocol));
            }
            WebSocket::new_with_str_sequence(&self.config.url, &protocols)?
        };

        // Set binary type
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // Set up event handlers
        self.setup_handlers(&ws)?;

        *self.ws.borrow_mut() = Some(ws);
        Ok(())
    }

    fn setup_handlers(&self, ws: &WebSocket) -> Result<(), JsValue> {
        let state = self.state.clone();
        let handlers = self.handlers.clone();
        let config = self.config.clone();
        let reconnect_timer = self.reconnect_timer.clone();
        let reconnect_attempts = self.reconnect_attempts.clone();
        let ws_ref = self.ws.clone();

        // On open
        let on_open = {
            let state = state.clone();
            let handlers = handlers.clone();
            let reconnect_attempts_clone = reconnect_attempts.clone();

            Closure::<dyn FnMut()>::new(move || {
                *state.borrow_mut() = WsState::Connected;
                // Reset reconnection attempts on successful connection
                *reconnect_attempts_clone.borrow_mut() = 0;

                if let Some(handler) = &handlers.borrow().on_open {
                    handler();
                }

                // Start ping interval if configured
                // TODO: Implement ping/pong
            })
        };
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        // On message
        let on_message = {
            let handlers = handlers.clone();

            Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Some(handler) = &handlers.borrow().on_message {
                    if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                        handler(WsMessage::Text(text.into()));
                    } else if let Ok(array_buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                        let array = js_sys::Uint8Array::new(&array_buffer);
                        let vec = array.to_vec();
                        handler(WsMessage::Binary(vec));
                    }
                }
            })
        };
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        // On error
        let on_error = {
            let state = state.clone();
            let handlers = handlers.clone();

            Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
                let error_msg = format!("WebSocket error: {}", e.message());
                *state.borrow_mut() = WsState::Error(error_msg.clone());

                if let Some(handler) = &handlers.borrow().on_error {
                    handler(error_msg);
                }
            })
        };
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        // On close
        let on_close = {
            let state = state.clone();
            let handlers = handlers.clone();

            Closure::<dyn FnMut(_)>::new(move |e: CloseEvent| {
                *state.borrow_mut() = WsState::Disconnected;

                if let Some(handler) = &handlers.borrow().on_close {
                    handler(e.code(), e.reason());
                }

                // Handle reconnection
                if config.reconnect && e.code() != 1000 {
                    Self::schedule_reconnection(
                        ws_ref.clone(),
                        state.clone(),
                        handlers.clone(),
                        config.clone(),
                        reconnect_timer.clone(),
                        reconnect_attempts.clone(),
                    );
                }
            })
        };
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        on_close.forget();

        Ok(())
    }

    fn schedule_reconnection(
        ws_ref: Rc<RefCell<Option<WebSocket>>>,
        state: Rc<RefCell<WsState>>,
        handlers: Rc<RefCell<WsHandlers>>,
        config: WsConfig,
        reconnect_timer: Rc<RefCell<Option<i32>>>,
        reconnect_attempts: Rc<RefCell<u32>>,
    ) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        // Clear any existing reconnection timer
        if let Some(timer_id) = reconnect_timer.borrow_mut().take() {
            window.clear_timeout_with_handle(timer_id);
        }

        let current_attempts = *reconnect_attempts.borrow();
        if current_attempts >= config.max_reconnect_attempts {
            // Max attempts reached, stop trying
            return;
        }

        // Calculate delay with exponential backoff
        let base_delay = config.reconnect_interval;
        let delay = base_delay * 2u32.pow(current_attempts.min(10)); // Cap exponential growth at 2^10

        let reconnect_timer_clone = reconnect_timer.clone();
        let closure = Closure::<dyn FnMut()>::new(move || {
            // Increment attempts
            *reconnect_attempts.borrow_mut() += 1;

            // Update state to connecting
            *state.borrow_mut() = WsState::Connecting;

            // Create new WebSocket
            let new_ws = if config.protocols.is_empty() {
                match WebSocket::new(&config.url) {
                    Ok(ws) => ws,
                    Err(_) => {
                        *state.borrow_mut() = WsState::Error("Failed to create WebSocket".to_string());
                        // Schedule another retry
                        Self::schedule_reconnection(
                            ws_ref.clone(),
                            state.clone(),
                            handlers.clone(),
                            config.clone(),
                            reconnect_timer_clone.clone(),
                            reconnect_attempts.clone(),
                        );
                        return;
                    }
                }
            } else {
                let protocols = js_sys::Array::new();
                for protocol in &config.protocols {
                    protocols.push(&JsValue::from_str(protocol));
                }
                match WebSocket::new_with_str_sequence(&config.url, &protocols) {
                    Ok(ws) => ws,
                    Err(_) => {
                        *state.borrow_mut() = WsState::Error("Failed to create WebSocket".to_string());
                        // Schedule another retry
                        Self::schedule_reconnection(
                            ws_ref.clone(),
                            state.clone(),
                            handlers.clone(),
                            config.clone(),
                            reconnect_timer_clone.clone(),
                            reconnect_attempts.clone(),
                        );
                        return;
                    }
                }
            };

            // Set binary type
            new_ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

            // Setup handlers for the new WebSocket
            Self::setup_handlers_for_reconnection(
                &new_ws,
                ws_ref.clone(),
                state.clone(),
                handlers.clone(),
                config.clone(),
                reconnect_timer_clone.clone(),
                reconnect_attempts.clone(),
            );

            // Store the new WebSocket
            *ws_ref.borrow_mut() = Some(new_ws);
        });

        match window.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            delay as i32,
        ) {
            Ok(handle) => {
                *reconnect_timer.borrow_mut() = Some(handle);
            }
            Err(_) => {
                // Failed to set timer
            }
        }

        closure.forget();
    }

    fn setup_handlers_for_reconnection(
        ws: &WebSocket,
        ws_ref: Rc<RefCell<Option<WebSocket>>>,
        state: Rc<RefCell<WsState>>,
        handlers: Rc<RefCell<WsHandlers>>,
        config: WsConfig,
        reconnect_timer: Rc<RefCell<Option<i32>>>,
        reconnect_attempts: Rc<RefCell<u32>>,
    ) {
        // On open
        let on_open = {
            let state = state.clone();
            let handlers = handlers.clone();
            let reconnect_attempts_clone = reconnect_attempts.clone();

            Closure::<dyn FnMut()>::new(move || {
                *state.borrow_mut() = WsState::Connected;
                // Reset reconnection attempts on successful connection
                *reconnect_attempts_clone.borrow_mut() = 0;

                if let Some(handler) = &handlers.borrow().on_open {
                    handler();
                }
            })
        };
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        // On message
        let on_message = {
            let handlers = handlers.clone();

            Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Some(handler) = &handlers.borrow().on_message {
                    if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                        handler(WsMessage::Text(text.into()));
                    } else if let Ok(array_buffer) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                        let array = js_sys::Uint8Array::new(&array_buffer);
                        let vec = array.to_vec();
                        handler(WsMessage::Binary(vec));
                    }
                }
            })
        };
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        // On error
        let on_error = {
            let state = state.clone();
            let handlers = handlers.clone();

            Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
                let error_msg = format!("WebSocket error: {}", e.message());
                *state.borrow_mut() = WsState::Error(error_msg.clone());

                if let Some(handler) = &handlers.borrow().on_error {
                    handler(error_msg);
                }
            })
        };
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        on_error.forget();

        // On close
        let on_close = {
            let state = state.clone();
            let handlers = handlers.clone();
            let ws_ref = ws_ref.clone();
            let config = config.clone();
            let reconnect_timer = reconnect_timer.clone();
            let reconnect_attempts = reconnect_attempts.clone();

            Closure::<dyn FnMut(_)>::new(move |e: CloseEvent| {
                *state.borrow_mut() = WsState::Disconnected;

                if let Some(handler) = &handlers.borrow().on_close {
                    handler(e.code(), e.reason());
                }

                // Handle reconnection
                if config.reconnect && e.code() != 1000 {
                    Self::schedule_reconnection(
                        ws_ref.clone(),
                        state.clone(),
                        handlers.clone(),
                        config.clone(),
                        reconnect_timer.clone(),
                        reconnect_attempts.clone(),
                    );
                }
            })
        };
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        on_close.forget();
    }

    pub fn send(&self, message: WsMessage) -> Result<(), JsValue> {
        if let Some(ws) = &*self.ws.borrow() {
            match message {
                WsMessage::Text(text) => ws.send_with_str(&text),
                WsMessage::Binary(data) => {
                    let array = js_sys::Uint8Array::from(&data[..]);
                    ws.send_with_array_buffer(&array.buffer())
                }
            }
        } else {
            Err(JsValue::from_str("WebSocket not connected"))
        }
    }

    pub fn send_json<T: Serialize>(&self, data: &T) -> Result<(), JsValue> {
        let json = serde_json::to_string(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.send(WsMessage::Text(json))
    }

    pub fn close(&self) -> Result<(), JsValue> {
        // Clear any pending reconnection timer
        if let Some(timer_id) = self.reconnect_timer.borrow_mut().take() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(timer_id);
            }
        }
        
        // Close the WebSocket connection
        if let Some(ws) = &*self.ws.borrow() {
            ws.close()?;
        }
        Ok(())
    }

    pub fn state(&self) -> WsState {
        self.state.borrow().clone()
    }

    pub fn on_open(self, handler: impl Fn() + 'static) -> Self {
        self.handlers.borrow_mut().on_open = Some(Box::new(handler));
        self
    }

    pub fn on_message(self, handler: impl Fn(WsMessage) + 'static) -> Self {
        self.handlers.borrow_mut().on_message = Some(Box::new(handler));
        self
    }

    pub fn on_error(self, handler: impl Fn(String) + 'static) -> Self {
        self.handlers.borrow_mut().on_error = Some(Box::new(handler));
        self
    }

    pub fn on_close(self, handler: impl Fn(u16, String) + 'static) -> Self {
        self.handlers.borrow_mut().on_close = Some(Box::new(handler));
        self
    }
}

/// WebSocket hook
pub fn use_websocket(url: impl Into<String>) -> Result<WsHandle, JsValue> {
    let config = WsConfig {
        url: url.into(),
        protocols: vec![],
        reconnect: true,
        reconnect_interval: 3000,
        max_reconnect_attempts: 5,
        ping_interval: Some(30000),
    };

    let conn = WsConnection::new(config)?;

    Ok(WsHandle {
        conn: Rc::new(conn),
    })
}

/// WebSocket handle for components
pub struct WsHandle {
    conn: Rc<WsConnection>,
}

impl WsHandle {
    pub fn send(&self, message: WsMessage) -> Result<(), JsValue> {
        self.conn.send(message)
    }

    pub fn send_json<T: Serialize>(&self, data: &T) -> Result<(), JsValue> {
        self.conn.send_json(data)
    }

    pub fn state(&self) -> WsState {
        self.conn.state()
    }

    pub fn close(&self) -> Result<(), JsValue> {
        self.conn.close()
    }
}

/// Real-time data subscription
pub struct RealtimeSubscription<T> {
    topic: String,
    data: Rc<RefCell<Option<T>>>,
    ws: WsHandle,
}

impl<T: for<'de> Deserialize<'de> + Clone + 'static> RealtimeSubscription<T> {
    pub fn new(topic: impl Into<String>, ws: WsHandle) -> Self {
        let topic = topic.into();
        let data = Rc::new(RefCell::new(None));

        // Subscribe to topic
        let subscribe_msg = serde_json::json!({
            "type": "subscribe",
            "topic": &topic,
        });

        let _ = ws.send_json(&subscribe_msg);

        RealtimeSubscription { topic, data, ws }
    }

    pub fn data(&self) -> Option<T> {
        self.data.borrow().clone()
    }

    pub fn update(&self, new_data: T) {
        *self.data.borrow_mut() = Some(new_data);
        // Trigger re-render
    }
}

impl<T> Drop for RealtimeSubscription<T> {
    fn drop(&mut self) {
        // Unsubscribe from topic
        let unsubscribe_msg = serde_json::json!({
            "type": "unsubscribe",
            "topic": &self.topic,
        });

        let _ = self.ws.send_json(&unsubscribe_msg);
    }
}

/// Hook for real-time data
pub fn use_realtime<T: for<'de> Deserialize<'de> + Clone + 'static>(
    topic: impl Into<String>,
    ws: &WsHandle,
) -> Option<T> {
    let subscription = RealtimeSubscription::new(topic, ws.clone());
    subscription.data()
}

// Implement Clone for WsHandle
impl Clone for WsHandle {
    fn clone(&self) -> Self {
        WsHandle {
            conn: self.conn.clone(),
        }
    }
}

impl Clone for WsConfig {
    fn clone(&self) -> Self {
        WsConfig {
            url: self.url.clone(),
            protocols: self.protocols.clone(),
            reconnect: self.reconnect,
            reconnect_interval: self.reconnect_interval,
            max_reconnect_attempts: self.max_reconnect_attempts,
            ping_interval: self.ping_interval,
        }
    }
}
