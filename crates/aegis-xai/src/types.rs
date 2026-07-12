use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request body for POST /v1/responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponseRequest {
    pub model: String,
    pub input: Vec<InputItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDef>>,
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
