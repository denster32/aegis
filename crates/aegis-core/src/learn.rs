//! Self-heal and end-of-run reflection hooks.

use aegis_memory::{
    fingerprint, find_known_fix, reflection_json_schema, reflection_system_prompt, FailureRecord,
    Lesson, NewFailure, ProjectMemory, RunReflection,
};
use aegis_xai::{
    system_msg, user_msg, CreateResponseRequest, ResponsesClient, TextConfig, TextFormat,
};
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

pub struct LearnRuntime {
    pub memory: ProjectMemory,
    pub enabled: bool,
    pub run_id: String,
    heal_counts: HashMap<String, u32>,
    max_heal_per_fp: u32,
    transcript: Vec<String>,
}

impl LearnRuntime {
    pub fn open(cwd: &std::path::Path, enabled: bool) -> Result<Self> {
        let mut memory = ProjectMemory::open(cwd)?;
        let run_id = Uuid::new_v4().to_string();
        if enabled {
            memory.record_run_start(&run_id)?;
        }
        Ok(Self {
            memory,
            enabled,
            run_id,
            heal_counts: HashMap::new(),
            max_heal_per_fp: 2,
            transcript: Vec::new(),
        })
    }

    pub fn note(&mut self, line: impl Into<String>) {
        self.transcript.push(line.into());
        if self.transcript.len() > 200 {
            self.transcript.drain(0..50);
        }
    }

    /// On tool failure: return optional heal injection for the next model turn.
    pub fn on_tool_error(&mut self, tool: &str, error: &str) -> Option<String> {
        if !self.enabled {
            return None;
        }
        self.memory.metrics.heal_attempts += 1;
        let _ = self.memory.save_metrics();

        let fp = fingerprint(tool, error);
        let count = self.heal_counts.entry(fp.clone()).or_insert(0);
        *count += 1;
        if *count > self.max_heal_per_fp {
            return Some(format!(
                "Heal budget exhausted for this error fingerprint. Report to user and stop auto-retrying.\nTool: {tool}\nError: {}",
                truncate(error, 500)
            ));
        }

        let failures = self.memory.load_failures().unwrap_or_default();
        if let Some(known) = find_known_fix(&failures, &fp, 0.55) {
            info!(%fp, "known fix for tool error");
            return Some(format!(
                "SELF-HEAL (known project fix):\n\
                 Tool: {tool}\n\
                 Pattern: {}\n\
                 Root cause: {}\n\
                 Apply this fix first (minimal change):\n{}\n\
                 Then re-run the failing command/tool.",
                known.pattern, known.root_cause, known.fix
            ));
        }

        Some(format!(
            "SELF-HEAL (attempt {}):\n\
             Tool `{tool}` failed:\n{}\n\
             Diagnose root cause, apply a minimal fix, re-run verification. \
             If fixed, remember the pattern for this project.",
            count,
            truncate(error, 800)
        ))
    }

    pub fn record_successful_heal(&mut self, tool: &str, pattern: &str, fix: &str) {
        if !self.enabled {
            return;
        }
        self.memory.metrics.heal_successes += 1;
        let _ = self.memory.save_metrics();
        let fp = fingerprint(tool, pattern);
        let rec = FailureRecord {
            id: Uuid::new_v4().to_string(),
            ts: Utc::now().to_rfc3339(),
            fingerprint: fp,
            tool: tool.into(),
            pattern: pattern.into(),
            root_cause: "auto-healed".into(),
            fix: fix.into(),
            confidence: 0.7,
            hits: 1,
        };
        let _ = self.memory.append_failure(&rec);
        let lesson = Lesson {
            id: Uuid::new_v4().to_string(),
            ts: Utc::now().to_rfc3339(),
            kind: "fix".into(),
            summary: format!("heal {tool}"),
            detail: fix.into(),
            tags: vec!["heal".into(), tool.into()],
            confidence: 0.7,
            hits: 1,
        };
        let _ = self.memory.append_lesson(&lesson);
    }

    pub async fn reflect(
        &mut self,
        client: &ResponsesClient,
        model: &str,
    ) -> Result<RunReflection> {
        if !self.enabled {
            return Ok(RunReflection::default());
        }
        let summary = self.transcript.join("\n");
        let summary = if summary.len() > 12_000 {
            format!("…{}", &summary[summary.len() - 12_000..])
        } else {
            summary
        };

        let req = CreateResponseRequest {
            model: model.into(),
            input: vec![
                system_msg(reflection_system_prompt()),
                user_msg(format!(
                    "Project: {}\nRun: {}\n\nSession notes:\n{summary}",
                    self.memory.paths.root.display(),
                    self.run_id
                )),
            ],
            tools: None,
            tool_choice: None,
            previous_response_id: None,
            store: Some(false),
            stream: Some(false),
            temperature: Some(0.2),
            max_output_tokens: Some(4096),
            parallel_tool_calls: None,
            text: Some(TextConfig {
                format: TextFormat::JsonSchema {
                    name: "run_reflection".into(),
                    schema: reflection_json_schema(),
                    strict: Some(true),
                },
            }),
            include: None,
        };

        let text = match client.create(req).await {
            Ok(r) => r.output_text(),
            Err(e) => {
                warn!(error = %e, "reflection schema failed; skip");
                return Ok(RunReflection::default());
            }
        };
        let text = extract_json(&text).unwrap_or(text);
        let reflection: RunReflection = serde_json::from_str(&text).unwrap_or_default();
        self.apply_reflection(&reflection)?;
        Ok(reflection)
    }

    pub fn apply_reflection(&mut self, r: &RunReflection) -> Result<()> {
        self.memory.merge_memory_sections(
            r.stack_notes.as_deref(),
            r.command_notes.as_deref(),
            r.gotchas.as_deref(),
            None,
        )?;
        if !r.conventions_learned.is_empty() {
            let body = r
                .conventions_learned
                .iter()
                .map(|c| format!("- {c}"))
                .collect::<Vec<_>>()
                .join("\n");
            self.memory
                .merge_memory_sections(None, None, None, Some(&body))?;
        }

        for l in &r.new_lessons {
            let lesson = Lesson {
                id: Uuid::new_v4().to_string(),
                ts: Utc::now().to_rfc3339(),
                kind: l.kind.clone(),
                summary: l.summary.clone(),
                detail: l.detail.clone(),
                tags: l.tags.clone(),
                confidence: l.confidence,
                hits: 1,
            };
            self.memory.append_lesson(&lesson)?;
        }
        for f in &r.failure_records {
            self.record_failure_new(f)?;
        }
        for s in &r.skill_updates {
            self.memory.write_skill(&s.name, &s.body)?;
        }

        let summary = serde_json::json!({
            "run_id": self.run_id,
            "wins": r.wins,
            "failures": r.failures,
            "agents_md_suggestion": r.agents_md_suggestion,
        });
        self.memory.write_run_summary(&self.run_id, &summary)?;
        Ok(())
    }

    fn record_failure_new(&self, f: &NewFailure) -> Result<()> {
        let fp = fingerprint(&f.tool, &f.pattern);
        let rec = FailureRecord {
            id: Uuid::new_v4().to_string(),
            ts: Utc::now().to_rfc3339(),
            fingerprint: fp,
            tool: f.tool.clone(),
            pattern: f.pattern.clone(),
            root_cause: f.root_cause.clone(),
            fix: f.fix.clone(),
            confidence: f.confidence,
            hits: 1,
        };
        self.memory.append_failure(&rec)
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
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
