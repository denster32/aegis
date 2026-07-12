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
        self.oidc_issuer.as_deref().unwrap_or(crate::DEFAULT_ISSUER)
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
    let grok =
        read_auth_file(&paths.grok)?.context("no ~/.grok/auth.json — run `grok login` first")?;
    let (_, entry) = first_entry(&grok).context("empty grok auth file")?;
    upsert_entry(&paths.aegis, entry.clone())?;
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn sample_entry(key: &str) -> AuthEntry {
        AuthEntry {
            key: key.into(),
            auth_mode: Some("oauth".into()),
            create_time: None,
            user_id: Some("u1".into()),
            email: Some("user@x.ai".into()),
            first_name: None,
            last_name: None,
            principal_type: None,
            principal_id: None,
            team_id: Some("team".into()),
            refresh_token: Some("refresh".into()),
            expires_at: None,
            oidc_issuer: None,
            oidc_client_id: None,
            coding_data_retention_opt_out: None,
        }
    }

    #[test]
    fn auth_source_as_str() {
        assert_eq!(AuthSource::EnvToken.as_str(), "env-token");
        assert_eq!(AuthSource::AegisFile.as_str(), "aegis-auth.json");
        assert_eq!(AuthSource::GrokFile.as_str(), "grok-auth.json");
        assert_eq!(AuthSource::ApiKey.as_str(), "api-key");
    }

    #[test]
    fn entry_defaults_and_map_key() {
        let e = sample_entry("tok");
        assert_eq!(e.client_id(), GROK_OIDC_CLIENT_ID);
        assert_eq!(e.issuer(), crate::DEFAULT_ISSUER);
        assert_eq!(
            e.map_key(),
            format!("{}::{}", crate::DEFAULT_ISSUER, GROK_OIDC_CLIENT_ID)
        );
    }

    #[test]
    fn needs_refresh_respects_expiry() {
        let mut e = sample_entry("tok");
        assert!(!e.needs_refresh(120));

        e.expires_at = Some((Utc::now() - Duration::seconds(10)).to_rfc3339());
        assert!(e.needs_refresh(120));

        e.expires_at = Some((Utc::now() + Duration::hours(2)).to_rfc3339());
        assert!(!e.needs_refresh(120));

        // Within skew window
        e.expires_at = Some((Utc::now() + Duration::seconds(30)).to_rfc3339());
        assert!(e.needs_refresh(120));
    }

    #[test]
    fn first_entry_skips_empty_keys() {
        let mut file = AuthFile::new();
        file.insert("empty".into(), sample_entry(""));
        assert!(first_entry(&file).is_none());
        file.insert("good".into(), sample_entry("real-token"));
        let (k, e) = first_entry(&file).unwrap();
        assert!(!e.key.is_empty());
        assert!(k == "empty" || k == "good");
        // ensure we got the non-empty one
        assert_eq!(e.key, "real-token");
    }

    #[test]
    fn access_token_alias_deserializes() {
        let json = r#"{
            "https://auth.x.ai::cid": {
                "access_token": "jwt-here",
                "email": "a@b.c"
            }
        }"#;
        let file: AuthFile = serde_json::from_str(json).unwrap();
        let (_, e) = first_entry(&file).unwrap();
        assert_eq!(e.key, "jwt-here");
        assert_eq!(e.email.as_deref(), Some("a@b.c"));
    }

    #[test]
    fn write_read_clear_auth_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        assert!(read_auth_file(&path).unwrap().is_none());

        let mut file = AuthFile::new();
        let entry = sample_entry("access-xyz");
        file.insert(entry.map_key(), entry.clone());
        write_auth_file(&path, &file).unwrap();
        assert!(path.exists());

        let loaded = read_auth_file(&path).unwrap().unwrap();
        let (_, got) = first_entry(&loaded).unwrap();
        assert_eq!(got.key, "access-xyz");
        assert_eq!(got.email.as_deref(), Some("user@x.ai"));

        clear_auth_file(&path).unwrap();
        assert!(!path.exists());
        assert!(read_auth_file(&path).unwrap().is_none());
        // clearing missing is ok
        clear_auth_file(&path).unwrap();
    }

    #[test]
    fn upsert_entry_merges() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let e1 = sample_entry("v1");
        upsert_entry(&path, e1).unwrap();
        let mut e2 = sample_entry("v2");
        e2.email = Some("other@x.ai".into());
        upsert_entry(&path, e2).unwrap();
        let loaded = read_auth_file(&path).unwrap().unwrap();
        assert_eq!(loaded.len(), 1);
        let (_, got) = first_entry(&loaded).unwrap();
        assert_eq!(got.key, "v2");
        assert_eq!(got.email.as_deref(), Some("other@x.ai"));
    }

    #[test]
    fn empty_file_reads_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.json");
        std::fs::write(&path, "   \n").unwrap();
        assert!(read_auth_file(&path).unwrap().is_none());
    }
}
