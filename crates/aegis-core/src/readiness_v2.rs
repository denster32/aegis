//! Agent Readiness L1–L5 with Factory-inspired pillars (local checks).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessV2Report {
    pub level: u8,
    pub level_name: String,
    pub score_pct: u8,
    pub pillars: Vec<PillarScore>,
    pub failing: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PillarScore {
    pub name: String,
    pub passed: u32,
    pub total: u32,
    pub criteria: Vec<CriterionResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionResult {
    pub id: String,
    pub level: u8,
    pub passed: bool,
    pub detail: String,
}

pub fn assess_v2(root: &Path) -> ReadinessV2Report {
    let mut criteria: Vec<CriterionResult> = Vec::new();

    // --- L1 Functional ---
    criteria.push(crit(
        "l1_readme",
        1,
        root.join("README.md").exists(),
        "README.md present",
    ));
    criteria.push(crit(
        "l1_vcs",
        1,
        root.join(".git").exists(),
        "git repository",
    ));
    let has_manifest = root.join("Cargo.toml").exists()
        || root.join("package.json").exists()
        || root.join("pyproject.toml").exists()
        || root.join("go.mod").exists();
    criteria.push(crit("l1_manifest", 1, has_manifest, "project manifest"));
    criteria.push(crit(
        "l1_tests",
        1,
        has_tests(root),
        "unit tests or tests/ dir",
    ));
    criteria.push(crit(
        "l1_linter",
        1,
        root.join("rustfmt.toml").exists()
            || root.join(".eslintrc").exists()
            || root.join(".eslintrc.js").exists()
            || root.join("clippy.toml").exists()
            || root.join(".prettierrc").exists(),
        "formatter/linter config",
    ));

    // --- L2 Documented ---
    criteria.push(crit(
        "l2_agents",
        2,
        root.join("AGENTS.md").exists()
            || root.join(".aegis/MEMORY.md").exists()
            || root.join(".aegis/rules.md").exists()
            || root.join("CLAUDE.md").exists(),
        "agent docs (AGENTS.md / .aegis)",
    ));
    criteria.push(crit(
        "l2_env_example",
        2,
        root.join(".env.example").exists(),
        ".env.example",
    ));
    criteria.push(crit(
        "l2_contributing",
        2,
        root.join("CONTRIBUTING.md").exists(),
        "CONTRIBUTING.md",
    ));
    criteria.push(crit(
        "l2_license",
        2,
        root.join("LICENSE").exists() || root.join("LICENSE.md").exists(),
        "LICENSE",
    ));
    criteria.push(crit(
        "l2_gitignore",
        2,
        root.join(".gitignore").exists(),
        ".gitignore",
    ));

    // --- L3 Standardized ---
    criteria.push(crit(
        "l3_ci",
        3,
        root.join(".github/workflows").exists(),
        "CI workflows",
    ));
    criteria.push(crit(
        "l3_lockfile",
        3,
        root.join("Cargo.lock").exists()
            || root.join("package-lock.json").exists()
            || root.join("pnpm-lock.yaml").exists()
            || root.join("yarn.lock").exists(),
        "dependency lockfile",
    ));
    criteria.push(crit(
        "l3_pr_template",
        3,
        root.join(".github/PULL_REQUEST_TEMPLATE.md").exists()
            || root.join(".github/pull_request_template.md").exists(),
        "PR template",
    ));
    criteria.push(crit(
        "l3_issue_template",
        3,
        root.join(".github/ISSUE_TEMPLATE").exists(),
        "issue templates",
    ));
    criteria.push(crit(
        "l3_security",
        3,
        root.join("SECURITY.md").exists(),
        "SECURITY.md",
    ));

    // --- L4 Optimized ---
    criteria.push(crit(
        "l4_aegis_memory",
        4,
        root.join(".aegis/MEMORY.md").exists()
            || root.join(".aegis/LESSONS.jsonl").exists(),
        "Aegis project memory",
    ));
    criteria.push(crit(
        "l4_wiki",
        4,
        root.join("docs/wiki").exists() || root.join(".aegis/wiki").exists(),
        "project wiki",
    ));
    criteria.push(crit(
        "l4_qa_skill",
        4,
        root.join(".aegis/skills/qa").exists() || root.join(".aegis/skills/qa/SKILL.md").exists(),
        "QA skill installed",
    ));
    criteria.push(crit(
        "l4_review_workflow",
        4,
        workflow_contains(root, "review") || workflow_contains(root, "aegis-review"),
        "code review automation",
    ));
    criteria.push(crit(
        "l4_tests_run",
        4,
        tests_runnable(root),
        "tests execute successfully",
    ));

    // --- L5 Autonomous ---
    criteria.push(crit(
        "l5_dream",
        5,
        root.join(".aegis/dreams").exists()
            || root.join(".aegis/automations").exists(),
        "dream/automations present",
    ));
    criteria.push(crit(
        "l5_missions",
        5,
        root.join(".aegis/missions").exists(),
        "missions history",
    ));
    criteria.push(crit(
        "l5_self_improve",
        5,
        root.join(".aegis/metrics.json").exists(),
        "learning metrics",
    ));
    criteria.push(crit(
        "l5_factory",
        5,
        root.join(".aegis/factory").exists()
            || root.join(".aegis/automations").exists(),
        "software factory coverage files",
    ));

    // Pillar grouping (map criteria into pillars for display)
    let pillars = group_pillars(&criteria);

    // Level progression: 80% of current level criteria to unlock next
    let level = compute_level(&criteria);
    let level_name = match level {
        1 => "L1 Functional",
        2 => "L2 Documented",
        3 => "L3 Standardized",
        4 => "L4 Optimized",
        5 => "L5 Autonomous",
        _ => "L1 Functional",
    }
    .to_string();

    let passed = criteria.iter().filter(|c| c.passed).count() as u8;
    let total = criteria.len().max(1) as u8;
    let score_pct = ((passed as f32 / total as f32) * 100.0) as u8;

    let failing: Vec<String> = criteria
        .iter()
        .filter(|c| !c.passed)
        .map(|c| format!("{}: {}", c.id, c.detail))
        .collect();

    let recommendations = recommend(&failing);

    let report = ReadinessV2Report {
        level,
        level_name: level_name.to_string(),
        score_pct,
        pillars,
        failing,
        recommendations,
    };

    // persist
    let dir = root.join(".aegis/readiness");
    let _ = fs::create_dir_all(&dir);
    if let Ok(j) = serde_json::to_string_pretty(&report) {
        let _ = fs::write(dir.join("report.json"), j);
    }

    report
}

fn crit(id: &str, level: u8, passed: bool, detail: &str) -> CriterionResult {
    CriterionResult {
        id: id.into(),
        level,
        passed,
        detail: detail.into(),
    }
}

fn has_tests(root: &Path) -> bool {
    if root.join("tests").exists() {
        return true;
    }
    // rust #[cfg(test)] common — look for test modules in src
    walk_has_test_markers(root)
}

fn walk_has_test_markers(root: &Path) -> bool {
    let patterns = ["#[cfg(test)]", "#[test]", "describe(", "it(", "def test_"];
    walk_limited(root, 0, 3, &patterns)
}

fn walk_limited(dir: &Path, depth: u32, max_depth: u32, patterns: &[&str]) -> bool {
    if depth > max_depth {
        return false;
    }
    let Ok(rd) = fs::read_dir(dir) else {
        return false;
    };
    for e in rd.flatten() {
        let p = e.path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name == "target" || name == "node_modules" || name == ".git" {
            continue;
        }
        if p.is_dir() {
            if walk_limited(&p, depth + 1, max_depth, patterns) {
                return true;
            }
        } else {
            let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "rs" | "ts" | "tsx" | "js" | "py" | "go") {
                continue;
            }
            if let Ok(s) = fs::read_to_string(&p) {
                if patterns.iter().any(|pat| s.contains(pat)) {
                    return true;
                }
            }
        }
    }
    false
}

fn workflow_contains(root: &Path, needle: &str) -> bool {
    let dir = root.join(".github/workflows");
    let Ok(rd) = fs::read_dir(dir) else {
        return false;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_lowercase();
        if name.contains(needle) {
            return true;
        }
        if let Ok(s) = fs::read_to_string(e.path()) {
            if s.to_lowercase().contains(needle) {
                return true;
            }
        }
    }
    false
}

fn tests_runnable(root: &Path) -> bool {
    if root.join("Cargo.toml").exists() {
        let out = Command::new("cargo")
            .args(["test", "--no-run", "-q"])
            .current_dir(root)
            .output();
        return out.map(|o| o.status.success()).unwrap_or(false);
    }
    if root.join("package.json").exists() {
        return true; // don't run npm install in assess
    }
    false
}

fn compute_level(criteria: &[CriterionResult]) -> u8 {
    let mut level = 1u8;
    for try_level in 1u8..=5 {
        let at: Vec<_> = criteria.iter().filter(|c| c.level == try_level).collect();
        if at.is_empty() {
            continue;
        }
        let pass = at.iter().filter(|c| c.passed).count() as f32;
        let pct = pass / at.len() as f32;
        if pct >= 0.80 {
            level = try_level;
        } else {
            break;
        }
    }
    level
}

fn group_pillars(criteria: &[CriterionResult]) -> Vec<PillarScore> {
    // Map criterion ids to pillars
    let mapping: &[(&str, &[&str])] = &[
        (
            "Style & Validation",
            &["l1_linter"],
        ),
        ("Build System", &["l1_manifest", "l3_lockfile", "l1_vcs"]),
        ("Testing", &["l1_tests", "l4_tests_run", "l4_qa_skill"]),
        (
            "Documentation",
            &["l1_readme", "l2_agents", "l2_contributing", "l4_wiki"],
        ),
        ("Development Environment", &["l2_env_example", "l2_gitignore"]),
        ("Debugging & Observability", &[]),
        ("Security", &["l3_security", "l2_license"]),
        (
            "Task Discovery",
            &["l3_pr_template", "l3_issue_template"],
        ),
        (
            "Product & Experimentation",
            &["l3_ci", "l4_review_workflow", "l5_dream", "l5_missions", "l5_self_improve", "l5_factory"],
        ),
    ];

    let mut pillars = Vec::new();
    for (name, ids) in mapping {
        let mut list = Vec::new();
        for id in *ids {
            if let Some(c) = criteria.iter().find(|c| c.id == *id) {
                list.push(c.clone());
            }
        }
        // also attach unmapped by scanning — skip for brevity
        let passed = list.iter().filter(|c| c.passed).count() as u32;
        let total = list.len() as u32;
        pillars.push(PillarScore {
            name: (*name).into(),
            passed,
            total,
            criteria: list,
        });
    }
    pillars
}

fn recommend(failing: &[String]) -> Vec<String> {
    let mut r = Vec::new();
    for f in failing {
        if f.contains("l1_readme") {
            r.push("Add a README.md with build/test instructions".into());
        } else if f.contains("l2_agents") {
            r.push("Add AGENTS.md or run aegis once to seed .aegis/MEMORY.md".into());
        } else if f.contains("l3_ci") {
            r.push("Add .github/workflows CI".into());
        } else if f.contains("l4_wiki") {
            r.push("Run: aegis wiki generate".into());
        } else if f.contains("l4_qa") {
            r.push("Run: aegis install-qa".into());
        } else if f.contains("l4_review") {
            r.push("Run: aegis install-code-review".into());
        } else if f.contains("l5_dream") {
            r.push("Run: aegis dream once; aegis dream install".into());
        }
    }
    r.dedup();
    r.truncate(8);
    r
}

pub fn format_report(r: &ReadinessV2Report) -> String {
    let mut s = format!(
        "Agent Readiness: {} (level {}) · overall {}%\n\n",
        r.level_name, r.level, r.score_pct
    );
    for p in &r.pillars {
        if p.total == 0 {
            continue;
        }
        s.push_str(&format!(
            "  {}  {}/{}  {}\n",
            if p.passed == p.total { "✓" } else { "·" },
            p.passed,
            p.total,
            p.name
        ));
        for c in &p.criteria {
            s.push_str(&format!(
                "      {} {}\n",
                if c.passed { "✓" } else { "✗" },
                c.detail
            ));
        }
    }
    if !r.recommendations.is_empty() {
        s.push_str("\nRecommendations:\n");
        for rec in &r.recommendations {
            s.push_str(&format!("  → {rec}\n"));
        }
    }
    s
}
