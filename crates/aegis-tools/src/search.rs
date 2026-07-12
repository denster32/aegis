use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use ignore::WalkBuilder;
use serde_json::{json, Value};
use std::path::Path;

pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern under the workspace (respects .gitignore)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Glob pattern, e.g. **/*.rs" },
                "path": { "type": "string", "description": "Root directory (default: workspace)" },
                "max_results": { "type": "integer", "default": 200 }
            },
            "required": ["pattern"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let pattern = match args.get("pattern").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => return ToolResult::err("missing pattern"),
        };
        let root = args
            .get("path")
            .and_then(|v| v.as_str())
            .map(|p| ctx.resolve_path(p))
            .unwrap_or_else(|| ctx.cwd.clone());
        let max = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(200) as usize;

        let glob = match globset_from(&pattern) {
            Ok(g) => g,
            Err(e) => return ToolResult::err(e),
        };

        let root_clone = root.clone();
        let matches = tokio::task::spawn_blocking(move || {
            let mut out = Vec::new();
            let walker = WalkBuilder::new(&root_clone)
                .hidden(false)
                .git_ignore(true)
                .build();
            for entry in walker.flatten() {
                if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    continue;
                }
                let path = entry.path();
                let rel = path
                    .strip_prefix(&root_clone)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                if glob.is_match(Path::new(&rel)) || glob.is_match(path) {
                    out.push(rel);
                    if out.len() >= max {
                        break;
                    }
                }
            }
            out
        })
        .await;

        match matches {
            Ok(list) => {
                if list.is_empty() {
                    ToolResult::ok("(no matches)")
                } else {
                    ToolResult::ok(list.join("\n"))
                }
            }
            Err(e) => ToolResult::err(format!("join: {e}")),
        }
    }
}

fn globset_from(pattern: &str) -> Result<ignore::gitignore::Gitignore, String> {
    // Use Gitignore builder for simple glob matching of patterns relative to root
    let mut builder = ignore::gitignore::GitignoreBuilder::new("");
    builder
        .add_line(None, pattern)
        .map_err(|e| e.to_string())?;
    // Also accept **/pattern if bare
    if !pattern.contains('/') && !pattern.starts_with("**") {
        let _ = builder.add_line(None, &format!("**/{pattern}"));
    }
    builder.build().map_err(|e| e.to_string())
}

// Gitignore::is_match returns Match — wrap
trait GlobMatch {
    fn is_match(&self, path: &Path) -> bool;
}

impl GlobMatch for ignore::gitignore::Gitignore {
    fn is_match(&self, path: &Path) -> bool {
        matches!(
            self.matched(path, false),
            ignore::Match::Ignore(_)
        )
    }
}
