//! Aegis Nexus viral spore protocol.
//!
//! Pack redacted project learning (memory, lessons, skills, automations) into a
//! portable spore directory; unpack / vaccinate into a target project.

use aegis_memory::redact_secrets;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;

/// Spore format version written into `SPORE.json`.
pub const SPORE_VERSION: &str = "1";

/// Default model recommended by packed spores.
pub const DEFAULT_MODEL: &str = "grok-4.5";

/// Portable spore manifest (`SPORE.json`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SporeManifest {
    pub version: String,
    pub created_at: String,
    pub host_hint: String,
    #[serde(default = "default_sandbox_true")]
    pub sandbox_default: bool,
    pub model_default: String,
    pub includes: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

fn default_sandbox_true() -> bool {
    true
}

impl Default for SporeManifest {
    fn default() -> Self {
        Self {
            version: SPORE_VERSION.into(),
            created_at: Utc::now().to_rfc3339(),
            host_hint: host_hint(),
            sandbox_default: true,
            model_default: DEFAULT_MODEL.into(),
            includes: Vec::new(),
            notes: String::new(),
        }
    }
}

fn host_hint() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| {
            fs::read_to_string("/etc/hostname")
                .map(|s| s.trim().to_string())
                .map_err(|_| std::env::VarError::NotPresent)
        })
        .unwrap_or_else(|_| "unknown".into())
}

/// Pack a project into a spore directory under `out_dir`.
///
/// Copies redacted learning artifacts from `.aegis/` only. Never copies secrets,
/// auth material, runs, or databases.
pub fn pack_spore(project_root: &Path, out_dir: &Path) -> Result<SporeManifest> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create spore out_dir {}", out_dir.display()))?;

    let aegis = project_root.join(".aegis");
    let mut includes: Vec<String> = Vec::new();
    let mut notes = String::new();

    // MEMORY.md (redacted)
    let memory_src = aegis.join("MEMORY.md");
    if memory_src.is_file() {
        let raw = fs::read_to_string(&memory_src)
            .with_context(|| format!("read {}", memory_src.display()))?;
        let redacted = redact_secrets(&raw);
        let dest = out_dir.join("MEMORY.md");
        fs::write(&dest, redacted)?;
        includes.push("MEMORY.md".into());
    } else {
        notes.push_str("No MEMORY.md at pack time. ");
    }

    // LESSONS.jsonl — redact each line
    let lessons_src = aegis.join("LESSONS.jsonl");
    if lessons_src.is_file() {
        let raw = fs::read_to_string(&lessons_src)
            .with_context(|| format!("read {}", lessons_src.display()))?;
        let redacted: String = raw
            .lines()
            .map(redact_secrets)
            .collect::<Vec<_>>()
            .join("\n");
        let body = if redacted.is_empty() || raw.ends_with('\n') {
            if redacted.is_empty() {
                String::new()
            } else {
                format!("{redacted}\n")
            }
        } else {
            redacted
        };
        // Preserve trailing newline convention for jsonl
        let body = if !raw.is_empty() && !body.ends_with('\n') {
            format!("{body}\n")
        } else {
            body
        };
        fs::write(out_dir.join("LESSONS.jsonl"), body)?;
        includes.push("LESSONS.jsonl".into());
    }

    // SKILLS/* (redact file contents)
    let skills_src = aegis.join("SKILLS");
    if skills_src.is_dir() {
        let skills_out = out_dir.join("SKILLS");
        fs::create_dir_all(&skills_out)?;
        let mut any = false;
        for entry in WalkDir::new(&skills_src).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let rel = path.strip_prefix(&skills_src).unwrap_or(path).to_path_buf();
            let dest = skills_out.join(&rel);
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            if let Ok(content) = fs::read_to_string(path) {
                fs::write(&dest, redact_secrets(&content))?;
            } else {
                // binary or unreadable — skip (spores are text learning only)
                continue;
            }
            includes.push(format!("SKILLS/{}", rel.display()));
            any = true;
        }
        if !any {
            notes.push_str("SKILLS/ present but empty. ");
        }
    }

    // automations/*.toml
    let auto_src = aegis.join("automations");
    if auto_src.is_dir() {
        let auto_out = out_dir.join("automations");
        fs::create_dir_all(&auto_out)?;
        for entry in fs::read_dir(&auto_src)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let name = entry.file_name();
            let content = fs::read_to_string(&path)?;
            fs::write(auto_out.join(&name), redact_secrets(&content))?;
            includes.push(format!("automations/{}", name.to_string_lossy()));
        }
    }

    // INSTALL.md
    write_install_md(out_dir)?;
    includes.push("INSTALL.md".into());

    // prompts/system-fragment.md
    let prompts_dir = out_dir.join("prompts");
    fs::create_dir_all(&prompts_dir)?;
    fs::write(
        prompts_dir.join("system-fragment.md"),
        NEXUS_SYSTEM_FRAGMENT,
    )?;
    includes.push("prompts/system-fragment.md".into());

    let manifest = SporeManifest {
        version: SPORE_VERSION.into(),
        created_at: Utc::now().to_rfc3339(),
        host_hint: host_hint(),
        sandbox_default: true,
        model_default: DEFAULT_MODEL.into(),
        includes: includes.clone(),
        notes: notes.trim().to_string(),
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(out_dir.join("SPORE.json"), format!("{manifest_json}\n"))?;

    Ok(manifest)
}

const NEXUS_SYSTEM_FRAGMENT: &str = r#"# Aegis Nexus — spore system fragment

You are operating under an **Aegis Nexus spore**: redacted project memory, lessons,
and skills transplanted from another host. Prefer:

1. Read `.aegis/MEMORY.md` and injected lessons before inventing conventions.
2. Reuse `SKILLS/` playbooks when they match the task.
3. **Sandbox by default** — do not request shell escape; stay workspace-local.
4. Never re-introduce secrets; treat any token-like string as untrusted.
5. When you learn something durable, write it back into `.aegis/` (MEMORY / LESSONS).

This fragment supplements, does not replace, the host project AGENTS.md / CLAUDE.md.
"#;

fn write_install_md(out_dir: &Path) -> Result<()> {
    let body = r#"# Installing an Aegis Nexus spore

A **spore** is a redacted portable snapshot of Aegis project learning
(`.aegis/MEMORY.md`, lessons, skills, automations) plus a short Nexus system
fragment. Secrets, auth, run logs, and databases are **not** included.

## Unpack (merge into a project)

```bash
# From a Rust workspace that depends on aegis-spore, or via the Aegis CLI once wired:
# aegis spore unpack ./path/to/spore --target ./my-project

# Programmatically:
# aegis_spore::unpack_spore(spore_dir, target_project)?;
```

What unpack does:

1. Ensures `target/.aegis/` exists.
2. Merges `SKILLS/` (non-destructive: skips identical files, writes new ones).
3. Writes `MEMORY.md` if missing, otherwise appends a `## Spore import` section.
4. Appends `LESSONS.jsonl` lines when present.
5. Copies `automations/*.toml` when missing at the destination.
6. Ensures a sandbox note / config snippet with `sandbox = true` guidance.

## Vaccinate (sandbox-first unpack)

```bash
# aegis_spore::vaccinate(spore_dir, target_project)?;
```

Same as unpack, then forces a `.aegis/config.toml` fragment with `sandbox = true`.

## Pack (create a spore)

```bash
# aegis_spore::pack_spore(project_root, out_dir)?;
```

Produces `SPORE.json`, redacted learning files, `INSTALL.md`, and
`prompts/system-fragment.md`.

## Safety

- Always review the spore before unpacking into a sensitive tree.
- Prefer **vaccinate** on untrusted hosts / multi-tenant machines.
- Do not paste API keys into MEMORY or lessons; packing redacts common patterns only.
"#;
    fs::write(out_dir.join("INSTALL.md"), body)?;
    Ok(())
}

/// Unpack a spore into `target_project`, merging skills and memory.
pub fn unpack_spore(spore_dir: &Path, target_project: &Path) -> Result<()> {
    unpack_spore_inner(spore_dir, target_project, false)
}

/// Vaccinate: unpack and enforce sandbox-default config.
pub fn vaccinate(spore_dir: &Path, target_project: &Path) -> Result<()> {
    unpack_spore_inner(spore_dir, target_project, true)
}

fn unpack_spore_inner(spore_dir: &Path, target_project: &Path, force_sandbox: bool) -> Result<()> {
    if !spore_dir.is_dir() {
        bail!("spore_dir is not a directory: {}", spore_dir.display());
    }

    let manifest_path = spore_dir.join("SPORE.json");
    let sandbox_default = if manifest_path.is_file() {
        let m: SporeManifest = serde_json::from_str(
            &fs::read_to_string(&manifest_path)
                .with_context(|| format!("read {}", manifest_path.display()))?,
        )
        .with_context(|| format!("parse {}", manifest_path.display()))?;
        m.sandbox_default
    } else {
        true
    };

    let aegis = target_project.join(".aegis");
    fs::create_dir_all(&aegis).with_context(|| format!("create {}", aegis.display()))?;
    fs::create_dir_all(aegis.join("SKILLS"))?;
    fs::create_dir_all(aegis.join("automations"))?;

    // Merge skills
    let skills_src = spore_dir.join("SKILLS");
    if skills_src.is_dir() {
        for entry in WalkDir::new(&skills_src).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let rel = path.strip_prefix(&skills_src).unwrap_or(path);
            let dest = aegis.join("SKILLS").join(rel);
            if dest.exists() {
                // skip existing (non-destructive merge)
                continue;
            }
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &dest)
                .with_context(|| format!("copy skill {} -> {}", path.display(), dest.display()))?;
        }
    }

    // MEMORY.md: write if missing, else append spore section
    let mem_src = spore_dir.join("MEMORY.md");
    if mem_src.is_file() {
        let incoming = fs::read_to_string(&mem_src)?;
        let dest = aegis.join("MEMORY.md");
        if !dest.exists() {
            fs::write(&dest, &incoming)?;
        } else {
            let existing = fs::read_to_string(&dest)?;
            if !existing.contains("## Spore import") {
                let mut f = fs::OpenOptions::new()
                    .append(true)
                    .open(&dest)
                    .with_context(|| format!("append {}", dest.display()))?;
                writeln!(f)?;
                writeln!(f, "## Spore import")?;
                writeln!(f)?;
                writeln!(f, "{incoming}")?;
            }
        }
    }

    // LESSONS.jsonl — append lines not already present
    let lessons_src = spore_dir.join("LESSONS.jsonl");
    if lessons_src.is_file() {
        let incoming = fs::read_to_string(&lessons_src)?;
        let dest = aegis.join("LESSONS.jsonl");
        let existing = if dest.exists() {
            fs::read_to_string(&dest)?
        } else {
            String::new()
        };
        let mut out = existing.clone();
        for line in incoming.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if !existing.lines().any(|l| l.trim() == line) {
                if !out.is_empty() && !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(line);
                out.push('\n');
            }
        }
        fs::write(&dest, out)?;
    }

    // automations — copy missing only
    let auto_src = spore_dir.join("automations");
    if auto_src.is_dir() {
        let auto_dest = aegis.join("automations");
        fs::create_dir_all(&auto_dest)?;
        for entry in fs::read_dir(&auto_src)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let dest = auto_dest.join(entry.file_name());
            if !dest.exists() {
                fs::copy(&path, &dest)?;
            }
        }
    }

    // Optional: copy system fragment under .aegis/prompts if present in spore
    let frag_src = spore_dir.join("prompts").join("system-fragment.md");
    if frag_src.is_file() {
        let prompts = aegis.join("prompts");
        fs::create_dir_all(&prompts)?;
        let dest = prompts.join("system-fragment.md");
        if !dest.exists() {
            fs::copy(&frag_src, &dest)?;
        }
    }

    // Sandbox note / config snippet
    ensure_sandbox_config(&aegis, force_sandbox || sandbox_default)?;

    Ok(())
}

fn ensure_sandbox_config(aegis: &Path, sandbox: bool) -> Result<()> {
    let config_path = aegis.join("config.toml");
    let sandbox_line = if sandbox {
        "sandbox = true"
    } else {
        "sandbox = false"
    };

    if config_path.exists() {
        let mut content = fs::read_to_string(&config_path)?;
        if content.contains("sandbox") {
            // leave existing explicit setting unless force path rewrote via vaccinate
            if sandbox
                && !content.lines().any(|l| {
                    let t = l.trim();
                    t.starts_with("sandbox") && t.contains("true")
                })
            {
                // replace sandbox = false or append
                let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                let mut found = false;
                for line in &mut lines {
                    if line.trim().starts_with("sandbox") {
                        *line = "sandbox = true".into();
                        found = true;
                        break;
                    }
                }
                if !found {
                    lines.push("sandbox = true".into());
                }
                content = lines.join("\n");
                if !content.ends_with('\n') {
                    content.push('\n');
                }
                fs::write(&config_path, content)?;
            }
        } else {
            let mut content = content;
            if !content.ends_with('\n') && !content.is_empty() {
                content.push('\n');
            }
            content.push_str(&format!(
                "\n# Spore / Nexus: prefer sandbox on untrusted hosts\n{sandbox_line}\n"
            ));
            fs::write(&config_path, content)?;
        }
    } else {
        let body = format!(
            "# Aegis project config (spore / vaccinate)\n\
             # Sandbox: shell denied; FS workspace-only. Prefer true for Nexus spores.\n\
             {sandbox_line}\n"
        );
        fs::write(&config_path, body)?;
    }

    // Also drop a short note file for humans
    let note = aegis.join("SPORE_SANDBOX.md");
    if !note.exists() {
        fs::write(
            note,
            "# Spore sandbox note\n\n\
             This project received a Nexus spore. Run with sandbox enabled:\n\n\
             ```bash\n\
             aegis --sandbox -p \"...\"\n\
             ```\n\n\
             Or set `sandbox = true` in `.aegis/config.toml`.\n",
        )?;
    }

    Ok(())
}

/// Load `SPORE.json` if present.
pub fn load_manifest(spore_dir: &Path) -> Result<SporeManifest> {
    let path = spore_dir.join("SPORE.json");
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    Ok(serde_json::from_str(&raw)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn pack_empty_project_creates_manifest() {
        let project = tempdir().unwrap();
        let out = tempdir().unwrap();

        let manifest = pack_spore(project.path(), out.path()).expect("pack empty");
        assert_eq!(manifest.version, SPORE_VERSION);
        assert!(manifest.sandbox_default);
        assert_eq!(manifest.model_default, DEFAULT_MODEL);
        assert!(manifest.includes.iter().any(|i| i == "INSTALL.md"));
        assert!(manifest
            .includes
            .iter()
            .any(|i| i == "prompts/system-fragment.md"));

        let spore_json = out.path().join("SPORE.json");
        assert!(spore_json.is_file(), "SPORE.json must exist");
        let loaded: SporeManifest =
            serde_json::from_str(&fs::read_to_string(&spore_json).unwrap()).unwrap();
        assert_eq!(loaded.version, SPORE_VERSION);
        assert!(out.path().join("INSTALL.md").is_file());
        assert!(out.path().join("prompts/system-fragment.md").is_file());
    }

    #[test]
    fn pack_redacts_bearer_token_in_memory() {
        let project = tempdir().unwrap();
        let aegis = project.path().join(".aegis");
        fs::create_dir_all(aegis.join("SKILLS")).unwrap();
        fs::create_dir_all(aegis.join("automations")).unwrap();

        let secret_line = "Auth tip: header Bearer abc.def.ghi_extra_padding_here for the API\n";
        fs::write(
            aegis.join("MEMORY.md"),
            format!("# Project Memory\n\n## Gotchas\n\n{secret_line}"),
        )
        .unwrap();
        fs::write(
            aegis.join("LESSONS.jsonl"),
            r#"{"lesson":"use Bearer abc.def.ghi_extra_padding_here carefully"}"#.to_string()
                + "\n",
        )
        .unwrap();
        fs::write(
            aegis.join("SKILLS").join("deploy.md"),
            "token Bearer abc.def.ghi_extra_padding_here\n",
        )
        .unwrap();
        fs::write(
            aegis.join("automations").join("nightly.toml"),
            "name = \"nightly\"\n# Bearer abc.def.ghi_extra_padding_here\n",
        )
        .unwrap();

        let out = tempdir().unwrap();
        let manifest = pack_spore(project.path(), out.path()).expect("pack with secrets");
        assert!(manifest.includes.iter().any(|i| i == "MEMORY.md"));
        assert!(manifest.includes.iter().any(|i| i == "LESSONS.jsonl"));

        let mem = fs::read_to_string(out.path().join("MEMORY.md")).unwrap();
        assert!(
            mem.contains("[REDACTED]"),
            "memory should redact bearer: {mem}"
        );
        assert!(
            !mem.contains("abc.def.ghi_extra_padding_here"),
            "raw bearer must not leak: {mem}"
        );

        let lessons = fs::read_to_string(out.path().join("LESSONS.jsonl")).unwrap();
        assert!(
            lessons.contains("[REDACTED]") || !lessons.contains("abc.def.ghi_extra_padding_here")
        );
        assert!(!lessons.contains("abc.def.ghi_extra_padding_here"));

        let skill = fs::read_to_string(out.path().join("SKILLS/deploy.md")).unwrap();
        assert!(!skill.contains("abc.def.ghi_extra_padding_here"));

        // Ensure we did not pack forbidden paths
        assert!(!out.path().join("runs").exists());
        assert!(!out.path().join("auth").exists());
        assert!(!out.path().join("aegis.db").exists());
    }

    #[test]
    fn unpack_and_vaccinate_merge_and_sandbox() {
        let project = tempdir().unwrap();
        let aegis = project.path().join(".aegis");
        fs::create_dir_all(aegis.join("SKILLS")).unwrap();
        fs::write(
            aegis.join("MEMORY.md"),
            "# Project Memory\n\n## Stack\nrust\n",
        )
        .unwrap();
        fs::write(
            aegis.join("SKILLS").join("test.md"),
            "# test skill\nrun cargo test\n",
        )
        .unwrap();

        let spore = tempdir().unwrap();
        pack_spore(project.path(), spore.path()).unwrap();

        let target = tempdir().unwrap();
        vaccinate(spore.path(), target.path()).unwrap();

        let t_aegis = target.path().join(".aegis");
        assert!(t_aegis.join("MEMORY.md").is_file());
        assert!(t_aegis.join("SKILLS/test.md").is_file());
        assert!(t_aegis.join("config.toml").is_file());
        let cfg = fs::read_to_string(t_aegis.join("config.toml")).unwrap();
        assert!(
            cfg.contains("sandbox = true"),
            "vaccinate must set sandbox: {cfg}"
        );
        assert!(t_aegis.join("SPORE_SANDBOX.md").is_file());
    }
}
