use crate::failures::FailureRecord;
use crate::lessons::Lesson;
use crate::redact::redact_secrets;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ProjectPaths {
    pub root: PathBuf,
    pub aegis_dir: PathBuf,
    pub memory_md: PathBuf,
    pub lessons: PathBuf,
    pub failures: PathBuf,
    pub skills: PathBuf,
    pub runs: PathBuf,
    pub missions: PathBuf,
    pub metrics: PathBuf,
}

impl ProjectPaths {
    pub fn for_cwd(cwd: &Path) -> Self {
        let aegis_dir = cwd.join(".aegis");
        Self {
            root: cwd.to_path_buf(),
            memory_md: aegis_dir.join("MEMORY.md"),
            lessons: aegis_dir.join("LESSONS.jsonl"),
            failures: aegis_dir.join("FAILURES.jsonl"),
            skills: aegis_dir.join("SKILLS"),
            runs: aegis_dir.join("runs"),
            missions: aegis_dir.join("missions"),
            metrics: aegis_dir.join("metrics.json"),
            aegis_dir,
        }
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.aegis_dir)?;
        fs::create_dir_all(&self.skills)?;
        fs::create_dir_all(&self.runs)?;
        fs::create_dir_all(&self.missions)?;
        if !self.memory_md.exists() {
            fs::write(
                &self.memory_md,
                "# Project Memory\n\n## Stack\n\n## Commands\n\n## Gotchas\n\n## Conventions\n",
            )?;
        }
        if !self.lessons.exists() {
            fs::write(&self.lessons, "")?;
        }
        if !self.failures.exists() {
            fs::write(&self.failures, "")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMetrics {
    pub run_count: u64,
    pub heal_attempts: u64,
    pub heal_successes: u64,
    pub last_run_id: Option<String>,
    pub last_run_at: Option<String>,
}

pub struct ProjectMemory {
    pub paths: ProjectPaths,
    pub metrics: ProjectMetrics,
}

impl ProjectMemory {
    pub fn open(cwd: &Path) -> Result<Self> {
        let paths = ProjectPaths::for_cwd(cwd);
        paths.ensure()?;
        let metrics = if paths.metrics.exists() {
            serde_json::from_str(&fs::read_to_string(&paths.metrics)?).unwrap_or_default()
        } else {
            ProjectMetrics::default()
        };
        Ok(Self { paths, metrics })
    }

    pub fn read_memory_md(&self) -> Result<String> {
        Ok(fs::read_to_string(&self.paths.memory_md).unwrap_or_default())
    }

    pub fn write_memory_md(&self, content: &str) -> Result<()> {
        let content = redact_secrets(content);
        // size cap ~48k
        let clipped = if content.len() > 48_000 {
            format!("{}\n\n…[truncated for size]", &content[..48_000])
        } else {
            content
        };
        fs::write(&self.paths.memory_md, clipped)?;
        Ok(())
    }

    pub fn merge_memory_sections(
        &self,
        stack: Option<&str>,
        commands: Option<&str>,
        gotchas: Option<&str>,
        conventions: Option<&str>,
    ) -> Result<()> {
        let mut md = self.read_memory_md()?;
        for (heading, body) in [
            ("## Stack", stack),
            ("## Commands", commands),
            ("## Gotchas", gotchas),
            ("## Conventions", conventions),
        ] {
            if let Some(b) = body {
                if b.trim().is_empty() {
                    continue;
                }
                md = upsert_section(&md, heading, b.trim());
            }
        }
        self.write_memory_md(&md)
    }

    pub fn append_lesson(&self, lesson: &Lesson) -> Result<()> {
        append_jsonl(&self.paths.lessons, lesson)
    }

    pub fn append_failure(&self, failure: &FailureRecord) -> Result<()> {
        append_jsonl(&self.paths.failures, failure)
    }

    pub fn load_lessons(&self) -> Result<Vec<Lesson>> {
        load_jsonl(&self.paths.lessons)
    }

    pub fn load_failures(&self) -> Result<Vec<FailureRecord>> {
        load_jsonl(&self.paths.failures)
    }

    pub fn save_metrics(&mut self) -> Result<()> {
        fs::write(
            &self.paths.metrics,
            serde_json::to_string_pretty(&self.metrics)?,
        )?;
        Ok(())
    }

    pub fn record_run_start(&mut self, run_id: &str) -> Result<()> {
        self.metrics.run_count += 1;
        self.metrics.last_run_id = Some(run_id.into());
        self.metrics.last_run_at = Some(Utc::now().to_rfc3339());
        self.save_metrics()
    }

    pub fn write_run_summary(&self, run_id: &str, summary: &serde_json::Value) -> Result<()> {
        let path = self.paths.runs.join(format!("{run_id}.json"));
        fs::write(path, serde_json::to_string_pretty(summary)?)?;
        Ok(())
    }

    pub fn write_skill(&self, name: &str, body: &str) -> Result<PathBuf> {
        let safe: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let path = self.paths.skills.join(format!("{safe}.md"));
        fs::write(&path, redact_secrets(body))?;
        Ok(path)
    }

    pub fn list_skills(&self) -> Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        if !self.paths.skills.exists() {
            return Ok(out);
        }
        for e in fs::read_dir(&self.paths.skills)? {
            let e = e?;
            if e.path().extension().and_then(|x| x.to_str()) == Some("md") {
                let name = e
                    .path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("skill")
                    .to_string();
                let body = fs::read_to_string(e.path()).unwrap_or_default();
                out.push((name, body));
            }
        }
        Ok(out)
    }

    pub fn summary_report(&self) -> Result<String> {
        let lessons = self.load_lessons()?.len();
        let failures = self.load_failures()?.len();
        let skills = self.list_skills()?.len();
        let mem_len = self.read_memory_md()?.len();
        let last = self
            .metrics
            .last_run_at
            .clone()
            .unwrap_or_else(|| "—".into());
        // Monochrome key/value layout (SpaceX / xAI).
        Ok(format!(
            "MEMORY\n\
             ────────────────────────────────────────────────────────\n\
               project        {}\n\
               store          {}\n\
               runs           {}\n\
               lessons        {lessons}\n\
               failures       {failures}\n\
               skills         {skills}\n\
               memory.md      {mem_len} B\n\
               heal           {}/{}\n\
               last_run       {last}\n\
             ────────────────────────────────────────────────────────\n",
            self.paths.root.display(),
            self.paths.aegis_dir.display(),
            self.metrics.run_count,
            self.metrics.heal_successes,
            self.metrics.heal_attempts,
        ))
    }

    pub fn clear_lessons_failures(&self) -> Result<()> {
        fs::write(&self.paths.lessons, "")?;
        fs::write(&self.paths.failures, "")?;
        Ok(())
    }
}

fn append_jsonl<T: Serialize>(path: &Path, item: &T) -> Result<()> {
    let line = serde_json::to_string(item)?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(f, "{line}")?;
    Ok(())
}

fn load_jsonl<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str(line) {
            out.push(v);
        }
    }
    Ok(out)
}

fn upsert_section(md: &str, heading: &str, body: &str) -> String {
    if let Some(start) = md.find(heading) {
        let after = start + heading.len();
        let rest = &md[after..];
        let end = rest.find("\n## ").map(|i| after + i).unwrap_or(md.len());
        format!(
            "{}{}\n\n{}\n{}",
            &md[..start],
            heading,
            body,
            &md[end..].trim_start_matches('\n')
        )
    } else {
        format!("{}\n\n{}\n\n{}\n", md.trim_end(), heading, body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lessons::Lesson;

    #[test]
    fn open_and_lesson() {
        let dir = std::env::temp_dir().join(format!("aegis-mem-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mem = ProjectMemory::open(&dir).unwrap();
        let lesson = Lesson {
            id: "1".into(),
            ts: Utc::now().to_rfc3339(),
            kind: "convention".into(),
            summary: "use cargo test".into(),
            detail: "run from root".into(),
            tags: vec!["test".into()],
            confidence: 0.8,
            hits: 1,
        };
        mem.append_lesson(&lesson).unwrap();
        assert_eq!(mem.load_lessons().unwrap().len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }
}
