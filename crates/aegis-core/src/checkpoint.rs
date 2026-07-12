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
    root.join(".aegis/checkpoints")
}

/// Create a lightweight checkpoint: prefer `git stash push --include-untracked` message ref,
/// also record HEAD.
pub fn create(root: &Path, label: &str) -> Result<Checkpoint> {
    let root = root
        .canonicalize()
        .with_context(|| format!("canonicalize {}", root.display()))?;
    let dir = checkpoint_dir(&root);
    fs::create_dir_all(&dir)
        .with_context(|| format!("mkdir {}", dir.display()))?;
    let id = format!("{}", Utc::now().format("%Y%m%d_%H%M%S"));
    let head = git_out(&root, &["rev-parse", "HEAD"]).ok();
    let mut stash_ref = None;

    // Snapshot via git write-tree / commit-tree style is heavy; use a simple tar of tracked+dirty files
    // Prefer: save status snapshot file, optional stash
    let status = git_out(&root, &["status", "--porcelain"]).unwrap_or_default();
    let dirty = !status.trim().is_empty();
    if dirty {
        let msg = format!("aegis-checkpoint-{id}-{label}");
        let stash_out = Command::new("/usr/bin/git")
            .args(["stash", "push", "-u", "-m", &msg])
            .current_dir(&root)
            .output();
        match stash_out {
            Ok(out) if out.status.success() => {
                if let Ok(list) = git_out(&root, &["stash", "list"]) {
                    if let Some(line) = list.lines().find(|l| l.contains(&msg)) {
                        stash_ref = line.split(':').next().map(|s| s.to_string());
                    }
                }
                if stash_ref.is_some() {
                    let _ = Command::new("/usr/bin/git")
                        .args(["stash", "apply", "stash@{0}"])
                        .current_dir(&root)
                        .output();
                }
            }
            Ok(out) => {
                // stash failed (e.g. nothing to stash edge) — still write checkpoint meta
                let _ = String::from_utf8_lossy(&out.stderr);
            }
            Err(_) => {}
        }
    }

    // Always write a status snapshot for rollback context
    let _ = fs::write(dir.join(format!("{id}.status.txt")), &status);

    let cp = Checkpoint {
        id: id.clone(),
        created_at: Utc::now().to_rfc3339(),
        label: label.into(),
        stash_ref,
        commit: head,
        note: "Use `aegis checkpoint restore <id>` to attempt rollback".into(),
    };
    let path = dir.join(format!("{id}.json"));
    fs::write(&path, serde_json::to_string_pretty(&cp)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(cp)
}

pub fn list(root: &Path) -> Result<Vec<Checkpoint>> {
    let dir = checkpoint_dir(root);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out: Vec<Checkpoint> = Vec::new();
    for e in fs::read_dir(dir)? {
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
        let out = Command::new("/usr/bin/git")
            .args(["stash", "apply", stash])
            .current_dir(root)
            .output()
            .context("stash apply")?;
        if !out.status.success() {
            bail!(
                "stash apply failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        return Ok(format!("restored stash {stash} from checkpoint {}", cp.id));
    }
    if let Some(ref commit) = cp.commit {
        // hard reset is dangerous — only soft note
        return Ok(format!(
            "checkpoint {} had clean tree at commit {commit}; no stash to apply. Use git carefully if needed.",
            cp.id
        ));
    }
    Ok(format!("checkpoint {} had nothing to restore", cp.id))
}

fn git_out(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("/usr/bin/git")
        .args(args)
        .current_dir(root)
        .env(
            "PATH",
            format!(
                "{}:/usr/bin:/bin:{}",
                std::env::var("PATH").unwrap_or_default(),
                std::env::var("HOME").map(|h| format!("{h}/.local/bin")).unwrap_or_default()
            ),
        )
        .output()
        .with_context(|| format!("spawn git {:?} in {}", args, root.display()))?;
    if !out.status.success() {
        bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
