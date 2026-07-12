QA skill notes for Aegis workspace:
- Prefer cargo test --workspace and existing .github/workflows/aegis-qa.yml.
- When new modules land under crates/aegis-core/src/ (dream, factory, qa, readiness_v2, review, wiki, automations), add smoke coverage and keep docs/*.md in sync.
- Readiness gaps (e.g. missing code-review automation) should be filed as actionable checklist items, not silent skips.
- Never generate exploit payloads; security work stays defensive (SECURITY.md, review automation, dependency hygiene).