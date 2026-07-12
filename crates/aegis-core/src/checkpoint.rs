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
            // Prefer immutable stash commit SHA over drifting stash@{N} indices.
            if let Ok(sha) = run_git(root, &["rev-parse", "stash@{0}"]) {
                let sha = sha.trim().to_string();
                if !sha.is_empty() {
                    stash_ref = Some(sha);
                }
            }
            if stash_ref.is_none() {
                if let Ok(list) = run_git(root, &["stash", "list"]) {
                    if let Some(line) = list.lines().find(|l| l.contains(&msg)) {
                        stash_ref = line.split(':').next().map(|s| s.trim().to_string());
                    }
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
    let matches: Vec<_> = list
        .into_iter()
        .filter(|c| c.id == id || c.id.starts_with(id))
        .collect();
    let cp = match matches.len() {
        0 => bail!("checkpoint not found"),
        1 => matches.into_iter().next().unwrap(),
        _ => bail!(
            "ambiguous checkpoint id {id:?}; use full id (matches: {})",
            matches
                .iter()
                .map(|c| c.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    if let Some(ref stash) = cp.stash_ref {
        // Accept SHA or stash@{N} ref stored at create time.
        if run_git_status(root, &["stash", "apply", stash]).unwrap_or(false) {
            return Ok(format!(
                "restored stash {} from checkpoint {} (best-effort re-apply; not a hard reset)",
                &stash[..8.min(stash.len())],
                cp.id
            ));
        }
        bail!("stash apply failed for {stash}");
    }
    if let Some(ref commit) = cp.commit {
        return Ok(format!(
            "checkpoint {} recorded clean/HEAD {commit}; no stash to apply (no hard reset)",
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn git_init_with_commit(root: &Path) {
        let run = |args: &[&str]| {
            let st = Command::new("git")
                .args(args)
                .current_dir(root)
                .status()
                .expect("git");
            assert!(st.success(), "git {args:?}");
        };
        run(&["init"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(root.join("README.md"), "hi\n").unwrap();
        run(&["add", "README.md"]);
        run(&["commit", "-m", "init"]);
    }

    #[test]
    fn create_list_checkpoint_clean_tree() {
        let dir = tempdir().unwrap();
        git_init_with_commit(dir.path());
        let cp = create(dir.path(), "test-label").unwrap();
        assert!(!cp.id.is_empty());
        assert_eq!(cp.label, "test-label");
        assert!(cp.commit.is_some());
        assert!(dir
            .path()
            .join(".aegis/checkpoints")
            .join(format!("{}.json", cp.id))
            .exists());
        let listed = list(dir.path()).unwrap();
        assert!(listed.iter().any(|c| c.id == cp.id));
    }

    #[test]
    fn create_with_dirty_tree_may_stash() {
        let dir = tempdir().unwrap();
        git_init_with_commit(dir.path());
        std::fs::write(dir.path().join("dirty.txt"), "x\n").unwrap();
        let cp = create(dir.path(), "dirty").unwrap();
        assert!(dir
            .path()
            .join(".aegis/checkpoints")
            .join(format!("{}.status.txt", cp.id))
            .exists());
        // stash_ref is optional depending on git stash behavior; file must exist.
        assert!(!cp.id.is_empty());
    }

    #[test]
    fn restore_missing_and_ambiguous() {
        let dir = tempdir().unwrap();
        git_init_with_commit(dir.path());
        let err = restore(dir.path(), "no-such-id").unwrap_err();
        assert!(err.to_string().contains("not found"));

        // Two checkpoints with same second-precision id is unlikely; craft ambiguous
        // by writing two JSON files sharing a prefix.
        let d = checkpoint_dir(dir.path());
        std::fs::create_dir_all(&d).unwrap();
        for id in ["20260101_000001", "20260101_000002"] {
            let cp = Checkpoint {
                id: id.into(),
                created_at: "t".into(),
                label: "l".into(),
                stash_ref: None,
                commit: Some("abc".into()),
                note: "n".into(),
            };
            std::fs::write(
                d.join(format!("{id}.json")),
                serde_json::to_string(&cp).unwrap(),
            )
            .unwrap();
        }
        let amb = restore(dir.path(), "20260101").unwrap_err();
        assert!(amb.to_string().contains("ambiguous"), "{}", amb);
    }

    #[test]
    fn list_empty_without_dir() {
        let dir = tempdir().unwrap();
        let listed = list(dir.path()).unwrap();
        assert!(listed.is_empty());
    }
}
