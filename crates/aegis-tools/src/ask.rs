use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct AskUserTool;

#[async_trait]
impl Tool for AskUserTool {
    fn name(&self) -> &str {
        "ask_user"
    }

    fn description(&self) -> &str {
        "Ask the human a question and wait for their answer. Use when blocked or need a preference."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "question": { "type": "string" },
                "options": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional multiple-choice options"
                }
            },
            "required": ["question"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let question = match args.get("question").and_then(|v| v.as_str()) {
            Some(q) => q,
            None => return ToolResult::err("missing question"),
        };
        let options: Vec<String> = args
            .get("options")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut prompt = format!("\n❓ {question}\n");
        for (i, o) in options.iter().enumerate() {
            prompt.push_str(&format!("  {}) {}\n", i + 1, o));
        }
        prompt.push_str("Answer: ");

        if let Some(ask) = &ctx.ask {
            let ans = ask(&prompt);
            ToolResult::ok(ans.trim().to_string())
        } else {
            ToolResult::err("ask_user unavailable in non-interactive mode; use --yolo defaults or provide enough context")
        }
    }
}
