use crate::registry::{PermissionMode, Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::timeout;

pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Run a shell command in the workspace. Prefer non-interactive commands. Captures stdout+stderr."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to run" },
                "timeout_ms": { "type": "integer", "description": "Timeout in ms (default 120000)" },
                "cwd": { "type": "string", "description": "Optional working directory" }
            },
            "required": ["command"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let command = match args.get("command").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::err("missing command"),
        };
        let timeout_ms = args
            .get("timeout_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(120_000);
        let cwd = args
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(|p| ctx.resolve_path(p))
            .unwrap_or_else(|| ctx.cwd.clone());

        if matches!(ctx.permission, PermissionMode::Deny) {
            return ToolResult::err("bash disabled by permission mode");
        }
        if matches!(ctx.permission, PermissionMode::Prompt) {
            if !ctx.approve(&format!("run bash: {command}")) {
                return ToolResult::err("bash denied by user");
            }
        }

        let mut child = match Command::new("bash")
            .arg("-lc")
            .arg(command)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .kill_on_drop(true)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return ToolResult::err(format!("spawn: {e}")),
        };

        let fut = async {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut out) = child.stdout.take() {
                let _ = out.read_to_end(&mut stdout).await;
            }
            if let Some(mut err) = child.stderr.take() {
                let _ = err.read_to_end(&mut stderr).await;
            }
            let status = child.wait().await?;
            Ok::<_, std::io::Error>((status, stdout, stderr))
        };

        match timeout(Duration::from_millis(timeout_ms), fut).await {
            Ok(Ok((status, stdout, stderr))) => {
                let mut out = String::new();
                out.push_str(&String::from_utf8_lossy(&stdout));
                if !stderr.is_empty() {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str("--- stderr ---\n");
                    out.push_str(&String::from_utf8_lossy(&stderr));
                }
                let code = status.code().unwrap_or(-1);
                if out.len() > 200_000 {
                    out.truncate(200_000);
                    out.push_str("\n...[truncated]...");
                }
                if out.is_empty() {
                    out = format!("(no output, exit {code})");
                } else {
                    out.push_str(&format!("\n(exit {code})"));
                }
                if status.success() {
                    ToolResult::ok(out)
                } else {
                    ToolResult::err(out)
                }
            }
            Ok(Err(e)) => ToolResult::err(format!("io: {e}")),
            Err(_) => {
                let _ = child.kill().await;
                ToolResult::err(format!("timeout after {timeout_ms}ms"))
            }
        }
    }
}
