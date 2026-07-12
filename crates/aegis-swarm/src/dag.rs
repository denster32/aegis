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

    fn task(id: &str, deps: &[&str]) -> TaskNode {
        TaskNode {
            id: id.into(),
            title: id.to_uppercase(),
            description: String::new(),
            depends_on: deps.iter().map(|s| (*s).to_string()).collect(),
            needs_reasoning: false,
            model_hint: None,
        }
    }

    #[test]
    fn topo_simple() {
        let g = MissionGraph {
            goal: "t".into(),
            tasks: vec![task("a", &[]), task("b", &["a"])],
        };
        g.validate().unwrap();
        assert_eq!(g.topo_order().unwrap(), vec!["a", "b"]);
    }

    #[test]
    fn validate_empty_graph() {
        let g = MissionGraph {
            goal: "x".into(),
            tasks: vec![],
        };
        assert!(matches!(g.validate(), Err(DagError::Empty)));
    }

    #[test]
    fn validate_duplicate_id() {
        let g = MissionGraph {
            goal: "x".into(),
            tasks: vec![task("a", &[]), task("a", &[])],
        };
        assert!(matches!(g.validate(), Err(DagError::DuplicateId(id)) if id == "a"));
    }

    #[test]
    fn validate_unknown_dep() {
        let g = MissionGraph {
            goal: "x".into(),
            tasks: vec![task("a", &["missing"])],
        };
        match g.validate() {
            Err(DagError::UnknownDep { task, dep }) => {
                assert_eq!(task, "a");
                assert_eq!(dep, "missing");
            }
            other => panic!("expected UnknownDep, got {other:?}"),
        }
    }

    #[test]
    fn cycle_detection() {
        let g = MissionGraph {
            goal: "cycle".into(),
            tasks: vec![task("a", &["b"]), task("b", &["a"])],
        };
        assert!(matches!(g.validate(), Err(DagError::Cycle(_))));
        assert!(matches!(g.topo_order(), Err(DagError::Cycle(_))));
    }

    #[test]
    fn self_cycle() {
        let g = MissionGraph {
            goal: "self".into(),
            tasks: vec![task("a", &["a"])],
        };
        assert!(matches!(g.topo_order(), Err(DagError::Cycle(_))));
    }

    #[test]
    fn diamond_topo_and_ready() {
        //   a
        //  / \
        // b   c
        //  \ /
        //   d
        let g = MissionGraph {
            goal: "diamond".into(),
            tasks: vec![
                task("a", &[]),
                task("b", &["a"]),
                task("c", &["a"]),
                task("d", &["b", "c"]),
            ],
        };
        g.validate().unwrap();
        let order = g.topo_order().unwrap();
        assert_eq!(order.len(), 4);
        let pos = |id: &str| order.iter().position(|x| x == id).unwrap();
        assert!(pos("a") < pos("b"));
        assert!(pos("a") < pos("c"));
        assert!(pos("b") < pos("d"));
        assert!(pos("c") < pos("d"));

        let done = HashSet::new();
        let running = HashSet::new();
        let ready = g.ready_tasks(&done, &running);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "a");

        let mut done = HashSet::new();
        done.insert("a".into());
        let ready = g.ready_tasks(&done, &running);
        let ids: HashSet<_> = ready.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids, HashSet::from(["b", "c"]));

        done.insert("b".into());
        done.insert("c".into());
        let ready = g.ready_tasks(&done, &running);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "d");
    }

    #[test]
    fn ready_skips_running() {
        let g = MissionGraph {
            goal: "r".into(),
            tasks: vec![task("a", &[]), task("b", &[])],
        };
        let done = HashSet::new();
        let mut running = HashSet::new();
        running.insert("a".into());
        let ready = g.ready_tasks(&done, &running);
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "b");
    }

    #[test]
    fn json_schemas_are_objects() {
        let s = MissionGraph::json_schema();
        assert_eq!(s["type"], "object");
        assert!(s["properties"]["tasks"].is_object());
        let p = MissionGraph::plan_json_schema();
        assert!(p["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "milestones"));
        let v = MissionGraph::validation_json_schema();
        assert!(
            v["properties"]["passed"].is_object() || v["properties"]["passed"]["type"] == "boolean"
        );
    }
}
