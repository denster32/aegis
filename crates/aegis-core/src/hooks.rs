//! Project hooks: `.aegis/hooks/pre-tool` and `post-tool` executables/scripts.

use std::path::Path;
use std::process::Command;
use tracing::debug;

/// Run hook if present. Non-zero exit is logged but does not fail the agent by default.
pub fn run_hook(root: &Path, name: &str, env: &[(&str, &str)]) -> Option<String> {
    let path = root.join(".aegis/hooks").join(name);
    if !path.exists() {
        return None;
    }
    let mut cmd = Command::new(&path);
    cmd.current_dir(root);
    for (k, v) in env {
        cmd.env(k, v);
    }
    match cmd.output() {
        Ok(out) => {
            let s = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            debug!(hook = name, status = ?out.status, "hook ran");
            Some(s)
        }
        Err(e) => Some(format!("hook {name} error: {e}")),
    }
}
