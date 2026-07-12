use crate::file::{
    auth_paths, first_entry, read_auth_file, upsert_entry, AuthEntry, AuthPaths, AuthSource,
};
use crate::refresh::refresh_entry;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, warn};

const REFRESH_SKEW_SECS: i64 = 120;

#[derive(Debug, Clone)]
pub struct Credential {
    pub token: String,
    pub source: AuthSource,
    pub email: Option<String>,
    pub expires_at: Option<String>,
    pub is_api_key: bool,
}

#[derive(Debug, Clone)]
pub struct AuthStatus {
    pub source: AuthSource,
    pub email: Option<String>,
    pub expires_at: Option<String>,
    pub team_id: Option<String>,
    pub auth_mode: Option<String>,
    pub path: Option<String>,
    pub needs_refresh: bool,
}

#[async_trait]
pub trait TokenSource: Send + Sync {
    async fn token(&self) -> Result<String>;
    /// Force refresh if OIDC; no-op for static API keys.
    async fn force_refresh(&self) -> Result<()>;
    fn status(&self) -> AuthStatus;
}

/// Resolves and refreshes credentials for Aegis.
pub struct AuthProvider {
    inner: Mutex<ProviderState>,
    paths: AuthPaths,
    http_refresh: bool,
}

struct ProviderState {
    cred: Credential,
    /// OIDC entry when applicable (for refresh).
    entry: Option<AuthEntry>,
    entry_path: Option<std::path::PathBuf>,
}

impl AuthProvider {
    /// Resolve credentials from env + auth files.
    pub fn resolve() -> Result<Arc<Self>> {
        let paths = auth_paths();

        // 1. Explicit access token
        if let Ok(t) = std::env::var("AEGIS_ACCESS_TOKEN")
            .or_else(|_| std::env::var("XAI_ACCESS_TOKEN"))
        {
            if !t.is_empty() {
                return Ok(Arc::new(Self {
                    inner: Mutex::new(ProviderState {
                        cred: Credential {
                            token: t,
                            source: AuthSource::EnvToken,
                            email: None,
                            expires_at: None,
                            is_api_key: false,
                        },
                        entry: None,
                        entry_path: None,
                    }),
                    paths,
                    http_refresh: false,
                }));
            }
        }

        // 2. Aegis auth file
        if let Some(file) = read_auth_file(&paths.aegis)? {
            if let Some((_, entry)) = first_entry(&file) {
                return Ok(Arc::new(Self::from_entry(
                    entry,
                    AuthSource::AegisFile,
                    Some(paths.aegis.clone()),
                    paths,
                )));
            }
        }

        // 3. Grok auth file
        if let Some(file) = read_auth_file(&paths.grok)? {
            if let Some((_, entry)) = first_entry(&file) {
                return Ok(Arc::new(Self::from_entry(
                    entry,
                    AuthSource::GrokFile,
                    Some(paths.grok.clone()),
                    paths,
                )));
            }
        }

        // 4. API key
        if let Ok(k) = std::env::var("XAI_API_KEY").or_else(|_| std::env::var("SPACEXAI_API_KEY")) {
            if !k.is_empty() {
                return Ok(Arc::new(Self {
                    inner: Mutex::new(ProviderState {
                        cred: Credential {
                            token: k,
                            source: AuthSource::ApiKey,
                            email: None,
                            expires_at: None,
                            is_api_key: true,
                        },
                        entry: None,
                        entry_path: None,
                    }),
                    paths,
                    http_refresh: false,
                }));
            }
        }

        bail!(
            "Not signed in. Run `aegis login` or `grok login`, or set XAI_API_KEY.\n\
             Tip: Aegis reuses ~/.grok/auth.json automatically after `grok login`."
        )
    }

    fn from_entry(
        entry: AuthEntry,
        source: AuthSource,
        entry_path: Option<std::path::PathBuf>,
        paths: AuthPaths,
    ) -> Self {
        Self {
            inner: Mutex::new(ProviderState {
                cred: Credential {
                    token: entry.key.clone(),
                    source,
                    email: entry.email.clone(),
                    expires_at: entry.expires_at.clone(),
                    is_api_key: false,
                },
                entry: Some(entry),
                entry_path,
            }),
            paths,
            http_refresh: true,
        }
    }

    pub fn paths(&self) -> &AuthPaths {
        &self.paths
    }

    pub fn status_snapshot(&self) -> AuthStatus {
        let g = self.inner.lock();
        let needs = g
            .entry
            .as_ref()
            .map(|e| e.needs_refresh(REFRESH_SKEW_SECS))
            .unwrap_or(false);
        AuthStatus {
            source: g.cred.source,
            email: g.cred.email.clone(),
            expires_at: g.cred.expires_at.clone(),
            team_id: g.entry.as_ref().and_then(|e| e.team_id.clone()),
            auth_mode: g.entry.as_ref().and_then(|e| e.auth_mode.clone()),
            path: g
                .entry_path
                .as_ref()
                .map(|p| p.display().to_string()),
            needs_refresh: needs,
        }
    }

    async fn ensure_fresh_locked(&self) -> Result<()> {
        let (needs, entry) = {
            let g = self.inner.lock();
            if !self.http_refresh {
                return Ok(());
            }
            let Some(entry) = g.entry.clone() else {
                return Ok(());
            };
            (entry.needs_refresh(REFRESH_SKEW_SECS), entry)
        };
        if !needs {
            return Ok(());
        }
        debug!("access token near expiry; refreshing");
        self.apply_refresh(entry).await
    }

    async fn apply_refresh(&self, entry: AuthEntry) -> Result<()> {
        let next = refresh_entry(&entry).await?;
        let path = {
            let g = self.inner.lock();
            g.entry_path.clone()
        };
        if let Some(path) = path {
            if let Err(e) = upsert_entry(&path, next.clone()) {
                warn!(error = %e, "failed to persist refreshed token");
            }
        }
        let mut g = self.inner.lock();
        g.cred.token = next.key.clone();
        g.cred.expires_at = next.expires_at.clone();
        g.cred.email = next.email.clone().or(g.cred.email.clone());
        g.entry = Some(next);
        Ok(())
    }
}

#[async_trait]
impl TokenSource for AuthProvider {
    async fn token(&self) -> Result<String> {
        self.ensure_fresh_locked().await?;
        Ok(self.inner.lock().cred.token.clone())
    }

    async fn force_refresh(&self) -> Result<()> {
        let entry = {
            let g = self.inner.lock();
            g.entry.clone().context("no OIDC entry to refresh")?
        };
        self.apply_refresh(entry).await
    }

    fn status(&self) -> AuthStatus {
        self.status_snapshot()
    }
}

/// Static token source (tests / simple).
#[allow(dead_code)]
pub struct StaticToken(pub String);

#[async_trait]
impl TokenSource for StaticToken {
    async fn token(&self) -> Result<String> {
        Ok(self.0.clone())
    }
    async fn force_refresh(&self) -> Result<()> {
        Ok(())
    }
    fn status(&self) -> AuthStatus {
        AuthStatus {
            source: AuthSource::EnvToken,
            email: None,
            expires_at: None,
            team_id: None,
            auth_mode: None,
            path: None,
            needs_refresh: false,
        }
    }
}
