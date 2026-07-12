//! Software Factory — local SDLC coverage map.

use crate::automations;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct FactoryStatus {
    pub stages: Vec<StageHealth>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StageHealth {
    pub name: String,
    pub healthy: bool,
    pub coverage: String,
    pub detail: String,
}

pub fn factory_status(root: &Path) -> FactoryStatus {
    let autos = automations::list(root).unwrap_or_default();
    let has_auto = |stage: &str| {
        autos
            .iter()
            .any(|a| a.enabled && a.stage.eq_ignore_ascii_case(stage))
    };
    let has_wf = |needle: &str| workflow_has(root, needle);

    let stages = vec![
        StageHealth {
            name: "Triage".into(),
            healthy: has_auto("triage") || has_wf("triage"),
            coverage: if has_auto("triage") { "automation" } else { "none" }.into(),
            detail: "Issue intake / routing automations".into(),
        },
        StageHealth {
            name: "Code-gen".into(),
            healthy: true, // Aegis present
            coverage: "aegis agent/missions".into(),
            detail: "Agent + Missions available".into(),
        },
        StageHealth {
            name: "Validate".into(),
            healthy: has_wf("review")
                || has_wf("aegis-review")
                || has_wf("qa")
                || root.join(".aegis/skills/qa").exists(),
            coverage: format!(
                "review={} qa={}",
                has_wf("review") || has_wf("aegis-review"),
                root.join(".aegis/skills/qa").exists()
            ),
            detail: "Code review + QA".into(),
        },
        StageHealth {
            name: "Release".into(),
            healthy: has_wf("release") || root.join("CHANGELOG.md").exists(),
            coverage: if has_wf("release") { "workflow" } else { "docs" }.into(),
            detail: "Release workflows / changelog".into(),
        },
        StageHealth {
            name: "Document".into(),
            healthy: root.join("docs/wiki").exists() || has_wf("wiki"),
            coverage: if root.join("docs/wiki").exists() {
                "wiki"
            } else {
                "none"
            }
            .into(),
            detail: "AutoWiki / docs/wiki".into(),
        },
        StageHealth {
            name: "Monitor".into(),
            healthy: has_auto("monitor")
                || root.join(".aegis/dreams").exists()
                || root.join(".aegis/automations/nightly-dream.toml").exists(),
            coverage: if root.join(".aegis/dreams").exists() {
                "dream"
            } else {
                "none"
            }
            .into(),
            detail: "Nightly dream / monitoring automations".into(),
        },
    ];

    let status = FactoryStatus { stages };
    let dir = root.join(".aegis/factory");
    let _ = fs::create_dir_all(&dir);
    if let Ok(j) = serde_json::to_string_pretty(&status) {
        let _ = fs::write(dir.join("status.json"), j);
    }
    status
}

pub fn format_factory(s: &FactoryStatus) -> String {
    let mut out = String::from("Software Factory — SDLC coverage\n\n");
    for st in &s.stages {
        out.push_str(&format!(
            "  {} {:<10}  [{:^12}]  {}\n",
            if st.healthy { "✓" } else { "·" },
            st.name,
            st.coverage,
            st.detail
        ));
    }
    out.push_str("\nTip: aegis install-qa · install-code-review · dream install · wiki generate\n");
    out
}

fn workflow_has(root: &Path, needle: &str) -> bool {
    let dir = root.join(".github/workflows");
    let Ok(rd) = fs::read_dir(dir) else {
        return false;
    };
    for e in rd.flatten() {
        let n = e.file_name().to_string_lossy().to_lowercase();
        if n.contains(needle) {
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
