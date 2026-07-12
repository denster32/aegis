# Conventions

Project norms for contributors and agents. See [Home](Home.md), [Architecture](Architecture.md), [Modules](Modules.md), [Commands](Commands.md).

## Workspace layout

- **Rust workspace**: `crates/aegis` (CLI) + `crates/aegis-core` (library) and sibling crates (`aegis-auth`, `aegis-xai`, `aegis-tools`, …).
- **Feature modules** live as sibling files under `crates/aegis-core/src/` (e.g. `dream.rs`, `factory.rs`, `qa.rs`, `readiness_v2.rs`, `review.rs`, `wiki.rs`, `automations.rs`).
- **User-facing docs** mirror features under `docs/*.md` plus this wiki under `docs/wiki/`.
- **Agent / project memory** and readiness state live under `.aegis/`.

## Coding & process

- Prefer **small durable memory updates** over large refactors; write lessons into `.aegis/` memory, not only chat.
- Prefer **`aegis` CLI entrypoints** over one-off scripts when a subcommand exists.
- Keep **SECURITY.md**, **LICENSE**, PR/issue templates, and **CHANGELOG.md** current with releases.
- Formatter/linter: workspace `rustfmt.toml`; validate with `cargo build --workspace` and `cargo test`.
- Do **not** invent exploit/PoC content; security posture is via `SECURITY.md` and review automation.

## UI / UX

- Terminal UI is **monochrome** (SpaceX / xAI): white primary, dim secondary, thin rules, geometric marks `● · ▸ ×`.
- Version badge target: **0.8.0**; Rust **1.96+**; license **MIT**.

## Git & tracking

- High-value modules and docs (dream, factory, qa, readiness, review, wiki, automations, `.aegis/`, `aegis-qa` workflow, assets) must stay **committed**—easy to lose if left untracked.
- CI: GitHub Actions including `aegis-qa` (`.github/workflows/aegis-qa.yml`).

## Readiness

- Readiness is scored L1–L5 (often ~L5/95%).
- Remaining gap is often **code-review automation** → `aegis install-code-review`.

## Docs map

| Path | Purpose |
|------|---------|
| `docs/features.md` | Feature matrix |
| `docs/nexus.md` | Nexus / immune system |
| `docs/architecture.md` | Crates |
| `docs/xai-capabilities.md` | Grok 4.5 knobs |
| `docs/learning.md` | Memory & heal |
| `docs/missions.md` | Factory Missions |
| `docs/stress.md` | Stress harness |
| `docs/RELEASE.md` | Release / verify |
| `docs/wiki/` | Wiki source (synced to GitHub Wiki) |
| `SECURITY.md` | Threat model |
| `CONTRIBUTING.md` / `CODE_OF_CONDUCT.md` | Community |
