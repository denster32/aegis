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
    fn utf8_truncate_multibyte() {
        let s = "αβγδεζηθ";
        let t = utf8_truncate(s, 4);
        assert_eq!(t.chars().count(), 4);
        assert!(t.ends_with('…'));
    }
}
