//! # Skulk
//!
//! MCP (Model Context Protocol) connection manager - sneaking connections to the outside.
//!
//! This crate manages connections to MCP servers and provides
//! tool discovery and execution capabilities.
//!
//! ## Features
//!
//! - Connect to MCP servers via stdio, socket, or HTTP
//! - Automatic tool discovery
//! - Tool schema caching
//! - Health monitoring
//! - Sandbox state notifications
//!
//! ## Example
//!
//! ```rust,ignore
//! use skulk::{McpManager, McpServerConfig, McpTransport};
//!
//! let mut manager = McpManager::new();
//!
//! // Connect to an MCP server
//! let config = McpServerConfig {
//!     id: "my-server".into(),
//!     name: "My MCP Server".into(),
//!     transport: McpTransport::Stdio {
//!         command: "my-mcp-server".into(),
//!         args: vec![],
//!     },
//!     env: Default::default(),
//! };
//!
//! manager.connect(config).await?;
//!
//! // Discover tools
//! let tools = manager.list_tools().await?;
//! ```

pub mod manager;
pub mod connection;
pub mod transport;
pub mod types;
pub mod error;

pub use manager::McpManager;
pub use connection::McpConnection;
pub use transport::McpTransport;
pub use types::*;
pub use error::McpError;

// Re-export protocol types
pub use warhorn::McpServerConfig;
