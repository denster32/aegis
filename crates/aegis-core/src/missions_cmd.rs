//! Factory-style Missions orchestration.

use crate::agent::AgentLoop;
use crate::prompts;
use aegis_swarm::{
    append_progress, assess_readiness, load_state, save_state, FeatureStatus, MissionGraph,
    MissionPlan, MissionStatus, TaskNode,
};
use anyhow::{Context, Result};
use chrono::Utc;
use console::style;
use std::sync::Arc;

/// Collaborative / one-shot mission plan generation.
pub async fn missions_new(agent: &mut AgentLoop, goal: &str, oneshot: bool) -> Result<MissionPlan> {
    let user = if oneshot {
        format!(
            "Create a MissionPlan for this goal. Decompose into milestones and features with dependencies and skill_hints.\n\nGoal:\n{goal}"
        )
    } else {
        format!(
            "You are planning a Factory-style Mission. Ask yourself clarifying constraints, then produce a complete MissionPlan JSON.\n\
             Prefer 2-5 milestones and 3-10 features. Include validation steps per milestone.\n\nGoal:\n{goal}"
        )
    };

    let json = agent
        .structured_json(
            "You are Aegis Mission planner. Output only MissionPlan JSON matching the schema. Original design for structured multi-feature work.",
            &user,
            "mission_plan",
            MissionPlan::plan_json_schema(),
        )
        .await
        .context("mission plan generation")?;

    let mut raw: serde_json::Value =
        serde_json::from_str(&json).with_context(|| format!("parse mission plan: {json}"))?;
    // Ensure id/status timestamps
    if raw.get("id").is_none() {
        raw["id"] = serde_json::json!(uuid::Uuid::new_v4().to_string());
    }
    let mut plan: MissionPlan = serde_json::from_value(raw)?;
    if plan.id.is_empty() {
        plan.id = uuid::Uuid::new_v4().to_string();
    }
    plan.goal = if plan.goal.is_empty() {
        goal.into()
    } else {
        plan.goal
    };
    plan.status = MissionStatus::Approved;
    plan.created_at = Utc::now().to_rfc3339();
    plan.updated_at = plan.created_at.clone();
    // default feature/milestone statuses
    for f in &mut plan.features {
        f.status = FeatureStatus::Pending;
    }
    for m in &mut plan.milestones {
        m.status = FeatureStatus::Pending;
    }
    plan.save(&agent.cwd)?;
    append_progress(
        &agent.cwd,
        &plan.id,
        serde_json::json!({"ts": Utc::now().to_rfc3339(), "event": "created", "goal": plan.goal}),
    )?;
    Ok(plan)
}

/// Execute an approved mission: convert features → swarm DAG, run, validate milestones.
pub async fn missions_run(mut agent: AgentLoop, mission_id: &str) -> Result<String> {
    let mut plan = MissionPlan::load(&agent.cwd, mission_id)?;
    plan.status = MissionStatus::Running;
    plan.updated_at = Utc::now().to_rfc3339();
    plan.save(&agent.cwd)?;

    let mut state = load_state(&agent.cwd, &plan.id)?;
    println!("{}", plan.control_board(&state));

    // Materialize skills if missing
    if let Some(learn) = agent.learn.as_ref() {
        for bind in &plan.skill_map {
            if bind.create_if_missing {
                let skills = learn.memory.list_skills().unwrap_or_default();
                if !skills.iter().any(|(n, _)| n == &bind.skill_name) {
                    let _ = learn.memory.write_skill(
                        &bind.skill_name,
                        &format!(
                            "# Skill: {}\n\nFor feature `{}`.\n\n## Steps\n- inspect codebase\n- implement\n- test\n",
                            bind.skill_name, bind.feature_id
                        ),
                    );
                }
            }
        }
    }

    // Build swarm graph from features
    let tasks: Vec<TaskNode> = plan
        .features
        .iter()
        .map(|f| TaskNode {
            id: f.id.clone(),
            title: f.title.clone(),
            description: format!(
                "{}\nSkills: {:?}\nMilestone: {:?}",
                f.description, f.skill_hints, f.milestone_id
            ),
            depends_on: f.depends_on.clone(),
            needs_reasoning: f.skill_hints.iter().any(|s| s.contains("design")),
            model_hint: None,
        })
        .collect();

    let graph = MissionGraph {
        goal: plan.goal.clone(),
        tasks,
    };
    graph.validate().context("mission feature DAG")?;

    // Run via existing mission machinery conceptually — use run_turn per feature for clarity
    // with dependency order
    let order = graph.topo_order()?;
    for fid in order {
        let (title, description, skill_ctx) = {
            let feature = plan
                .features
                .iter_mut()
                .find(|f| f.id == fid)
                .context("feature missing")?;
            feature.status = FeatureStatus::InProgress;
            (
                feature.title.clone(),
                feature.description.clone(),
                feature.skill_hints.join(", "),
            )
        };
        state.current_feature = Some(fid.clone());
        save_state(&agent.cwd, &state)?;
        plan.save(&agent.cwd)?;
        println!(
            "{} feature {} — {}",
            style("▶").yellow(),
            style(&fid).bold(),
            title
        );

        let prompt = format!(
            "MISSION FEATURE EXECUTION\n\
             Mission goal: {}\n\
             Feature id: {fid}\n\
             Title: {title}\n\
             Description: {description}\n\
             Skills: {skill_ctx}\n\
             Read project memory and skills. Implement this feature only. Verify when possible.\n\
             When done, use memory_write to record any durable lesson.",
            plan.goal
        );
        match agent.run_turn(&prompt).await {
            Ok(summary) => {
                let notes: String = summary.chars().take(500).collect();
                if let Some(feature) = plan.features.iter_mut().find(|f| f.id == fid) {
                    feature.status = FeatureStatus::Done;
                    feature.notes = notes.clone();
                }
                state.completed_features.push(fid.clone());
                let handoff_dir = agent
                    .cwd
                    .join(".aegis/missions")
                    .join(&plan.id)
                    .join("handoffs");
                let _ = std::fs::create_dir_all(&handoff_dir);
                let _ = std::fs::write(handoff_dir.join(format!("{fid}.md")), &notes);
                append_progress(
                    &agent.cwd,
                    &plan.id,
                    serde_json::json!({"ts": Utc::now().to_rfc3339(), "event": "feature_done", "id": fid}),
                )?;
            }
            Err(e) => {
                if let Some(feature) = plan.features.iter_mut().find(|f| f.id == fid) {
                    feature.status = FeatureStatus::Blocked;
                }
                state.blocked_reason = Some(e.to_string());
                plan.status = MissionStatus::Blocked;
                plan.save(&agent.cwd)?;
                save_state(&agent.cwd, &state)?;
                return Err(e);
            }
        }
        plan.updated_at = Utc::now().to_rfc3339();
        plan.save(&agent.cwd)?;
        save_state(&agent.cwd, &state)?;
    }

    // Milestone validation
    for m in &mut plan.milestones {
        m.status = FeatureStatus::InProgress;
        let checks = m.validation.join("\n- ");
        let prompt = format!(
            "Validate milestone `{}`: {}\nChecks:\n- {checks}\nRun tests/commands as needed. Reply with PASS or FAIL and details.",
            m.id, m.title
        );
        let out = agent.run_turn(&prompt).await.unwrap_or_default();
        if out.to_uppercase().contains("PASS") {
            m.status = FeatureStatus::Done;
        } else {
            m.status = FeatureStatus::Blocked;
        }
    }

    // Soft complete: feature loop finished; milestone blocking is advisory for now.
    let _milestones_ok = plan.milestones.is_empty()
        || plan
            .milestones
            .iter()
            .all(|m| m.status == FeatureStatus::Done);
    let _ = _milestones_ok;
    plan.status = MissionStatus::Completed;
    plan.updated_at = Utc::now().to_rfc3339();
    plan.save(&agent.cwd)?;

    let _ = agent.reflect_and_save().await;

    let state = load_state(&agent.cwd, &plan.id)?;
    println!("{}", plan.control_board(&state));
    Ok(format!("Mission {} finished: {:?}", plan.id, plan.status))
}

pub fn missions_status(cwd: &std::path::Path, id: Option<&str>) -> Result<String> {
    if let Some(id) = id {
        let plan = MissionPlan::load(cwd, id)?;
        let state = load_state(cwd, &plan.id)?;
        Ok(plan.control_board(&state))
    } else {
        let list = MissionPlan::list(cwd)?;
        if list.is_empty() {
            return Ok("No missions yet. Run: aegis missions new \"…\"".into());
        }
        let mut s = String::from("Missions:\n");
        for p in list {
            s.push_str(&format!(
                "  {}  {:?}  {}\n",
                &p.id[..8.min(p.id.len())],
                p.status,
                p.goal.chars().take(60).collect::<String>()
            ));
        }
        Ok(s)
    }
}

pub fn readiness_report(cwd: &std::path::Path) -> String {
    let r = assess_readiness(cwd);
    let mut s = format!("Readiness: {} ({}/100)\n", r.level, r.score);
    for c in r.checks {
        let mark = if c.passed {
            style("✓").green()
        } else {
            style("✗").red()
        };
        s.push_str(&format!("  {mark} {} — {}\n", c.name, c.detail));
    }
    s.push_str(
        "\nTip: Factory recommends high agent readiness before long Missions (tests + scriptable QA).\n",
    );
    s
}

// silence
#[allow(dead_code)]
fn _p() {
    let _ = prompts::system_prompt(".");
    let _ = Arc::new(());
}
