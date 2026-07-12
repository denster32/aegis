//! Minimal JSON-RPC MCP stdio client.
//!
//! Supports: initialize, tools/list, tools/call.

use aegis_tools::{Tool, ToolContext, ToolRegistry, ToolResult};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Vec<(String, String)>,
}

struct McpSession {
    #[allow(dead_code)]
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<tokio::process::ChildStdout>,
    next_id: AtomicU64,
}

impl McpSession {
    async fn spawn(cfg: &McpServerConfig) -> Result<Self> {
        let mut cmd = Command::new(&cfg.command);
        cmd.args(&cfg.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        for (k, v) in &cfg.env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().with_context(|| format!("spawn MCP {}", cfg.name))?;
        let stdin = child.stdin.take().context("mcp stdin")?;
        let stdout = child.stdout.take().context("mcp stdout")?;
        let mut session = Self {
            child,
            stdin,
            reader: BufReader::new(stdout),
            next_id: AtomicU64::new(1),
        };
        session
            .request(
                "initialize",
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": { "name": "aegis", "version": env!("CARGO_PKG_VERSION") }
                }),
            )
            .await?;
        // notifications/initialized
        session
            .notify("notifications/initialized", json!({}))
            .await?;
        Ok(session)
    }

    async fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        let line = serde_json::to_string(&msg)?;
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let msg = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        let line = serde_json::to_string(&msg)?;
        debug!(%method, %id, "MCP request");
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;

        // Read until matching id
        loop {
            let mut buf = String::new();
            let n = self.reader.read_line(&mut buf).await?;
            if n == 0 {
                bail!("MCP server closed stdout");
            }
            let v: Value = match serde_json::from_str(buf.trim()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v.get("id").and_then(|i| i.as_u64()) == Some(id)
                || v.get("id").and_then(|i| i.as_i64()) == Some(id as i64)
            {
                if let Some(err) = v.get("error") {
                    bail!("MCP error: {err}");
                }
                return Ok(v.get("result").cloned().unwrap_or(Value::Null));
            }
        }
    }

    async fn list_tools(&mut self) -> Result<Vec<McpToolInfo>> {
        let result = self.request("tools/list", json!({})).await?;
        let tools = result
            .get("tools")
            .and_then(|t| t.as_array())
            .cloned()
            .unwrap_or_default();
        let mut out = Vec::new();
        for t in tools {
            let name = t
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            if name.is_empty() {
                continue;
            }
            out.push(McpToolInfo {
                name,
                description: t
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string(),
                input_schema: t
                    .get("inputSchema")
                    .cloned()
                    .unwrap_or_else(|| json!({"type":"object","properties":{}})),
            });
        }
        Ok(out)
    }

    async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<String> {
        let result = self
            .request(
                "tools/call",
                json!({ "name": name, "arguments": arguments }),
            )
            .await?;
        // content array
        if let Some(arr) = result.get("content").and_then(|c| c.as_array()) {
            let mut parts = Vec::new();
            for item in arr {
                if let Some(t) = item.get("text").and_then(|t| t.as_str()) {
                    parts.push(t.to_string());
                } else {
                    parts.push(item.to_string());
                }
            }
            return Ok(parts.join("\n"));
        }
        Ok(result.to_string())
    }
}

#[derive(Clone)]
struct McpToolInfo {
    name: String,
    description: String,
    input_schema: Value,
}

struct McpTool {
    server: String,
    info: McpToolInfo,
    session: Arc<Mutex<McpSession>>,
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        // prefix to avoid collisions
        // Tool trait returns &str — store owned name
        self.info.name.as_str()
    }

    fn description(&self) -> &str {
        self.info.description.as_str()
    }

    fn parameters_schema(&self) -> Value {
        self.info.input_schema.clone()
    }

    async fn call(&self, args: Value, _ctx: &ToolContext) -> ToolResult {
        let mut s = self.session.lock().await;
        match s.call_tool(&self.info.name, args).await {
            Ok(out) => ToolResult::ok(format!("[mcp:{}] {out}", self.server)),
            Err(e) => ToolResult::err(format!("[mcp:{}] {e:#}", self.server)),
        }
    }
}

/// Connect configured MCP servers and register their tools.
pub async fn register_mcp_tools(reg: &mut ToolRegistry, servers: &[McpServerConfig]) -> Result<()> {
    for cfg in servers {
        info!(name = %cfg.name, "starting MCP server");
        match McpSession::spawn(cfg).await {
            Ok(mut session) => {
                let tools = match session.list_tools().await {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::warn!(name = %cfg.name, error = %e, "tools/list failed");
                        continue;
                    }
                };
                let session = Arc::new(Mutex::new(session));
                for info in tools {
                    // Prefix name with server to avoid collisions
                    let prefixed = format!("mcp__{}__{}", cfg.name, info.name);
                    let mut info = info;
                    let original = info.name.clone();
                    info.name = prefixed.clone();
                    // keep original for call - store in description? Better custom wrapper
                    let tool = McpTool {
                        server: cfg.name.clone(),
                        info: McpToolInfo {
                            name: original,
                            description: format!("[MCP {}] {}", cfg.name, info.description),
                            input_schema: info.input_schema,
                        },
                        session: session.clone(),
                    };
                    // Register under prefixed name by wrapping
                    reg.register(Arc::new(PrefixedMcpTool {
                        prefixed,
                        inner: tool,
                    }));
                }
            }
            Err(e) => {
                tracing::warn!(name = %cfg.name, error = %e, "failed to start MCP server");
            }
        }
    }
    Ok(())
}

struct PrefixedMcpTool {
    prefixed: String,
    inner: McpTool,
}

#[async_trait]
impl Tool for PrefixedMcpTool {
    fn name(&self) -> &str {
        &self.prefixed
    }
    fn description(&self) -> &str {
        self.inner.description()
    }
    fn parameters_schema(&self) -> Value {
        self.inner.parameters_schema()
    }
    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        self.inner.call(args, ctx).await
    }
}
