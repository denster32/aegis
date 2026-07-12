# Release 0.5.2 — finalized

**Status:** personal production-ready  
**Date:** 2026-07-12  
**Tag:** `v0.5.2`

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
aegis --version   # 0.5.2
grok login && aegis auth status
```

## Verify

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./scripts/live_smoke.sh
# optional long:
./scripts/stress_test.sh
```

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
- Public GitHub Actions reliability (private runner quota)
- Large unit-test expansion (stress harness is primary)

## Maintain

- Prefer OAuth over console API keys
- Use `--yolo` only in trusted sandboxes
- See [SECURITY.md](../SECURITY.md) and [features.md](features.md)
