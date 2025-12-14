//! MCP error types

use thiserror::Error;

/// Errors that can occur in MCP operations
#[derive(Debug, Error)]
pub enum McpError {
    /// Server not found
    #[error("Server not found: {0}")]
    ServerNotFound(String),

    /// Not connected to server
    #[error("Not connected to MCP server")]
    NotConnected,

    /// Transport error
    #[error("Transport error: {0}")]
    TransportError(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// JSON-RPC error
    #[error("RPC error {code}: {message}")]
    RpcError {
        code: i64,
        message: String,
    },

    /// Tool execution error
    #[error("Tool error: {0}")]
    ToolError(String),

    /// Connection timeout
    #[error("Connection timeout")]
    Timeout,

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
