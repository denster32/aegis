use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::process::Command;

pub struct GitStatusTool;

#[async_trait]
impl Tool for GitStatusTool {
    fn name(&self) -> &str {
        "git_status"
    }

    fn description(&self) -> &str {
        "Show git status --short and current branch for the workspace."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    async fn call(&self, _args: Value, ctx: &ToolContext) -> ToolResult {
        run_git(&ctx.cwd, &["status", "--short", "--branch"]).await
    }
}

pub struct GitDiffTool;

#[async_trait]
impl Tool for GitDiffTool {
    fn name(&self) -> &str {
        "git_diff"
    }

    fn description(&self) -> &str {
        "Show git diff (optionally staged). Caps output size."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "staged": { "type": "boolean", "default": false },
                "path": { "type": "string" }
            },
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let staged = args
            .get("staged")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let mut cmd = vec!["diff".to_string()];
        if staged {
            cmd.push("--staged".into());
        }
        if let Some(p) = args.get("path").and_then(|v| v.as_str()) {
            cmd.push("--".into());
            cmd.push(p.into());
        }
        let refs: Vec<&str> = cmd.iter().map(|s| s.as_str()).collect();
        run_git(&ctx.cwd, &refs).await
    }
}

async fn run_git(cwd: &std::path::Path, args: &[&str]) -> ToolResult {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;
    match out {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
            if !o.stderr.is_empty() {
                s.push_str(&format!(
                    "\n{}",
                    String::from_utf8_lossy(&o.stderr)
                ));
            }
            if s.len() > 100_000 {
                s.truncate(100_000);
                s.push_str("\n…[truncated]");
            }
            if o.status.success() {
                ToolResult::ok(if s.is_empty() {
                    "(clean)".into()
                } else {
                    s
                })
            } else {
                ToolResult::err(s)
            }
        }
        Err(e) => ToolResult::err(e.to_string()),
    }
}
