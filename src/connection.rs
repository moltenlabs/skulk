//! Single MCP server connection

use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use warhorn::McpServerConfig;
use crate::transport::McpTransport;
use crate::types::{ToolSchema, ServerInfo};
use crate::error::McpError;

/// Connection to a single MCP server
pub struct McpConnection {
    /// Server configuration
    config: McpServerConfig,
    /// Transport layer
    transport: Mutex<Option<Box<dyn McpTransport>>>,
    /// Whether connected
    connected: AtomicBool,
    /// Server info (after initialization)
    server_info: Mutex<Option<ServerInfo>>,
    /// Request ID counter
    request_id: std::sync::atomic::AtomicU64,
}

impl McpConnection {
    /// Create a new connection (but don't connect yet)
    pub async fn new(config: McpServerConfig) -> Result<Self, McpError> {
        Ok(Self {
            config,
            transport: Mutex::new(None),
            connected: AtomicBool::new(false),
            server_info: Mutex::new(None),
            request_id: std::sync::atomic::AtomicU64::new(0),
        })
    }

    /// Initialize the connection
    pub async fn initialize(&self) -> Result<ServerInfo, McpError> {
        info!(server_id = %self.config.id, "Initializing MCP connection");
        
        // Create transport based on config
        let transport = crate::transport::create_transport(&self.config).await?;
        *self.transport.lock().await = Some(transport);
        
        // Send initialize request
        let init_response = self.send_request("initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "sampling": {}
            },
            "clientInfo": {
                "name": "lair",
                "version": env!("CARGO_PKG_VERSION")
            }
        })).await?;
        
        // Parse server info
        let server_info: ServerInfo = serde_json::from_value(init_response)
            .map_err(|e| McpError::ProtocolError(format!("Invalid server info: {}", e)))?;
        
        *self.server_info.lock().await = Some(server_info.clone());
        self.connected.store(true, Ordering::SeqCst);
        
        // Send initialized notification
        self.send_notification("notifications/initialized", serde_json::json!({})).await?;
        
        info!(
            server_id = %self.config.id,
            server_name = %server_info.name,
            "MCP connection initialized"
        );
        
        Ok(server_info)
    }

    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<ToolSchema>, McpError> {
        let response = self.send_request("tools/list", serde_json::json!({})).await?;
        
        let tools: Vec<ToolSchema> = response["tools"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();
        
        debug!(server_id = %self.config.id, num_tools = tools.len(), "Listed tools");
        Ok(tools)
    }

    /// Call a tool
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        debug!(server_id = %self.config.id, tool = %name, "Calling tool");
        
        let response = self.send_request("tools/call", serde_json::json!({
            "name": name,
            "arguments": arguments
        })).await?;
        
        // Check for error in response
        if let Some(error) = response.get("error") {
            return Err(McpError::ToolError(error.to_string()));
        }
        
        Ok(response["content"].clone())
    }

    /// Send sandbox state notification
    pub async fn notify_sandbox_state(&self, enabled: bool, policy: &str) -> Result<(), McpError> {
        self.send_notification("notifications/sandbox_state", serde_json::json!({
            "enabled": enabled,
            "policy": policy
        })).await
    }

    /// Ping the server
    pub async fn ping(&self) -> Result<(), McpError> {
        self.send_request("ping", serde_json::json!({})).await?;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Get server info
    pub async fn server_info(&self) -> Option<ServerInfo> {
        self.server_info.lock().await.clone()
    }

    /// Shutdown the connection
    pub async fn shutdown(&self) -> Result<(), McpError> {
        self.connected.store(false, Ordering::SeqCst);
        
        if let Some(transport) = self.transport.lock().await.take() {
            transport.close().await?;
        }
        
        info!(server_id = %self.config.id, "MCP connection shutdown");
        Ok(())
    }

    /// Send a JSON-RPC request
    async fn send_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        
        let transport = self.transport.lock().await;
        let transport = transport.as_ref()
            .ok_or_else(|| McpError::NotConnected)?;
        
        let response = transport.send_request(request).await?;
        
        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(McpError::RpcError {
                code: error["code"].as_i64().unwrap_or(-1),
                message: error["message"].as_str().unwrap_or("Unknown error").to_string(),
            });
        }
        
        Ok(response["result"].clone())
    }

    /// Send a JSON-RPC notification (no response expected)
    async fn send_notification(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<(), McpError> {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        
        let transport = self.transport.lock().await;
        let transport = transport.as_ref()
            .ok_or_else(|| McpError::NotConnected)?;
        
        transport.send_notification(notification).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require mock transport
}
