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
    // Prefer structured host (handles IPv6 bracket form).
    let host = match u.host() {
        Some(url::Host::Domain(d)) => d.to_lowercase(),
        Some(url::Host::Ipv4(ip)) => {
            if is_blocked_ip(std::net::IpAddr::V4(ip)) {
                return Err("private IP blocked".into());
            }
            return Ok(());
        }
        Some(url::Host::Ipv6(ip)) => {
            if is_blocked_ip(std::net::IpAddr::V6(ip)) {
                return Err("private IP blocked".into());
            }
            return Ok(());
        }
        None => return Err("blocked host".into()),
    };
    if host.is_empty()
        || host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host == "metadata.google.internal"
        || host == "metadata"
        || host.ends_with(".internal")
    {
        return Err("blocked host".into());
    }
    // Domain that happens to be a literal IP string (rare but cheap to check).
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if is_blocked_ip(ip) {
            return Err("private IP blocked".into());
        }
    }
    Ok(())
}

fn is_blocked_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || v4.is_documentation()
                // CGNAT / shared address space 100.64.0.0/10
                || (v4.octets()[0] == 100 && (v4.octets()[1] & 0xc0) == 64)
                // Benchmarking 198.18.0.0/15
                || (v4.octets()[0] == 198 && (v4.octets()[1] & 0xfe) == 18)
        }
        std::net::IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_unique_local()
                || v6.is_unicast_link_local()
                // IPv4-mapped / compatible IPv6 — re-check embedded v4.
                || v6
                    .to_ipv4()
                    .map(|v4| is_blocked_ip(std::net::IpAddr::V4(v4)))
                    .unwrap_or(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssrf_blocks_private_and_localhost() {
        assert!(ssrf_check("http://127.0.0.1/").is_err());
        assert!(ssrf_check("http://localhost/").is_err());
        assert!(ssrf_check("http://10.0.0.1/").is_err());
        assert!(ssrf_check("http://192.168.1.1/").is_err());
        assert!(ssrf_check("http://172.16.0.1/").is_err());
        assert!(ssrf_check("http://169.254.169.254/").is_err());
        assert!(ssrf_check("http://[::1]/").is_err());
        assert!(ssrf_check("http://[fc00::1]/").is_err());
        assert!(ssrf_check("http://[fe80::1]/").is_err());
        assert!(ssrf_check("http://[::ffff:127.0.0.1]/").is_err());
        assert!(ssrf_check("http://metadata.google.internal/").is_err());
        assert!(ssrf_check("http://0.0.0.0/").is_err());
        assert!(ssrf_check("http://100.64.0.1/").is_err());
    }

    #[test]
    fn ssrf_allows_public_https() {
        assert!(ssrf_check("https://example.com/path").is_ok());
        assert!(ssrf_check("https://1.1.1.1/").is_ok());
    }
}
