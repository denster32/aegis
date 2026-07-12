use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read a text file. Optional 1-based offset and limit (lines). Binary files are rejected."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Absolute or workspace-relative path" },
                "offset": { "type": "integer", "description": "1-based start line" },
                "limit": { "type": "integer", "description": "Max lines to return" }
            },
            "required": ["path"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => ctx.resolve_path(p),
            None => return ToolResult::err("missing path"),
        };
        let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        match fs::read(&path).await {
            Ok(bytes) => {
                if bytes.iter().take(8192).any(|&b| b == 0) {
                    return ToolResult::err("binary file (null bytes detected)");
                }
                let text = String::from_utf8_lossy(&bytes);
                let lines: Vec<&str> = text.lines().collect();
                let start = offset.saturating_sub(1).min(lines.len());
                let end = match limit {
                    Some(l) => (start + l).min(lines.len()),
                    None => lines.len(),
                };
                let mut out = String::new();
                for (i, line) in lines[start..end].iter().enumerate() {
                    out.push_str(&format!("{:6}|{}\n", start + i + 1, line));
                }
                if out.is_empty() {
                    out = "(empty file or range)\n".into();
                }
                ToolResult::ok(out)
            }
            Err(e) => ToolResult::err(format!("read {}: {e}", path.display())),
        }
    }
}

pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Create or overwrite a text file with the given contents."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => ctx.resolve_path(p),
            None => return ToolResult::err("missing path"),
        };
        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => return ToolResult::err("missing content"),
        };

        if !ctx.is_within_cwd(&path)
            && !ctx.approve(&format!("write outside cwd: {}", path.display()))
        {
            return ToolResult::err("permission denied: write outside workspace");
        }

        let lock = ctx.lock_path(&path).await;
        let _g = lock.lock().await;

        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return ToolResult::err(format!("mkdir: {e}"));
            }
        }
        match fs::write(&path, content).await {
            Ok(()) => {
                ToolResult::ok(format!("wrote {} bytes to {}", content.len(), path.display()))
            }
            Err(e) => ToolResult::err(format!("write {}: {e}", path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{PermissionMode, ToolContext};
    use std::path::PathBuf;

    #[tokio::test]
    async fn write_and_read_roundtrip() {
        let dir = std::env::temp_dir().join(format!("aegis-tool-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let ctx = ToolContext::new(dir.clone(), "test".into(), PermissionMode::Yolo);
        let w = WriteFileTool;
        let r = w
            .call(
                json!({"path": "a.txt", "content": "hello"}),
                &ctx,
            )
            .await;
        assert!(r.ok, "{}", r.output);
        let reader = ReadFileTool;
        let out = reader
            .call(json!({"path": "a.txt"}), &ctx)
            .await;
        assert!(out.ok);
        assert!(out.output.contains("hello"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
