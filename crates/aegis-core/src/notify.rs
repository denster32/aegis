//! Optional completion notifications.

use std::env;
use tracing::info;

/// Fire-and-forget notify via webhook URL in AEGIS_NOTIFY_WEBHOOK or .aegis/notify.url
pub fn notify(title: &str, body: &str) {
    let url = env::var("AEGIS_NOTIFY_WEBHOOK").ok().or_else(|| {
        std::fs::read_to_string(
            std::env::current_dir()
                .unwrap_or_default()
                .join(".aegis/notify.url"),
        )
        .ok()
        .map(|s| s.trim().to_string())
    });
    let Some(url) = url else {
        return;
    };
    if url.is_empty() {
        return;
    }
    let payload = serde_json::json!({ "title": title, "body": body, "source": "aegis" });
    // blocking-ish via curl to avoid extra deps in async contexts
    let _ = std::process::Command::new("curl")
        .args([
            "-sS",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload.to_string(),
            &url,
        ])
        .output();
    info!(%title, "notify sent");
}
