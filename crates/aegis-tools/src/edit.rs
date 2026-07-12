use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;

pub struct EditFileTool;

#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Replace exact text in a file. old_string must be unique unless replace_all is true."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "old_string": { "type": "string" },
                "new_string": { "type": "string" },
                "replace_all": { "type": "boolean", "default": false }
            },
            "required": ["path", "old_string", "new_string"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => ctx.resolve_path(p),
            None => return ToolResult::err("missing path"),
        };
        let old = match args.get("old_string").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult::err("missing old_string"),
        };
        let new = match args.get("new_string").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult::err("missing new_string"),
        };
        let replace_all = args
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if old == new {
            return ToolResult::err("old_string and new_string are identical");
        }

        if !ctx.allow_path(&path, "edit") {
            return ToolResult::err("permission denied: edit outside workspace");
        }

        let lock = ctx.lock_path(&path).await;
        let _g = lock.lock().await;

        let content = match fs::read_to_string(&path).await {
            Ok(c) => c,
            Err(e) => return ToolResult::err(format!("read {}: {e}", path.display())),
        };

        let count = content.matches(old).count();
        if count == 0 {
            return ToolResult::err("old_string not found in file");
        }
        if count > 1 && !replace_all {
            return ToolResult::err(format!(
                "old_string found {count} times; set replace_all=true or provide a unique string"
            ));
        }

        let updated = if replace_all {
            content.replace(old, new)
        } else {
            content.replacen(old, new, 1)
        };

        match fs::write(&path, &updated).await {
            Ok(()) => ToolResult::ok(format!(
                "edited {} ({} replacement{})",
                path.display(),
                if replace_all { count } else { 1 },
                if replace_all && count != 1 { "s" } else { "" }
            )),
            Err(e) => ToolResult::err(format!("write: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{PermissionMode, ToolContext};

    fn yolo_ctx(dir: &std::path::Path) -> ToolContext {
        ToolContext::new(dir.to_path_buf(), "test".into(), PermissionMode::Yolo)
    }

    #[tokio::test]
    async fn edit_single_replacement() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f.txt"), "hello world").unwrap();
        let ctx = yolo_ctx(dir.path());
        let r = EditFileTool
            .call(
                json!({
                    "path": "f.txt",
                    "old_string": "world",
                    "new_string": "aegis"
                }),
                &ctx,
            )
            .await;
        assert!(r.ok, "{}", r.output);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("f.txt")).unwrap(),
            "hello aegis"
        );
    }

    #[tokio::test]
    async fn edit_replace_all() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f.txt"), "aa ba aa").unwrap();
        let ctx = yolo_ctx(dir.path());
        let r = EditFileTool
            .call(
                json!({
                    "path": "f.txt",
                    "old_string": "aa",
                    "new_string": "xx",
                    "replace_all": true
                }),
                &ctx,
            )
            .await;
        assert!(r.ok, "{}", r.output);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("f.txt")).unwrap(),
            "xx ba xx"
        );
    }

    #[tokio::test]
    async fn edit_requires_unique_without_replace_all() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f.txt"), "aa ba aa").unwrap();
        let ctx = yolo_ctx(dir.path());
        let r = EditFileTool
            .call(
                json!({
                    "path": "f.txt",
                    "old_string": "aa",
                    "new_string": "xx"
                }),
                &ctx,
            )
            .await;
        assert!(!r.ok);
        assert!(r.output.contains("2 times") || r.output.contains("replace_all"));
    }

    #[tokio::test]
    async fn edit_not_found_and_identical() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f.txt"), "only").unwrap();
        let ctx = yolo_ctx(dir.path());
        let missing = EditFileTool
            .call(
                json!({
                    "path": "f.txt",
                    "old_string": "nope",
                    "new_string": "x"
                }),
                &ctx,
            )
            .await;
        assert!(!missing.ok);
        let same = EditFileTool
            .call(
                json!({
                    "path": "f.txt",
                    "old_string": "only",
                    "new_string": "only"
                }),
                &ctx,
            )
            .await;
        assert!(!same.ok);
        assert!(same.output.contains("identical"));
    }
}
