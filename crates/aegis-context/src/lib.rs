//! Project context packing for Grok-optimized prompts.

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_TREE_ENTRIES: usize = 200;
const MAX_RULE_CHARS: usize = 12_000;

/// Build a bootstrap context block for the workspace.
pub fn pack_workspace(cwd: &Path) -> String {
    pack_workspace_with_memory(cwd, true)
}

/// Pack workspace; when `include_memory` is true, inject project learning.
pub fn pack_workspace_with_memory(cwd: &Path, include_memory: bool) -> String {
    let mut parts = Vec::new();
    parts.push(format!("## Workspace\n{}", cwd.display()));

    if let Some(git) = git_status(cwd) {
        parts.push(format!("## Git\n{git}"));
    }

    if let Some(tree) = file_tree(cwd) {
        parts.push(format!("## File tree (capped)\n{tree}"));
    }

    for name in ["AGENTS.md", "CLAUDE.md", ".aegis/rules.md", "README.md"] {
        let path = cwd.join(name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let clipped = clip(&content, MAX_RULE_CHARS);
            parts.push(format!("## {name}\n{clipped}"));
        }
    }

    if include_memory {
        if let Ok(mem) = aegis_memory::ProjectMemory::open(cwd) {
            let block = aegis_memory::inject_memory_block(&mem, Some(6_000));
            if block.len() > 80 {
                parts.push(block);
            }
        }
    }

    parts.join("\n\n")
}

fn clip(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…\n[truncated {} chars]", &s[..max], s.len() - max)
    }
}

fn git_status(cwd: &Path) -> Option<String> {
    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !branch.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&branch.stdout).trim().to_string();
    let status = Command::new("git")
        .args(["status", "--short"])
        .current_dir(cwd)
        .output()
        .ok()?;
    let st = String::from_utf8_lossy(&status.stdout);
    let st = if st.len() > 4000 {
        format!("{}…", &st[..4000])
    } else {
        st.to_string()
    };
    Some(format!("branch: {branch}\n{st}"))
}

fn file_tree(cwd: &Path) -> Option<String> {
    let mut entries: Vec<String> = Vec::new();
    let walker = WalkBuilder::new(cwd)
        .hidden(false)
        .git_ignore(true)
        .max_depth(Some(4))
        .build();
    for entry in walker.flatten() {
        if entries.len() >= MAX_TREE_ENTRIES {
            entries.push("…".into());
            break;
        }
        let path = entry.path();
        if path == cwd {
            continue;
        }
        let rel = path.strip_prefix(cwd).unwrap_or(path);
        let s = rel.to_string_lossy().replace('\\', "/");
        // skip heavy dirs by name even if not gitignored
        if s.contains("node_modules") || s.contains("target/") || s.starts_with("target") {
            continue;
        }
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            entries.push(format!("{s}/"));
        } else {
            entries.push(s);
        }
    }
    if entries.is_empty() {
        None
    } else {
        Some(entries.join("\n"))
    }
}

/// Rough token estimate (~4 chars/token).
pub fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Build a compaction user message when context is large.
pub fn compaction_prompt(summary_so_far: &str, recent: &str) -> String {
    format!(
        "Summarize the work so far for context re-anchoring. Keep: goals, decisions, file paths touched, open tasks, failures.\n\
         Existing summary:\n{summary_so_far}\n\nRecent exchange:\n{recent}\n\n\
         Reply with a compact bullet summary only."
    )
}

pub fn resolve_project_config(cwd: &Path) -> Option<PathBuf> {
    let p = cwd.join(".aegis").join("config.toml");
    if p.exists() {
        Some(p)
    } else {
        None
    }
}
