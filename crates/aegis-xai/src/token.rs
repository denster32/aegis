use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Pluggable bearer token provider (API key or OAuth).
#[async_trait]
pub trait TokenSource: Send + Sync {
    async fn bearer_token(&self) -> Result<String>;
    /// Called after HTTP 401 — refresh OAuth if possible.
    async fn on_unauthorized(&self) -> Result<()> {
        Ok(())
    }
}

/// Static string token (API key or pre-fetched JWT).
pub struct StaticToken {
    pub token: String,
}

#[async_trait]
impl TokenSource for StaticToken {
    async fn bearer_token(&self) -> Result<String> {
        Ok(self.token.clone())
    }
}

/// Adapter from any async token fn.
pub struct ArcToken(pub Arc<dyn TokenSource>);

#[async_trait]
impl TokenSource for ArcToken {
    async fn bearer_token(&self) -> Result<String> {
        self.0.bearer_token().await
    }
    async fn on_unauthorized(&self) -> Result<()> {
        self.0.on_unauthorized().await
    }
}
