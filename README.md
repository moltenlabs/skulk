# 🕵️ Skulk

MCP (Model Context Protocol) connection manager - sneaking connections to the outside.

[![Crates.io](https://img.shields.io/crates/v/skulk.svg)](https://crates.io/crates/skulk)
[![Documentation](https://docs.rs/skulk/badge.svg)](https://docs.rs/skulk)
[![License](https://img.shields.io/crates/l/skulk.svg)](LICENSE)

## Overview

Skulk provides a Rust client for the Model Context Protocol (MCP), managing connections to MCP servers and enabling tool discovery and execution.

## Features

- 🔌 Multiple transport support (stdio, socket, HTTP)
- 🔧 Automatic tool discovery
- 📦 Tool schema caching
- 💓 Health monitoring
- 🔒 Sandbox state notifications

## Installation

```toml
[dependencies]
skulk = "0.1"
```

## Usage

```rust
use skulk::{McpManager, McpServerConfig};
use warhorn::McpTransport;

#[tokio::main]
async fn main() -> Result<(), skulk::McpError> {
    let mut manager = McpManager::new();

    // Connect to an MCP server
    let config = McpServerConfig {
        id: "my-server".into(),
        name: "My MCP Server".into(),
        transport: McpTransport::Stdio {
            command: "my-mcp-server".into(),
            args: vec![],
        },
        env: Default::default(),
    };

    manager.connect(config).await?;

    // Discover available tools
    let tools = manager.list_tools();
    for tool in tools {
        println!("Found tool: {} - {}", tool.name, tool.description);
    }

    // Call a tool
    let result = manager.call_tool(
        "my-server",
        "some_tool",
        serde_json::json!({"arg": "value"})
    ).await?;

    Ok(())
}
```

## Transport Types

```rust
use warhorn::McpTransport;

// Stdio (spawn a process)
let stdio = McpTransport::Stdio {
    command: "npx".into(),
    args: vec!["-y", "@modelcontextprotocol/server-filesystem"].into_iter().map(String::from).collect(),
};

// Unix socket
let socket = McpTransport::Socket {
    path: "/tmp/mcp.sock".into(),
};

// HTTP/SSE
let http = McpTransport::Http {
    url: "http://localhost:3000/mcp".into(),
};
```

## Part of the Goblin Family

- [warhorn](https://crates.io/crates/warhorn) - Protocol types
- [trinkets](https://crates.io/crates/trinkets) - Tool registry
- [wardstone](https://crates.io/crates/wardstone) - Sandboxing
- **skulk** - MCP connections (you are here)
- [hutch](https://crates.io/crates/hutch) - Checkpoints
- [ambush](https://crates.io/crates/ambush) - Task planning
- [cabal](https://crates.io/crates/cabal) - Orchestration

## License

MIT OR Apache-2.0
