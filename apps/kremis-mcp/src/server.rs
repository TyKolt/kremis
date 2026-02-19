//! # Kremis MCP Server
//!
//! Implements `ServerHandler` with 7 MCP tools that proxy to the Kremis HTTP API.

use crate::client::KremisClient;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;

// =============================================================================
// MCP SERVER
// =============================================================================

/// MCP server that bridges to a Kremis HTTP API.
#[derive(Clone)]
pub struct KremisMcp {
    client: KremisClient,
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
}

// =============================================================================
// TOOL PARAMETER STRUCTS
// =============================================================================

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IngestParams {
    /// The entity ID (numeric identifier).
    #[schemars(description = "The entity ID (numeric identifier)")]
    pub entity_id: u64,
    /// The attribute name (e.g. "name", "type", "connected_to").
    #[schemars(description = "The attribute name (e.g. 'name', 'type', 'connected_to')")]
    pub attribute: String,
    /// The value for this attribute.
    #[schemars(description = "The value for this attribute")]
    pub value: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LookupParams {
    /// The entity ID to look up.
    #[schemars(description = "The entity ID to look up")]
    pub entity_id: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TraverseParams {
    /// The starting node ID.
    #[schemars(description = "The starting node ID")]
    pub node_id: u64,
    /// Traversal depth (default: 2, max: 10).
    #[schemars(description = "Traversal depth (default: 2, max: 10)")]
    pub depth: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PathParams {
    /// Starting node ID.
    #[schemars(description = "Starting node ID")]
    pub start: u64,
    /// Ending node ID.
    #[schemars(description = "Ending node ID")]
    pub end: u64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IntersectParams {
    /// List of node IDs to find common connections between.
    #[schemars(description = "List of node IDs to find common connections between")]
    pub nodes: Vec<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PropertiesParams {
    /// The node ID to get properties for.
    #[schemars(description = "The node ID to get properties for")]
    pub node_id: u64,
}

// =============================================================================
// TOOL IMPLEMENTATIONS
// =============================================================================

#[tool_router]
impl KremisMcp {
    pub fn new(client: KremisClient) -> Self {
        Self {
            client,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Add an entity or relation to the Kremis knowledge graph")]
    async fn kremis_ingest(
        &self,
        params: Parameters<IngestParams>,
    ) -> Result<CallToolResult, McpError> {
        let IngestParams {
            entity_id,
            attribute,
            value,
        } = params.0;
        let result = self.client.ingest(entity_id, &attribute, &value).await;
        match result {
            Ok(resp) => {
                let text = if let Some(node_id) = resp.get("node_id").and_then(|v| v.as_u64()) {
                    format!("Ingested successfully. Node ID: {node_id}")
                } else if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
                    format!("Ingest failed: {err}")
                } else {
                    format!("Ingest response: {resp}")
                };
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "Look up an entity in the graph by its entity ID")]
    async fn kremis_lookup(
        &self,
        params: Parameters<LookupParams>,
    ) -> Result<CallToolResult, McpError> {
        let query = serde_json::json!({
            "type": "lookup",
            "entity_id": params.0.entity_id,
        });
        match self.client.query(query).await {
            Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                format_query_response(&resp),
            )])),
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "Traverse the graph from a node to discover connected entities")]
    async fn kremis_traverse(
        &self,
        params: Parameters<TraverseParams>,
    ) -> Result<CallToolResult, McpError> {
        let depth = params.0.depth.unwrap_or(2);
        let query = serde_json::json!({
            "type": "traverse",
            "node_id": params.0.node_id,
            "depth": depth,
        });
        match self.client.query(query).await {
            Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                format_query_response(&resp),
            )])),
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "Find the strongest weighted path between two nodes")]
    async fn kremis_path(
        &self,
        params: Parameters<PathParams>,
    ) -> Result<CallToolResult, McpError> {
        let query = serde_json::json!({
            "type": "strongest_path",
            "start": params.0.start,
            "end": params.0.end,
        });
        match self.client.query(query).await {
            Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                format_query_response(&resp),
            )])),
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "Find common connections between multiple nodes")]
    async fn kremis_intersect(
        &self,
        params: Parameters<IntersectParams>,
    ) -> Result<CallToolResult, McpError> {
        let query = serde_json::json!({
            "type": "intersect",
            "nodes": params.0.nodes,
        });
        match self.client.query(query).await {
            Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                format_query_response(&resp),
            )])),
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "Get current graph statistics (node count, edge count, density)")]
    async fn kremis_status(&self) -> Result<CallToolResult, McpError> {
        match self.client.status().await {
            Ok(resp) => {
                let node_count = resp.get("node_count").and_then(|v| v.as_u64()).unwrap_or(0);
                let edge_count = resp.get("edge_count").and_then(|v| v.as_u64()).unwrap_or(0);
                let stable = resp
                    .get("stable_edges")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let density = resp
                    .get("density_millionths")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let text = format!(
                    "Graph Status:\n  Nodes: {node_count}\n  Edges: {edge_count}\n  Stable edges: {stable}\n  Density: {density} millionths"
                );
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }

    #[tool(description = "Get all properties (attributes and values) of a specific node")]
    async fn kremis_properties(
        &self,
        params: Parameters<PropertiesParams>,
    ) -> Result<CallToolResult, McpError> {
        let query = serde_json::json!({
            "type": "properties",
            "node_id": params.0.node_id,
        });
        match self.client.query(query).await {
            Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                format_query_response(&resp),
            )])),
            Err(e) => Err(McpError::internal_error(format!("{e}"), None)),
        }
    }
}

// =============================================================================
// SERVER HANDLER
// =============================================================================

#[tool_handler]
impl ServerHandler for KremisMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Kremis knowledge graph server. Use tools to ingest entities, \
                 query relationships, traverse the graph, and inspect properties."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

// =============================================================================
// RESPONSE FORMATTING
// =============================================================================

/// Format a query response JSON into human-readable text.
fn format_query_response(resp: &serde_json::Value) -> String {
    let found = resp.get("found").and_then(|v| v.as_bool()).unwrap_or(false);
    if !found {
        let grounding = resp
            .get("grounding")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return format!("Not found.\nGrounding: {grounding}");
    }

    let mut parts = Vec::new();

    // Path
    if let Some(path) = resp.get("path").and_then(|v| v.as_array())
        && !path.is_empty()
    {
        let ids: Vec<String> = path
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n.to_string()))
            .collect();
        parts.push(format!("Path: [{}]", ids.join(" -> ")));
    }

    // Edges
    if let Some(edges) = resp.get("edges").and_then(|v| v.as_array())
        && !edges.is_empty()
    {
        parts.push(format!("Edges ({}):", edges.len()));
        for edge in edges {
            let from = edge.get("from").and_then(|v| v.as_u64()).unwrap_or(0);
            let to = edge.get("to").and_then(|v| v.as_u64()).unwrap_or(0);
            let weight = edge.get("weight").and_then(|v| v.as_i64()).unwrap_or(0);
            parts.push(format!("  {from} --({weight})--> {to}"));
        }
    }

    // Properties
    if let Some(props) = resp.get("properties").and_then(|v| v.as_array())
        && !props.is_empty()
    {
        parts.push(format!("Properties ({}):", props.len()));
        for prop in props {
            let attr = prop
                .get("attribute")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let val = prop.get("value").and_then(|v| v.as_str()).unwrap_or("?");
            parts.push(format!("  {attr}: {val}"));
        }
    }

    if let Some(grounding) = resp.get("grounding").and_then(|v| v.as_str()) {
        parts.push(format!("Grounding: {grounding}"));
    }

    if parts.is_empty() {
        "Found (no details).".to_string()
    } else {
        parts.join("\n")
    }
}
