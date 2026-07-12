# Release 0.6.0

**Status:** production-ready (sandbox + thick tests + CI)  
**Date:** 2026-07-12  
**Tag:** `v0.6.0`

## Product

Sovereign Grok-native coding agent in Rust:

- OAuth via `~/.grok/auth.json`
- Agent tools + Missions + project learning
- Platform: dream, readiness, factory, wiki, QA, review, automations
- Monochrome SpaceX / xAI CLI and brand

## Install

```bash
git clone https://github.com/denster32/aegis.git
cd aegis && ./install.sh
aegis --version   # 0.6.0
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
# optional long:
./scripts/stress_test.sh
```

## Continuous integration

Core CI (`.github/workflows/ci.yml`) runs on push to `main`/`master`, pull requests, and
`workflow_dispatch`. It needs **no external secrets**:

| Step | Command |
|------|---------|
| Cache | `Swatinem/rust-cache@v2` |
| Format | `cargo fmt --all -- --check` |
| Tests | `cargo test --workspace --locked` |
| Lint | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Release build | `cargo build --release -p aegis --locked` |

`Cargo.lock` is committed; all cargo steps use `--locked` so CI matches the lockfile.

Optional live QA (`.github/workflows/aegis-qa.yml`) always runs unit tests (failures fail the job).
Live `aegis qa` runs only when repository secret `XAI_API_KEY` is set; otherwise that path is
skipped. Binary install happens before live QA.

If private-repo Actions has no available runners, workflows still define the correct green path —
re-run via **Actions → CI → Run workflow** once runners are available.

## Assets (canonical)

| File | Role |
|------|------|
| `assets/logo.svg` | Mark (README) |
| `assets/logo.png` | Raster mark (vision fallback / tooling) |
| `assets/banner.svg` | Hero (README) |
| `assets/banner.png` | Raster hero |
| `assets/learning-loop.svg` | Learning diagram (README) |
| `assets/mission-control.svg` | Mission board art (README) |
| `assets/dream-cycle.svg` | Dream pipeline |
| `assets/og.png` | Social / preview (monochrome) |
| `assets/screenshots/*` | Product shots (brand-kit, cli-surfaces, readme-preview) |

## Not in scope

- Multi-tenant sandboxing
- Guaranteed GitHub-hosted runner availability (private-repo quota is infra, not workflow config)
- Large unit-test expansion (stress harness is primary)

## Maintain

- Prefer OAuth over console API keys
- Use `--yolo` only in trusted sandboxes
- See [SECURITY.md](../SECURITY.md) and [features.md](features.md)
