//! Git checkpoint / rollback for missions and dreams.

use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub created_at: String,
    pub label: String,
    pub stash_ref: Option<String>,
    pub commit: Option<String>,
    pub note: String,
}

pub fn checkpoint_dir(root: &Path) -> PathBuf {
    root.join(".aegis").join("checkpoints")
}

/// Create a lightweight checkpoint: HEAD + status snapshot + optional stash.
pub fn create(root: &Path, label: &str) -> Result<Checkpoint> {
    // Ensure parent .aegis exists first (explicit two-step avoids rare mkdir edge cases)
    let aegis = root.join(".aegis");
    fs::create_dir_all(&aegis).with_context(|| format!("mkdir {}", aegis.display()))?;
    let dir = aegis.join("checkpoints");
    fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;

    let id = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let head = run_git(root, &["rev-parse", "HEAD"]).ok();
    let status = run_git(root, &["status", "--porcelain"]).unwrap_or_default();
    let _ = fs::write(dir.join(format!("{id}.status.txt")), &status);

    let mut stash_ref = None;
    if !status.trim().is_empty() {
        let msg = format!("aegis-checkpoint-{id}-{label}");
        if run_git_status(root, &["stash", "push", "-u", "-m", &msg]).unwrap_or(false) {
            if let Ok(list) = run_git(root, &["stash", "list"]) {
                if let Some(line) = list.lines().find(|l| l.contains(&msg)) {
                    stash_ref = line.split(':').next().map(|s| s.trim().to_string());
                }
            }
            // restore working tree
            let _ = run_git_status(root, &["stash", "apply", "stash@{0}"]);
        }
    }

    let cp = Checkpoint {
        id: id.clone(),
        created_at: Utc::now().to_rfc3339(),
        label: label.into(),
        stash_ref,
        commit: head.map(|s| s.trim().to_string()),
        note: "Use `aegis checkpoint restore <id>` to attempt rollback".into(),
    };
    let path = dir.join(format!("{id}.json"));
    let json = serde_json::to_string_pretty(&cp)?;
    fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(cp)
}

pub fn list(root: &Path) -> Result<Vec<Checkpoint>> {
    let dir = checkpoint_dir(root);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut out: Vec<Checkpoint> = Vec::new();
    for e in fs::read_dir(&dir)? {
        let e = e?;
        if e.path().extension().and_then(|x| x.to_str()) == Some("json") {
            if let Ok(c) = serde_json::from_str::<Checkpoint>(&fs::read_to_string(e.path())?) {
                out.push(c);
            }
        }
    }
    out.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(out)
}

pub fn restore(root: &Path, id: &str) -> Result<String> {
    let list = list(root)?;
    let cp = list
        .into_iter()
        .find(|c| c.id == id || c.id.starts_with(id))
        .context("checkpoint not found")?;
    if let Some(ref stash) = cp.stash_ref {
        if run_git_status(root, &["stash", "apply", stash]).unwrap_or(false) {
            return Ok(format!("restored stash {stash} from checkpoint {}", cp.id));
        }
        bail!("stash apply failed for {stash}");
    }
    if let Some(ref commit) = cp.commit {
        return Ok(format!(
            "checkpoint {} recorded clean/HEAD {commit}; no stash to apply",
            cp.id
        ));
    }
    Ok(format!("checkpoint {} had nothing to restore", cp.id))
}

fn run_git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("git {:?}", args))?;
    if !out.status.success() {
        bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn run_git_status(root: &Path, args: &[&str]) -> Result<bool> {
    let out = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("git {:?}", args))?;
    Ok(out.status.success())
}
