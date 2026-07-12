//! Aegis Nexus evolution: genes, mutation proposals, local fitness, and run persistence.
//!
//! Offline-friendly types and scoring first; `EvolutionEngine::propose` calls Grok via
//! the Responses API with a JSON schema for structured gene arrays.

use aegis_xai::{
    system_msg, user_msg, CreateResponseRequest, ResponsesClient, TextConfig, TextFormat,
};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Kind of evolutionary gene (mutation candidate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneKind {
    Plan,
    Diff,
    Test,
    Skill,
}

impl GeneKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Diff => "diff",
            Self::Test => "test",
            Self::Skill => "skill",
        }
    }
}

/// A single mutation candidate produced by the evolution loop.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Gene {
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: GeneKind,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub fitness_hints: Vec<String>,
}

impl Gene {
    /// Stable content fingerprint (sha256 hex) over kind/title/description/payload.
    pub fn content_hash(&self) -> String {
        let mut h = Sha256::new();
        h.update(self.kind.as_str().as_bytes());
        h.update(b"\0");
        h.update(self.title.as_bytes());
        h.update(b"\0");
        h.update(self.description.as_bytes());
        h.update(b"\0");
        if let Ok(bytes) = serde_json::to_vec(&self.payload) {
            h.update(&bytes);
        }
        hex::encode(h.finalize())
    }
}

/// Request to propose mutation genes for a goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationRequest {
    pub goal: String,
    pub max_genes: usize,
    #[serde(default)]
    pub context: String,
}

/// Local fitness evaluation result for one gene (score in \[0, 1\]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FitnessScore {
    pub gene_id: String,
    pub score: f32,
    pub passed_tests: bool,
    pub notes: String,
}

/// Observable signals used by the offline fitness function.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FitnessSignals {
    /// Project readiness percent, typically 0–100 (also accepts 0–1).
    pub readiness_pct: f32,
    /// Result of a cargo check/test when known.
    pub cargo_ok: Option<bool>,
    /// How many memory/lesson hits relate to this gene or goal.
    pub lesson_hits: u32,
}

/// One persisted evolution experiment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvolutionRun {
    pub id: String,
    pub goal: String,
    #[serde(default)]
    pub genes: Vec<Gene>,
    #[serde(default)]
    pub scores: Vec<FitnessScore>,
    pub created_at: String,
    pub best_gene_id: Option<String>,
}

impl EvolutionRun {
    pub fn new(goal: impl Into<String>, genes: Vec<Gene>, scores: Vec<FitnessScore>) -> Self {
        let best_gene_id = scores
            .iter()
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.gene_id.clone());
        Self {
            id: Uuid::new_v4().to_string(),
            goal: goal.into(),
            genes,
            scores,
            created_at: Utc::now().to_rfc3339(),
            best_gene_id,
        }
    }

    /// Recompute `best_gene_id` from current scores.
    pub fn recompute_best(&mut self) {
        self.best_gene_id = self
            .scores
            .iter()
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|s| s.gene_id.clone());
    }
}

/// Directory for evolution run JSON: `{project_root}/.aegis/nexus/evolution`.
pub fn evolution_dir(project_root: &Path) -> PathBuf {
    project_root.join(".aegis/nexus/evolution")
}

/// JSON Schema (object root) for Grok structured output producing an array of genes.
///
/// Root is an object with `genes` because Responses API structured outputs require
/// an object schema (not a bare array).
pub fn gene_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "genes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Stable short id for the gene (uuid or slug)"
                        },
                        "title": {
                            "type": "string",
                            "description": "Short human title"
                        },
                        "description": {
                            "type": "string",
                            "description": "What this mutation does and why"
                        },
                        "kind": {
                            "type": "string",
                            "enum": ["plan", "diff", "test", "skill"]
                        },
                        "payload": {
                            "type": "object",
                            "description": "Kind-specific body (plan steps, diff text, test plan, skill body)",
                            "additionalProperties": true
                        },
                        "fitness_hints": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Signals that should raise fitness if present offline"
                        }
                    },
                    "required": [
                        "id",
                        "title",
                        "description",
                        "kind",
                        "payload",
                        "fitness_hints"
                    ],
                    "additionalProperties": false
                }
            }
        },
        "required": ["genes"],
        "additionalProperties": false
    })
}

/// Offline fitness: map readiness, cargo status, lesson hits, and gene hints to \[0, 1\].
pub fn fitness_score_local(gene: &Gene, signals: &FitnessSignals) -> FitnessScore {
    // Normalize readiness: accept 0–1 or 0–100.
    let readiness = if signals.readiness_pct > 1.0 {
        (signals.readiness_pct / 100.0).clamp(0.0, 1.0)
    } else {
        signals.readiness_pct.clamp(0.0, 1.0)
    };

    let cargo_component: f32 = match signals.cargo_ok {
        Some(true) => 1.0,
        Some(false) => 0.0,
        None => 0.5,
    };

    // Diminishing returns on lesson hits (0 → 0, 1 → ~0.5, 4+ → ~1).
    let lesson_component = 1.0 - (-0.4 * signals.lesson_hits as f32).exp();
    let lesson_component = lesson_component.clamp(0.0, 1.0);

    // Hints: each non-empty hint adds a small boost (cap at 5).
    let hint_n = gene
        .fitness_hints
        .iter()
        .filter(|h| !h.trim().is_empty())
        .count();
    let hint_component = (hint_n as f32 / 5.0).clamp(0.0, 1.0);

    // Kind prior: tests/diffs that ship code slightly preferred when cargo is green.
    let kind_boost: f32 = match gene.kind {
        GeneKind::Test => 0.05,
        GeneKind::Diff => 0.04,
        GeneKind::Skill => 0.03,
        GeneKind::Plan => 0.02,
    };

    // Weighted blend.
    let mut score = 0.40 * readiness
        + 0.30 * cargo_component
        + 0.20 * lesson_component
        + 0.10 * hint_component
        + kind_boost;

    // Hard penalty if cargo explicitly failed.
    if signals.cargo_ok == Some(false) {
        score *= 0.55;
    }

    let score = score.clamp(0.0, 1.0);
    let passed_tests = signals.cargo_ok.unwrap_or(false) && score >= 0.45;

    let notes = format!(
        "readiness={:.2} cargo={:?} lessons={} hints={} kind={} hash={}",
        readiness,
        signals.cargo_ok,
        signals.lesson_hits,
        hint_n,
        gene.kind.as_str(),
        &gene.content_hash()[..12.min(gene.content_hash().len())]
    );

    FitnessScore {
        gene_id: gene.id.clone(),
        score,
        passed_tests,
        notes,
    }
}

/// Score every gene and build a new run (does not persist).
pub fn score_run(
    goal: impl Into<String>,
    genes: Vec<Gene>,
    signals: &FitnessSignals,
) -> EvolutionRun {
    let scores: Vec<FitnessScore> = genes
        .iter()
        .map(|g| fitness_score_local(g, signals))
        .collect();
    EvolutionRun::new(goal, genes, scores)
}

/// Persist run to `{project_root}/.aegis/nexus/evolution/{id}.json`.
pub fn save_run(project_root: &Path, run: &EvolutionRun) -> Result<PathBuf> {
    let dir = evolution_dir(project_root);
    fs::create_dir_all(&dir).with_context(|| format!("create evolution dir {}", dir.display()))?;
    let path = dir.join(format!("{}.json", run.id));
    let body = serde_json::to_string_pretty(run).context("serialize evolution run")?;
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

/// Load run from `{project_root}/.aegis/nexus/evolution/{id}.json`.
pub fn load_run(project_root: &Path, id: &str) -> Result<EvolutionRun> {
    let path = evolution_dir(project_root).join(format!("{id}.json"));
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let run: EvolutionRun = serde_json::from_str(&raw).context("parse evolution run")?;
    Ok(run)
}

/// List run ids present under the evolution directory (no extension).
pub fn list_run_ids(project_root: &Path) -> Result<Vec<String>> {
    let dir = evolution_dir(project_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut ids = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("read dir {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                ids.push(stem.to_string());
            }
        }
    }
    ids.sort();
    Ok(ids)
}

/// LLM-backed engine that proposes genes for a mutation request.
pub struct EvolutionEngine {
    client: ResponsesClient,
    model: String,
}

impl EvolutionEngine {
    pub fn new(client: ResponsesClient, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// Propose genes via Responses API + JsonSchema structured output.
    pub async fn propose(&self, req: &MutationRequest) -> Result<Vec<Gene>> {
        let max = req.max_genes.clamp(1, 16);
        let schema = gene_schema();

        let system = "You are Aegis Nexus Evolution. Propose concrete, high-value mutation genes \
            for the given coding-agent project goal. Prefer small, testable changes. \
            Each gene must have kind plan|diff|test|skill, a useful payload, and fitness_hints \
            that offline scoring can use. No secrets. Respond only with JSON matching the schema.";

        let user = format!(
            "Goal:\n{}\n\nMax genes: {}\n\nContext:\n{}",
            req.goal,
            max,
            if req.context.is_empty() {
                "(none)"
            } else {
                &req.context
            }
        );

        let api_req = CreateResponseRequest {
            model: self.model.clone(),
            input: vec![system_msg(system), user_msg(user)],
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
                    name: "evolution_genes".into(),
                    schema,
                    strict: Some(true),
                },
            }),
            include: None,
            reasoning: Some(aegis_xai::ReasoningConfig::medium()),
            prompt_cache_key: Some("aegis-evolution".into()),
        };

        let resp = self
            .client
            .create(api_req)
            .await
            .context("evolution propose LLM")?;
        let text = resp.output_text();
        let text = extract_json(&text).unwrap_or(text);
        let genes = parse_genes_payload(&text)?;
        Ok(genes.into_iter().take(max).collect())
    }
}

#[derive(Debug, Deserialize)]
struct GenesEnvelope {
    #[serde(default)]
    genes: Vec<Gene>,
}

fn parse_genes_payload(text: &str) -> Result<Vec<Gene>> {
    let t = text.trim();
    // Prefer envelope { "genes": [...] }
    if let Ok(env) = serde_json::from_str::<GenesEnvelope>(t) {
        if !env.genes.is_empty() || t.contains("\"genes\"") {
            return Ok(fill_gene_ids(env.genes));
        }
    }
    // Bare array fallback
    if let Ok(genes) = serde_json::from_str::<Vec<Gene>>(t) {
        return Ok(fill_gene_ids(genes));
    }
    bail!("could not parse genes from model output");
}

fn fill_gene_ids(mut genes: Vec<Gene>) -> Vec<Gene> {
    for g in &mut genes {
        if g.id.trim().is_empty() {
            g.id = Uuid::new_v4().to_string();
        }
    }
    genes
}

fn extract_json(text: &str) -> Option<String> {
    let t = text.trim();
    if t.starts_with('{') || t.starts_with('[') {
        return Some(t.to_string());
    }
    if let Some(start) = t.find('{') {
        if let Some(end) = t.rfind('}') {
            if end > start {
                return Some(t[start..=end].to_string());
            }
        }
    }
    if let Some(start) = t.find('[') {
        if let Some(end) = t.rfind(']') {
            if end > start {
                return Some(t[start..=end].to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_gene(kind: GeneKind, id: &str) -> Gene {
        Gene {
            id: id.into(),
            title: format!("Title {id}"),
            description: "desc".into(),
            kind,
            payload: json!({"steps": ["a", "b"]}),
            fitness_hints: vec!["cargo test".into(), "readiness".into()],
        }
    }

    #[test]
    fn gene_schema_is_valid_json_object() {
        let schema = gene_schema();
        assert!(schema.is_object(), "schema root must be object");
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["genes"].is_object());
        assert_eq!(schema["properties"]["genes"]["type"], "array");
        let items = &schema["properties"]["genes"]["items"];
        assert_eq!(items["type"], "object");
        let kinds = items["properties"]["kind"]["enum"]
            .as_array()
            .expect("kind enum");
        assert!(kinds.iter().any(|v| v == "plan"));
        assert!(kinds.iter().any(|v| v == "diff"));
        assert!(kinds.iter().any(|v| v == "test"));
        assert!(kinds.iter().any(|v| v == "skill"));
        assert_eq!(schema["required"][0], "genes");
        // Round-trip serialize
        let s = serde_json::to_string(&schema).unwrap();
        let back: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(back["type"], "object");
    }

    #[test]
    fn fitness_scoring_bounds_and_signals() {
        let gene = sample_gene(GeneKind::Test, "g1");

        let high = fitness_score_local(
            &gene,
            &FitnessSignals {
                readiness_pct: 90.0,
                cargo_ok: Some(true),
                lesson_hits: 5,
            },
        );
        assert!((0.0..=1.0).contains(&high.score));
        assert!(high.passed_tests);
        assert_eq!(high.gene_id, "g1");
        assert!(high.score > 0.7);

        let low = fitness_score_local(
            &gene,
            &FitnessSignals {
                readiness_pct: 10.0,
                cargo_ok: Some(false),
                lesson_hits: 0,
            },
        );
        assert!((0.0..=1.0).contains(&low.score));
        assert!(!low.passed_tests);
        assert!(low.score < high.score);

        // readiness already 0–1
        let mid = fitness_score_local(
            &Gene {
                fitness_hints: vec![],
                ..sample_gene(GeneKind::Plan, "g2")
            },
            &FitnessSignals {
                readiness_pct: 0.5,
                cargo_ok: None,
                lesson_hits: 1,
            },
        );
        assert!((0.0..=1.0).contains(&mid.score));
        assert!(!mid.notes.is_empty());
    }

    #[test]
    fn save_and_load_run_roundtrip() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let genes = vec![
            sample_gene(GeneKind::Diff, "a"),
            sample_gene(GeneKind::Skill, "b"),
        ];
        let signals = FitnessSignals {
            readiness_pct: 70.0,
            cargo_ok: Some(true),
            lesson_hits: 2,
        };
        let run = score_run("improve tests", genes, &signals);
        assert!(run.best_gene_id.is_some());

        let path = save_run(root, &run).unwrap();
        assert!(path.exists());
        assert!(path.ends_with(format!("{}.json", run.id)));

        let loaded = load_run(root, &run.id).unwrap();
        assert_eq!(loaded, run);

        let ids = list_run_ids(root).unwrap();
        assert_eq!(ids, vec![run.id.clone()]);
    }

    #[test]
    fn content_hash_stable() {
        let g1 = sample_gene(GeneKind::Plan, "x");
        let mut g2 = g1.clone();
        g2.id = "different-id".into();
        // id is not part of content hash
        assert_eq!(g1.content_hash(), g2.content_hash());
        let mut g3 = g1.clone();
        g3.title = "other".into();
        assert_ne!(g1.content_hash(), g3.content_hash());
        assert_eq!(g1.content_hash().len(), 64);
    }

    #[test]
    fn parse_genes_envelope_and_array() {
        let env = r#"{"genes":[{"id":"1","title":"t","description":"d","kind":"plan","payload":{},"fitness_hints":[]}]}"#;
        let genes = parse_genes_payload(env).unwrap();
        assert_eq!(genes.len(), 1);
        assert_eq!(genes[0].kind, GeneKind::Plan);

        let arr = r#"[{"id":"","title":"t","description":"d","kind":"test","payload":{"n":1},"fitness_hints":["h"]}]"#;
        let genes = parse_genes_payload(arr).unwrap();
        assert_eq!(genes.len(), 1);
        assert!(!genes[0].id.is_empty());
        assert_eq!(genes[0].kind, GeneKind::Test);
    }

    #[test]
    fn evolution_run_recompute_best() {
        let mut run = EvolutionRun {
            id: "r".into(),
            goal: "g".into(),
            genes: vec![],
            scores: vec![
                FitnessScore {
                    gene_id: "a".into(),
                    score: 0.2,
                    passed_tests: false,
                    notes: String::new(),
                },
                FitnessScore {
                    gene_id: "b".into(),
                    score: 0.9,
                    passed_tests: true,
                    notes: String::new(),
                },
            ],
            created_at: Utc::now().to_rfc3339(),
            best_gene_id: None,
        };
        run.recompute_best();
        assert_eq!(run.best_gene_id.as_deref(), Some("b"));
    }
}
