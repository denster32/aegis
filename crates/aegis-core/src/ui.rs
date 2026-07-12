//! Terminal chrome — SpaceX / xAI design language.
//!
//! Black void · white primary · dim secondary · no noise.
//! Thin rules, uppercase micro-labels, geometric marks.

use console::style;

/// Preferred content width for boards and headers.
pub const WIDTH: usize = 52;

// ── primitives ──────────────────────────────────────────────

/// Horizontal rule (dim).
pub fn rule() -> String {
    style("─".repeat(WIDTH)).dim().to_string()
}

/// Thin rule at custom width.
pub fn rule_w(w: usize) -> String {
    style("─".repeat(w)).dim().to_string()
}

/// Brand wordmark.
pub fn wordmark(version: &str) -> String {
    format!(
        "{}  {}",
        style("AEGIS").bold().white(),
        style(version).dim()
    )
}

/// Section header: TITLE + rule + trailing newline.
pub fn header(title: &str) -> String {
    format!(
        "{}\n{}\n",
        style(title.to_uppercase()).bold().white(),
        rule()
    )
}

/// Micro label (uppercase, dim) — no rule.
pub fn label(title: &str) -> String {
    style(title.to_uppercase()).dim().to_string()
}

/// Key / value row. Key is dim fixed-width; value is white.
pub fn kv(key: &str, value: impl AsRef<str>) -> String {
    format!(
        "  {}  {}",
        style(format!("{key:<14}")).dim(),
        style(value.as_ref()).white()
    )
}

/// Key / value without forcing color (for pre-styled values).
pub fn kv_raw(key: &str, value: impl AsRef<str>) -> String {
    format!(
        "  {}  {}",
        style(format!("{key:<14}")).dim(),
        value.as_ref()
    )
}

// ── text tokens (route all ad-hoc styling here) ─────────────

/// Primary emphasis (white).
pub fn primary(s: impl AsRef<str>) -> String {
    style(s.as_ref()).white().to_string()
}

/// Bold primary (ids, titles).
pub fn primary_bold(s: impl AsRef<str>) -> String {
    style(s.as_ref()).white().bold().to_string()
}

/// Secondary / muted (dim).
pub fn dim(s: impl AsRef<str>) -> String {
    style(s.as_ref()).dim().to_string()
}

/// Empty-state line.
pub fn empty(msg: impl AsRef<str>) -> String {
    format!("  {}", dim(msg))
}

/// Compact board row: mark · primary · dim extras…
pub fn row(mark: &str, primary_text: impl AsRef<str>, secondary: impl AsRef<str>) -> String {
    let sec = secondary.as_ref();
    if sec.is_empty() {
        format!("  {}  {}", mark, primary(primary_text))
    } else {
        format!("  {}  {}  {}", mark, primary(primary_text), dim(sec))
    }
}

/// Dense list line without mark: primary + dim tail.
pub fn list_item(primary_text: impl AsRef<str>, secondary: impl AsRef<str>) -> String {
    let sec = secondary.as_ref();
    if sec.is_empty() {
        format!("  {}", primary(primary_text))
    } else {
        format!("  {}  {}", primary(primary_text), dim(sec))
    }
}

/// Hint / help footer (dim, indented).
pub fn hint(msg: impl AsRef<str>) -> String {
    format!("  {}", dim(msg))
}

// ── status marks (geometric, monochrome-first) ──────────────

/// Complete / healthy.
pub fn mark_ok() -> String {
    style("●").white().to_string()
}

/// Failed / blocked.
pub fn mark_fail() -> String {
    style("×").red().to_string()
}

/// Pending / partial.
pub fn mark_idle() -> String {
    style("·").dim().to_string()
}

/// In progress / active step.
pub fn mark_active() -> String {
    style("▸").white().to_string()
}

/// Skipped.
pub fn mark_skip() -> String {
    style("–").dim().to_string()
}

pub fn mark_bool(ok: bool) -> String {
    if ok {
        mark_ok()
    } else {
        mark_fail()
    }
}

// ── events ──────────────────────────────────────────────────

/// Tool invocation (agent → tool).
pub fn tool_call(name: &str) -> String {
    format!("{} {}", style("▸").dim(), style(name).white())
}

/// Tool completion (tool → agent).
pub fn tool_done(name: &str) -> String {
    format!("{} {}", style("·").dim(), style(name).dim())
}

/// System / lifecycle event.
pub fn event(tag: &str, msg: impl AsRef<str>) -> String {
    format!(
        "{}  {}",
        style(tag.to_uppercase()).dim(),
        style(msg.as_ref()).white()
    )
}

/// Soft success note (learn, reflect).
pub fn note(msg: impl AsRef<str>) -> String {
    format!("{}  {}", style("·").dim(), style(msg.as_ref()).dim())
}

pub fn error_line(msg: impl AsRef<str>) -> String {
    format!("{}  {}", style("ERR").red().bold(), msg.as_ref())
}

pub fn warn_line(msg: impl AsRef<str>) -> String {
    format!("{}  {}", style("WARN").dim().bold(), msg.as_ref())
}

/// Closing bar after a run.
pub fn footer_done() -> String {
    format!("{}\n{}", rule(), style("done").dim())
}

/// REPL prompt glyph.
pub fn prompt_glyph() -> String {
    style("›").white().bold().to_string()
}

// ── composite layouts ───────────────────────────────────────

/// Full REPL open banner.
pub fn repl_banner(
    version: &str,
    session8: &str,
    model: &str,
    effort: &str,
    cwd: &str,
    yolo: bool,
    sandbox: bool,
) -> String {
    let mode = if sandbox {
        "sandbox"
    } else if yolo {
        "yolo"
    } else {
        "prompt"
    };
    format!(
        "\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n\n",
        wordmark(version),
        rule(),
        kv("session", session8),
        kv("model", model),
        kv("effort", effort),
        kv("mode", mode),
        kv("cwd", cwd),
        rule(),
        hint(
            "/plan · /mission · /missions · /memory · /yolo · /cost · /compact · /model · /clear · /quit"
        )
    )
}

/// Auth status block.
pub fn auth_status(
    source: &str,
    email: &str,
    expires: &str,
    team: &str,
    mode: &str,
    path: &str,
    needs_refresh: bool,
) -> String {
    let refresh = if needs_refresh { "yes" } else { "no" };
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        header("auth"),
        kv("source", source),
        kv("email", email),
        kv("expires", expires),
        kv("team", team),
        kv("mode", mode),
        kv("path", path),
        kv("refresh", refresh),
    )
}

/// Unsigned auth.
pub fn auth_unsigned(detail: &str) -> String {
    format!(
        "{}\n  {}\n  {}\n",
        header("auth"),
        primary("not signed in"),
        dim(detail)
    )
}

/// One-shot run header (non-interactive -p).
pub fn run_header(model: &str, effort: &str, session8: &str) -> String {
    format!(
        "\n{}  {}  {}  {}\n{}\n",
        style("AEGIS").bold().white(),
        style(model).dim(),
        style(effort).dim(),
        style(session8).dim(),
        rule_w(48)
    )
}

/// Pad / truncate to width for board columns.
pub fn ellipsis(s: &str, max: usize) -> String {
    if s.chars().count() > max && max > 1 {
        let mut t: String = s.chars().take(max - 1).collect();
        t.push('…');
        t
    } else {
        s.chars().take(max).collect()
    }
}

/// Left-pad string to width (char-aware).
pub fn pad_right(s: &str, w: usize) -> String {
    let n = s.chars().count();
    if n >= w {
        ellipsis(s, w)
    } else {
        format!("{s}{}", " ".repeat(w - n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ellipsis_short() {
        assert_eq!(ellipsis("hi", 8), "hi");
    }

    #[test]
    fn ellipsis_long() {
        let s = ellipsis("abcdefghij", 5);
        assert_eq!(s.chars().count(), 5);
        assert!(s.ends_with('…'));
    }

    #[test]
    fn row_and_empty() {
        let r = row(&mark_ok(), "id", "tail");
        assert!(r.contains("id"));
        assert!(empty("none").contains("none"));
        assert!(hint("help").contains("help"));
    }

    #[test]
    fn primary_dim_tokens() {
        assert!(!primary("x").is_empty());
        assert!(!dim("y").is_empty());
        assert!(!primary_bold("z").is_empty());
    }
}
