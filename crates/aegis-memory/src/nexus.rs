//! Nexus neural summary — distilled immune memory for long Missions.

use crate::project::ProjectMemory;
use crate::redact::redact_secrets;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NeuralSummary {
    pub version: u32,
    pub created_at: String,
    pub stack: String,
    pub conventions: Vec<String>,
    pub active_risks: Vec<String>,
    pub durable_lessons: Vec<String>,
    pub style_priors: Vec<String>,
    pub open_threads: Vec<String>,
    /// Free-form distillation from Grok.
    pub narrative: String,
}

impl NeuralSummary {
    pub fn path(root: &Path) -> PathBuf {
        root.join(".aegis/nexus/neural-summary.json")
    }

    pub fn load(root: &Path) -> Result<Option<Self>> {
        let p = Self::path(root);
        if !p.exists() {
            return Ok(None);
        }
        let s = fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
        Ok(Some(serde_json::from_str(&s)?))
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let p = Self::path(root);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut v = serde_json::to_value(self)?;
        if let Some(obj) = v.as_object_mut() {
            for key in ["stack", "narrative"] {
                if let Some(serde_json::Value::String(s)) = obj.get_mut(key) {
                    *s = redact_secrets(s);
                }
            }
            for key in [
                "conventions",
                "active_risks",
                "durable_lessons",
                "style_priors",
                "open_threads",
            ] {
                if let Some(serde_json::Value::Array(arr)) = obj.get_mut(key) {
                    for item in arr {
                        if let serde_json::Value::String(s) = item {
                            *s = redact_secrets(s);
                        }
                    }
                }
            }
        }
        fs::write(&p, serde_json::to_string_pretty(&v)?)?;
        Ok(())
    }

    /// Local offline distillation without Grok (fallback).
    pub fn from_project_local(mem: &ProjectMemory) -> Result<Self> {
        let lessons = mem.load_lessons().unwrap_or_default();
        let failures = mem.load_failures().unwrap_or_default();
        let memory_md = mem.read_memory_md().unwrap_or_default();
        let durable: Vec<String> = lessons
            .iter()
            .rev()
            .take(8)
            .map(|l| redact_secrets(&l.summary))
            .collect();
        let risks: Vec<String> = failures
            .iter()
            .rev()
            .take(6)
            .map(|f| redact_secrets(&format!("{} → {}", f.pattern, f.fix)))
            .collect();
        let narrative = redact_secrets(&format!(
            "Local neural summary. Memory {} chars, {} lessons, {} failure patterns.",
            memory_md.len(),
            lessons.len(),
            failures.len()
        ));
        Ok(Self {
            version: 1,
            created_at: Utc::now().to_rfc3339(),
            stack: first_stack_line(&memory_md),
            conventions: extract_bullets(&memory_md, 6),
            active_risks: risks,
            durable_lessons: durable,
            style_priors: vec![
                "Prefer minimal monochrome CLI output".into(),
                "Prefer verified cargo test before claim done".into(),
            ],
            open_threads: vec![],
            narrative,
        })
    }

    pub fn inject_block(&self, max_chars: usize) -> String {
        let mut s = String::from("## Nexus neural summary\n");
        s.push_str(&format!("updated: {}\n", self.created_at));
        if !self.stack.is_empty() {
            s.push_str(&format!("stack: {}\n", self.stack));
        }
        if !self.narrative.is_empty() {
            s.push_str(&format!("narrative: {}\n", self.narrative));
        }
        if !self.durable_lessons.is_empty() {
            s.push_str("lessons:\n");
            for l in &self.durable_lessons {
                s.push_str(&format!("- {l}\n"));
            }
        }
        if !self.active_risks.is_empty() {
            s.push_str("risks:\n");
            for r in &self.active_risks {
                s.push_str(&format!("- {r}\n"));
            }
        }
        if s.len() > max_chars {
            format!("{}…\n[truncated]", &s[..max_chars.saturating_sub(1)])
        } else {
            s
        }
    }
}

fn first_stack_line(md: &str) -> String {
    for line in md.lines() {
        let t = line.trim();
        if t.to_lowercase().contains("stack") || t.starts_with("- rust") || t.starts_with("Rust") {
            return redact_secrets(t).chars().take(120).collect();
        }
    }
    "unknown".into()
}

fn extract_bullets(md: &str, n: usize) -> Vec<String> {
    md.lines()
        .filter(|l| l.trim_start().starts_with('-'))
        .take(n)
        .map(|l| redact_secrets(l.trim().trim_start_matches('-').trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lessons::Lesson;
    use crate::project::ProjectMemory;
    use uuid::Uuid;

    #[test]
    fn local_summary_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mem = ProjectMemory::open(dir.path()).unwrap();
        mem.append_lesson(&Lesson {
            id: Uuid::new_v4().to_string(),
            ts: Utc::now().to_rfc3339(),
            kind: "command".into(),
            summary: "Always run cargo test".into(),
            detail: "Verify with cargo test before claiming done".into(),
            tags: vec!["test".into()],
            confidence: 0.9,
            hits: 1,
        })
        .unwrap();
        let sum = NeuralSummary::from_project_local(&mem).unwrap();
        sum.save(dir.path()).unwrap();
        let loaded = NeuralSummary::load(dir.path()).unwrap().unwrap();
        assert_eq!(loaded.version, 1);
        assert!(!loaded.durable_lessons.is_empty());
        let block = loaded.inject_block(2000);
        assert!(block.contains("Nexus"));
    }
}
