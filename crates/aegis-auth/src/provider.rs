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
        if let Ok(t) =
            std::env::var("AEGIS_ACCESS_TOKEN").or_else(|_| std::env::var("XAI_ACCESS_TOKEN"))
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
            path: g.entry_path.as_ref().map(|p| p.display().to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    /// Serialize env-var mutation across tests in this crate.
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        keys: Vec<&'static str>,
        saved: Vec<Option<String>>,
    }

    impl EnvGuard {
        fn capture(keys: &[&'static str]) -> Self {
            let saved = keys.iter().map(|k| std::env::var(k).ok()).collect();
            for k in keys {
                std::env::remove_var(k);
            }
            Self {
                keys: keys.to_vec(),
                saved,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (k, v) in self.keys.iter().zip(self.saved.iter()) {
                match v {
                    Some(val) => std::env::set_var(k, val),
                    None => std::env::remove_var(k),
                }
            }
        }
    }

    const AUTH_ENV_KEYS: &[&str] = &[
        "AEGIS_ACCESS_TOKEN",
        "XAI_ACCESS_TOKEN",
        "XAI_API_KEY",
        "SPACEXAI_API_KEY",
    ];

    #[test]
    fn resolve_prefers_aegis_access_token() {
        let _g = env_lock().lock().unwrap();
        let _env = EnvGuard::capture(AUTH_ENV_KEYS);
        std::env::set_var("AEGIS_ACCESS_TOKEN", "env-access-token");
        std::env::set_var("XAI_API_KEY", "should-not-win");
        let p = AuthProvider::resolve().unwrap();
        let st = p.status_snapshot();
        assert_eq!(st.source, AuthSource::EnvToken);
        assert!(!st.needs_refresh);
        // token() is async; use runtime-free field via status + force path
        let tok = futures_executor_token(&p);
        assert_eq!(tok, "env-access-token");
    }

    #[test]
    fn resolve_xai_access_token_alias() {
        let _g = env_lock().lock().unwrap();
        let _env = EnvGuard::capture(AUTH_ENV_KEYS);
        std::env::set_var("XAI_ACCESS_TOKEN", "xai-access");
        let p = AuthProvider::resolve().unwrap();
        assert_eq!(p.status_snapshot().source, AuthSource::EnvToken);
        assert_eq!(futures_executor_token(&p), "xai-access");
    }

    #[test]
    fn resolve_api_key_when_no_access_token() {
        let _g = env_lock().lock().unwrap();
        let _env = EnvGuard::capture(AUTH_ENV_KEYS);
        // Auth files on the host may still resolve — only assert API key
        // wins over nothing when those are absent, or when we force API key
        // with empty access tokens. If files exist, source may be Aegis/Grok.
        // Prefer testing StaticToken + explicit ApiKey construction path via env
        // only when resolve hits step 4: clear tokens, set key, and accept
        // file-or-key outcomes that are never EnvToken.
        std::env::set_var("XAI_API_KEY", "sk-test-api-key");
        let p = AuthProvider::resolve();
        // Must succeed somehow (file or api key)
        let p = p.expect("resolve should find file or api key");
        let src = p.status_snapshot().source;
        assert!(
            matches!(
                src,
                AuthSource::ApiKey | AuthSource::AegisFile | AuthSource::GrokFile
            ),
            "unexpected source {src:?}"
        );
        if src == AuthSource::ApiKey {
            assert_eq!(futures_executor_token(&p), "sk-test-api-key");
            assert!(p.status_snapshot().source == AuthSource::ApiKey);
        }
    }

    #[test]
    fn resolve_spacexai_api_key_alias() {
        let _g = env_lock().lock().unwrap();
        let _env = EnvGuard::capture(AUTH_ENV_KEYS);
        std::env::set_var("SPACEXAI_API_KEY", "spacex-key");
        let p = AuthProvider::resolve().expect("file or spacex key");
        if p.status_snapshot().source == AuthSource::ApiKey {
            assert_eq!(futures_executor_token(&p), "spacex-key");
        }
    }

    #[test]
    fn empty_access_token_falls_through() {
        let _g = env_lock().lock().unwrap();
        let _env = EnvGuard::capture(AUTH_ENV_KEYS);
        std::env::set_var("AEGIS_ACCESS_TOKEN", "");
        std::env::set_var("XAI_API_KEY", "from-api");
        let p = AuthProvider::resolve().expect("should resolve");
        // Empty access token must not be treated as EnvToken
        assert_ne!(p.status_snapshot().source, AuthSource::EnvToken);
    }

    #[test]
    fn static_token_source() {
        let s = StaticToken("static".into());
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert_eq!(rt.block_on(s.token()).unwrap(), "static");
        rt.block_on(s.force_refresh()).unwrap();
        assert_eq!(s.status().source, AuthSource::EnvToken);
    }

    fn futures_executor_token(p: &AuthProvider) -> String {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(p.token()).unwrap()
    }
}
