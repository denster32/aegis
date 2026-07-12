use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

/// Structured mission graph produced by Grok 4.5 (boss).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionGraph {
    pub goal: String,
    #[serde(default)]
    pub tasks: Vec<TaskNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Prefer cheaper model unless true.
    #[serde(default)]
    pub needs_reasoning: bool,
    #[serde(default)]
    pub model_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub goal: String,
    pub milestones: Vec<Milestone>,
    #[serde(default)]
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub title: String,
    #[serde(default)]
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub passed: bool,
    pub checks: Vec<CheckResult>,
    #[serde(default)]
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    #[serde(default)]
    pub detail: String,
}

#[derive(Debug, Error)]
pub enum DagError {
    #[error("empty task list")]
    Empty,
    #[error("duplicate task id: {0}")]
    DuplicateId(String),
    #[error("unknown dependency {dep} on task {task}")]
    UnknownDep { task: String, dep: String },
    #[error("cycle detected involving task {0}")]
    Cycle(String),
}

impl MissionGraph {
    pub fn validate(&self) -> Result<(), DagError> {
        if self.tasks.is_empty() {
            return Err(DagError::Empty);
        }
        let mut seen = HashSet::new();
        for t in &self.tasks {
            if !seen.insert(t.id.clone()) {
                return Err(DagError::DuplicateId(t.id.clone()));
            }
        }
        for t in &self.tasks {
            for d in &t.depends_on {
                if !seen.contains(d) {
                    return Err(DagError::UnknownDep {
                        task: t.id.clone(),
                        dep: d.clone(),
                    });
                }
            }
        }
        // cycle check via Kahn
        let _ = self.topo_order()?;
        Ok(())
    }

    pub fn topo_order(&self) -> Result<Vec<String>, DagError> {
        let mut indeg: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for t in &self.tasks {
            indeg.entry(t.id.clone()).or_insert(0);
            for d in &t.depends_on {
                adj.entry(d.clone()).or_default().push(t.id.clone());
                *indeg.entry(t.id.clone()).or_insert(0) += 1;
            }
        }
        let mut q: VecDeque<String> = indeg
            .iter()
            .filter(|(_, &d)| d == 0)
            .map(|(k, _)| k.clone())
            .collect();
        let mut order = Vec::new();
        while let Some(n) = q.pop_front() {
            order.push(n.clone());
            if let Some(nexts) = adj.get(&n) {
                for m in nexts {
                    if let Some(e) = indeg.get_mut(m) {
                        *e -= 1;
                        if *e == 0 {
                            q.push_back(m.clone());
                        }
                    }
                }
            }
        }
        if order.len() != self.tasks.len() {
            let bad = self
                .tasks
                .iter()
                .map(|t| t.id.clone())
                .find(|id| !order.contains(id))
                .unwrap_or_else(|| "unknown".into());
            return Err(DagError::Cycle(bad));
        }
        Ok(order)
    }

    pub fn ready_tasks(&self, done: &HashSet<String>, running: &HashSet<String>) -> Vec<&TaskNode> {
        self.tasks
            .iter()
            .filter(|t| {
                !done.contains(&t.id)
                    && !running.contains(&t.id)
                    && t.depends_on.iter().all(|d| done.contains(d))
            })
            .collect()
    }

    pub fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goal": { "type": "string" },
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "title": { "type": "string" },
                            "description": { "type": "string" },
                            "depends_on": {
                                "type": "array",
                                "items": { "type": "string" }
                            },
                            "needs_reasoning": { "type": "boolean" },
                            "model_hint": { "type": ["string", "null"] }
                        },
                        "required": ["id", "title", "depends_on", "needs_reasoning"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["goal", "tasks"],
            "additionalProperties": false
        })
    }

    pub fn plan_json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goal": { "type": "string" },
                "milestones": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": { "type": "string" },
                            "steps": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        },
                        "required": ["title", "steps"],
                        "additionalProperties": false
                    }
                },
                "risks": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": ["goal", "milestones", "risks"],
            "additionalProperties": false
        })
    }

    pub fn validation_json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "passed": { "type": "boolean" },
                "checks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "passed": { "type": "boolean" },
                            "detail": { "type": "string" }
                        },
                        "required": ["name", "passed", "detail"],
                        "additionalProperties": false
                    }
                },
                "next_actions": {
                    "type": "array",
                    "items": { "type": "string" }
                }
            },
            "required": ["passed", "checks", "next_actions"],
            "additionalProperties": false
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topo_simple() {
        let g = MissionGraph {
            goal: "t".into(),
            tasks: vec![
                TaskNode {
                    id: "a".into(),
                    title: "A".into(),
                    description: String::new(),
                    depends_on: vec![],
                    needs_reasoning: false,
                    model_hint: None,
                },
                TaskNode {
                    id: "b".into(),
                    title: "B".into(),
                    description: String::new(),
                    depends_on: vec!["a".into()],
                    needs_reasoning: false,
                    model_hint: None,
                },
            ],
        };
        g.validate().unwrap();
        assert_eq!(g.topo_order().unwrap(), vec!["a", "b"]);
    }
}
