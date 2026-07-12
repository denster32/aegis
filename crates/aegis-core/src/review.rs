//! Code review for PR or local diff (Factory review-inspired).

use aegis_xai::{
    system_msg, user_msg, CreateResponseRequest, ResponsesClient, TextConfig, TextFormat,
};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewReport {
    pub summary: String,
    pub findings: Vec<ReviewFinding>,
    pub approve: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub severity: String,
    pub title: String,
    pub detail: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub line: Option<u32>,
}

pub async fn review_diff(
    client: &ResponsesClient,
    model: &str,
    root: &Path,
    depth: &str,
) -> Result<ReviewReport> {
    let diff = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(root)
        .output()
        .context("git diff")?;
    let mut text = String::from_utf8_lossy(&diff.stdout).to_string();
    if text.trim().is_empty() {
        // include untracked via status for review context
        let st = Command::new("git")
            .args(["status", "--short"])
            .current_dir(root)
            .output()
            .context("git status")?;
        let st = String::from_utf8_lossy(&st.stdout).to_string();
        if st.trim().is_empty() {
            // still produce a trivial pass report instead of hard fail
            return Ok(ReviewReport {
                summary: "No local diff or dirty files to review.".into(),
                findings: vec![],
                approve: true,
            });
        }
        text = format!("# git status --short\n{st}\n");
    }
    let report = review_text(client, model, &text, depth).await?;
    let dir = root.join(".aegis/reviews");
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join(format!("diff-{}.md", chrono::Utc::now().format("%Y%m%d_%H%M%S"))),
        format_report_md(&report),
    )?;
    Ok(report)
}

pub async fn review_pr(
    client: &ResponsesClient,
    model: &str,
    root: &Path,
    pr: u64,
    depth: &str,
) -> Result<ReviewReport> {
    let out = Command::new("gh")
        .args(["pr", "diff", &pr.to_string()])
        .current_dir(root)
        .output()
        .context("gh pr diff")?;
    if !out.status.success() {
        bail!(
            "gh pr diff failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let report = review_text(client, model, &text, depth).await?;

    // write report; try post summary comment
    let dir = root.join(".aegis/reviews");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("pr-{pr}.md"));
    fs::write(&path, format_report_md(&report))?;

    let body = format!(
        "## Aegis review ({} depth)\n\n{}\n\n### Findings ({})\n{}",
        depth,
        report.summary,
        report.findings.len(),
        report
            .findings
            .iter()
            .map(|f| format!("- **{}**: {} — {}", f.severity, f.title, f.detail))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let _ = Command::new("gh")
        .args(["pr", "comment", &pr.to_string(), "--body", &body])
        .current_dir(root)
        .output();

    Ok(report)
}

async fn review_text(
    client: &ResponsesClient,
    model: &str,
    diff: &str,
    depth: &str,
) -> Result<ReviewReport> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "summary": { "type": "string" },
            "approve": { "type": "boolean" },
            "findings": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "severity": { "type": "string" },
                        "title": { "type": "string" },
                        "detail": { "type": "string" },
                        "path": { "type": ["string", "null"] },
                        "line": { "type": ["integer", "null"] }
                    },
                    "required": ["severity", "title", "detail", "path", "line"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["summary", "approve", "findings"],
        "additionalProperties": false
    });

    let guidance = if depth == "shallow" {
        "Focus on clear bugs and security only. Max 5 findings."
    } else {
        "Thorough review: bugs, security, correctness, races, resource leaks. Skip pure style."
    };

    let req = CreateResponseRequest {
        model: model.into(),
        input: vec![
            system_msg(format!(
                "You are Aegis code review. {guidance} Do not nitpick formatting."
            )),
            user_msg(format!(
                "Review this diff:\n\n```\n{}\n```",
                diff.chars().take(80_000).collect::<String>()
            )),
        ],
        tools: None,
        tool_choice: None,
        previous_response_id: None,
        store: Some(false),
        stream: Some(false),
        temperature: Some(0.1),
        max_output_tokens: Some(4096),
        parallel_tool_calls: None,
        text: Some(TextConfig {
            format: TextFormat::JsonSchema {
                name: "review_report".into(),
                schema,
                strict: Some(true),
            },
        }),
        include: None,
        reasoning: Some(aegis_xai::ReasoningConfig::high()),
        prompt_cache_key: Some("aegis-structured".into()),
    };

    let resp = client.create(req).await?;
    let text = resp.output_text();
    let text = extract_json(&text).unwrap_or(text);
    serde_json::from_str(&text).context("parse review")
}

pub fn format_report_md(r: &ReviewReport) -> String {
    let mut s = format!(
        "# Aegis Code Review\n\n{}\n\nApprove: {}\n\n## Findings\n",
        r.summary, r.approve
    );
    for f in &r.findings {
        s.push_str(&format!(
            "### {} — {}\n{}\n\n",
            f.severity, f.title, f.detail
        ));
    }
    s
}

pub fn install_review_workflow(root: &Path) -> Result<PathBuf> {
    let dir = root.join(".github/workflows");
    fs::create_dir_all(&dir)?;
    let path = dir.join("aegis-review.yml");
    fs::write(
        &path,
        r#"name: Aegis Code Review
on:
  pull_request:
    types: [opened, synchronize, reopened, ready_for_review]
jobs:
  review:
    if: github.event.pull_request.draft == false
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install aegis
        run: cargo install --path crates/aegis --force
        continue-on-error: true
      - name: Review PR
        env:
          XAI_API_KEY: ${{ secrets.XAI_API_KEY }}
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: aegis review --pr ${{ github.event.pull_request.number }} --depth deep || true
"#,
    )?;
    Ok(path)
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
