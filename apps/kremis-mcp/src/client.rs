//! # Kremis HTTP Client
//!
//! Wrapper around the Kremis REST API for use by the MCP server.

use serde_json::Value;

/// Errors from the HTTP client layer.
#[derive(Debug)]
pub enum ClientError {
    /// Cannot reach the Kremis server.
    ConnectionFailed(String),
    /// 401 Unauthorized - invalid or missing API key.
    Unauthorized,
    /// 429 Too Many Requests.
    RateLimited,
    /// Server returned a 5xx error.
    ServerError(u16, String),
    /// Failed to parse response body.
    ParseError(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed(url) => write!(f, "Cannot connect to Kremis at {url}"),
            Self::Unauthorized => write!(f, "Unauthorized: invalid or missing API key"),
            Self::RateLimited => write!(f, "Rate limited: too many requests"),
            Self::ServerError(status, msg) => write!(f, "Server error ({status}): {msg}"),
            Self::ParseError(msg) => write!(f, "Parse error: {msg}"),
        }
    }
}

impl std::error::Error for ClientError {}

/// HTTP client that wraps calls to the Kremis REST API.
#[derive(Clone)]
pub struct KremisClient {
    http: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

#[allow(dead_code)]
impl KremisClient {
    /// Create a new client pointing at the given Kremis server URL.
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    /// Build a request with optional Bearer auth.
    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method, &url);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }
        req
    }

    /// Handle HTTP response: check status codes and parse JSON.
    async fn handle_response(&self, resp: reqwest::Response) -> Result<Value, ClientError> {
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ClientError::Unauthorized);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ClientError::RateLimited);
        }
        if status.is_server_error() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ClientError::ServerError(status.as_u16(), body));
        }
        resp.json::<Value>()
            .await
            .map_err(|e| ClientError::ParseError(e.to_string()))
    }

    /// Send a request and handle connection errors.
    async fn send(&self, req: reqwest::RequestBuilder) -> Result<reqwest::Response, ClientError> {
        req.send()
            .await
            .map_err(|e| ClientError::ConnectionFailed(format!("{}: {e}", self.base_url)))
    }

    /// GET /health
    pub async fn health(&self) -> Result<Value, ClientError> {
        let req = self.request(reqwest::Method::GET, "/health");
        let resp = self.send(req).await?;
        self.handle_response(resp).await
    }

    /// GET /status → graph statistics.
    pub async fn status(&self) -> Result<Value, ClientError> {
        let req = self.request(reqwest::Method::GET, "/status");
        let resp = self.send(req).await?;
        self.handle_response(resp).await
    }

    /// POST /signal → ingest a signal.
    pub async fn ingest(
        &self,
        entity_id: u64,
        attribute: &str,
        value: &str,
    ) -> Result<Value, ClientError> {
        let body = serde_json::json!({
            "entity_id": entity_id,
            "attribute": attribute,
            "value": value,
        });
        let req = self.request(reqwest::Method::POST, "/signal").json(&body);
        let resp = self.send(req).await?;
        self.handle_response(resp).await
    }

    /// POST /query → execute a graph query (generic JSON body).
    pub async fn query(&self, request: Value) -> Result<Value, ClientError> {
        let req = self.request(reqwest::Method::POST, "/query").json(&request);
        let resp = self.send(req).await?;
        self.handle_response(resp).await
    }

    /// POST /export → export graph in canonical format.
    pub async fn export(&self) -> Result<Value, ClientError> {
        let req = self.request(reqwest::Method::POST, "/export");
        let resp = self.send(req).await?;
        self.handle_response(resp).await
    }
}
