# Contributing to Aegis

Patches welcome. Keep changes small, tested, and monochrome.

## Setup

```bash
cargo build --locked
cargo test --workspace --locked
./scripts/live_smoke.sh   # needs Grok OAuth (`grok login`)
```

`Cargo.lock` is committed. Prefer `--locked` so local builds match CI.

## Architecture (crate map)

| Crate | Role |
|-------|------|
| `aegis` | CLI binary |
| `aegis-core` | agent loop, missions, dream, factory, UI chrome |
| `aegis-tools` | coding tools + permissions + path locks |
| `aegis-xai` | Responses API client |
| `aegis-auth` | Grok OAuth |
| `aegis-memory` | `.aegis/` learning |
| `aegis-swarm` | DAG / Mission Control |
| `aegis-store` | SQLite sessions |
| `aegis-context` | workspace pack |
| `aegis-mcp` | optional MCP |
| `aegis-evolution` / `spore` / `hardware` | Nexus |

UI: all terminal chrome goes through `aegis_core::ui` (white / dim / red errors only).

## Continuous integration

| Workflow | When | Secrets | What |
|----------|------|---------|------|
| **CI** | push / PR / `workflow_dispatch` | none | fmt, test, clippy `-D warnings`, release build |
| **Aegis QA** | PR / manual | optional `XAI_API_KEY` | unit tests always; live QA if secret set |
| **Release** | tag `v*` | `GITHUB_TOKEN` | linux binary + checksums |

Before opening a PR:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Agent-path changes should also pass:

```bash
./scripts/live_smoke.sh
STRESS_LONG=1 ./scripts/stress_test.sh   # hard-fail; no soft-pass
```

## Style

- `cargo fmt`
- Prefer small, focused crates
- Do not commit secrets, `.env`, or `auth.json`
- Sandbox and permission changes need unit tests

## Project learning

When changing agent behavior, update `docs/learning.md` and consider fixtures under `examples/`.

## Security

See [SECURITY.md](SECURITY.md). Never paste tokens into issues or PRs.
