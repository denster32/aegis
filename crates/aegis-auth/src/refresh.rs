use crate::file::AuthEntry;
use anyhow::{bail, Context, Result};
use chrono::{Duration, Utc};
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    token_type: Option<String>,
}

/// Refresh an OIDC entry via the issuer token endpoint.
pub async fn refresh_entry(entry: &AuthEntry) -> Result<AuthEntry> {
    let refresh = entry
        .refresh_token
        .as_deref()
        .context("no refresh_token on auth entry")?;
    let issuer = entry.issuer().trim_end_matches('/');
    let url = format!("{issuer}/oauth2/token");
    let client_id = entry.client_id();

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let resp = http
        .post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh),
            ("client_id", client_id),
        ])
        .send()
        .await
        .context("token refresh request")?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        bail!("token refresh failed ({status}): {body}");
    }

    let tr: TokenResponse = serde_json::from_str(&body).context("parse token response")?;
    if tr.token_type.as_deref().unwrap_or("bearer").to_lowercase() != "bearer" {
        // still accept
    }

    let mut next = entry.clone();
    next.key = tr.access_token;
    if let Some(rt) = tr.refresh_token {
        next.refresh_token = Some(rt);
    }
    let expires_in = tr.expires_in.unwrap_or(21600); // default 6h
    next.expires_at = Some((Utc::now() + Duration::seconds(expires_in)).to_rfc3339());
    info!(expires_in, "refreshed OIDC access token");
    Ok(next)
}
