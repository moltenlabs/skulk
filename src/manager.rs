//! MCP connection manager

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info, warn, error};

use warhorn::McpServerConfig;
use crate::connection::McpConnection;
use crate::types::{ToolSchema, ServerHealth, ServerInfo};
use crate::error::McpError;

/// Manages connections to multiple MCP servers
pub struct McpManager {
    /// Active connections by server ID
    connections: RwLock<HashMap<String, Arc<McpConnection>>>,
    /// Cached tool schemas
    tool_cache: RwLock<HashMap<String, Vec<ToolSchema>>>,
    /// Server health status
    health: RwLock<HashMap<String, ServerHealth>>,
}

impl McpManager {
    /// Create a new MCP manager
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            tool_cache: RwLock::new(HashMap::new()),
            health: RwLock::new(HashMap::new()),
        }
    }

    /// Connect to an MCP server
    pub async fn connect(&self, config: McpServerConfig) -> Result<(), McpError> {
        let server_id = config.id.clone();
        
        info!(server_id = %server_id, "Connecting to MCP server");
        
        let connection = McpConnection::new(config).await?;
        let connection = Arc::new(connection);
        
        // Initialize connection
        connection.initialize().await?;
        
        // Discover tools
        let tools = connection.list_tools().await?;
        
        // Store connection and tools
        self.connections.write().insert(server_id.clone(), connection);
        self.tool_cache.write().insert(server_id.clone(), tools);
        self.health.write().insert(server_id.clone(), ServerHealth::Healthy);
        
        info!(server_id = %server_id, "Connected to MCP server");
        Ok(())
    }

    /// Disconnect from an MCP server
    pub async fn disconnect(&self, server_id: &str) -> Result<(), McpError> {
        let connection = self.connections.write().remove(server_id);
        
        if let Some(conn) = connection {
            conn.shutdown().await?;
        }
        
        self.tool_cache.write().remove(server_id);
        self.health.write().remove(server_id);
        
        info!(server_id = %server_id, "Disconnected from MCP server");
        Ok(())
    }

    /// Get a connection by server ID
    pub fn get_connection(&self, server_id: &str) -> Option<Arc<McpConnection>> {
        self.connections.read().get(server_id).cloned()
    }

    /// List all connected server IDs
    pub fn server_ids(&self) -> Vec<String> {
        self.connections.read().keys().cloned().collect()
    }

    /// List all available tools across all servers
    pub fn list_tools(&self) -> Vec<ToolSchema> {
        let cache = self.tool_cache.read();
        cache.values().flatten().cloned().collect()
    }

    /// List tools from a specific server
    pub fn list_server_tools(&self, server_id: &str) -> Vec<ToolSchema> {
        self.tool_cache.read()
            .get(server_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Find a tool by name (returns server_id, tool_name)
    pub fn find_tool(&self, name: &str) -> Option<(String, ToolSchema)> {
        let cache = self.tool_cache.read();
        for (server_id, tools) in cache.iter() {
            if let Some(tool) = tools.iter().find(|t| t.name == name) {
                return Some((server_id.clone(), tool.clone()));
            }
        }
        None
    }

    /// Call a tool on a specific server
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let connection = self.get_connection(server_id)
            .ok_or_else(|| McpError::ServerNotFound(server_id.to_string()))?;
        
        connection.call_tool(tool_name, arguments).await
    }

    /// Get health status of a server
    pub fn server_health(&self, server_id: &str) -> Option<ServerHealth> {
        self.health.read().get(server_id).cloned()
    }

    /// Refresh tools from a server
    pub async fn refresh_tools(&self, server_id: &str) -> Result<Vec<ToolSchema>, McpError> {
        let connection = self.get_connection(server_id)
            .ok_or_else(|| McpError::ServerNotFound(server_id.to_string()))?;
        
        let tools = connection.list_tools().await?;
        self.tool_cache.write().insert(server_id.to_string(), tools.clone());
        
        debug!(server_id = %server_id, num_tools = tools.len(), "Refreshed tools");
        Ok(tools)
    }

    /// Notify all servers of sandbox state change
    pub async fn notify_sandbox_state(&self, enabled: bool, policy: &str) {
        for connection in self.connections.read().values() {
            if let Err(e) = connection.notify_sandbox_state(enabled, policy).await {
                warn!(error = %e, "Failed to notify sandbox state");
            }
        }
    }

    /// Check health of all connections
    pub async fn health_check(&self) {
        for (server_id, connection) in self.connections.read().iter() {
            let health = if connection.is_connected() {
                match connection.ping().await {
                    Ok(_) => ServerHealth::Healthy,
                    Err(_) => ServerHealth::Unhealthy,
                }
            } else {
                ServerHealth::Disconnected
            };
            
            self.health.write().insert(server_id.clone(), health);
        }
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = McpManager::new();
        assert!(manager.server_ids().is_empty());
        assert!(manager.list_tools().is_empty());
    }
}
