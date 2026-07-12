use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request body for POST /v1/responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseRequest {
    pub model: String,
    pub input: Vec<InputItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Structured output mode when we need a JSON schema (planning / DAG).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// Grok 4.5 reasoning depth (Responses API: reasoning.effort).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
    /// Sticky cache routing key — highly recommended for multi-turn agents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
}

/// Controls reasoning depth for grok-4.5 (default on API is high if omitted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningConfig {
    pub effort: String, // "low" | "medium" | "high"
}

impl ReasoningConfig {
    pub fn low() -> Self {
        Self {
            effort: "low".into(),
        }
    }
    pub fn medium() -> Self {
        Self {
            effort: "medium".into(),
        }
    }
    pub fn high() -> Self {
        Self {
            effort: "high".into(),
        }
    }
    /// Parse effort name (`low` / `medium` / `high`). Unknown values map to high.
    pub fn parse_effort(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::low(),
            "medium" => Self::medium(),
            _ => Self::high(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    pub format: TextFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextFormat {
    Text,
    JsonObject,
    JsonSchema {
        name: String,
        schema: Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputItem {
    Message(Message),
    FunctionCallOutput(FunctionCallOutput),
    /// Pass-through for chaining previous output items when store=false.
    Raw(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    InputText {
        text: String,
    },
    OutputText {
        text: String,
    },
    /// Multimodal image input (data URL or https URL)
    InputImage {
        #[serde(default)]
        image_url: Option<String>,
        #[serde(default)]
        detail: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallOutput {
    #[serde(rename = "type")]
    pub kind: String, // "function_call_output"
    pub call_id: String,
    pub output: String,
}

impl FunctionCallOutput {
    pub fn new(call_id: impl Into<String>, output: impl Into<String>) -> Self {
        Self {
            kind: "function_call_output".into(),
            call_id: call_id.into(),
            output: output.into(),
        }
    }
}

/// Client function tool or xAI server-side built-in tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolSpec {
    Function(ToolDef),
    /// Built-in: web_search | x_search | code_execution
    Server(ServerTool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerTool {
    #[serde(rename = "type")]
    pub kind: String,
}

impl ServerTool {
    pub fn web_search() -> ToolSpec {
        ToolSpec::Server(Self {
            kind: "web_search".into(),
        })
    }
    pub fn x_search() -> ToolSpec {
        ToolSpec::Server(Self {
            kind: "x_search".into(),
        })
    }
    pub fn code_execution() -> ToolSpec {
        ToolSpec::Server(Self {
            kind: "code_execution".into(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    #[serde(rename = "type")]
    pub kind: String, // "function"
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

impl ToolDef {
    pub fn function(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) -> Self {
        Self {
            kind: "function".into(),
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }

    pub fn as_spec(self) -> ToolSpec {
        ToolSpec::Function(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    Auto(String), // "auto" | "required" | "none"
    Forced {
        #[serde(rename = "type")]
        kind: String,
        name: String,
    },
}

impl ToolChoice {
    pub fn auto() -> Self {
        Self::Auto("auto".into())
    }
    pub fn required() -> Self {
        Self::Auto("required".into())
    }
    pub fn none() -> Self {
        Self::Auto("none".into())
    }
}

/// Full response from /v1/responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseObject {
    pub id: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub output: Vec<OutputItem>,
    #[serde(default)]
    pub usage: Option<Usage>,
    #[serde(default)]
    pub error: Option<ApiErrorBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub total_tokens: Option<u64>,
    #[serde(default)]
    pub reasoning_tokens: Option<u64>,
    /// Nested details when API provides cache hits
    #[serde(default)]
    pub input_tokens_details: Option<InputTokenDetails>,
    #[serde(default)]
    pub output_tokens_details: Option<OutputTokenDetails>,
    #[serde(default)]
    pub prompt_tokens_details: Option<InputTokenDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputTokenDetails {
    #[serde(default)]
    pub cached_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputTokenDetails {
    #[serde(default)]
    pub reasoning_tokens: Option<u64>,
}

impl Usage {
    pub fn cached_tokens(&self) -> u64 {
        self.input_tokens_details
            .as_ref()
            .and_then(|d| d.cached_tokens)
            .or_else(|| {
                self.prompt_tokens_details
                    .as_ref()
                    .and_then(|d| d.cached_tokens)
            })
            .unwrap_or(0)
    }

    pub fn reasoning_token_count(&self) -> u64 {
        self.reasoning_tokens
            .or_else(|| {
                self.output_tokens_details
                    .as_ref()
                    .and_then(|d| d.reasoning_tokens)
            })
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorBody {
    pub message: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputItem {
    Message {
        id: Option<String>,
        role: Option<String>,
        status: Option<String>,
        content: Vec<OutputContent>,
    },
    FunctionCall {
        id: Option<String>,
        call_id: String,
        name: String,
        arguments: String,
        status: Option<String>,
    },
    Reasoning {
        id: Option<String>,
        status: Option<String>,
        #[serde(default)]
        summary: Vec<Value>,
        #[serde(default)]
        encrypted_content: Option<String>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputContent {
    OutputText {
        text: String,
        #[serde(default)]
        annotations: Vec<Value>,
    },
    #[serde(other)]
    Other,
}

impl ResponseObject {
    pub fn output_text(&self) -> String {
        let mut out = String::new();
        for item in &self.output {
            if let OutputItem::Message { content, .. } = item {
                for part in content {
                    if let OutputContent::OutputText { text, .. } = part {
                        out.push_str(text);
                    }
                }
            }
        }
        out
    }

    pub fn function_calls(&self) -> Vec<FunctionCallRef<'_>> {
        self.output
            .iter()
            .filter_map(|item| match item {
                OutputItem::FunctionCall {
                    call_id,
                    name,
                    arguments,
                    ..
                } => Some(FunctionCallRef {
                    call_id,
                    name,
                    arguments,
                }),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct FunctionCallRef<'a> {
    pub call_id: &'a str,
    pub name: &'a str,
    pub arguments: &'a str,
}

// --- helpers for building input ---

pub fn system_msg(text: impl Into<String>) -> InputItem {
    InputItem::Message(Message {
        role: "system".into(),
        content: MessageContent::Text(text.into()),
    })
}

pub fn user_msg(text: impl Into<String>) -> InputItem {
    InputItem::Message(Message {
        role: "user".into(),
        content: MessageContent::Text(text.into()),
    })
}

pub fn assistant_msg(text: impl Into<String>) -> InputItem {
    InputItem::Message(Message {
        role: "assistant".into(),
        content: MessageContent::Text(text.into()),
    })
}

#[derive(Debug, thiserror::Error)]
pub enum XaiError {
    #[error("HTTP error {status}: {body}")]
    Http { status: u16, body: String },
    #[error("rate limited, retry after {retry_after_ms:?}ms")]
    RateLimited { retry_after_ms: Option<u64> },
    #[error("auth error: {0}")]
    Auth(String),
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

#[cfg(test)]
mod grok45_tests {
    use super::*;

    fn minimal_req() -> CreateResponseRequest {
        CreateResponseRequest {
            model: "grok-4.5".into(),
            input: vec![user_msg("hi")],
            tools: None,
            tool_choice: None,
            previous_response_id: None,
            store: None,
            stream: None,
            temperature: None,
            max_output_tokens: None,
            parallel_tool_calls: None,
            text: None,
            include: None,
            reasoning: None,
            prompt_cache_key: None,
        }
    }

    #[test]
    fn request_serializes_reasoning_and_cache_key() {
        let mut req = minimal_req();
        req.tools = Some(vec![ServerTool::web_search(), ServerTool::code_execution()]);
        req.store = Some(true);
        req.parallel_tool_calls = Some(true);
        req.reasoning = Some(ReasoningConfig::low());
        req.prompt_cache_key = Some("sess-123".into());
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["reasoning"]["effort"], "low");
        assert_eq!(v["prompt_cache_key"], "sess-123");
        assert!(v["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["type"] == "web_search"));
        assert!(v["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["type"] == "code_execution"));
    }

    #[test]
    fn reasoning_omitted_when_none() {
        let req = minimal_req();
        let v = serde_json::to_value(&req).unwrap();
        assert!(v.get("reasoning").is_none());
        assert!(v.get("prompt_cache_key").is_none());
    }

    #[test]
    fn tools_and_tool_choice_omitted_when_none() {
        let req = minimal_req();
        let v = serde_json::to_value(&req).unwrap();
        assert!(v.get("tools").is_none());
        assert!(v.get("tool_choice").is_none());
        assert!(v.get("previous_response_id").is_none());
        assert!(v.get("store").is_none());
        assert!(v.get("stream").is_none());
        assert!(v.get("temperature").is_none());
        assert!(v.get("max_output_tokens").is_none());
        assert!(v.get("parallel_tool_calls").is_none());
        assert!(v.get("text").is_none());
        assert!(v.get("include").is_none());
    }

    #[test]
    fn tool_choice_serializes() {
        let mut req = minimal_req();
        req.tool_choice = Some(ToolChoice::auto());
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["tool_choice"], "auto");

        req.tool_choice = Some(ToolChoice::Forced {
            kind: "function".into(),
            name: "read_file".into(),
        });
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["tool_choice"]["type"], "function");
        assert_eq!(v["tool_choice"]["name"], "read_file");
    }

    #[test]
    fn reasoning_config_parse_effort() {
        assert_eq!(ReasoningConfig::parse_effort("low").effort, "low");
        assert_eq!(ReasoningConfig::parse_effort("MEDIUM").effort, "medium");
        assert_eq!(ReasoningConfig::parse_effort("high").effort, "high");
        assert_eq!(ReasoningConfig::parse_effort("weird").effort, "high");
    }

    #[test]
    fn function_call_output_new() {
        let f = FunctionCallOutput::new("call_1", "result");
        assert_eq!(f.kind, "function_call_output");
        assert_eq!(f.call_id, "call_1");
        let item = InputItem::FunctionCallOutput(f);
        let v = serde_json::to_value(&item).unwrap();
        assert_eq!(v["type"], "function_call_output");
        assert_eq!(v["output"], "result");
    }

    #[test]
    fn tool_def_function_spec() {
        let def = ToolDef::function(
            "read_file",
            "Read a file",
            serde_json::json!({"type": "object"}),
        );
        let v = serde_json::to_value(def.as_spec()).unwrap();
        assert_eq!(v["type"], "function");
        assert_eq!(v["name"], "read_file");
    }

    #[test]
    fn response_output_text_and_function_calls() {
        let resp = ResponseObject {
            id: "r1".into(),
            model: Some("grok-4.5".into()),
            status: Some("completed".into()),
            output: vec![
                OutputItem::Message {
                    id: None,
                    role: Some("assistant".into()),
                    status: None,
                    content: vec![OutputContent::OutputText {
                        text: "Hello ".into(),
                        annotations: vec![],
                    }],
                },
                OutputItem::Message {
                    id: None,
                    role: Some("assistant".into()),
                    status: None,
                    content: vec![OutputContent::OutputText {
                        text: "world".into(),
                        annotations: vec![],
                    }],
                },
                OutputItem::FunctionCall {
                    id: None,
                    call_id: "c1".into(),
                    name: "bash".into(),
                    arguments: r#"{"cmd":"ls"}"#.into(),
                    status: None,
                },
            ],
            usage: None,
            error: None,
        };
        assert_eq!(resp.output_text(), "Hello world");
        let calls = resp.function_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "bash");
        assert_eq!(calls[0].call_id, "c1");
    }

    #[test]
    fn usage_cached_and_reasoning_tokens() {
        let u = Usage {
            input_tokens: Some(100),
            output_tokens: Some(50),
            total_tokens: Some(150),
            reasoning_tokens: None,
            input_tokens_details: Some(InputTokenDetails {
                cached_tokens: Some(40),
            }),
            output_tokens_details: Some(OutputTokenDetails {
                reasoning_tokens: Some(12),
            }),
            prompt_tokens_details: None,
        };
        assert_eq!(u.cached_tokens(), 40);
        assert_eq!(u.reasoning_token_count(), 12);

        let empty = Usage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: None,
            reasoning_tokens: Some(7),
            input_tokens_details: None,
            output_tokens_details: None,
            prompt_tokens_details: None,
        };
        assert_eq!(empty.cached_tokens(), 0);
        assert_eq!(empty.reasoning_token_count(), 7);
    }

    #[test]
    fn message_helpers_roles() {
        let s = serde_json::to_value(system_msg("sys")).unwrap();
        let u = serde_json::to_value(user_msg("usr")).unwrap();
        let a = serde_json::to_value(assistant_msg("asst")).unwrap();
        assert_eq!(s["role"], "system");
        assert_eq!(u["role"], "user");
        assert_eq!(a["role"], "assistant");
        assert_eq!(u["content"], "usr");
    }
}

#[cfg(test)]
mod server_tools_tests {
    use crate::server_tools;

    #[test]
    fn server_tools_flags() {
        assert!(server_tools(false, false, false).is_empty());
        let all = server_tools(true, true, true);
        assert_eq!(all.len(), 3);
        let v = serde_json::to_value(&all).unwrap();
        let kinds: Vec<_> = v
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["type"].as_str().unwrap().to_string())
            .collect();
        assert!(kinds.contains(&"web_search".into()));
        assert!(kinds.contains(&"x_search".into()));
        assert!(kinds.contains(&"code_execution".into()));
    }
}
