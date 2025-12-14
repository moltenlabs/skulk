//! MCP transport implementations

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, error};

use warhorn::McpServerConfig;
use crate::error::McpError;

/// Transport trait for MCP communication
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Send a request and wait for response
    async fn send_request(&self, request: serde_json::Value) -> Result<serde_json::Value, McpError>;
    
    /// Send a notification (no response)
    async fn send_notification(&self, notification: serde_json::Value) -> Result<(), McpError>;
    
    /// Close the transport
    async fn close(self: Box<Self>) -> Result<(), McpError>;
}

/// Create a transport from config
pub async fn create_transport(
    config: &McpServerConfig,
) -> Result<Box<dyn McpTransport>, McpError> {
    match &config.transport {
        warhorn::McpTransport::Stdio { command, args } => {
            let transport = StdioTransport::new(command, args, &config.env).await?;
            Ok(Box::new(transport))
        }
        warhorn::McpTransport::Socket { path: _ } => {
            // Socket transport not yet implemented
            Err(McpError::TransportError("Socket transport not implemented".into()))
        }
        warhorn::McpTransport::Http { url: _ } => {
            // HTTP transport not yet implemented
            Err(McpError::TransportError("HTTP transport not implemented".into()))
        }
    }
}

/// Stdio-based transport (spawns a child process)
pub struct StdioTransport {
    child: tokio::sync::Mutex<Child>,
    stdin: tokio::sync::Mutex<tokio::process::ChildStdin>,
    stdout: tokio::sync::Mutex<BufReader<tokio::process::ChildStdout>>,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub async fn new(
        command: &str,
        args: &[String],
        env: &std::collections::HashMap<String, String>,
    ) -> Result<Self, McpError> {
        debug!(command = %command, "Starting MCP server process");
        
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .kill_on_drop(true);
        
        for (key, value) in env {
            cmd.env(key, value);
        }
        
        let mut child = cmd.spawn()
            .map_err(|e| McpError::TransportError(format!("Failed to spawn: {}", e)))?;
        
        let stdin = child.stdin.take()
            .ok_or_else(|| McpError::TransportError("No stdin".into()))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| McpError::TransportError("No stdout".into()))?;
        
        Ok(Self {
            child: tokio::sync::Mutex::new(child),
            stdin: tokio::sync::Mutex::new(stdin),
            stdout: tokio::sync::Mutex::new(BufReader::new(stdout)),
        })
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send_request(&self, request: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let request_str = serde_json::to_string(&request)
            .map_err(|e| McpError::ProtocolError(format!("JSON error: {}", e)))?;
        
        // Send request
        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(request_str.as_bytes()).await
                .map_err(|e| McpError::TransportError(format!("Write error: {}", e)))?;
            stdin.write_all(b"\n").await
                .map_err(|e| McpError::TransportError(format!("Write error: {}", e)))?;
            stdin.flush().await
                .map_err(|e| McpError::TransportError(format!("Flush error: {}", e)))?;
        }
        
        // Read response
        let mut response_line = String::new();
        {
            let mut stdout = self.stdout.lock().await;
            stdout.read_line(&mut response_line).await
                .map_err(|e| McpError::TransportError(format!("Read error: {}", e)))?;
        }
        
        let response: serde_json::Value = serde_json::from_str(&response_line)
            .map_err(|e| McpError::ProtocolError(format!("Invalid JSON response: {}", e)))?;
        
        Ok(response)
    }

    async fn send_notification(&self, notification: serde_json::Value) -> Result<(), McpError> {
        let notification_str = serde_json::to_string(&notification)
            .map_err(|e| McpError::ProtocolError(format!("JSON error: {}", e)))?;
        
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(notification_str.as_bytes()).await
            .map_err(|e| McpError::TransportError(format!("Write error: {}", e)))?;
        stdin.write_all(b"\n").await
            .map_err(|e| McpError::TransportError(format!("Write error: {}", e)))?;
        stdin.flush().await
            .map_err(|e| McpError::TransportError(format!("Flush error: {}", e)))?;
        
        Ok(())
    }

    async fn close(self: Box<Self>) -> Result<(), McpError> {
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Transport tests would require mock processes
}
