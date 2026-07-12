//! Vision: describe local images via xAI (used by CLI; agent can call as tool).

use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

pub struct VisionDescribeTool;

#[async_trait]
impl Tool for VisionDescribeTool {
    fn name(&self) -> &str {
        "vision_describe"
    }

    fn description(&self) -> &str {
        "Describe a local image file (png/jpg) using the vision model. Path relative to workspace or absolute."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "question": {
                    "type": "string",
                    "description": "What to look for",
                    "default": "Describe this image and note any UI/layout issues."
                }
            },
            "required": ["path"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => ctx.resolve_path(p),
            None => return ToolResult::err("missing path"),
        };
        let question = args
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("Describe this image and note any UI/layout issues.");
        if !path.exists() {
            return ToolResult::err(format!("not found: {}", path.display()));
        }
        match describe_image_file(&path, question).await {
            Ok(s) => ToolResult::ok(s),
            Err(e) => ToolResult::err(e.to_string()),
        }
    }
}

/// Encode image as data URL and call Responses API with image input.
pub async fn describe_image_file(
    path: &std::path::Path,
    question: &str,
) -> anyhow::Result<String> {
    let bytes = fs::read(path)?;
    if bytes.len() > 15_000_000 {
        anyhow::bail!("image too large");
    }
    let mime = match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "image/png",
    };
    use base64::{engine::general_purpose::STANDARD, Engine};
    let b64 = STANDARD.encode(&bytes);
    let data_url = format!("data:{mime};base64,{b64}");

    // Resolve token via env or grok auth file using existing CLI path: curl with bearer
    let token = resolve_bearer()?;
    let body = serde_json::json!({
        "model": std::env::var("AEGIS_VISION_MODEL").unwrap_or_else(|_| "grok-4.5".into()),
        "input": [{
            "role": "user",
            "content": [
                { "type": "input_text", "text": question },
                { "type": "input_image", "image_url": data_url }
            ]
        }],
        "store": false
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let resp = client
        .post("https://api.x.ai/v1/responses")
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        // fallback: try chat completions style
        anyhow::bail!("vision API {status}: {}", text.chars().take(400).collect::<String>());
    }
    // extract output_text
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
        let mut out = String::new();
        if let Some(arr) = v.get("output").and_then(|o| o.as_array()) {
            for item in arr {
                if item.get("type").and_then(|t| t.as_str()) == Some("message") {
                    if let Some(content) = item.get("content").and_then(|c| c.as_array()) {
                        for part in content {
                            if let Some(t) = part.get("text").and_then(|t| t.as_str()) {
                                out.push_str(t);
                            }
                        }
                    }
                }
            }
        }
        if !out.is_empty() {
            return Ok(out);
        }
    }
    Ok(text.chars().take(2000).collect())
}

fn resolve_bearer() -> anyhow::Result<String> {
    if let Ok(t) = std::env::var("AEGIS_ACCESS_TOKEN").or_else(|_| std::env::var("XAI_ACCESS_TOKEN"))
    {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    if let Ok(k) = std::env::var("XAI_API_KEY") {
        if !k.is_empty() {
            return Ok(k);
        }
    }
    // grok auth.json
    let home = std::env::var("HOME")?;
    let path = std::path::PathBuf::from(home).join(".grok/auth.json");
    let text = fs::read_to_string(path)?;
    let v: serde_json::Value = serde_json::from_str(&text)?;
    if let Some(map) = v.as_object() {
        for (_k, entry) in map {
            if let Some(key) = entry.get("key").and_then(|k| k.as_str()) {
                return Ok(key.to_string());
            }
        }
    }
    anyhow::bail!("no auth token for vision")
}

pub struct ScreenshotTool;

#[async_trait]
impl Tool for ScreenshotTool {
    fn name(&self) -> &str {
        "screenshot"
    }

    fn description(&self) -> &str {
        "Capture a screenshot of the screen or a file preview to .aegis/screenshots/ (requires import or scrot)."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "default": "shot" }
            },
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, ctx: &ToolContext) -> ToolResult {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("shot");
        let dir = ctx.cwd.join(".aegis/screenshots");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join(format!("{name}.png"));
        // try import (ImageMagick), then scrot, then gnome-screenshot
        let cmds = [
            vec![
                "import".into(),
                "-window".into(),
                "root".into(),
                path.display().to_string(),
            ],
            vec!["scrot".into(), path.display().to_string()],
            vec![
                "gnome-screenshot".into(),
                "-f".into(),
                path.display().to_string(),
            ],
        ];
        for c in cmds {
            let (bin, args) = c.split_first().unwrap();
            if Command::new(bin).args(args).status().map(|s| s.success()).unwrap_or(false)
                && path.exists()
            {
                return ToolResult::ok(path.display().to_string());
            }
        }
        // fallback: copy logo if exists in repo
        let logo = ctx.cwd.join("assets/logo.png");
        if logo.exists() {
            let _ = fs::copy(&logo, &path);
            return ToolResult::ok(format!(
                "{} (fallback copy of assets/logo.png — no screenshot tool)",
                path.display()
            ));
        }
        ToolResult::err("no screenshot utility (import/scrot) available")
    }
}
