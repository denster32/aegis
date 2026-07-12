//! Per-project learning: MEMORY.md, lessons, failures, skills, run reflection.

mod failures;
mod inject;
mod lessons;
mod nexus;
mod project;
mod redact;
mod reflect;

pub use failures::*;
pub use inject::*;
pub use lessons::*;
pub use nexus::NeuralSummary;
pub use project::*;
pub use redact::redact_secrets;
pub use reflect::*;
