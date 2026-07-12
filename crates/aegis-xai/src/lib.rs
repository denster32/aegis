//! xAI Responses API client optimized for Grok 4.5 coding-agent loops.

mod client;
mod token;
mod types;

pub use client::{ResponsesClient, StreamEvent};
pub use token::{ArcToken, StaticToken, TokenSource};
pub use types::*;

/// Build server-side tool list from flags.
pub fn server_tools(web: bool, x: bool, code: bool) -> Vec<ToolSpec> {
    let mut t = Vec::new();
    if web {
        t.push(ServerTool::web_search());
    }
    if x {
        t.push(ServerTool::x_search());
    }
    if code {
        t.push(ServerTool::code_execution());
    }
    t
}
