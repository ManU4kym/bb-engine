use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub id: Uuid,
    pub url: String,
    pub method: String,
    pub discovered_at: DateTime<Utc>,
    pub source: EndpointSource,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndpointSource {
    Crawled,
    Wordlist,
    JsAnalysis,
    ApiSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub location: ParamLocation,
    pub param_type: ParamType,
    pub example_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamLocation {
    Query,
    Body,
    Header,
    Path,
    Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub id: Uuid,
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub id: Uuid,
    pub request_id: Uuid,
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub evidence: Evidence,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub request: HttpRequest,
    pub response: HttpResponse,
    pub matched_pattern: Option<String>,
    pub notes: Vec<String>,
}

impl Endpoint {
    pub fn new(url: String, method: String, source: EndpointSource) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            method,
            discovered_at: Utc::now(),
            source,
            parameters: Vec::new(),
        }
    }
}

impl Finding {
    pub fn new(
        endpoint_id: Uuid,
        severity: Severity,
        title: String,
        description: String,
        evidence: Evidence,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            endpoint_id,
            severity,
            title,
            description,
            evidence,
            discovered_at: Utc::now(),
        }
    }
}
