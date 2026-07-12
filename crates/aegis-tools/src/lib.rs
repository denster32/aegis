//! Built-in coding tools for Aegis.

mod ask;
mod bash;
mod edit;
mod fs_tools;
mod git_tools;
mod grep;
mod memory_tools;
mod registry;
mod search;
mod todo;
mod web;

pub use registry::{
    default_registry, PermissionMode, TodoStore, Tool, ToolContext, ToolRegistry, ToolResult,
};
