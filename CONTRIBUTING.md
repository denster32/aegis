# Contributing to Aegis

Personal project — patches welcome.

## Setup

```bash
cargo build --locked
cargo test --workspace --locked
./scripts/live_smoke.sh   # needs Grok OAuth (`grok login`)
```

`Cargo.lock` is committed. Prefer `--locked` so local builds match CI.

## Continuous integration

| Workflow | When | Secrets | What |
|----------|------|---------|------|
| **CI** (`.github/workflows/ci.yml`) | push to `main`/`master`, PRs, manual `workflow_dispatch` | none | fmt, `cargo test --workspace --locked`, clippy `-D warnings`, `cargo build --release -p aegis --locked` |
| **Aegis QA** (`.github/workflows/aegis-qa.yml`) | PRs, manual | optional `XAI_API_KEY` | unit tests always (must pass); live `aegis qa` only if secret is set |

Core CI must stay green without any secrets. Use **Actions → CI → Run workflow** if you need a manual re-run.

Before opening a PR:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## Style

- `cargo fmt`
- Prefer small, focused crates
- Do not commit secrets, `.env`, or `auth.json`

## Project learning

When changing agent behavior, update `docs/learning.md` and consider fixtures under `examples/toy-project`.
