use crate::file::{upsert_entry, AuthEntry, AuthPaths};
use crate::{DEFAULT_ISSUER, GROK_OIDC_CLIENT_ID};
use anyhow::{bail, Context, Result};
use chrono::{Duration, Utc};
use serde::Deserialize;
use std::io::{self, Write};
use std::time::Duration as StdDuration;
use tracing::info;

const SCOPES: &str = "openid profile email offline_access api:access grok-cli:access";

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default)]
    verification_uri_complete: Option<String>,
    expires_in: u64,
    interval: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    error: Option<String>,
    error_description: Option<String>,
}

/// Device-code OAuth login (headless-friendly).
pub async fn device_login(paths: &AuthPaths, write_aegis: bool) -> Result<AuthEntry> {
    let issuer = DEFAULT_ISSUER;
    let client_id = GROK_OIDC_CLIENT_ID;
    let http = reqwest::Client::builder()
        .timeout(StdDuration::from_secs(60))
        .build()?;

    let dc_url = format!("{issuer}/oauth2/device/code");
    let resp = http
        .post(&dc_url)
        .form(&[
            ("client_id", client_id),
            ("scope", SCOPES),
        ])
        .send()
        .await
        .context("device code request")?;
    let status = resp.status();
    let body = resp.text().await?;
    if !status.is_success() {
        bail!("device code failed ({status}): {body}");
    }
    let dc: DeviceCodeResponse = serde_json::from_str(&body)?;

    println!("\n  Aegis login (device code)");
    println!("  ─────────────────────────");
    println!("  Open:  {}", dc.verification_uri);
    if let Some(ref full) = dc.verification_uri_complete {
        println!("  Or:    {full}");
    }
    println!("  Code:  {}\n", dc.user_code);
    println!("  Waiting for approval…");
    let _ = io::stdout().flush();

    let token_url = format!("{issuer}/oauth2/token");
    let interval = dc.interval.unwrap_or(5).max(1);
    let deadline = std::time::Instant::now() + StdDuration::from_secs(dc.expires_in);

    loop {
        if std::time::Instant::now() > deadline {
            bail!("device login timed out");
        }
        tokio::time::sleep(StdDuration::from_secs(interval)).await;

        let resp = http
            .post(&token_url)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", dc.device_code.as_str()),
                ("client_id", client_id),
            ])
            .send()
            .await?;
        let body = resp.text().await?;
        let tr: TokenResponse = serde_json::from_str(&body).unwrap_or(TokenResponse {
            access_token: None,
            refresh_token: None,
            expires_in: None,
            error: Some("parse_error".into()),
            error_description: Some(body.clone()),
        });

        if let Some(err) = tr.error.as_deref() {
            match err {
                "authorization_pending" | "slow_down" => {
                    if err == "slow_down" {
                        tokio::time::sleep(StdDuration::from_secs(interval)).await;
                    }
                    print!(".");
                    let _ = io::stdout().flush();
                    continue;
                }
                other => {
                    bail!(
                        "device login error: {other} {}",
                        tr.error_description.unwrap_or_default()
                    );
                }
            }
        }

        let access = tr.access_token.context("no access_token in response")?;
        let expires_in = tr.expires_in.unwrap_or(21600);
        let entry = AuthEntry {
            key: access,
            auth_mode: Some("oidc".into()),
            create_time: Some(Utc::now().to_rfc3339()),
            user_id: None,
            email: None,
            first_name: None,
            last_name: None,
            principal_type: Some("User".into()),
            principal_id: None,
            team_id: None,
            refresh_token: tr.refresh_token,
            expires_at: Some((Utc::now() + Duration::seconds(expires_in)).to_rfc3339()),
            oidc_issuer: Some(issuer.into()),
            oidc_client_id: Some(client_id.into()),
            coding_data_retention_opt_out: None,
        };

        // Enrich from userinfo if possible
        let mut entry = entry;
        if let Ok(ui) = fetch_userinfo(&http, issuer, &entry.key).await {
            entry.email = ui.email.or(entry.email);
            entry.user_id = ui.sub.or(entry.user_id);
            entry.first_name = ui.given_name.or(entry.first_name);
            entry.last_name = ui.family_name.or(entry.last_name);
        }

        if write_aegis {
            upsert_entry(&paths.aegis, entry.clone())?;
            info!(path = %paths.aegis.display(), "wrote aegis auth");
        } else {
            upsert_entry(&paths.grok, entry.clone())?;
            info!(path = %paths.grok.display(), "wrote grok auth");
        }
        println!("\n  Logged in.\n");
        return Ok(entry);
    }
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    sub: Option<String>,
    email: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
}

async fn fetch_userinfo(
    http: &reqwest::Client,
    issuer: &str,
    token: &str,
) -> Result<UserInfo> {
    let url = format!("{}/oauth2/userinfo", issuer.trim_end_matches('/'));
    let resp = http
        .get(url)
        .bearer_auth(token)
        .send()
        .await?;
    if !resp.status().is_success() {
        bail!("userinfo failed");
    }
    Ok(resp.json().await?)
}
