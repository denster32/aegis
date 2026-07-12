use crate::failures::FailureRecord;
use crate::lessons::{top_lessons, Lesson};
use crate::project::ProjectMemory;

const DEFAULT_BUDGET: usize = 6_000;

/// Build a prompt block from project memory (budgeted).
pub fn inject_memory_block(mem: &ProjectMemory, budget: Option<usize>) -> String {
    let budget = budget.unwrap_or(DEFAULT_BUDGET);
    let mut parts = Vec::new();
    parts.push("## Project memory (learned on prior runs — prefer these)".to_string());

    if let Ok(md) = mem.read_memory_md() {
        let clipped = clip(&md, budget / 2);
        if clipped.trim().len() > 40 {
            parts.push(clipped);
        }
    }

    if let Ok(lessons) = mem.load_lessons() {
        let top = top_lessons(lessons, 8);
        if !top.is_empty() {
            parts.push("### Top lessons".into());
            for l in top {
                parts.push(format_lesson(&l));
            }
        }
    }

    if let Ok(failures) = mem.load_failures() {
        let mut ranked = failures;
        ranked.sort_by(|a, b| {
            b.hits
                .cmp(&a.hits)
                .then_with(|| {
                    b.confidence
                        .partial_cmp(&a.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });
        ranked.truncate(5);
        if !ranked.is_empty() {
            parts.push("### Known failure fixes".into());
            for f in ranked {
                parts.push(format_failure(&f));
            }
        }
    }

    if let Ok(skills) = mem.list_skills() {
        if !skills.is_empty() {
            parts.push("### Project skills".into());
            for (name, body) in skills.into_iter().take(4) {
                parts.push(format!("#### skill:{name}\n{}", clip(&body, 800)));
            }
        }
    }

    let mut out = parts.join("\n\n");
    if out.len() > budget {
        out.truncate(budget);
        out.push_str("\n…[memory truncated]");
    }
    out
}

fn format_lesson(l: &Lesson) -> String {
    format!(
        "- [{} conf={:.2} hits={}] {}: {}",
        l.kind, l.confidence, l.hits, l.summary, l.detail
    )
}

fn format_failure(f: &FailureRecord) -> String {
    format!(
        "- tool={} pattern={} → fix: {} (conf={:.2})",
        f.tool, f.pattern, f.fix, f.confidence
    )
}

fn clip(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}
