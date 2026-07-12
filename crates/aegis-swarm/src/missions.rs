//! Factory-inspired Missions: collaborative plans, features, milestones, Mission Control.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionPlan {
    pub id: String,
    pub goal: String,
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default)]
    pub milestones: Vec<MissionMilestone>,
    #[serde(default)]
    pub success_criteria: Vec<String>,
    #[serde(default)]
    pub skill_map: Vec<SkillBinding>,
    #[serde(default)]
    pub status: MissionStatus,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    #[default]
    Draft,
    Approved,
    Running,
    Blocked,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub milestone_id: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub skill_hints: Vec<String>,
    #[serde(default)]
    pub status: FeatureStatus,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureStatus {
    #[default]
    Pending,
    InProgress,
    Done,
    Blocked,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionMilestone {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub validation: Vec<String>,
    #[serde(default)]
    pub status: FeatureStatus,
    #[serde(default)]
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBinding {
    pub feature_id: String,
    pub skill_name: String,
    #[serde(default)]
    pub create_if_missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MissionState {
    pub mission_id: String,
    pub current_milestone: Option<String>,
    pub current_feature: Option<String>,
    #[serde(default)]
    pub completed_features: Vec<String>,
    #[serde(default)]
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Error)]
pub enum MissionError {
    #[error("mission not found: {0}")]
    NotFound(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

pub fn missions_root(project_root: &Path) -> PathBuf {
    project_root.join(".aegis").join("missions")
}

pub fn mission_dir(project_root: &Path, id: &str) -> PathBuf {
    missions_root(project_root).join(id)
}

impl MissionPlan {
    pub fn new(goal: impl Into<String>) -> Self {
        let now = chrono_lite_now();
        Self {
            id: Uuid::new_v4().to_string(),
            goal: goal.into(),
            features: vec![],
            milestones: vec![],
            success_criteria: vec![],
            skill_map: vec![],
            status: MissionStatus::Draft,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn plan_json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goal": { "type": "string" },
                "features": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "title": { "type": "string" },
                            "description": { "type": "string" },
                            "milestone_id": { "type": ["string", "null"] },
                            "depends_on": { "type": "array", "items": { "type": "string" } },
                            "skill_hints": { "type": "array", "items": { "type": "string" } }
                        },
                        "required": ["id", "title", "description", "milestone_id", "depends_on", "skill_hints"],
                        "additionalProperties": false
                    }
                },
                "milestones": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "title": { "type": "string" },
                            "description": { "type": "string" },
                            "validation": { "type": "array", "items": { "type": "string" } },
                            "order": { "type": "integer" }
                        },
                        "required": ["id", "title", "description", "validation", "order"],
                        "additionalProperties": false
                    }
                },
                "success_criteria": { "type": "array", "items": { "type": "string" } },
                "skill_map": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "feature_id": { "type": "string" },
                            "skill_name": { "type": "string" },
                            "create_if_missing": { "type": "boolean" }
                        },
                        "required": ["feature_id", "skill_name", "create_if_missing"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["goal", "features", "milestones", "success_criteria", "skill_map"],
            "additionalProperties": false
        })
    }

    pub fn save(&self, project_root: &Path) -> Result<PathBuf, MissionError> {
        let dir = mission_dir(project_root, &self.id);
        fs::create_dir_all(&dir)?;
        fs::create_dir_all(dir.join("handoffs"))?;
        fs::create_dir_all(dir.join("evidence"))?;
        let path = dir.join("plan.json");
        fs::write(&path, serde_json::to_string_pretty(self)?)?;
        let state = MissionState {
            mission_id: self.id.clone(),
            current_milestone: self
                .milestones
                .iter()
                .min_by_key(|m| m.order)
                .map(|m| m.id.clone()),
            current_feature: None,
            completed_features: vec![],
            blocked_reason: None,
        };
        fs::write(
            dir.join("state.json"),
            serde_json::to_string_pretty(&state)?,
        )?;
        if !dir.join("progress.jsonl").exists() {
            fs::write(dir.join("progress.jsonl"), "")?;
        }
        Ok(path)
    }

    pub fn load(project_root: &Path, id: &str) -> Result<Self, MissionError> {
        // allow prefix match
        let root = missions_root(project_root);
        if !root.exists() {
            return Err(MissionError::NotFound(id.into()));
        }
        let full = if root.join(id).join("plan.json").exists() {
            root.join(id)
        } else {
            let mut found = None;
            for e in fs::read_dir(&root)? {
                let e = e?;
                if e.file_name().to_string_lossy().starts_with(id) {
                    found = Some(e.path());
                    break;
                }
            }
            found.ok_or_else(|| MissionError::NotFound(id.into()))?
        };
        let plan: MissionPlan = serde_json::from_str(&fs::read_to_string(full.join("plan.json"))?)?;
        Ok(plan)
    }

    pub fn list(project_root: &Path) -> Result<Vec<MissionPlan>, MissionError> {
        let root = missions_root(project_root);
        if !root.exists() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        for e in fs::read_dir(root)? {
            let e = e?;
            let plan_path = e.path().join("plan.json");
            if plan_path.exists() {
                if let Ok(p) = serde_json::from_str::<MissionPlan>(&fs::read_to_string(plan_path)?)
                {
                    out.push(p);
                }
            }
        }
        out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(out)
    }

    /// Mission Control board — minimal monochrome (SpaceX / xAI).
    pub fn control_board(&self, state: &MissionState) -> String {
        let w = 56;
        let rule = "─".repeat(w);
        let mut s = String::new();
        s.push_str("MISSION CONTROL\n");
        s.push_str(&rule);
        s.push('\n');
        s.push_str(&format!(
            "  id          {}\n",
            &self.id[..8.min(self.id.len())]
        ));
        s.push_str(&format!("  status      {:?}\n", self.status));
        s.push_str(&format!("  goal        {}\n", truncate(&self.goal, 42)));
        s.push_str(&format!(
            "  milestone   {}\n",
            state.current_milestone.as_deref().unwrap_or("—")
        ));
        s.push_str(&rule);
        s.push('\n');
        s.push_str("MILESTONES\n");
        let mut miles = self.milestones.clone();
        miles.sort_by_key(|m| m.order);
        for m in &miles {
            let mark = match m.status {
                FeatureStatus::Done => "●",
                FeatureStatus::InProgress => "▸",
                FeatureStatus::Blocked => "×",
                _ => "·",
            };
            s.push_str(&format!(
                "  {}  {:>2}  {}\n",
                mark,
                m.order,
                truncate(&m.title, 40)
            ));
        }
        s.push_str(&rule);
        s.push('\n');
        s.push_str("FEATURES\n");
        for f in &self.features {
            let mark = match f.status {
                FeatureStatus::Done => "●",
                FeatureStatus::InProgress => "▸",
                FeatureStatus::Blocked => "×",
                FeatureStatus::Skipped => "–",
                FeatureStatus::Pending => "·",
            };
            s.push_str(&format!(
                "  {}  {:<10}  {}\n",
                mark,
                truncate(&f.id, 10),
                truncate(&f.title, 36)
            ));
        }
        if let Some(ref br) = state.blocked_reason {
            s.push_str(&rule);
            s.push('\n');
            s.push_str(&format!("  blocked     {}\n", truncate(br, 42)));
        }
        s.push_str(&rule);
        s.push('\n');
        s
    }

    /// Convert features into a swarm-style task list (id, title, deps).
    pub fn as_swarm_tasks(&self) -> Vec<(String, String, Vec<String>, String)> {
        self.features
            .iter()
            .map(|f| {
                (
                    f.id.clone(),
                    f.title.clone(),
                    f.depends_on.clone(),
                    f.description.clone(),
                )
            })
            .collect()
    }
}

pub fn load_state(project_root: &Path, id: &str) -> Result<MissionState, MissionError> {
    let plan = MissionPlan::load(project_root, id)?;
    let path = mission_dir(project_root, &plan.id).join("state.json");
    if path.exists() {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    } else {
        Ok(MissionState {
            mission_id: plan.id,
            ..Default::default()
        })
    }
}

pub fn save_state(project_root: &Path, state: &MissionState) -> Result<(), MissionError> {
    let path = mission_dir(project_root, &state.mission_id).join("state.json");
    fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

pub fn append_progress(
    project_root: &Path,
    mission_id: &str,
    event: serde_json::Value,
) -> Result<(), MissionError> {
    use std::io::Write;
    let path = mission_dir(project_root, mission_id).join("progress.jsonl");
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{}", serde_json::to_string(&event)?)?;
    Ok(())
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n.saturating_sub(1)])
    }
}

fn chrono_lite_now() -> String {
    // avoid chrono dep in swarm if not present — use simple UTC via std? use chrono from workspace
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

/// Project readiness score (Factory agent-readiness inspired, simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessReport {
    pub score: u8,
    pub level: String,
    pub checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCheck {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

pub fn assess_readiness(project_root: &Path) -> ReadinessReport {
    let mut checks = Vec::new();
    let has_git = project_root.join(".git").exists();
    checks.push(ReadinessCheck {
        name: "git_repo".into(),
        passed: has_git,
        detail: if has_git {
            "git present".into()
        } else {
            "no .git".into()
        },
    });
    let has_aegis = project_root.join(".aegis").exists();
    checks.push(ReadinessCheck {
        name: "aegis_dir".into(),
        passed: has_aegis,
        detail: if has_aegis {
            ".aegis present".into()
        } else {
            "run aegis once to create".into()
        },
    });
    let has_tests = project_root.join("tests").exists()
        || project_root.join("Cargo.toml").exists()
        || project_root.join("package.json").exists();
    checks.push(ReadinessCheck {
        name: "test_or_build_manifest".into(),
        passed: has_tests,
        detail: "Cargo.toml / package.json / tests/".into(),
    });
    let has_agents = project_root.join("AGENTS.md").exists()
        || project_root.join(".aegis/rules.md").exists()
        || project_root.join(".aegis/MEMORY.md").exists();
    checks.push(ReadinessCheck {
        name: "agent_docs".into(),
        passed: has_agents,
        detail: "AGENTS.md or .aegis memory/rules".into(),
    });
    let has_ci = project_root.join(".github/workflows").exists();
    checks.push(ReadinessCheck {
        name: "ci".into(),
        passed: has_ci,
        detail: if has_ci {
            "workflows present".into()
        } else {
            "optional CI".into()
        },
    });

    let passed = checks.iter().filter(|c| c.passed).count();
    let score = ((passed as f32 / checks.len() as f32) * 100.0) as u8;
    let level = match score {
        90..=100 => "L4 Optimized",
        70..=89 => "L3 Ready",
        40..=69 => "L2 Partial",
        _ => "L1 Basic",
    }
    .to_string();
    ReadinessReport {
        score,
        level,
        checks,
    }
}
