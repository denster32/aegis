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
    pub fn from_str(s: &str) -> Self {
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
    InputText { text: String },
    OutputText { text: String },
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
    pub fn function(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
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

    #[test]
    fn request_serializes_reasoning_and_cache_key() {
        let req = CreateResponseRequest {
            model: "grok-4.5".into(),
            input: vec![user_msg("hi")],
            tools: Some(vec![ServerTool::web_search(), ServerTool::code_execution()]),
            tool_choice: None,
            previous_response_id: None,
            store: Some(true),
            stream: None,
            temperature: None,
            max_output_tokens: None,
            parallel_tool_calls: Some(true),
            text: None,
            include: None,
            reasoning: Some(ReasoningConfig::low()),
            prompt_cache_key: Some("sess-123".into()),
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["reasoning"]["effort"], "low");
        assert_eq!(v["prompt_cache_key"], "sess-123");
        assert!(v["tools"].as_array().unwrap().iter().any(|t| t["type"] == "web_search"));
        assert!(v["tools"].as_array().unwrap().iter().any(|t| t["type"] == "code_execution"));
    }
}
