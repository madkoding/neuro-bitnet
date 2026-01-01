//! MCP Server implementation
//!
//! Handles JSON-RPC communication over stdio

use std::io::{BufRead, Write};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::{
    get_tools, execute_tool,
    protocol::*,
};

/// MCP Server
pub struct McpServer {
    model_path: String,
}

impl McpServer {
    pub fn new(model_path: String) -> Self {
        Self { model_path }
    }

    /// Run the MCP server (stdio transport)
    pub async fn run(self) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        let (tx, mut rx) = mpsc::channel::<String>(100);

        // Spawn a task to write responses
        let write_handle = tokio::spawn(async move {
            while let Some(response) = rx.recv().await {
                let mut stdout = std::io::stdout();
                if let Err(e) = writeln!(stdout, "{}", response) {
                    error!("Failed to write response: {}", e);
                }
                let _ = stdout.flush();
            }
        });

        // Read requests from stdin
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    error!("Failed to read line: {}", e);
                    continue;
                }
            };

            if line.is_empty() {
                continue;
            }

            debug!("Received: {}", line);

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let response = JsonRpcResponse::error(
                        None,
                        JsonRpcError::parse_error(),
                    );
                    let _ = writeln!(stdout, "{}", serde_json::to_string(&response)?);
                    let _ = stdout.flush();
                    error!("Parse error: {}", e);
                    continue;
                }
            };

            let response = self.handle_request(request).await;

            if let Some(resp) = response {
                let json = serde_json::to_string(&resp)?;
                debug!("Sending: {}", json);
                writeln!(stdout, "{}", json)?;
                stdout.flush()?;
            }
        }

        drop(tx);
        let _ = write_handle.await;

        Ok(())
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = request.id.clone();

        match request.method.as_str() {
            // Lifecycle methods
            "initialize" => {
                let result = self.handle_initialize(request.params);
                Some(JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()))
            }
            "initialized" => {
                // Notification, no response needed
                info!("Client initialized");
                None
            }
            "shutdown" => {
                info!("Shutdown requested");
                Some(JsonRpcResponse::success(id, serde_json::Value::Null))
            }

            // Tools
            "tools/list" => {
                let result = ListToolsResult { tools: get_tools() };
                Some(JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()))
            }
            "tools/call" => {
                let result = self.handle_tool_call(request.params).await;
                match result {
                    Ok(r) => Some(JsonRpcResponse::success(id, serde_json::to_value(r).unwrap())),
                    Err(e) => Some(JsonRpcResponse::error(id, JsonRpcError::internal_error(&e.to_string()))),
                }
            }

            // Prompts (not implemented)
            "prompts/list" => {
                Some(JsonRpcResponse::success(id, serde_json::json!({ "prompts": [] })))
            }

            // Resources (not implemented)
            "resources/list" => {
                Some(JsonRpcResponse::success(id, serde_json::json!({ "resources": [] })))
            }

            // Unknown method
            method => {
                Some(JsonRpcResponse::error(id, JsonRpcError::method_not_found(method)))
            }
        }
    }

    fn handle_initialize(&self, _params: Option<serde_json::Value>) -> InitializeResult {
        InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability { list_changed: false }),
                prompts: None,
                resources: None,
            },
            server_info: ServerInfo {
                name: "neuro-bitnet".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    async fn handle_tool_call(&self, params: Option<serde_json::Value>) -> anyhow::Result<CallToolResult> {
        let params: CallToolParams = match params {
            Some(p) => serde_json::from_value(p)?,
            None => anyhow::bail!("Missing parameters"),
        };

        Ok(execute_tool(&params.name, params.arguments, &self.model_path).await)
    }
}
