//! File-based automations (schedule / manual / github tags).

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Automation {
    pub name: String,
    #[serde(default = "default_trigger")]
    pub trigger: String,
    #[serde(default)]
    pub cron: Option<String>,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_stage")]
    pub stage: String,
}

fn default_trigger() -> String {
    "manual".into()
}
fn default_true() -> bool {
    true
}
fn default_stage() -> String {
    "monitor".into()
}

pub fn list(root: &Path) -> Result<Vec<Automation>> {
    let dir = root.join(".aegis/automations");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for e in fs::read_dir(dir)? {
        let e = e?;
        if e.path().extension().and_then(|x| x.to_str()) != Some("toml") {
            continue;
        }
        let text = fs::read_to_string(e.path())?;
        if let Ok(a) = toml::from_str::<Automation>(&text) {
            out.push(a);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn run(root: &Path, name: &str) -> Result<String> {
    let autos = list(root)?;
    let a = autos
        .into_iter()
        .find(|x| x.name == name || x.name.starts_with(name))
        .with_context(|| format!("automation not found: {name}"))?;
    if !a.enabled {
        bail!("automation disabled: {}", a.name);
    }
    let aegis = crate::dream::which_aegis_pub();
    let mut cmd = Command::new(&aegis);
    cmd.arg("--cwd").arg(root);
    cmd.arg(&a.command);
    for arg in &a.args {
        cmd.arg(arg);
    }
    let out = cmd.output().context("spawn automation")?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        bail!("automation failed: {stderr}{stdout}");
    }
    Ok(format!("{stdout}{stderr}"))
}

pub fn ensure_defaults(root: &Path) -> Result<()> {
    let dir = root.join(".aegis/automations");
    fs::create_dir_all(&dir)?;
    let defaults = [
        (
            "nightly-dream.toml",
            r#"name = "nightly-dream"
trigger = "schedule"
cron = "0 3 * * *"
command = "dream"
args = ["--apply"]
enabled = true
stage = "monitor"
"#,
        ),
        (
            "wiki-refresh.toml",
            r#"name = "wiki-refresh"
trigger = "manual"
command = "wiki"
args = ["generate"]
enabled = true
stage = "document"
"#,
        ),
    ];
    for (name, body) in defaults {
        let p = dir.join(name);
        if !p.exists() {
            fs::write(p, body)?;
        }
    }
    Ok(())
}

pub fn format_list(autos: &[Automation]) -> String {
    if autos.is_empty() {
        return "No automations. Run: aegis automation ensure\n".into();
    }
    let mut s = String::from("Automations:\n");
    for a in autos {
        s.push_str(&format!(
            "  {} {:<18} trigger={:<10} stage={:<10} {} {}\n",
            if a.enabled { "●" } else { "○" },
            a.name,
            a.trigger,
            a.stage,
            a.command,
            a.args.join(" ")
        ));
    }
    s
}
