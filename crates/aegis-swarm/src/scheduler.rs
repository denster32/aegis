use crate::dag::{MissionGraph, TaskNode};
use aegis_store::{Store, TaskRow};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::info;

/// Callback type for executing a single task node.
pub type TaskRunner = Arc<
    dyn Fn(TaskNode, String /*mission_id*/) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send>>
        + Send
        + Sync,
>;

pub struct SwarmScheduler {
    pub max_workers: usize,
    /// If true, one failed task fails the mission. If false, continue other branches.
    pub fail_fast: bool,
}

impl SwarmScheduler {
    pub fn new(max_workers: usize) -> Self {
        Self {
            max_workers: max_workers.max(1),
            fail_fast: true,
        }
    }

    pub fn soft_fail(mut self) -> Self {
        self.fail_fast = false;
        self
    }

    /// Persist graph tasks to the blackboard.
    pub fn seed_tasks(&self, store: &Store, mission_id: &str, graph: &MissionGraph) -> anyhow::Result<()> {
        for t in &graph.tasks {
            store.upsert_task(&TaskRow {
                id: t.id.clone(),
                mission_id: mission_id.to_string(),
                title: t.title.clone(),
                status: "pending".into(),
                depends_on: serde_json::to_string(&t.depends_on)?,
                result: None,
                model_hint: t.model_hint.clone(),
                needs_reasoning: t.needs_reasoning,
            })?;
        }
        Ok(())
    }

    /// Run until all tasks complete or one fails (fail-fast).
    pub async fn run(
        &self,
        store: Arc<Store>,
        mission_id: &str,
        graph: MissionGraph,
        runner: TaskRunner,
    ) -> anyhow::Result<()> {
        graph.validate()?;
        self.seed_tasks(&store, mission_id, &graph)?;
        store.update_mission_status(mission_id, "running")?;

        let mut done: HashSet<String> = HashSet::new();
        let mut failed: HashSet<String> = HashSet::new();
        let sem = Arc::new(Semaphore::new(self.max_workers));
        let graph = Arc::new(graph);

        loop {
            if done.len() + failed.len() >= graph.tasks.len() {
                break;
            }
            // re-read statuses in case of external updates
            let tasks = store.list_tasks(mission_id)?;
            for t in &tasks {
                if t.status == "done" {
                    done.insert(t.id.clone());
                } else if t.status == "failed" {
                    failed.insert(t.id.clone());
                }
            }
            if !failed.is_empty() && self.fail_fast {
                store.update_mission_status(mission_id, "failed")?;
                anyhow::bail!("task(s) failed: {:?}", failed);
            }
            if done.len() >= graph.tasks.len() {
                break;
            }

            let running: HashSet<String> = tasks
                .iter()
                .filter(|t| t.status == "running")
                .map(|t| t.id.clone())
                .collect();

            let ready: Vec<TaskNode> = graph
                .ready_tasks(&done, &running)
                .into_iter()
                .cloned()
                .collect();

            if ready.is_empty() && running.is_empty() {
                if done.len() < graph.tasks.len() {
                    anyhow::bail!("deadlock: no ready tasks and none running");
                }
                break;
            }

            if ready.is_empty() {
                // wait a bit for running tasks — we spawn join handles below so this path is rare
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                continue;
            }

            let mut handles = Vec::new();
            for node in ready {
                let permit = sem.clone().acquire_owned().await?;
                let store = store.clone();
                let mission_id = mission_id.to_string();
                let runner = runner.clone();
                let node_id = node.id.clone();
                store.set_task_status(&node_id, "running", None)?;
                info!(task = %node_id, title = %node.title, "swarm task start");
                handles.push(tokio::spawn(async move {
                    let _permit = permit;
                    let res = runner(node.clone(), mission_id.clone()).await;
                    match res {
                        Ok(summary) => {
                            let _ = store.set_task_status(&node_id, "done", Some(&summary));
                            let _ = store.add_note(&mission_id, Some(&node_id), &summary);
                            Ok::<_, anyhow::Error>(node_id)
                        }
                        Err(e) => {
                            let msg = format!("error: {e:#}");
                            let _ = store.set_task_status(&node_id, "failed", Some(&msg));
                            let _ = store.add_note(&mission_id, Some(&node_id), &msg);
                            Err(anyhow::anyhow!(msg))
                        }
                    }
                }));
            }

            for h in handles {
                match h.await {
                    Ok(Ok(id)) => {
                        done.insert(id);
                    }
                    Ok(Err(e)) => {
                        if self.fail_fast {
                            store.update_mission_status(mission_id, "failed")?;
                            return Err(e);
                        }
                        tracing::warn!(error = %e, "swarm task failed (soft-fail)");
                        // mark as done for dependency purposes so dependents aren't stuck
                        // (already marked failed in DB)
                    }
                    Err(e) => {
                        if self.fail_fast {
                            store.update_mission_status(mission_id, "failed")?;
                            anyhow::bail!("join error: {e}");
                        }
                        tracing::warn!(error = %e, "swarm join error (soft-fail)");
                    }
                }
            }
        }

        let tasks = store.list_tasks(mission_id)?;
        let any_fail = tasks.iter().any(|t| t.status == "failed");
        store.update_mission_status(
            mission_id,
            if any_fail { "completed_with_errors" } else { "completed" },
        )?;
        if any_fail && self.fail_fast {
            anyhow::bail!("mission completed with failures");
        }
        Ok(())
    }
}
