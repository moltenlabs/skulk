//! MCP type definitions

use serde::{Deserialize, Serialize};

/// Tool schema from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name
    pub name: String,
    /// Tool description
    #[serde(default)]
    pub description: String,
    /// Input schema (JSON Schema)
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Server information returned on initialize
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    #[serde(default)]
    pub version: String,
    /// Protocol version
    #[serde(default)]
    pub protocol_version: String,
    /// Server capabilities
    #[serde(default)]
    pub capabilities: ServerCapabilities,
}

/// Server capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tools capability
    #[serde(default)]
    pub tools: Option<ToolsCapability>,
    /// Resources capability
    #[serde(default)]
    pub resources: Option<ResourcesCapability>,
    /// Prompts capability
    #[serde(default)]
    pub prompts: Option<PromptsCapability>,
    /// Sampling capability
    #[serde(default)]
    pub sampling: Option<SamplingCapability>,
}

/// Tools capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// List changed notifications supported
    #[serde(default)]
    pub list_changed: bool,
}

/// Resources capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Subscribe supported
    #[serde(default)]
    pub subscribe: bool,
    /// List changed notifications supported
    #[serde(default)]
    pub list_changed: bool,
}

/// Prompts capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// List changed notifications supported
    #[serde(default)]
    pub list_changed: bool,
}

/// Sampling capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SamplingCapability {}

/// Server health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerHealth {
    /// Server is healthy
    Healthy,
    /// Server is unhealthy (failed ping)
    Unhealthy,
    /// Server is disconnected
    Disconnected,
    /// Health unknown
    Unknown,
}

impl Default for ServerHealth {
    fn default() -> Self {
        ServerHealth::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_schema_deserialize() {
        let json = r#"{
            "name": "test_tool",
            "description": "A test tool",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }
        }"#;

        let schema: ToolSchema = serde_json::from_str(json).unwrap();
        assert_eq!(schema.name, "test_tool");
        assert_eq!(schema.description, "A test tool");
    }
}
