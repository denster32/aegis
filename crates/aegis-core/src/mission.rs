use crate::agent::AgentLoop;
use crate::prompts;
use crate::ui;
use aegis_swarm::{MissionGraph, SwarmScheduler, ValidationReport};
use aegis_tools::PermissionMode;
use anyhow::{Context, Result};
use console::style;
use std::sync::Arc;
use tracing::info;

pub struct MissionOptions {
    pub auto_approve_graph: bool,
    pub max_validate_retries: u32,
    pub workers: usize,
}

impl Default for MissionOptions {
    fn default() -> Self {
        Self {
            auto_approve_graph: true,
            max_validate_retries: 1,
            workers: 4,
        }
    }
}

/// Full mission: plan DAG → parallel workers → validate.
pub async fn run_mission(mut boss: AgentLoop, goal: &str, opts: MissionOptions) -> Result<String> {
    println!("{}", ui::header("mission"));
    println!(
        "{}\n{}",
        ui::kv("phase", "plan"),
        ui::kv("model", &boss.config.model)
    );

    let graph_json = boss
        .structured_json(
            prompts::mission_boss_prompt(),
            &format!("Mission goal:\n{goal}"),
            "mission_graph",
            MissionGraph::json_schema(),
        )
        .await
        .context("boss graph generation")?;

    let graph: MissionGraph = serde_json::from_str(&graph_json)
        .with_context(|| format!("parse mission graph: {graph_json}"))?;
    graph.validate().context("invalid mission graph")?;

    println!();
    println!("{}", ui::kv("goal", &graph.goal));
    println!("{}", ui::kv("tasks", graph.tasks.len().to_string()));
    println!("{}", ui::rule());
    for t in &graph.tasks {
        println!(
            "  {}  {}  {}",
            ui::mark_idle(),
            style(&t.id).white().bold(),
            style(&t.title).dim()
        );
        if !t.depends_on.is_empty() {
            println!(
                "      {}  deps {}",
                style("·").dim(),
                style(format!("{:?}", t.depends_on)).dim()
            );
        }
    }
    println!();

    if !opts.auto_approve_graph {
        if let Some(ask) = &boss.ask_fn {
            let ans = ask("Approve this mission graph? [Y/n] ");
            if matches!(ans.trim().to_lowercase().as_str(), "n" | "no") {
                anyhow::bail!("mission cancelled by user");
            }
        }
    }

    let graph_str = serde_json::to_string_pretty(&graph)?;
    let mission_id = boss
        .store
        .create_mission(Some(&boss.session_id), goal, &graph_str)?;
    info!(%mission_id, "mission created");

    // Shared pieces for workers
    let client = boss.client.clone();
    let store = boss.store.clone();
    let tools = boss.tools.clone();
    let cwd = boss.cwd.clone();
    let permission = if boss.config.yolo {
        PermissionMode::Yolo
    } else {
        boss.permission
    };
    let print_fn = boss.print_fn.clone();
    let base_cfg = boss.config.clone();
    let mission_goal = graph.goal.clone();

    let runner = {
        let store = store.clone();
        let client = client.clone();
        let tools = tools.clone();
        let cwd = cwd.clone();
        let print_fn = print_fn.clone();
        let base_cfg = base_cfg.clone();
        let mission_goal = mission_goal.clone();
        Arc::new(move |node: aegis_swarm::TaskNode, mid: String| {
            let store = store.clone();
            let client = client.clone();
            let tools = tools.clone();
            let cwd = cwd.clone();
            let print_fn = print_fn.clone();
            let base_cfg = base_cfg.clone();
            let mission_goal = mission_goal.clone();
            Box::pin(async move {
                println!(
                    "\n{}  {}  {}",
                    ui::mark_active(),
                    style(&node.id).white().bold(),
                    style(&node.title).dim()
                );

                // Collect notes for context
                let notes = store
                    .list_notes(&mid)?
                    .into_iter()
                    .map(|(_, tid, body)| {
                        format!("[{}] {}", tid.unwrap_or_else(|| "-".into()), body)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let notes = if notes.is_empty() {
                    "(none yet)".into()
                } else {
                    notes
                };

                let model = if node.needs_reasoning {
                    base_cfg.model.clone()
                } else {
                    node.model_hint
                        .clone()
                        .unwrap_or_else(|| base_cfg.worker_model.clone())
                };

                let session = store.create_session(&cwd, &model)?;
                let mut worker = AgentLoop::new(
                    client,
                    store.clone(),
                    tools,
                    base_cfg.clone(),
                    cwd,
                    session.id,
                );
                worker.permission = permission;
                worker.print_fn = print_fn;
                worker.model_override = Some(model);
                worker.bootstrap_context = true;
                worker.system_override = Some(prompts::worker_system_prompt(
                    &node.title,
                    &node.description,
                    &mission_goal,
                    &notes,
                ));

                let prompt = format!(
                    "Execute your assigned task fully.\nTitle: {}\nDescription: {}",
                    node.title, node.description
                );
                let summary = worker.run_turn(&prompt).await?;
                store.add_note(&mid, Some(&node.id), &summary)?;
                Ok(summary)
            })
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send>,
                >
        })
    };

    let scheduler = SwarmScheduler::new(opts.workers);
    scheduler
        .run(store.clone(), &mission_id, graph.clone(), runner)
        .await
        .context("swarm scheduler")?;

    // Validation
    let mut last_report = String::new();
    for attempt in 0..=opts.max_validate_retries {
        println!(
            "\n{}\n{}",
            ui::label("validate"),
            ui::kv("pass", (attempt + 1).to_string())
        );
        let tasks = store.list_tasks(&mission_id)?;
        let summaries = tasks
            .iter()
            .map(|t| {
                format!(
                    "- {} [{}]: {}",
                    t.id,
                    t.status,
                    t.result.as_deref().unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Validation with tools enabled (not pure structured) — use agent turn then ask for JSON
        let validate_prompt = format!(
            "{}\n\nAfter inspecting, output a single JSON object matching ValidationReport \
             (passed, checks[], next_actions[]) as your final message, no markdown fences.",
            prompts::validation_prompt(goal, &summaries)
        );
        let text = boss.run_turn(&validate_prompt).await?;
        last_report = text.clone();

        if let Some(report) = extract_json::<ValidationReport>(&text) {
            println!(
                "{}",
                ui::kv("result", if report.passed { "pass" } else { "fail" })
            );
            for c in &report.checks {
                println!(
                    "  {}  {}  {}",
                    ui::mark_bool(c.passed),
                    style(&c.name).white(),
                    style(&c.detail).dim()
                );
            }
            if report.passed {
                store.update_mission_status(&mission_id, "validated")?;
                return Ok(format!(
                    "{}\n{}\n{}\n{}",
                    ui::header("mission complete"),
                    ui::kv("goal", &graph.goal),
                    ui::kv("validation", "pass"),
                    text
                ));
            }
            if attempt < opts.max_validate_retries && !report.next_actions.is_empty() {
                let fix = report.next_actions.join("\n- ");
                println!("{}", ui::event("heal", "applying fix actions"));
                let _ = boss
                    .run_turn(&format!(
                        "Validation failed. Fix these issues:\n- {fix}\nThen stop."
                    ))
                    .await?;
                continue;
            }
        }
        break;
    }

    store.update_mission_status(&mission_id, "completed")?;
    Ok(format!(
        "Mission finished (check validation).\nGoal: {}\n\n{last_report}",
        graph.goal
    ))
}

fn extract_json<T: serde::de::DeserializeOwned>(text: &str) -> Option<T> {
    // try whole text
    if let Ok(v) = serde_json::from_str(text) {
        return Some(v);
    }
    // fenced
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                if let Ok(v) = serde_json::from_str(&text[start..=end]) {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// Plan-only structured call.
pub async fn run_plan(agent: &mut AgentLoop, goal: &str) -> Result<aegis_swarm::Plan> {
    let json = agent
        .structured_json(
            prompts::plan_system_prompt(),
            &format!("Create an implementation plan for:\n{goal}"),
            "plan",
            MissionGraph::plan_json_schema(),
        )
        .await?;
    serde_json::from_str(&json).with_context(|| format!("plan parse: {json}"))
}
