use async_trait::async_trait;
use parking_lot::Mutex;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    /// Prompt for shell and writes outside cwd (interactive).
    Prompt,
    /// Auto-approve everything.
    Yolo,
    /// Deny dangerous ops (shell / outside writes).
    Deny,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub ok: bool,
    pub output: String,
}

impl ToolResult {
    pub fn ok(output: impl Into<String>) -> Self {
        Self {
            ok: true,
            output: output.into(),
        }
    }
    pub fn err(output: impl Into<String>) -> Self {
        Self {
            ok: false,
            output: output.into(),
        }
    }
}

/// Callback used for interactive prompts (`ask_user` / permission gates).
pub type AskFn = Arc<dyn Fn(&str) -> String + Send + Sync>;
/// Per-path async mutex map shared across parallel tool tasks.
pub type PathLockMap = Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>;

pub struct ToolContext {
    pub cwd: PathBuf,
    pub session_id: String,
    pub permission: PermissionMode,
    /// Optional callback for ask_user / permissions in interactive mode.
    pub ask: Option<AskFn>,
    /// Shared path locks to reduce parallel edit races.
    pub path_locks: PathLockMap,
    /// Persist todos / notes (injected by core).
    pub todo_store: Option<Arc<dyn TodoStore + Send + Sync>>,
}

pub trait TodoStore: Send + Sync {
    fn set_todos(&self, session_id: &str, todos_json: &str) -> anyhow::Result<()>;
    fn get_todos(&self, session_id: &str) -> anyhow::Result<Option<String>>;
}

impl ToolContext {
    pub fn new(cwd: PathBuf, session_id: String, permission: PermissionMode) -> Self {
        Self {
            cwd,
            session_id,
            permission,
            ask: None,
            path_locks: Arc::new(Mutex::new(HashMap::new())),
            todo_store: None,
        }
    }

    pub fn resolve_path(&self, path: &str) -> PathBuf {
        let p = Path::new(path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.cwd.join(p)
        }
    }

    pub fn is_within_cwd(&self, path: &Path) -> bool {
        let Ok(cwd) = self.cwd.canonicalize() else {
            return path.starts_with(&self.cwd);
        };
        let Ok(p) = path.canonicalize().or_else(|_| {
            path.parent()
                .and_then(|par| par.canonicalize().ok())
                .map(|par| par.join(path.file_name().unwrap_or_default()))
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no parent"))
        }) else {
            return path.starts_with(&self.cwd);
        };
        p.starts_with(&cwd)
    }

    pub async fn lock_path(&self, path: &Path) -> Arc<tokio::sync::Mutex<()>> {
        let key = path.display().to_string();
        let mut map = self.path_locks.lock();
        map.entry(key)
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone()
    }

    pub fn approve(&self, action: &str) -> bool {
        match self.permission {
            PermissionMode::Yolo => true,
            PermissionMode::Deny => false,
            PermissionMode::Prompt => {
                if let Some(ask) = &self.ask {
                    let ans = ask(&format!("{action} [y/N] "));
                    matches!(ans.trim().to_lowercase().as_str(), "y" | "yes")
                } else {
                    // Fail closed: non-interactive Prompt without ask callback denies.
                    false
                }
            }
        }
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.values().cloned().collect()
    }

    /// Returns (name, description, parameters_schema) for conversion to provider tool defs.
    pub fn to_xai_tools(&self) -> Vec<(String, String, Value)> {
        self.list()
            .into_iter()
            .map(|t| {
                (
                    t.name().to_string(),
                    t.description().to_string(),
                    t.parameters_schema(),
                )
            })
            .collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_registry() -> ToolRegistry {
    use crate::ask::AskUserTool;
    use crate::bash::BashTool;
    use crate::edit::EditFileTool;
    use crate::fs_tools::{ReadFileTool, WriteFileTool};
    use crate::git_tools::{GitDiffTool, GitStatusTool};
    use crate::grep::GrepTool;
    use crate::memory_tools::{MemoryReadTool, MemoryWriteTool};
    use crate::search::GlobTool;
    use crate::todo::TodoWriteTool;
    use crate::vision::{ScreenshotTool, VisionDescribeTool};
    use crate::web::WebFetchTool;

    let mut reg = ToolRegistry::new();
    reg.register(Arc::new(ReadFileTool));
    reg.register(Arc::new(WriteFileTool));
    reg.register(Arc::new(EditFileTool));
    reg.register(Arc::new(BashTool));
    reg.register(Arc::new(GlobTool));
    reg.register(Arc::new(GrepTool));
    reg.register(Arc::new(TodoWriteTool));
    reg.register(Arc::new(AskUserTool));
    reg.register(Arc::new(GitStatusTool));
    reg.register(Arc::new(GitDiffTool));
    reg.register(Arc::new(WebFetchTool));
    reg.register(Arc::new(MemoryReadTool));
    reg.register(Arc::new(MemoryWriteTool));
    reg.register(Arc::new(VisionDescribeTool));
    reg.register(Arc::new(ScreenshotTool));
    reg
}
