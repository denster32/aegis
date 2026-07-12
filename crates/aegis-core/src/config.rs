use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    #[default]
    Medium,
    High,
}

impl Effort {
    pub fn as_str(self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Effort::Low,
            "high" => Effort::High,
            _ => Effort::Medium,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AegisConfig {
    pub model: String,
    pub worker_model: String,
    pub base_url: String,
    pub max_tool_parallel: usize,
    pub max_swarm_workers: usize,
    pub max_agent_steps: usize,
    pub store_server_side: bool,
    pub yolo: bool,
    /// Rough token budget before auto-compaction kicks in.
    pub compact_token_threshold: usize,
    pub enable_web_fetch: bool,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfigSerde>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfigSerde {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl Default for AegisConfig {
    fn default() -> Self {
        Self {
            model: "grok-4.5".into(),
            worker_model: "grok-code-fast-1".into(),
            base_url: "https://api.x.ai/v1".into(),
            max_tool_parallel: 6,
            max_swarm_workers: 4,
            max_agent_steps: 40,
            store_server_side: true,
            yolo: false,
            compact_token_threshold: 120_000,
            enable_web_fetch: true,
            mcp_servers: Vec::new(),
        }
    }
}

impl AegisConfig {
    pub fn with_effort(mut self, effort: Effort) -> Self {
        match effort {
            Effort::High => {
                self.model = "grok-4.5".into();
                self.worker_model = "grok-4.5".into();
                self.max_tool_parallel = 8;
                self.max_swarm_workers = 6;
            }
            Effort::Medium => {
                self.model = "grok-4.5".into();
                self.worker_model = "grok-code-fast-1".into();
                self.max_tool_parallel = 6;
                self.max_swarm_workers = 4;
            }
            Effort::Low => {
                self.model = "grok-4-fast".into();
                self.worker_model = "grok-code-fast-1".into();
                self.max_tool_parallel = 4;
                self.max_swarm_workers = 2;
            }
        }
        self
    }

    pub fn load_layered(home_config: Option<&PathBuf>, project_config: Option<&PathBuf>) -> Self {
        let mut cfg = Self::default();
        if let Some(p) = home_config {
            if let Ok(s) = std::fs::read_to_string(p) {
                if let Ok(partial) = toml::from_str::<PartialConfig>(&s) {
                    partial.apply(&mut cfg);
                }
            }
        }
        if let Some(p) = project_config {
            if let Ok(s) = std::fs::read_to_string(p) {
                if let Ok(partial) = toml::from_str::<PartialConfig>(&s) {
                    partial.apply(&mut cfg);
                }
            }
        }
        cfg
    }
}

#[derive(Debug, Default, Deserialize)]
struct PartialConfig {
    model: Option<String>,
    worker_model: Option<String>,
    base_url: Option<String>,
    max_tool_parallel: Option<usize>,
    max_swarm_workers: Option<usize>,
    max_agent_steps: Option<usize>,
    store_server_side: Option<bool>,
    yolo: Option<bool>,
    compact_token_threshold: Option<usize>,
    enable_web_fetch: Option<bool>,
    #[serde(default)]
    mcp_servers: Option<Vec<McpServerConfigSerde>>,
}

impl PartialConfig {
    fn apply(self, cfg: &mut AegisConfig) {
        if let Some(v) = self.model {
            cfg.model = v;
        }
        if let Some(v) = self.worker_model {
            cfg.worker_model = v;
        }
        if let Some(v) = self.base_url {
            cfg.base_url = v;
        }
        if let Some(v) = self.max_tool_parallel {
            cfg.max_tool_parallel = v;
        }
        if let Some(v) = self.max_swarm_workers {
            cfg.max_swarm_workers = v;
        }
        if let Some(v) = self.max_agent_steps {
            cfg.max_agent_steps = v;
        }
        if let Some(v) = self.store_server_side {
            cfg.store_server_side = v;
        }
        if let Some(v) = self.yolo {
            cfg.yolo = v;
        }
        if let Some(v) = self.compact_token_threshold {
            cfg.compact_token_threshold = v;
        }
        if let Some(v) = self.enable_web_fetch {
            cfg.enable_web_fetch = v;
        }
        if let Some(v) = self.mcp_servers {
            cfg.mcp_servers = v;
        }
    }
}


