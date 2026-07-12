use crate::registry::{Tool, ToolContext, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;

pub struct WebFetchTool;

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch a public HTTPS URL and return text content (truncated). Blocks private/localhost addresses."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string" },
                "max_chars": { "type": "integer", "default": 50000 }
            },
            "required": ["url"],
            "additionalProperties": false
        })
    }

    async fn call(&self, args: Value, _ctx: &ToolContext) -> ToolResult {
        let url = match args.get("url").and_then(|v| v.as_str()) {
            Some(u) => u.to_string(),
            None => return ToolResult::err("missing url"),
        };
        let max = args
            .get("max_chars")
            .and_then(|v| v.as_u64())
            .unwrap_or(50_000) as usize;

        if let Err(e) = ssrf_check(&url) {
            return ToolResult::err(e);
        }

        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
        {
            Ok(c) => c,
            Err(e) => return ToolResult::err(e.to_string()),
        };

        match client.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status();
                match resp.text().await {
                    Ok(mut text) => {
                        if text.len() > max {
                            text.truncate(max);
                            text.push_str("\n…[truncated]");
                        }
                        ToolResult::ok(format!("HTTP {status}\n\n{text}"))
                    }
                    Err(e) => ToolResult::err(e.to_string()),
                }
            }
            Err(e) => ToolResult::err(e.to_string()),
        }
    }
}

fn ssrf_check(url: &str) -> Result<(), String> {
    let u = url::Url::parse(url).map_err(|e| e.to_string())?;
    if u.scheme() != "https" && u.scheme() != "http" {
        return Err("only http/https allowed".into());
    }
    let host = u.host_str().unwrap_or("").to_lowercase();
    if host.is_empty()
        || host == "localhost"
        || host.ends_with(".local")
        || host == "metadata.google.internal"
    {
        return Err("blocked host".into());
    }
    // block obvious private IP literals
    if let Ok(std::net::IpAddr::V4(ip)) = host.parse() {
        if ip.is_private() || ip.is_loopback() || ip.is_link_local() {
            return Err("private IP blocked".into());
        }
    }
    Ok(())
}
