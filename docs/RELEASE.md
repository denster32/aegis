# Release 0.8.0

**Status:** production-ready (bulletproof tests + hard stress + UI unify + OSS traffic pack)  
**Date:** 2026-07-12  
**Tag:** `v0.8.0`

## Product

Sovereign Grok-native coding agent in Rust:

- OAuth via `~/.grok/auth.json`
- Agent tools + Missions + project learning (heal credit fixed)
- Platform: dream, readiness, factory, wiki, QA, review, automations
- **Nexus:** evolve · spore · compress · hardware · capability map
- Monochrome SpaceX / xAI CLI (single `ui` chrome path)
- Hard-fail live stress S0–S27

## Install

```bash
git clone https://github.com/denster32/aegis.git
cd aegis && ./install.sh
aegis --version   # 0.8.0
grok login && aegis auth status
aegis --sandbox -p "Summarize README.md"
```

## Verify

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --release -p aegis --locked
./scripts/live_smoke.sh
STRESS_LONG=1 ./scripts/stress_test.sh   # hard-fail; FAIL must be 0
```

## Continuous integration

| Workflow | Notes |
|----------|--------|
| CI | fmt · test · clippy · release build (no secrets); optional macOS on main |
| QA | unit always; live if `XAI_API_KEY` |
| Release | tag `v*` → linux binary + SHA256SUMS |
| Dependabot | cargo + github-actions weekly |

## Maintain

- Prefer OAuth over console API keys
- Use `--yolo` only in trusted workspaces; prefer `--sandbox` for untrusted trees
- See [SECURITY.md](../SECURITY.md) and [features.md](features.md)
