# Conventions

## Workspace layout

- **Rust workspace**: `crates/aegis` (CLI) + `crates/aegis-core` (library).
- **Feature modules** are sibling files under `crates/aegis-core/src/` (`dream`, `factory`, `qa`, `readiness_v2`, `review`, `wiki`, `automations`, …).
- **User-facing docs** mirror modules under `docs/*.md` plus this wiki under `docs/wiki/`.
- **Agent/project memory** and readiness live under `.aegis/`.

## Memory & learning

- Prefer **small durable memory updates** over large refactors.
- Write lessons into `.aegis/` (`MEMORY.md`, `LESSONS.jsonl`, `FAILURES.jsonl`, `SKILLS/`) rather than relying on chat.
- Every project run should improve stack/commands/conventions knowledge.

## Code & process

- Validate with `cargo test` and `cargo build --workspace`.
- Keep `SECURITY.md`, `LICENSE`, PR/issue templates, and `CHANGELOG.md` current with releases.
- Do not invent exploit/PoC content; security posture via `SECURITY.md` and review automation.
- Commit high-value modules and docs (dream, factory, qa, readiness, review, wiki, automations, `.aegis/`, assets) so they are not lost as untracked files.

## CLI preference

Prefer `aegis` subcommands (`dream`, `qa`, `readiness`, `missions`, …) over one-off scripts when the CLI exposes the feature.

## Style

- Formatter/linter: workspace `rustfmt.toml` and project lint config.
- Cross-link wiki pages with relative links, e.g. [Architecture](Architecture.md).
- Keep wiki pages concise and accurate to the repo.

## Related

- [Home](Home.md) · [Architecture](Architecture.md) · [Modules](Modules.md) · [Commands](Commands.md)
