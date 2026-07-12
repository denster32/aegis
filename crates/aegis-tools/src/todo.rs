use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct TodoWriteTool;

#[async_trait]
impl Tool for TodoWriteTool {
    fn name(&self) -> &str {
        "todo_write"
    }

    fn description(&self) -> &str {
        "Replace the session todo list. Pass a JSON array of {id, content, status} items."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "content": { "type": "string" },
                            "status": {
                                "type": "string",
                                "enum": ["pending", "in_progress", "completed", "cancelled"]
                            }
                        },
                        "required": ["id", "content", "status"]
                    }
                }
            },
            "required": ["todos"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let todos = match args.get("todos") {
            Some(t) => t,
            None => return ToolResult::err("missing todos"),
        };
        let json = match serde_json::to_string_pretty(todos) {
            Ok(s) => s,
            Err(e) => return ToolResult::err(e.to_string()),
        };
        if let Some(store) = &ctx.todo_store {
            if let Err(e) = store.set_todos(&ctx.session_id, &json) {
                return ToolResult::err(format!("store: {e}"));
            }
        }
        ToolResult::ok(format!("updated todos:\n{json}"))
    }
}
