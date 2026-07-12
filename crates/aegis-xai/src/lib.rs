//! xAI Responses API client optimized for Grok 4.5 coding-agent loops.

mod client;
mod token;
mod types;

pub use client::{ResponsesClient, StreamEvent};
pub use token::{ArcToken, StaticToken, TokenSource};
pub use types::*;
