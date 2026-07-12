use serde::{Deserialize, Serialize};

/// Structured end-of-run reflection produced by Grok.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunReflection {
    #[serde(default)]
    pub wins: Vec<String>,
    #[serde(default)]
    pub failures: Vec<String>,
    #[serde(default)]
    pub conventions_learned: Vec<String>,
    #[serde(default)]
    pub stack_notes: Option<String>,
    #[serde(default)]
    pub command_notes: Option<String>,
    #[serde(default)]
    pub gotchas: Option<String>,
    #[serde(default)]
    pub new_lessons: Vec<NewLesson>,
    #[serde(default)]
    pub failure_records: Vec<NewFailure>,
    #[serde(default)]
    pub skill_updates: Vec<SkillUpdate>,
    #[serde(default)]
    pub agents_md_suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewLesson {
    pub kind: String,
    pub summary: String,
    pub detail: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "half")]
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFailure {
    pub tool: String,
    pub pattern: String,
    pub root_cause: String,
    pub fix: String,
    #[serde(default = "half")]
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdate {
    pub name: String,
    pub body: String,
}

fn half() -> f32 {
    0.6
}

pub fn reflection_json_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "wins": { "type": "array", "items": { "type": "string" } },
            "failures": { "type": "array", "items": { "type": "string" } },
            "conventions_learned": { "type": "array", "items": { "type": "string" } },
            "stack_notes": { "type": ["string", "null"] },
            "command_notes": { "type": ["string", "null"] },
            "gotchas": { "type": ["string", "null"] },
            "new_lessons": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string" },
                        "summary": { "type": "string" },
                        "detail": { "type": "string" },
                        "tags": { "type": "array", "items": { "type": "string" } },
                        "confidence": { "type": "number" }
                    },
                    "required": ["kind", "summary", "detail", "tags", "confidence"],
                    "additionalProperties": false
                }
            },
            "failure_records": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "tool": { "type": "string" },
                        "pattern": { "type": "string" },
                        "root_cause": { "type": "string" },
                        "fix": { "type": "string" },
                        "confidence": { "type": "number" }
                    },
                    "required": ["tool", "pattern", "root_cause", "fix", "confidence"],
                    "additionalProperties": false
                }
            },
            "skill_updates": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "body": { "type": "string" }
                    },
                    "required": ["name", "body"],
                    "additionalProperties": false
                }
            },
            "agents_md_suggestion": { "type": ["string", "null"] }
        },
        "required": [
            "wins", "failures", "conventions_learned", "stack_notes",
            "command_notes", "gotchas", "new_lessons", "failure_records",
            "skill_updates", "agents_md_suggestion"
        ],
        "additionalProperties": false
    })
}

pub fn reflection_system_prompt() -> &'static str {
    "You are Aegis project memory. Given a coding session transcript summary, extract durable lessons for THIS repository only. \
     No secrets. Prefer concrete commands, paths, and gotchas. Return the RunReflection JSON schema only."
}
