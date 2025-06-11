//! API Documentation System - L5/L6
//! OpenAPI/Swagger and GraphQL schema documentation

use crate::prelude::*;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OpenAPI 3.0 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiSpec {
    pub openapi: String,
    pub info: ApiInfo,
    pub servers: Vec<ApiServer>,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
    pub security: Option<Vec<SecurityRequirement>>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub terms_of_service: Option<String>,
    pub contact: Option<Contact>,
    pub license: Option<License>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServer {
    pub url: String,
    pub description: Option<String>,
    pub variables: Option<HashMap<String, ServerVariable>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerVariable {
    pub default: String,
    pub description: Option<String>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub tags: Option<Vec<String>>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    pub security: Option<Vec<SecurityRequirement>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
    pub headers: Option<HashMap<String, Header>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
    pub example: Option<Value>,
    pub examples: Option<HashMap<String, Example>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub format: Option<String>,
    pub description: Option<String>,
    pub properties: Option<HashMap<String, Schema>>,
    pub required: Option<Vec<String>>,
    pub items: Option<Box<Schema>>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<Value>>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub pattern: Option<String>,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub description: Option<String>,
    pub schema: Schema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub schemas: Option<HashMap<String, Schema>>,
    pub responses: Option<HashMap<String, Response>>,
    pub parameters: Option<HashMap<String, Parameter>>,
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub description: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "in")]
    pub location: Option<String>,
    pub scheme: Option<String>,
    pub bearer_format: Option<String>,
    pub flows: Option<OAuthFlows>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlows {
    pub implicit: Option<OAuthFlow>,
    pub password: Option<OAuthFlow>,
    pub client_credentials: Option<OAuthFlow>,
    pub authorization_code: Option<OAuthFlow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlow {
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub refresh_url: Option<String>,
    pub scopes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    #[serde(flatten)]
    pub requirements: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

/// OpenAPI builder
pub struct OpenApiBuilder {
    spec: OpenApiSpec,
}

impl OpenApiBuilder {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        OpenApiBuilder {
            spec: OpenApiSpec {
                openapi: "3.0.0".to_string(),
                info: ApiInfo {
                    title: title.into(),
                    version: version.into(),
                    description: None,
                    terms_of_service: None,
                    contact: None,
                    license: None,
                },
                servers: vec![],
                paths: HashMap::new(),
                components: None,
                security: None,
                tags: None,
            },
        }
    }
    
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.spec.info.description = Some(description.into());
        self
    }
    
    pub fn server(mut self, url: impl Into<String>, description: Option<String>) -> Self {
        self.spec.servers.push(ApiServer {
            url: url.into(),
            description,
            variables: None,
        });
        self
    }
    
    pub fn tag(mut self, name: impl Into<String>, description: Option<String>) -> Self {
        let tags = self.spec.tags.get_or_insert_with(Vec::new);
        tags.push(Tag {
            name: name.into(),
            description,
        });
        self
    }
    
    pub fn path(mut self, path: impl Into<String>, item: PathItem) -> Self {
        self.spec.paths.insert(path.into(), item);
        self
    }
    
    pub fn security_scheme(mut self, name: impl Into<String>, scheme: SecurityScheme) -> Self {
        let components = self.spec.components.get_or_insert_with(|| Components {
            schemas: None,
            responses: None,
            parameters: None,
            security_schemes: None,
        });
        
        let schemes = components.security_schemes.get_or_insert_with(HashMap::new);
        schemes.insert(name.into(), scheme);
        self
    }
    
    pub fn build(self) -> OpenApiSpec {
        self.spec
    }
}

/// Route documentation decorator
pub struct ApiDoc {
    operation: Operation,
}

impl ApiDoc {
    pub fn new() -> Self {
        ApiDoc {
            operation: Operation {
                tags: None,
                summary: None,
                description: None,
                operation_id: None,
                parameters: None,
                request_body: None,
                responses: HashMap::new(),
                security: None,
            },
        }
    }
    
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.operation.summary = Some(summary.into());
        self
    }
    
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.operation.description = Some(description.into());
        self
    }
    
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        let tags = self.operation.tags.get_or_insert_with(Vec::new);
        tags.push(tag.into());
        self
    }
    
    pub fn param(mut self, name: impl Into<String>, location: &str, schema: Schema, required: bool) -> Self {
        let params = self.operation.parameters.get_or_insert_with(Vec::new);
        params.push(Parameter {
            name: name.into(),
            location: location.to_string(),
            description: schema.description.clone(),
            required: Some(required),
            schema,
        });
        self
    }
    
    pub fn body(mut self, content_type: &str, schema: Schema, required: bool) -> Self {
        self.operation.request_body = Some(RequestBody {
            description: schema.description.clone(),
            content: {
                let mut content = HashMap::new();
                content.insert(content_type.to_string(), MediaType {
                    schema,
                    example: None,
                    examples: None,
                });
                content
            },
            required: Some(required),
        });
        self
    }
    
    pub fn response(mut self, status: &str, description: impl Into<String>, schema: Option<Schema>) -> Self {
        let content = schema.map(|s| {
            let mut content = HashMap::new();
            content.insert("application/json".to_string(), MediaType {
                schema: s,
                example: None,
                examples: None,
            });
            content
        });
        
        self.operation.responses.insert(status.to_string(), Response {
            description: description.into(),
            content,
            headers: None,
        });
        self
    }
    
    pub fn build(self) -> Operation {
        self.operation
    }
}

/// Schema builder helpers
pub struct SchemaBuilder;

impl SchemaBuilder {
    pub fn string() -> Schema {
        Schema {
            schema_type: Some("string".to_string()),
            format: None,
            description: None,
            properties: None,
            required: None,
            items: None,
            enum_values: None,
            minimum: None,
            maximum: None,
            pattern: None,
            reference: None,
        }
    }
    
    pub fn integer() -> Schema {
        Schema {
            schema_type: Some("integer".to_string()),
            format: Some("int32".to_string()),
            ..Self::string()
        }
    }
    
    pub fn number() -> Schema {
        Schema {
            schema_type: Some("number".to_string()),
            format: Some("double".to_string()),
            ..Self::string()
        }
    }
    
    pub fn boolean() -> Schema {
        Schema {
            schema_type: Some("boolean".to_string()),
            ..Self::string()
        }
    }
    
    pub fn array(items: Schema) -> Schema {
        Schema {
            schema_type: Some("array".to_string()),
            items: Some(Box::new(items)),
            ..Self::string()
        }
    }
    
    pub fn object(properties: HashMap<String, Schema>, required: Vec<String>) -> Schema {
        Schema {
            schema_type: Some("object".to_string()),
            properties: Some(properties),
            required: Some(required),
            ..Self::string()
        }
    }
    
    pub fn reference(path: impl Into<String>) -> Schema {
        Schema {
            reference: Some(path.into()),
            ..Self::string()
        }
    }
}

/// GraphQL schema documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLSchema {
    pub types: HashMap<String, GraphQLType>,
    pub query: String,
    pub mutation: Option<String>,
    pub subscription: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLType {
    pub kind: GraphQLTypeKind,
    pub description: Option<String>,
    pub fields: Option<HashMap<String, GraphQLField>>,
    pub values: Option<Vec<GraphQLEnumValue>>,
    pub interfaces: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphQLTypeKind {
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
    Scalar,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLField {
    pub field_type: String,
    pub description: Option<String>,
    pub args: Option<HashMap<String, GraphQLArgument>>,
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLArgument {
    pub arg_type: String,
    pub description: Option<String>,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLEnumValue {
    pub name: String,
    pub description: Option<String>,
    pub deprecation_reason: Option<String>,
}

/// Documentation viewer component
pub struct ApiDocsViewer {
    spec: OpenApiSpec,
}

impl ApiDocsViewer {
    pub fn new(spec: OpenApiSpec) -> Self {
        ApiDocsViewer { spec }
    }
}

impl Component for ApiDocsViewer {
    fn render(&self) -> Element {
        // Render Swagger UI or ReDoc style documentation
        view! {
            <div class="api-docs">
                <h1>{&self.spec.info.title}</h1>
                <p>{"Version: "}{&self.spec.info.version}</p>
                {if let Some(desc) = &self.spec.info.description {
                    view! { <p>{desc}</p> }
                } else {
                    view! { <div /> }
                }}
                
                <h2>{"Endpoints"}</h2>
                {self.spec.paths.iter().map(|(path, item)| {
                    view! {
                        <div class="endpoint">
                            <h3>{path}</h3>
                            {render_path_item(item)}
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        }
    }
}

fn render_path_item(item: &PathItem) -> Element {
    view! {
        <div class="path-methods">
            {item.get.as_ref().map(|op| render_operation("GET", op)).unwrap_or_else(|| view! { <div /> })}
            {item.post.as_ref().map(|op| render_operation("POST", op)).unwrap_or_else(|| view! { <div /> })}
            {item.put.as_ref().map(|op| render_operation("PUT", op)).unwrap_or_else(|| view! { <div /> })}
            {item.delete.as_ref().map(|op| render_operation("DELETE", op)).unwrap_or_else(|| view! { <div /> })}
        </div>
    }
}

fn render_operation(method: &str, op: &Operation) -> Element {
    view! {
        <div class="operation">
            <span class="method">{method}</span>
            {op.summary.as_ref().map(|s| view! { <span class="summary">{s}</span> }).unwrap_or_else(|| view! { <span /> })}
            {op.description.as_ref().map(|d| view! { <p class="description">{d}</p> }).unwrap_or_else(|| view! { <div /> })}
        </div>
    }
}

/// Export documentation
pub fn export_openapi_json(spec: &OpenApiSpec) -> String {
    serde_json::to_string_pretty(spec).unwrap()
}

pub fn export_openapi_yaml(spec: &OpenApiSpec) -> String {
    // In real implementation, use serde_yaml
    serde_json::to_string_pretty(spec).unwrap()
}

// Re-exports
use crate::component::{Component, Element};
use crate::view;