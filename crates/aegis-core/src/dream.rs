//! Nightly dream: deep project self-improve and reflect.

use crate::readiness_v2::{assess_v2, format_report};
use crate::wiki;
use aegis_memory::{Lesson, ProjectMemory, RunReflection};
use aegis_xai::{
    system_msg, user_msg, CreateResponseRequest, ResponsesClient, TextConfig, TextFormat,
};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DreamOptions {
    pub apply_memory: bool,
    pub apply_code: bool,
    pub budget_model: String,
    pub max_proposals: usize,
    pub refresh_wiki: bool,
}

impl Default for DreamOptions {
    fn default() -> Self {
        Self {
            apply_memory: true,
            apply_code: false,
            budget_model: "grok-4.5".into(),
            max_proposals: 5,
            refresh_wiki: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DreamJournal {
    pub id: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub snapshot: String,
    pub readiness_level: u8,
    pub proposals: Vec<DreamProposal>,
    pub applied: Vec<String>,
    pub reflection_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamProposal {
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub priority: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct DreamLlmOutput {
    #[serde(default)]
    memory_stack: Option<String>,
    #[serde(default)]
    memory_commands: Option<String>,
    #[serde(default)]
    memory_gotchas: Option<String>,
    #[serde(default)]
    memory_conventions: Option<String>,
    #[serde(default)]
    new_lessons: Vec<DreamLesson>,
    #[serde(default)]
    proposals: Vec<DreamProposal>,
    #[serde(default)]
    skill_updates: Vec<DreamSkill>,
    #[serde(default)]
    summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DreamLesson {
    kind: String,
    summary: String,
    detail: String,
    #[serde(default)]
    confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct DreamSkill {
    name: String,
    body: String,
}

pub async fn run_dream(
    client: &ResponsesClient,
    project_root: &Path,
    opts: DreamOptions,
) -> Result<DreamJournal> {
    let lock = project_root.join(".aegis/dream.lock");
    fs::create_dir_all(project_root.join(".aegis"))?;
    if lock.exists() {
        // stale > 2h
        if let Ok(meta) = fs::metadata(&lock) {
            if let Ok(modified) = meta.modified() {
                if modified.elapsed().map(|d| d.as_secs() < 7200).unwrap_or(false) {
                    bail!("dream already running (lock: {})", lock.display());
                }
            }
        }
    }
    fs::write(&lock, Utc::now().to_rfc3339())?;

    let result = run_dream_inner(client, project_root, opts).await;
    let _ = fs::remove_file(&lock);
    result
}

async fn run_dream_inner(
    client: &ResponsesClient,
    project_root: &Path,
    opts: DreamOptions,
) -> Result<DreamJournal> {
    let started = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    info!(%id, "dream start");

    let mut memory = ProjectMemory::open(project_root)?;
    let readiness = assess_v2(project_root);
    let snapshot = build_snapshot(project_root, &readiness, &memory)?;

    // LLM consolidate + proposals
    let llm = dream_llm(client, &opts.budget_model, &snapshot).await?;

    let mut applied = Vec::new();
    if opts.apply_memory {
        memory.merge_memory_sections(
            llm.memory_stack.as_deref(),
            llm.memory_commands.as_deref(),
            llm.memory_gotchas.as_deref(),
            llm.memory_conventions.as_deref(),
        )?;
        applied.push("MEMORY.md sections".into());
        for l in &llm.new_lessons {
            memory.append_lesson(&Lesson {
                id: Uuid::new_v4().to_string(),
                ts: Utc::now().to_rfc3339(),
                kind: l.kind.clone(),
                summary: l.summary.clone(),
                detail: l.detail.clone(),
                tags: vec!["dream".into()],
                confidence: if l.confidence == 0.0 { 0.7 } else { l.confidence },
                hits: 1,
            })?;
        }
        if !llm.new_lessons.is_empty() {
            applied.push(format!("{} lessons", llm.new_lessons.len()));
        }
        for s in &llm.skill_updates {
            memory.write_skill(&s.name, &s.body)?;
            applied.push(format!("skill:{}", s.name));
        }
    }

    if opts.refresh_wiki {
        if let Ok(n) = wiki::generate_wiki(project_root, client, &opts.budget_model).await {
            applied.push(format!("wiki ({n} pages)"));
        }
    }

    // also run standard reflection on snapshot
    let reflection = RunReflection {
        wins: vec!["nightly dream completed".into()],
        stack_notes: llm.memory_stack.clone(),
        command_notes: llm.memory_commands.clone(),
        gotchas: llm.memory_gotchas.clone(),
        ..Default::default()
    };
    let _ = memory.merge_memory_sections(
        reflection.stack_notes.as_deref(),
        reflection.command_notes.as_deref(),
        reflection.gotchas.as_deref(),
        None,
    );

    let mut proposals = llm.proposals;
    proposals.sort_by_key(|p| std::cmp::Reverse(p.priority));
    proposals.truncate(opts.max_proposals);

    let journal = DreamJournal {
        id: id.clone(),
        started_at: started,
        finished_at: Some(Utc::now().to_rfc3339()),
        snapshot: snapshot.chars().take(8000).collect(),
        readiness_level: readiness.level,
        proposals: proposals.clone(),
        applied: applied.clone(),
        reflection_summary: llm.summary.clone(),
    };

    let dreams_dir = project_root.join(".aegis/dreams");
    fs::create_dir_all(&dreams_dir)?;
    let date = Utc::now().format("%Y-%m-%d_%H%M%S");
    let path = dreams_dir.join(format!("{date}.md"));
    fs::write(&path, render_journal_md(&journal))?;
    fs::write(
        dreams_dir.join(format!("{date}.json")),
        serde_json::to_string_pretty(&journal)?,
    )?;

    // metrics
    memory.metrics.run_count += 1;
    memory.metrics.last_run_id = Some(id);
    memory.metrics.last_run_at = Some(Utc::now().to_rfc3339());
    memory.save_metrics()?;

    Ok(journal)
}

fn build_snapshot(
    root: &Path,
    readiness: &crate::readiness_v2::ReadinessV2Report,
    memory: &ProjectMemory,
) -> Result<String> {
    let mut s = String::new();
    s.push_str(&format!("# Dream snapshot\nProject: {}\n\n", root.display()));
    s.push_str("## Readiness\n");
    s.push_str(&format_report(readiness));
    s.push_str("\n## Git\n");
    s.push_str(&run_capture(root, &["git", "log", "-5", "--oneline"]).unwrap_or_default());
    s.push_str("\n");
    s.push_str(&run_capture(root, &["git", "status", "-sb"]).unwrap_or_default());
    s.push_str("\n## Memory\n");
    s.push_str(&memory.read_memory_md().unwrap_or_default().chars().take(4000).collect::<String>());
    s.push_str("\n## Lessons (recent)\n");
    if let Ok(lessons) = memory.load_lessons() {
        for l in lessons.iter().rev().take(15) {
            s.push_str(&format!("- [{}] {}\n", l.kind, l.summary));
        }
    }
    s.push_str("\n## Failures\n");
    if let Ok(f) = memory.load_failures() {
        for x in f.iter().rev().take(10) {
            s.push_str(&format!("- {} → {}\n", x.pattern, x.fix));
        }
    }
    // open PRs if gh
    if let Some(prs) = run_capture(root, &["gh", "pr", "list", "--limit", "5"]) {
        s.push_str("\n## Open PRs\n");
        s.push_str(&prs);
    }
    Ok(s)
}

async fn dream_llm(client: &ResponsesClient, model: &str, snapshot: &str) -> Result<DreamLlmOutput> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "memory_stack": { "type": ["string", "null"] },
            "memory_commands": { "type": ["string", "null"] },
            "memory_gotchas": { "type": ["string", "null"] },
            "memory_conventions": { "type": ["string", "null"] },
            "new_lessons": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string" },
                        "summary": { "type": "string" },
                        "detail": { "type": "string" },
                        "confidence": { "type": "number" }
                    },
                    "required": ["kind", "summary", "detail", "confidence"],
                    "additionalProperties": false
                }
            },
            "proposals": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string" },
                        "title": { "type": "string" },
                        "detail": { "type": "string" },
                        "priority": { "type": "integer" }
                    },
                    "required": ["kind", "title", "detail", "priority"],
                    "additionalProperties": false
                }
            },
            "skill_updates": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "body": { "type": "string" }
                    },
                    "required": ["name", "body"],
                    "additionalProperties": false
                }
            },
            "summary": { "type": "string" }
        },
        "required": ["memory_stack", "memory_commands", "memory_gotchas", "memory_conventions",
            "new_lessons", "proposals", "skill_updates", "summary"],
        "additionalProperties": false
    });

    let req = CreateResponseRequest {
        model: model.into(),
        input: vec![
            system_msg(
                "You are Aegis Nightly Dream. Consolidate project knowledge, extract durable lessons, \
                 propose small high-value improvements (docs, tests, skills). No secrets. Prefer memory over large refactors.",
            ),
            user_msg(format!(
                "Nightly dream over this project snapshot:\n\n{}",
                snapshot.chars().take(24000).collect::<String>()
            )),
        ],
        tools: None,
        tool_choice: None,
        previous_response_id: None,
        store: Some(false),
        stream: Some(false),
        temperature: Some(0.3),
        max_output_tokens: Some(8192),
        parallel_tool_calls: None,
        text: Some(TextConfig {
            format: TextFormat::JsonSchema {
                name: "dream_output".into(),
                schema,
                strict: Some(true),
            },
        }),
        include: None,
    };

    let resp = client.create(req).await.context("dream LLM")?;
    let text = resp.output_text();
    let text = extract_json(&text).unwrap_or(text);
    serde_json::from_str(&text).context("parse dream output")
}

fn render_journal_md(j: &DreamJournal) -> String {
    let mut s = format!(
        "# Dream {}\n\nStarted: {}\nFinished: {:?}\nReadiness level: {}\n\n## Summary\n{}\n\n## Applied\n",
        j.id, j.started_at, j.finished_at, j.readiness_level, j.reflection_summary
    );
    for a in &j.applied {
        s.push_str(&format!("- {a}\n"));
    }
    s.push_str("\n## Proposals\n");
    for p in &j.proposals {
        s.push_str(&format!(
            "### [{}] {} (p{})\n{}\n\n",
            p.kind, p.title, p.priority, p.detail
        ));
    }
    s
}

fn run_capture(cwd: &Path, args: &[&str]) -> Option<String> {
    let (cmd, rest) = args.split_first()?;
    let out = Command::new(cmd).args(rest).current_dir(cwd).output().ok()?;
    if !out.status.success() && out.stdout.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

fn extract_json(text: &str) -> Option<String> {
    let t = text.trim();
    if t.starts_with('{') {
        return Some(t.to_string());
    }
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    if end > start {
        Some(t[start..=end].to_string())
    } else {
        None
    }
}

/// Install user crontab entry for nightly dream.
pub fn install_dream_cron(project_root: &Path, hour: u8) -> Result<String> {
    let aegis = which_aegis();
    let script_dir = project_root.join("scripts");
    fs::create_dir_all(&script_dir)?;
    let script = script_dir.join("aegis-dream.sh");
    let body = format!(
        "#!/usr/bin/env bash\nset -euo pipefail\ncd \"{}\"\n\"{}\" --cwd \"{}\" dream --apply 2>&1 | tee -a \"{}\"\n",
        project_root.display(),
        aegis.display(),
        project_root.display(),
        project_root.join(".aegis/dreams/cron.log").display()
    );
    fs::write(&script, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
    }
    let line = format!("0 {hour} * * * {}\n", script.display());
    // append if missing
    let existing = Command::new("crontab").arg("-l").output();
    let mut cron = existing
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    if !cron.contains("aegis-dream.sh") {
        cron.push_str(&line);
        let mut child = Command::new("crontab")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("crontab")?;
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(cron.as_bytes())?;
        }
        let status = child.wait()?;
        if !status.success() {
            bail!("failed to install crontab");
        }
    }
    // also write automation file
    let auto_dir = project_root.join(".aegis/automations");
    fs::create_dir_all(&auto_dir)?;
    fs::write(
        auto_dir.join("nightly-dream.toml"),
        format!(
            "name = \"nightly-dream\"\ntrigger = \"schedule\"\ncron = \"0 {hour} * * *\"\ncommand = \"dream\"\nargs = [\"--apply\"]\nenabled = true\nstage = \"monitor\"\n"
        ),
    )?;
    Ok(format!("Installed cron: 0 {hour} * * * {}", script.display()))
}

pub fn which_aegis_pub() -> PathBuf {
    which_aegis()
}

fn which_aegis() -> PathBuf {
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let p = PathBuf::from(dir).join("aegis");
            if p.is_file() {
                return p;
            }
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".cargo/bin/aegis")
}
