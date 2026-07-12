use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use ignore::WalkBuilder;
use regex::RegexBuilder;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents with a regex. Respects .gitignore. Caps total matches."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string" },
                "path": { "type": "string", "description": "File or directory to search" },
                "case_insensitive": { "type": "boolean", "default": false },
                "max_matches": { "type": "integer", "default": 100 },
                "glob": { "type": "string", "description": "Optional file name glob filter" }
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
        let case_i = args
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let max = args
            .get("max_matches")
            .and_then(|v| v.as_u64())
            .unwrap_or(100) as usize;
        let root = args
            .get("path")
            .and_then(|v| v.as_str())
            .map(|p| ctx.resolve_path(p))
            .unwrap_or_else(|| ctx.cwd.clone());
        let name_glob = args
            .get("glob")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let re = match RegexBuilder::new(&pattern).case_insensitive(case_i).build() {
            Ok(r) => r,
            Err(e) => return ToolResult::err(format!("invalid regex: {e}")),
        };

        let result = tokio::task::spawn_blocking(move || {
            let mut hits = Vec::new();
            if root.is_file() {
                search_file(&root, &re, &mut hits, max);
                return hits;
            }
            let walker = WalkBuilder::new(&root)
                .hidden(false)
                .git_ignore(true)
                .build();
            for entry in walker.flatten() {
                if hits.len() >= max {
                    break;
                }
                if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    continue;
                }
                let path = entry.path();
                if let Some(ref g) = name_glob {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !simple_glob_match(g, name) {
                        continue;
                    }
                }
                // skip large / binary-ish
                if let Ok(meta) = entry.metadata() {
                    if meta.len() > 2_000_000 {
                        continue;
                    }
                }
                search_file(path, &re, &mut hits, max);
            }
            hits
        })
        .await;

        match result {
            Ok(hits) if hits.is_empty() => ToolResult::ok("(no matches)"),
            Ok(hits) => ToolResult::ok(hits.join("\n")),
            Err(e) => ToolResult::err(format!("join: {e}")),
        }
    }
}

fn search_file(path: &std::path::Path, re: &regex::Regex, hits: &mut Vec<String>, max: usize) {
    let Ok(data) = fs::read(path) else {
        return;
    };
    if data.iter().take(4096).any(|&b| b == 0) {
        return;
    }
    let Ok(text) = String::from_utf8(data) else {
        return;
    };
    for (i, line) in text.lines().enumerate() {
        if hits.len() >= max {
            break;
        }
        if re.is_match(line) {
            hits.push(format!("{}:{}:{}", path.display(), i + 1, line));
        }
    }
}

fn simple_glob_match(pat: &str, name: &str) -> bool {
    // very small * matcher
    if pat == "*" {
        return true;
    }
    if let Some(suf) = pat.strip_prefix('*') {
        return name.ends_with(suf);
    }
    if let Some(pre) = pat.strip_suffix('*') {
        return name.starts_with(pre);
    }
    name == pat
}

// silence unused
#[allow(dead_code)]
fn _pb() -> PathBuf {
    PathBuf::new()
}
