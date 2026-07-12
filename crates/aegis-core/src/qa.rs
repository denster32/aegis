//! Automated QA install + run (Factory QA-inspired, CLI-first MVP).

use aegis_memory::{fingerprint, FailureRecord, ProjectMemory};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

pub fn install_qa(root: &Path) -> Result<String> {
    let qa = root.join(".aegis/skills/qa");
    fs::create_dir_all(&qa)?;
    let is_rust = root.join("Cargo.toml").exists();
    let is_node = root.join("package.json").exists();

    fs::write(
        qa.join("config.yaml"),
        format!(
            "project: {}\ndefault_target: local\napps:\n  primary:\n    skill: qa-cli\n    test_command: {}\nfailure_learning: suggest_in_report\n",
            root.file_name().and_then(|s| s.to_str()).unwrap_or("project"),
            if is_rust {
                "cargo test"
            } else if is_node {
                "npm test"
            } else {
                "echo 'no test command configured'"
            }
        ),
    )?;

    fs::write(
        qa.join("SKILL.md"),
        r#"# QA Orchestrator

1. Read config.yaml
2. Map git diff to apps
3. Run qa-cli (or app-specific) flows
4. Write report under .aegis/qa/reports/
5. On failures, record to project FAILURES.jsonl via memory_write
"#,
    )?;

    let cli = root.join(".aegis/skills/qa-cli");
    fs::create_dir_all(&cli)?;
    fs::write(
        cli.join("SKILL.md"),
        r#"# QA CLI

## Flows
- build/typecheck if available
- unit tests
- smoke binary --help if present

## Evidence
Capture command output under .aegis/qa/evidence/
"#,
    )?;

    fs::write(
        qa.join("REPORT-TEMPLATE.md"),
        "# QA Report\n\n| # | Case | Result | Notes |\n|---|------|--------|-------|\n",
    )?;

    // optional workflow
    let wf = root.join(".github/workflows");
    fs::create_dir_all(&wf)?;
    fs::write(
        wf.join("aegis-qa.yml"),
        r#"name: Aegis QA
on:
  pull_request:
  workflow_dispatch:
env:
  CARGO_TERM_COLOR: always
jobs:
  qa:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      # Unit tests must fail the job (no continue-on-error).
      - name: unit tests
        run: cargo test --workspace --locked
      - name: Detect live QA secret
        id: secrets
        env:
          XAI_API_KEY: ${{ secrets.XAI_API_KEY }}
        run: |
          if [ -n "$XAI_API_KEY" ]; then
            echo "has_xai=true" >> "$GITHUB_OUTPUT"
          else
            echo "has_xai=false" >> "$GITHUB_OUTPUT"
            echo "XAI_API_KEY secret not set — skipping live QA (unit tests still required)"
          fi
      - name: install aegis
        if: steps.secrets.outputs.has_xai == 'true'
        run: cargo install --path crates/aegis --locked --force
      - name: install qa skills
        if: steps.secrets.outputs.has_xai == 'true'
        run: aegis install-qa
      - name: Aegis live QA
        if: steps.secrets.outputs.has_xai == 'true'
        env:
          XAI_API_KEY: ${{ secrets.XAI_API_KEY }}
        run: aegis qa
"#,
    )?;

    Ok(format!("Installed QA skills under {}", qa.display()))
}

pub fn run_qa(root: &Path, base: Option<&str>) -> Result<String> {
    let report_dir = root.join(".aegis/qa/reports");
    let evidence = root.join(".aegis/qa/evidence");
    fs::create_dir_all(&report_dir)?;
    fs::create_dir_all(&evidence)?;

    let mut cases: Vec<(String, bool, String)> = Vec::new();

    // diff summary
    let base_ref = base.unwrap_or("HEAD~1");
    let _diff = Command::new("git")
        .args(["diff", "--stat", base_ref])
        .current_dir(root)
        .output();

    if root.join("Cargo.toml").exists() {
        let out = Command::new("cargo")
            .args(["test", "--workspace"])
            .current_dir(root)
            .output()
            .context("cargo test")?;
        let ok = out.status.success();
        let log = format!(
            "{}\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        fs::write(evidence.join("cargo-test.log"), &log)?;
        cases.push(("cargo test --workspace".into(), ok, truncate(&log, 400)));
        if !ok {
            record_failure(root, "bash", "cargo test failed", &truncate(&log, 200))?;
        }
    }

    if root.join("package.json").exists() {
        let out = Command::new("npm")
            .args(["test", "--", "--watchAll=false"])
            .current_dir(root)
            .output();
        if let Ok(out) = out {
            let ok = out.status.success();
            cases.push((
                "npm test".into(),
                ok,
                truncate(&String::from_utf8_lossy(&out.stdout), 200),
            ));
        }
    }

    // binary help smoke
    if root.join("crates/aegis").exists() || which_in_path("aegis") {
        let out = Command::new("aegis").arg("--help").output();
        if let Ok(out) = out {
            cases.push((
                "aegis --help".into(),
                out.status.success(),
                "help renders".into(),
            ));
        }
    }

    let mut md = String::from("# QA Report\n\n");
    md.push_str(&format!("Generated: {}\n\n", Utc::now().to_rfc3339()));
    md.push_str("| # | Case | Result | Notes |\n|---|------|--------|-------|\n");
    for (i, (name, ok, notes)) in cases.iter().enumerate() {
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            i + 1,
            name,
            if *ok { "PASS" } else { "FAIL" },
            notes.replace('|', "/")
        ));
    }
    let passed = cases.iter().filter(|c| c.1).count();
    md.push_str(&format!(
        "\n**Summary:** {}/{} passed\n",
        passed,
        cases.len()
    ));

    let path = report_dir.join(format!("{}.md", Utc::now().format("%Y%m%d_%H%M%S")));
    fs::write(&path, &md)?;
    Ok(format!("QA report written to {}\n\n{md}", path.display()))
}

fn record_failure(root: &Path, tool: &str, pattern: &str, fix_hint: &str) -> Result<()> {
    let mem = ProjectMemory::open(root)?;
    let rec = FailureRecord {
        id: Uuid::new_v4().to_string(),
        ts: Utc::now().to_rfc3339(),
        fingerprint: fingerprint(tool, pattern),
        tool: tool.into(),
        pattern: pattern.into(),
        root_cause: "qa failure".into(),
        fix: fix_hint.into(),
        confidence: 0.5,
        hits: 1,
    };
    mem.append_failure(&rec)?;
    Ok(())
}

fn truncate(s: &str, n: usize) -> String {
    let s = s.replace('\n', " ");
    if s.len() <= n {
        s
    } else {
        format!("{}…", &s[..n])
    }
}

fn which_in_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(name).is_file()))
        .unwrap_or(false)
}
