//! Monitoring and Observability - L3/L4
//! Performance monitoring, error tracking, and distributed tracing

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::Performance;

/// Metrics types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Count(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Duration(f64),
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub name: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub tags: HashMap<String, String>,
    pub timestamp: f64,
}

/// Metrics collector
pub struct MetricsCollector {
    metrics: Rc<RefCell<Vec<MetricPoint>>>,
    exporters: Vec<Box<dyn MetricsExporter>>,
    buffer_size: usize,
}

impl MetricsCollector {
    pub fn new(buffer_size: usize) -> Self {
        MetricsCollector {
            metrics: Rc::new(RefCell::new(Vec::with_capacity(buffer_size))),
            exporters: vec![],
            buffer_size,
        }
    }

    pub fn add_exporter(mut self, exporter: impl MetricsExporter + 'static) -> Self {
        self.exporters.push(Box::new(exporter));
        self
    }

    pub fn record(&self, metric: MetricPoint) {
        let mut metrics = self.metrics.borrow_mut();
        metrics.push(metric);

        // Export if buffer is full
        if metrics.len() >= self.buffer_size {
            let batch = metrics.drain(..).collect::<Vec<_>>();
            drop(metrics);

            for exporter in &self.exporters {
                exporter.export(batch.clone());
            }
        }
    }

    pub fn flush(&self) {
        let mut metrics = self.metrics.borrow_mut();
        if !metrics.is_empty() {
            let batch = metrics.drain(..).collect::<Vec<_>>();
            drop(metrics);

            for exporter in &self.exporters {
                exporter.export(batch.clone());
            }
        }
    }

    pub fn counter(&self, name: impl Into<String>, value: u64, tags: HashMap<String, String>) {
        self.record(MetricPoint {
            name: name.into(),
            metric_type: MetricType::Counter,
            value: MetricValue::Count(value),
            tags,
            timestamp: js_sys::Date::now(),
        });
    }

    pub fn gauge(&self, name: impl Into<String>, value: f64, tags: HashMap<String, String>) {
        self.record(MetricPoint {
            name: name.into(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(value),
            tags,
            timestamp: js_sys::Date::now(),
        });
    }

    pub fn timer(&self, name: impl Into<String>, duration: f64, tags: HashMap<String, String>) {
        self.record(MetricPoint {
            name: name.into(),
            metric_type: MetricType::Timer,
            value: MetricValue::Duration(duration),
            tags,
            timestamp: js_sys::Date::now(),
        });
    }
}

/// Metrics exporter trait
pub trait MetricsExporter: 'static {
    fn export(&self, metrics: Vec<MetricPoint>);
}

/// Console metrics exporter
pub struct ConsoleExporter;

impl MetricsExporter for ConsoleExporter {
    fn export(&self, metrics: Vec<MetricPoint>) {
        for metric in metrics {
            web_sys::console::log_1(&format!("[METRIC] {:?}", metric).into());
        }
    }
}

/// HTTP metrics exporter
pub struct HttpExporter {
    endpoint: String,
    api_key: Option<String>,
}

impl HttpExporter {
    pub fn new(endpoint: impl Into<String>) -> Self {
        HttpExporter {
            endpoint: endpoint.into(),
            api_key: None,
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
}

impl MetricsExporter for HttpExporter {
    fn export(&self, metrics: Vec<MetricPoint>) {
        let endpoint = self.endpoint.clone();
        let api_key = self.api_key.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::to_string(&metrics).unwrap();
            let mut headers = HashMap::new();
            headers.insert("Content-Type".to_string(), "application/json".to_string());

            if let Some(key) = api_key {
                headers.insert("X-API-Key".to_string(), key);
            }

            let mut fetch_builder =
                crate::fetch::FetchBuilder::new(&endpoint).method(crate::fetch::Method::POST);

            for (key, value) in headers {
                fetch_builder = fetch_builder.header(key, value);
            }

            if let Ok(builder) =
                fetch_builder.json(&serde_json::from_str::<serde_json::Value>(&body).unwrap())
            {
                let _ = builder.send().await;
            }
        });
    }
}

/// Performance monitoring
pub struct PerformanceMonitor {
    entries: Rc<RefCell<HashMap<String, PerformanceEntry>>>,
    collector: Rc<MetricsCollector>,
}

#[derive(Clone)]
struct PerformanceEntry {
    start_time: f64,
    marks: HashMap<String, f64>,
}

impl PerformanceMonitor {
    pub fn new(collector: Rc<MetricsCollector>) -> Self {
        PerformanceMonitor {
            entries: Rc::new(RefCell::new(HashMap::new())),
            collector,
        }
    }

    pub fn start_timing(&self, name: impl Into<String>) -> TimingHandle {
        let name = name.into();
        let start_time = self.now();

        self.entries.borrow_mut().insert(
            name.clone(),
            PerformanceEntry {
                start_time,
                marks: HashMap::new(),
            },
        );

        TimingHandle {
            name,
            monitor: self.clone(),
        }
    }

    pub fn mark(&self, timing_name: &str, mark_name: impl Into<String>) {
        if let Some(entry) = self.entries.borrow_mut().get_mut(timing_name) {
            entry.marks.insert(mark_name.into(), self.now());
        }
    }

    pub fn end_timing(&self, name: &str, tags: HashMap<String, String>) {
        if let Some(entry) = self.entries.borrow_mut().remove(name) {
            let duration = self.now() - entry.start_time;
            self.collector.timer(name, duration, tags);

            // Also record marks as separate metrics
            for (mark_name, mark_time) in entry.marks {
                let mark_duration = mark_time - entry.start_time;
                self.collector.timer(
                    format!("{}.{}", name, mark_name),
                    mark_duration,
                    HashMap::new(),
                );
            }
        }
    }

    fn now(&self) -> f64 {
        Performance::now(&web_sys::window().unwrap().performance().unwrap())
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        PerformanceMonitor {
            entries: self.entries.clone(),
            collector: self.collector.clone(),
        }
    }
}

/// Timing handle for automatic timing
pub struct TimingHandle {
    name: String,
    monitor: PerformanceMonitor,
}

impl TimingHandle {
    pub fn mark(&self, mark_name: impl Into<String>) {
        self.monitor.mark(&self.name, mark_name);
    }
}

impl Drop for TimingHandle {
    fn drop(&mut self) {
        self.monitor.end_timing(&self.name, HashMap::new());
    }
}

/// Error tracking
#[derive(Clone)]
pub struct ErrorTracker {
    collector: Rc<MetricsCollector>,
    context: Rc<RefCell<HashMap<String, String>>>,
}

impl ErrorTracker {
    pub fn new(collector: Rc<MetricsCollector>) -> Self {
        ErrorTracker {
            collector,
            context: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn set_context(&self, key: impl Into<String>, value: impl Into<String>) {
        self.context.borrow_mut().insert(key.into(), value.into());
    }

    pub fn track_error(&self, error: &JsValue) {
        let mut tags = self.context.borrow().clone();

        // Extract error details
        if let Some(error_obj) = error.dyn_ref::<js_sys::Error>() {
            tags.insert("message".to_string(), error_obj.message().into());
            tags.insert("name".to_string(), error_obj.name().into());

            // Stack trace is not available on js_sys::Error in wasm-bindgen
            // We could use Reflect::get to access it if needed
        } else {
            tags.insert("message".to_string(), format!("{:?}", error));
        }

        self.collector.counter("error", 1, tags);
    }

    pub fn track_panic(&self, info: &std::panic::PanicHookInfo) {
        let mut tags = self.context.borrow().clone();

        tags.insert("type".to_string(), "panic".to_string());
        tags.insert("message".to_string(), info.to_string());

        if let Some(location) = info.location() {
            tags.insert("file".to_string(), location.file().to_string());
            tags.insert("line".to_string(), location.line().to_string());
            tags.insert("column".to_string(), location.column().to_string());
        }

        self.collector.counter("panic", 1, tags);
    }
}

/// Distributed tracing
pub struct TraceSpan {
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
    name: String,
    start_time: f64,
    attributes: HashMap<String, String>,
    events: Vec<SpanEvent>,
    collector: Rc<MetricsCollector>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SpanEvent {
    name: String,
    timestamp: f64,
    attributes: HashMap<String, String>,
}

impl TraceSpan {
    pub fn new(name: impl Into<String>, collector: Rc<MetricsCollector>) -> Self {
        TraceSpan {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            parent_span_id: None,
            name: name.into(),
            start_time: js_sys::Date::now(),
            attributes: HashMap::new(),
            events: vec![],
            collector,
        }
    }

    pub fn child(&self, name: impl Into<String>) -> Self {
        TraceSpan {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            name: name.into(),
            start_time: js_sys::Date::now(),
            attributes: HashMap::new(),
            events: vec![],
            collector: self.collector.clone(),
        }
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn add_event(&mut self, name: impl Into<String>, attributes: HashMap<String, String>) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: js_sys::Date::now(),
            attributes,
        });
    }

    pub fn end(self) {
        let duration = js_sys::Date::now() - self.start_time;

        let mut tags = self.attributes;
        tags.insert("trace_id".to_string(), self.trace_id);
        tags.insert("span_id".to_string(), self.span_id);

        if let Some(parent) = self.parent_span_id {
            tags.insert("parent_span_id".to_string(), parent);
        }

        self.collector
            .timer(format!("trace.{}", self.name), duration, tags);

        // Record events
        for event in self.events {
            self.collector.counter(
                format!("trace.{}.event.{}", self.name, event.name),
                1,
                event.attributes,
            );
        }
    }
}

/// Generate trace ID
fn generate_trace_id() -> String {
    format!("{:032x}", (js_sys::Math::random() * 1e16) as u64)
}

/// Generate span ID
fn generate_span_id() -> String {
    format!("{:016x}", (js_sys::Math::random() * 1e8) as u64)
}

/// User analytics
#[derive(Clone)]
pub struct Analytics {
    collector: Rc<MetricsCollector>,
    user_id: Option<String>,
    session_id: String,
}

impl Analytics {
    pub fn new(collector: Rc<MetricsCollector>) -> Self {
        Analytics {
            collector,
            user_id: None,
            session_id: generate_session_id(),
        }
    }

    pub fn identify(&mut self, user_id: impl Into<String>) {
        self.user_id = Some(user_id.into());
    }

    pub fn track_event(&self, event_name: impl Into<String>, properties: HashMap<String, String>) {
        let mut tags = properties;
        tags.insert("session_id".to_string(), self.session_id.clone());

        if let Some(user_id) = &self.user_id {
            tags.insert("user_id".to_string(), user_id.clone());
        }

        self.collector
            .counter(format!("event.{}", event_name.into()), 1, tags);
    }

    pub fn track_page_view(&self, path: &str, title: Option<&str>) {
        let mut properties = HashMap::new();
        properties.insert("path".to_string(), path.to_string());

        if let Some(title) = title {
            properties.insert("title".to_string(), title.to_string());
        }

        properties.insert(
            "referrer".to_string(),
            web_sys::window()
                .and_then(|w| w.document())
                .map(|d| d.referrer())
                .unwrap_or_default(),
        );

        self.track_event("page_view", properties);
    }

    pub fn track_click(&self, element_id: &str, element_text: Option<&str>) {
        let mut properties = HashMap::new();
        properties.insert("element_id".to_string(), element_id.to_string());

        if let Some(text) = element_text {
            properties.insert("element_text".to_string(), text.to_string());
        }

        self.track_event("click", properties);
    }
}

fn generate_session_id() -> String {
    format!("session_{}", js_sys::Date::now())
}

thread_local! {
    static MONITORING: RefCell<Option<MonitoringSystem>> = RefCell::new(None);
}

pub struct MonitoringSystem {
    pub metrics: Rc<MetricsCollector>,
    pub performance: PerformanceMonitor,
    pub errors: ErrorTracker,
    pub analytics: Analytics,
}

/// Initialize monitoring
pub fn init_monitoring() {
    let metrics = Rc::new(
        MetricsCollector::new(100)
            .add_exporter(ConsoleExporter)
            .add_exporter(HttpExporter::new("/api/metrics")),
    );

    let performance = PerformanceMonitor::new(metrics.clone());
    let errors = ErrorTracker::new(metrics.clone());
    let analytics = Analytics::new(metrics.clone());

    let system = MonitoringSystem {
        metrics,
        performance,
        errors,
        analytics,
    };

    MONITORING.with(|monitoring| {
        *monitoring.borrow_mut() = Some(system);
    });

    // Set up global error handler
    std::panic::set_hook(Box::new(|info| {
        MONITORING.with(|monitoring| {
            if let Some(m) = monitoring.borrow().as_ref() {
                m.errors.track_panic(info);
            }
        });
    }));

    // Set up periodic flush
    let closure = Closure::<dyn Fn()>::new(|| {
        MONITORING.with(|monitoring| {
            if let Some(m) = monitoring.borrow().as_ref() {
                m.metrics.flush();
            }
        });
    });

    web_sys::window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            10000, // Flush every 10 seconds
        )
        .unwrap();

    closure.forget();
}

/// Monitoring hooks
pub fn use_metrics() -> Rc<MetricsCollector> {
    MONITORING.with(|monitoring| {
        monitoring
            .borrow()
            .as_ref()
            .map(|m| m.metrics.clone())
            .expect("Monitoring not initialized")
    })
}

pub fn use_performance() -> PerformanceMonitor {
    MONITORING.with(|monitoring| {
        monitoring
            .borrow()
            .as_ref()
            .map(|m| m.performance.clone())
            .expect("Monitoring not initialized")
    })
}

pub fn use_error_tracker() -> ErrorTracker {
    MONITORING.with(|monitoring| {
        monitoring
            .borrow()
            .as_ref()
            .map(|m| m.errors.clone())
            .expect("Monitoring not initialized")
    })
}

pub fn use_analytics() -> Analytics {
    MONITORING.with(|monitoring| {
        monitoring
            .borrow()
            .as_ref()
            .map(|m| m.analytics.clone())
            .expect("Monitoring not initialized")
    })
}

// Re-exports
use wasm_bindgen::closure::Closure;
