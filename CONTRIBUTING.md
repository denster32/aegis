# Contributing to Aegis

Personal project — patches welcome.

## Setup

```bash
cargo build
cargo test --workspace
./scripts/live_smoke.sh   # needs Grok OAuth (`grok login`)
```

## Style

- `cargo fmt`
- Prefer small, focused crates
- Do not commit secrets, `.env`, or `auth.json`

## Project learning

When changing agent behavior, update `docs/learning.md` and consider fixtures under `examples/toy-project`.
