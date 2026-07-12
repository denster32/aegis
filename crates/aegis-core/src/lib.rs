//! Aegis agent loop, prompts, and mission orchestration.

mod agent;
pub mod automations;
mod checkpoint;
mod config;
mod dream;
mod factory;
mod hooks;
mod learn;
mod mission;
mod missions_cmd;
mod notify;
mod prompts;
mod qa;
mod readiness_v2;
mod review;
pub mod ui;
mod wiki;

pub use agent::*;
pub use checkpoint::{
    create as checkpoint_create, list as checkpoint_list, restore as checkpoint_restore, Checkpoint,
};
pub use config::*;
pub use dream::{install_dream_cron, run_dream, DreamJournal, DreamOptions};
pub use factory::{factory_status, format_factory, FactoryStatus};
pub use hooks::run_hook;
pub use learn::*;
pub use mission::*;
pub use missions_cmd::*;
pub use notify::notify;
pub use prompts::*;
pub use qa::{install_qa, run_qa};
pub use readiness_v2::{assess_v2, format_report as format_readiness_v2, ReadinessV2Report};
pub use review::{
    format_report_md as format_review_md, install_review_workflow, review_diff, review_pr,
    ReviewReport,
};
pub use wiki::{generate_wiki, install_wiki_workflow};

/// Whether this model accepts Responses API `reasoning.effort`.
/// `grok-code-fast-1` and similar workers return HTTP 400 if reasoning is sent.
pub fn model_supports_reasoning(model: &str) -> bool {
    let m = model.to_ascii_lowercase();
    m.contains("grok-4") || m.contains("grok4") || m.starts_with("grok-3")
}

/// Truncate to at most `max_chars` Unicode scalar values (UTF-8 safe).
pub fn utf8_truncate(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

#[cfg(test)]
mod util_tests {
    use super::*;

    #[test]
    fn reasoning_gate() {
        assert!(model_supports_reasoning("grok-4.5"));
        assert!(model_supports_reasoning("grok-4-fast"));
        assert!(!model_supports_reasoning("grok-code-fast-1"));
    }

    #[test]
    fn model_supports_reasoning_variants() {
        assert!(model_supports_reasoning("GROK-4.5"));
        assert!(model_supports_reasoning("grok4"));
        assert!(model_supports_reasoning("grok-3-mini"));
        assert!(model_supports_reasoning("grok-3"));
        assert!(!model_supports_reasoning("gpt-4o"));
        assert!(!model_supports_reasoning(""));
        assert!(!model_supports_reasoning("grok-2"));
    }

    #[test]
    fn utf8_truncate_multibyte() {
        let s = "αβγδεζηθ";
        let t = utf8_truncate(s, 4);
        assert_eq!(t.chars().count(), 4);
        assert!(t.ends_with('…'));
    }

    #[test]
    fn utf8_truncate_short_and_boundary() {
        assert_eq!(utf8_truncate("hi", 10), "hi");
        assert_eq!(utf8_truncate("hello", 5), "hello");
        let t = utf8_truncate("hello", 3);
        assert_eq!(t.chars().count(), 3);
        assert!(t.ends_with('…'));
        // max_chars 0 / 1 → just ellipsis (saturating_sub)
        let t0 = utf8_truncate("abc", 0);
        assert_eq!(t0, "…");
        let t1 = utf8_truncate("abc", 1);
        assert_eq!(t1, "…");
    }

    #[test]
    fn effort_parse_and_with_effort() {
        assert_eq!(Effort::parse("low"), Effort::Low);
        assert_eq!(Effort::parse("HIGH"), Effort::High);
        assert_eq!(Effort::parse("nope"), Effort::Medium);
        assert_eq!(Effort::Low.as_str(), "low");

        let high = AegisConfig::default().with_effort(Effort::High);
        assert_eq!(high.reasoning_effort, "high");
        assert_eq!(high.model, "grok-4.5");
        assert_eq!(high.worker_model, "grok-4.5");
        assert_eq!(high.max_swarm_workers, 6);

        let low = AegisConfig::default().with_effort(Effort::Low);
        assert_eq!(low.reasoning_effort, "low");
        assert_eq!(low.tool_reasoning_effort, "low");
        assert_eq!(low.max_tool_parallel, 4);

        let med = AegisConfig::default().with_effort(Effort::Medium);
        assert_eq!(med.reasoning_effort, "medium");
        assert_eq!(med.worker_model, "grok-code-fast-1");
    }

    #[test]
    fn factory_status_on_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        // empty project: Code-gen still healthy
        let status = factory_status(dir.path());
        assert_eq!(status.stages.len(), 6);
        let code = status.stages.iter().find(|s| s.name == "Code-gen").unwrap();
        assert!(code.healthy);

        // add release + wiki signals
        std::fs::write(dir.path().join("CHANGELOG.md"), "# changelog\n").unwrap();
        std::fs::create_dir_all(dir.path().join("docs/wiki")).unwrap();
        std::fs::create_dir_all(dir.path().join(".aegis/dreams")).unwrap();
        let status = factory_status(dir.path());
        assert!(status
            .stages
            .iter()
            .any(|s| s.name == "Release" && s.healthy));
        assert!(status
            .stages
            .iter()
            .any(|s| s.name == "Document" && s.healthy));
        assert!(status
            .stages
            .iter()
            .any(|s| s.name == "Monitor" && s.healthy));
        assert!(dir.path().join(".aegis/factory/status.json").exists());
    }

    #[test]
    fn readiness_assess_on_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        // Minimal project without Cargo.toml (avoids cargo test --no-run in L4).
        std::fs::write(dir.path().join("README.md"), "# toy\n").unwrap();
        std::fs::write(dir.path().join("package.json"), r#"{"name":"toy"}"#).unwrap();
        std::fs::create_dir_all(dir.path().join("tests")).unwrap();
        std::fs::write(dir.path().join("rustfmt.toml"), "edition = \"2021\"\n").unwrap();
        // Fake .git dir so l1_vcs passes without a real repo.
        std::fs::create_dir_all(dir.path().join(".git")).unwrap();
        std::fs::write(dir.path().join(".gitignore"), "node_modules\n").unwrap();
        std::fs::write(dir.path().join("LICENSE"), "MIT\n").unwrap();
        std::fs::write(dir.path().join("CONTRIBUTING.md"), "hi\n").unwrap();
        std::fs::write(dir.path().join(".env.example"), "KEY=\n").unwrap();
        std::fs::write(dir.path().join("AGENTS.md"), "rules\n").unwrap();

        let report = assess_v2(dir.path());
        assert!(report.level >= 1);
        assert!(report.score_pct <= 100);
        assert!(!report.pillars.is_empty());
        assert!(dir.path().join(".aegis/readiness/report.json").exists());
        // L1 (5/5) + L2 docs (5/5) → at least L2
        assert!(
            report.level >= 2,
            "expected >= L2, got L{} failing={:?}",
            report.level,
            report.failing
        );
        // Passed criteria should include readme
        let flat: Vec<_> = report
            .pillars
            .iter()
            .flat_map(|p| p.criteria.iter())
            .collect();
        assert!(flat.iter().any(|c| c.id == "l1_readme" && c.passed));
    }

    #[test]
    fn config_load_layered_from_toml() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home.toml");
        let proj = dir.path().join("proj.toml");
        std::fs::write(
            &home,
            r#"
model = "from-home"
reasoning_effort = "low"
"#,
        )
        .unwrap();
        std::fs::write(
            &proj,
            r#"
model = "from-project"
max_agent_steps = 7
"#,
        )
        .unwrap();
        let cfg = AegisConfig::load_layered(Some(&home), Some(&proj));
        assert_eq!(cfg.model, "from-project");
        assert_eq!(cfg.reasoning_effort, "low");
        assert_eq!(cfg.max_agent_steps, 7);
    }
}
