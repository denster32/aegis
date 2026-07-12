//! Grok OAuth / OIDC credentials for Aegis.
//!
//! Resolution order (highest first):
//! 1. `AEGIS_ACCESS_TOKEN` / `XAI_ACCESS_TOKEN`
//! 2. `~/.aegis/auth.json` (Aegis login)
//! 3. `~/.grok/auth.json` (Grok CLI OAuth — primary for most users)
//! 4. `XAI_API_KEY` / `SPACEXAI_API_KEY`

mod device;
mod file;
mod provider;
mod refresh;

pub use device::device_login;
pub use file::{
    auth_paths, clear_auth_file, import_grok_to_aegis, AuthEntry, AuthFile, AuthSource,
};
pub use provider::{AuthProvider, AuthStatus, Credential, TokenSource};
pub use refresh::refresh_entry;

/// Grok CLI public OIDC client id (from installed Grok Build).
pub const GROK_OIDC_CLIENT_ID: &str = "b1a00492-073a-47ea-816f-4c329264a828";
pub const DEFAULT_ISSUER: &str = "https://auth.x.ai";
