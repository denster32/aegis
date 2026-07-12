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
        if !ctx.allow_path(&path, "read") {
            return ToolResult::err("permission denied: read outside workspace");
        }
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

        if !ctx.allow_path(&path, "write") {
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
            Ok(()) => ToolResult::ok(format!(
                "wrote {} bytes to {}",
                content.len(),
                path.display()
            )),
            Err(e) => ToolResult::err(format!("write {}: {e}", path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{PermissionMode, ToolContext};

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "aegis-tool-test-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[tokio::test]
    async fn write_and_read_roundtrip() {
        let dir = temp_dir();
        let ctx = ToolContext::new(dir.clone(), "test".into(), PermissionMode::Yolo);
        let w = WriteFileTool;
        let r = w
            .call(json!({"path": "a.txt", "content": "hello"}), &ctx)
            .await;
        assert!(r.ok, "{}", r.output);
        let reader = ReadFileTool;
        let out = reader.call(json!({"path": "a.txt"}), &ctx).await;
        assert!(out.ok);
        assert!(out.output.contains("hello"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn sandbox_deny_allows_inside_blocks_outside_fs() {
        let dir = temp_dir();
        let ctx = ToolContext::new(dir.clone(), "test".into(), PermissionMode::Deny);
        let w = WriteFileTool;
        let reader = ReadFileTool;

        let inside = w
            .call(json!({"path": "safe.txt", "content": "inside"}), &ctx)
            .await;
        assert!(inside.ok, "{}", inside.output);
        let read_in = reader.call(json!({"path": "safe.txt"}), &ctx).await;
        assert!(read_in.ok, "{}", read_in.output);
        assert!(read_in.output.contains("inside"));

        let outside = std::env::temp_dir().join(format!(
            "aegis-sandbox-out-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::write(&outside, "secret").unwrap();
        let outside_s = outside.to_string_lossy().to_string();

        let read_out = reader.call(json!({"path": outside_s.clone()}), &ctx).await;
        assert!(!read_out.ok, "sandbox must deny outside read");
        assert!(read_out.output.contains("permission denied"));

        let write_out = w
            .call(json!({"path": outside_s, "content": "overwrite"}), &ctx)
            .await;
        assert!(!write_out.ok, "sandbox must deny outside write");
        assert!(write_out.output.contains("permission denied"));
        // Original outside content must be untouched.
        assert_eq!(std::fs::read_to_string(&outside).unwrap(), "secret");

        let _ = std::fs::remove_file(&outside);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn read_offset_and_limit() {
        let dir = temp_dir();
        let ctx = ToolContext::new(dir.clone(), "test".into(), PermissionMode::Yolo);
        WriteFileTool
            .call(
                json!({"path": "lines.txt", "content": "a\nb\nc\nd\ne\n"}),
                &ctx,
            )
            .await;
        let out = ReadFileTool
            .call(json!({"path": "lines.txt", "offset": 2, "limit": 2}), &ctx)
            .await;
        assert!(out.ok, "{}", out.output);
        assert!(out.output.contains("b"));
        assert!(out.output.contains("c"));
        assert!(!out.output.contains("|a\n") && !out.output.contains("|a|"));
        // line numbers 2 and 3 only
        assert!(out.output.contains("     2|"));
        assert!(out.output.contains("     3|"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn read_missing_file_errors() {
        let dir = temp_dir();
        let ctx = ToolContext::new(dir.clone(), "test".into(), PermissionMode::Yolo);
        let out = ReadFileTool
            .call(json!({"path": "missing.txt"}), &ctx)
            .await;
        assert!(!out.ok);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn write_nested_path_creates_parents() {
        let dir = temp_dir();
        let ctx = ToolContext::new(dir.clone(), "test".into(), PermissionMode::Yolo);
        let r = WriteFileTool
            .call(
                json!({"path": "sub/dir/file.txt", "content": "nested"}),
                &ctx,
            )
            .await;
        assert!(r.ok, "{}", r.output);
        let out = ReadFileTool
            .call(json!({"path": "sub/dir/file.txt"}), &ctx)
            .await;
        assert!(out.ok);
        assert!(out.output.contains("nested"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
