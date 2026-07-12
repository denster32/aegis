use crate::GROK_OIDC_CLIENT_ID;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthSource {
    EnvToken,
    AegisFile,
    GrokFile,
    ApiKey,
}

impl AuthSource {
    pub fn as_str(self) -> &'static str {
        match self {
            AuthSource::EnvToken => "env-token",
            AuthSource::AegisFile => "aegis-auth.json",
            AuthSource::GrokFile => "grok-auth.json",
            AuthSource::ApiKey => "api-key",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEntry {
    /// Access token (JWT) — Grok stores this as `key`.
    #[serde(alias = "access_token")]
    pub key: String,
    #[serde(default)]
    pub auth_mode: Option<String>,
    #[serde(default)]
    pub create_time: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub principal_type: Option<String>,
    #[serde(default)]
    pub principal_id: Option<String>,
    #[serde(default)]
    pub team_id: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub oidc_issuer: Option<String>,
    #[serde(default)]
    pub oidc_client_id: Option<String>,
    #[serde(default)]
    pub coding_data_retention_opt_out: Option<bool>,
}

impl AuthEntry {
    pub fn client_id(&self) -> &str {
        self.oidc_client_id
            .as_deref()
            .unwrap_or(GROK_OIDC_CLIENT_ID)
    }

    pub fn issuer(&self) -> &str {
        self.oidc_issuer
            .as_deref()
            .unwrap_or(crate::DEFAULT_ISSUER)
    }

    pub fn expires_at_dt(&self) -> Option<DateTime<Utc>> {
        self.expires_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc))
    }

    /// True if token expires within `skew_secs` (or already expired).
    pub fn needs_refresh(&self, skew_secs: i64) -> bool {
        match self.expires_at_dt() {
            Some(exp) => exp <= Utc::now() + chrono::Duration::seconds(skew_secs),
            // No expiry known — still refresh if we have a refresh_token and token looks old
            None => false,
        }
    }

    pub fn map_key(&self) -> String {
        format!("{}::{}", self.issuer(), self.client_id())
    }
}

pub type AuthFile = HashMap<String, AuthEntry>;

#[derive(Debug, Clone)]
pub struct AuthPaths {
    pub aegis: PathBuf,
    pub grok: PathBuf,
}

pub fn auth_paths() -> AuthPaths {
    let home = directories::UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    // Prefer XDG data for aegis auth alongside store; also allow ~/.aegis
    let aegis = directories::ProjectDirs::from("dev", "aegis", "aegis")
        .map(|p| p.data_dir().join("auth.json"))
        .unwrap_or_else(|| home.join(".aegis").join("auth.json"));
    let grok = home.join(".grok").join("auth.json");
    AuthPaths { aegis, grok }
}

pub fn read_auth_file(path: &Path) -> Result<Option<AuthFile>> {
    if !path.exists() {
        return Ok(None);
    }
    let mut f = File::open(path).with_context(|| format!("open {}", path.display()))?;
    // Shared lock for read
    let _ = f.lock_shared();
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let _ = f.unlock();
    if buf.trim().is_empty() {
        return Ok(None);
    }
    let map: AuthFile = serde_json::from_str(&buf)
        .with_context(|| format!("parse auth file {}", path.display()))?;
    Ok(Some(map))
}

pub fn write_auth_file(path: &Path, file: &AuthFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp)
            .with_context(|| format!("open tmp {}", tmp.display()))?;
        f.lock_exclusive()?;
        let data = serde_json::to_string_pretty(file)?;
        f.write_all(data.as_bytes())?;
        f.sync_all()?;
        f.unlock()?;
    }
    fs::rename(&tmp, path)?;
    // Restrict permissions best-effort
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Load first usable OIDC entry from a file.
pub fn first_entry(file: &AuthFile) -> Option<(String, AuthEntry)> {
    file.iter()
        .find(|(_, e)| !e.key.is_empty())
        .map(|(k, e)| (k.clone(), e.clone()))
}

pub fn upsert_entry(path: &Path, entry: AuthEntry) -> Result<()> {
    let mut file = read_auth_file(path)?.unwrap_or_default();
    let key = entry.map_key();
    file.insert(key, entry);
    write_auth_file(path, &file)
}

pub fn clear_auth_file(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Import Grok auth into Aegis auth file (copy).
pub fn import_grok_to_aegis() -> Result<AuthEntry> {
    let paths = auth_paths();
    let grok = read_auth_file(&paths.grok)?
        .context("no ~/.grok/auth.json — run `grok login` first")?;
    let (_, entry) = first_entry(&grok).context("empty grok auth file")?;
    upsert_entry(&paths.aegis, entry.clone())?;
    Ok(entry)
}
