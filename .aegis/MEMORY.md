# Project Memory

## Stack

Rust (Cargo workspace) agent platform: crates/aegis CLI + crates/aegis-core library. Features: Grok OAuth agent, learning/Missions, Dream cycle, Automations, Software Factory coverage, QA, Readiness v2, Code Review hooks, Wiki. Tooling: GitHub Actions (CI + aegis-qa), formatter/linter config, .env.example, agent docs (.aegis / AGENTS.md), project wiki under docs/wiki/.
## Commands

cargo test / cargo build --workspace for core validation; aegis install-code-review to close remaining readiness gap; run dream/automations via project modules (crates/aegis-core/src/dream.rs, automations.rs); QA via .github/workflows/aegis-qa.yml and crates/aegis-core/src/qa.rs; prefer aegis CLI entrypoints over ad-hoc scripts when available.
## Gotchas

Readiness is L5/95% but blocked on missing code-review automation (recommendation: aegis install-code-review). Many high-value modules and docs are still untracked (dream, factory, qa, readiness_v2, review, wiki, automations, .aegis/, aegis-qa workflow, assets/dream-cycle.svg)—easy to lose if not committed. Memory sections were empty at snapshot time; prefer writing durable lessons into memory over relying on chat. Do not invent exploit/PoC content; keep security posture via SECURITY.md and review automation.
## Conventions

Rust workspace: crates/aegis (CLI) + crates/aegis-core (library modules). Feature modules live as sibling files under crates/aegis-core/src/ (dream, factory, qa, readiness_v2, review, wiki, automations). User-facing docs mirror modules under docs/*.md plus docs/wiki/. Agent/project memory and readiness live under .aegis/. Prefer small durable memory updates over large refactors. Keep SECURITY.md, LICENSE, PR/issue templates, and CHANGELOG.md current with releases.
