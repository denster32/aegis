use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct MemoryReadTool;

#[async_trait]
impl Tool for MemoryReadTool {
    fn name(&self) -> &str {
        "memory_read"
    }

    fn description(&self) -> &str {
        "Read project learning files under .aegis/ (MEMORY.md, lessons summary, or a skill)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "what": {
                    "type": "string",
                    "enum": ["memory", "lessons", "failures", "skills", "skill"],
                    "description": "Which memory surface to read"
                },
                "name": {
                    "type": "string",
                    "description": "Skill name when what=skill"
                }
            },
            "required": ["what"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let what = args.get("what").and_then(|v| v.as_str()).unwrap_or("memory");
        let mem = match aegis_memory::ProjectMemory::open(&ctx.cwd) {
            Ok(m) => m,
            Err(e) => return ToolResult::err(e.to_string()),
        };
        match what {
            "memory" => ToolResult::ok(mem.read_memory_md().unwrap_or_default()),
            "lessons" => {
                let lessons = mem.load_lessons().unwrap_or_default();
                ToolResult::ok(serde_json::to_string_pretty(&lessons).unwrap_or_default())
            }
            "failures" => {
                let f = mem.load_failures().unwrap_or_default();
                ToolResult::ok(serde_json::to_string_pretty(&f).unwrap_or_default())
            }
            "skills" => {
                let s = mem.list_skills().unwrap_or_default();
                let names: Vec<_> = s.into_iter().map(|(n, _)| n).collect();
                ToolResult::ok(names.join("\n"))
            }
            "skill" => {
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let s = mem.list_skills().unwrap_or_default();
                if let Some((_, body)) = s.into_iter().find(|(n, _)| n == name) {
                    ToolResult::ok(body)
                } else {
                    ToolResult::err(format!("skill not found: {name}"))
                }
            }
            _ => ToolResult::err("unknown what"),
        }
    }
}

pub struct MemoryWriteTool;

#[async_trait]
impl Tool for MemoryWriteTool {
    fn name(&self) -> &str {
        "memory_write"
    }

    fn description(&self) -> &str {
        "Append a lesson or failure fix, or write a skill playbook under .aegis/ for this project."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "kind": {
                    "type": "string",
                    "enum": ["lesson", "failure", "skill", "gotcha"]
                },
                "summary": { "type": "string" },
                "detail": { "type": "string" },
                "tool": { "type": "string" },
                "fix": { "type": "string" },
                "skill_name": { "type": "string" },
                "skill_body": { "type": "string" }
            },
            "required": ["kind"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let kind = args.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        let mem = match aegis_memory::ProjectMemory::open(&ctx.cwd) {
            Ok(m) => m,
            Err(e) => return ToolResult::err(e.to_string()),
        };
        match kind {
            "lesson" | "gotcha" => {
                let summary = args
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let detail = args
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let lesson = aegis_memory::Lesson {
                    id: uuid::Uuid::new_v4().to_string(),
                    ts: chrono::Utc::now().to_rfc3339(),
                    kind: kind.into(),
                    summary,
                    detail: detail.clone(),
                    tags: vec![],
                    confidence: 0.7,
                    hits: 1,
                };
                if let Err(e) = mem.append_lesson(&lesson) {
                    return ToolResult::err(e.to_string());
                }
                if kind == "gotcha" {
                    let _ = mem.merge_memory_sections(None, None, Some(&detail), None);
                }
                ToolResult::ok("lesson recorded")
            }
            "failure" => {
                let tool = args.get("tool").and_then(|v| v.as_str()).unwrap_or("unknown");
                let pattern = args
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("error");
                let fix = args.get("fix").and_then(|v| v.as_str()).unwrap_or("");
                let fp = aegis_memory::fingerprint(tool, pattern);
                let rec = aegis_memory::FailureRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    ts: chrono::Utc::now().to_rfc3339(),
                    fingerprint: fp,
                    tool: tool.into(),
                    pattern: pattern.into(),
                    root_cause: args
                        .get("detail")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .into(),
                    fix: fix.into(),
                    confidence: 0.75,
                    hits: 1,
                };
                if let Err(e) = mem.append_failure(&rec) {
                    return ToolResult::err(e.to_string());
                }
                ToolResult::ok("failure fix recorded")
            }
            "skill" => {
                let name = args
                    .get("skill_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("skill");
                let body = args
                    .get("skill_body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                match mem.write_skill(name, body) {
                    Ok(p) => ToolResult::ok(format!("wrote {}", p.display())),
                    Err(e) => ToolResult::err(e.to_string()),
                }
            }
            _ => ToolResult::err("unknown kind"),
        }
    }
}
