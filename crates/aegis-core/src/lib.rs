//! Aegis agent loop, prompts, and mission orchestration.

mod agent;
pub mod automations;
mod config;
mod dream;
mod factory;
mod learn;
mod mission;
mod missions_cmd;
mod prompts;
mod qa;
mod readiness_v2;
mod review;
mod wiki;

pub use agent::*;
pub use config::*;
pub use dream::{install_dream_cron, run_dream, DreamJournal, DreamOptions};
pub use factory::{factory_status, format_factory, FactoryStatus};
pub use learn::*;
pub use mission::*;
pub use missions_cmd::*;
pub use prompts::*;
pub use qa::{install_qa, run_qa};
pub use readiness_v2::{assess_v2, format_report as format_readiness_v2, ReadinessV2Report};
pub use review::{
    format_report_md as format_review_md, install_review_workflow, review_diff, review_pr,
    ReviewReport,
};
pub use wiki::{generate_wiki, install_wiki_workflow};
