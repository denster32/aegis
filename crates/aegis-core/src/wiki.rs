//! Project wiki generation (AutoWiki-inspired, local markdown).

use aegis_memory::ProjectMemory;
use aegis_xai::{
    system_msg, user_msg, CreateResponseRequest, ResponsesClient, TextConfig, TextFormat,
};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct WikiBundle {
    home: String,
    architecture: String,
    modules: String,
    commands: String,
    conventions: String,
}

pub async fn generate_wiki(root: &Path, client: &ResponsesClient, model: &str) -> Result<usize> {
    let mem = ProjectMemory::open(root).ok();
    let memory = mem
        .as_ref()
        .and_then(|m| m.read_memory_md().ok())
        .unwrap_or_default();
    let readme = fs::read_to_string(root.join("README.md")).unwrap_or_default();
    let tree = list_top_level(root);

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "home": { "type": "string" },
            "architecture": { "type": "string" },
            "modules": { "type": "string" },
            "commands": { "type": "string" },
            "conventions": { "type": "string" }
        },
        "required": ["home", "architecture", "modules", "commands", "conventions"],
        "additionalProperties": false
    });

    let req = CreateResponseRequest {
        model: model.into(),
        input: vec![
            system_msg(
                "You write concise project wiki pages in Markdown. Cross-link with relative links \
                 like [Architecture](Architecture.md). Be accurate to provided context.",
            ),
            user_msg(format!(
                "Generate a 5-page wiki for this repo.\n\n## README\n{}\n\n## Memory\n{}\n\n## Tree\n{}\n",
                clip(&readme, 6000),
                clip(&memory, 4000),
                tree
            )),
        ],
        tools: None,
        tool_choice: None,
        previous_response_id: None,
        store: Some(false),
        stream: Some(false),
        temperature: Some(0.2),
        max_output_tokens: Some(8192),
        parallel_tool_calls: None,
        text: Some(TextConfig {
            format: TextFormat::JsonSchema {
                name: "wiki_bundle".into(),
                schema,
                strict: Some(true),
            },
        }),
        include: None,
        reasoning: if crate::model_supports_reasoning(model) {
            Some(aegis_xai::ReasoningConfig::high())
        } else {
            None
        },
        prompt_cache_key: Some("aegis-structured".into()),
    };

    let resp = client.create(req).await.context("wiki generate")?;
    let text = resp.output_text();
    let text = extract_json(&text).unwrap_or(text);
    let bundle: WikiBundle = serde_json::from_str(&text).context("parse wiki")?;

    let dir = root.join("docs/wiki");
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("Home.md"), bundle.home)?;
    fs::write(dir.join("Architecture.md"), bundle.architecture)?;
    fs::write(dir.join("Modules.md"), bundle.modules)?;
    fs::write(dir.join("Commands.md"), bundle.commands)?;
    fs::write(dir.join("Conventions.md"), bundle.conventions)?;
    fs::write(
        dir.join("_Sidebar.md"),
        "- [Home](Home)\n- [Architecture](Architecture)\n- [Modules](Modules)\n- [Commands](Commands)\n- [Conventions](Conventions)\n",
    )?;
    Ok(5)
}

pub fn install_wiki_workflow(root: &Path) -> Result<PathBuf> {
    let dir = root.join(".github/workflows");
    fs::create_dir_all(&dir)?;
    let path = dir.join("aegis-wiki-refresh.yml");
    fs::write(
        &path,
        r#"name: Aegis Wiki Refresh
on:
  push:
    branches: [main, master]
  workflow_dispatch:
jobs:
  wiki:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install aegis
        run: cargo install --path crates/aegis --force
        continue-on-error: true
      - name: Generate wiki
        env:
          XAI_API_KEY: ${{ secrets.XAI_API_KEY }}
        run: aegis wiki generate || true
"#,
    )?;
    Ok(path)
}

fn list_top_level(root: &Path) -> String {
    let mut lines = Vec::new();
    if let Ok(rd) = fs::read_dir(root) {
        for e in rd.flatten().take(40) {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && name != ".aegis" && name != ".github" {
                continue;
            }
            lines.push(name);
        }
    }
    lines.join("\n")
}

fn clip(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
}

fn extract_json(text: &str) -> Option<String> {
    let t = text.trim();
    if t.starts_with('{') {
        return Some(t.to_string());
    }
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    if end > start {
        Some(t[start..=end].to_string())
    } else {
        None
    }
}
