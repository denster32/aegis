# Aegis Wiki

**Sovereign Grok-native coding agent** — Rust · tools · Missions · project learning.

| | |
|---|---|
| Version | 0.8.0 |
| Stack | Rust 1.96+ (Cargo workspace) |
| License | MIT |
| UI | Monochrome terminal |

## Why Aegis

| Capability | Detail |
|------------|--------|
| **Grok OAuth** | Reuses `grok login` / `~/.grok/auth.json` |
| **Grok 4.5** | `reasoning.effort` · `prompt_cache_key` · server tools |
| **Tools** | read / write / edit / bash / glob / grep / git / web / memory / vision |
| **Learning** | Self-heal mid-run · reflect · `.aegis/` memory |
| **Missions** | Plan → Mission Control → execute → validate |
| **Platform** | Dream · readiness · factory · wiki · QA · review · automations |
| **Nexus** | Evolve · spore · compress · hardware · capability map |
| **Binary** | ~16 MB Rust CLI · ~2 ms cold start · no Node runtime |

## Wiki pages

- [Architecture](Architecture.md) — crates, runtime flow, binary profile
- [Modules](Modules.md) — core platform and Nexus modules
- [Commands](Commands.md) — CLI, flags, smoke/stress
- [Nexus](Nexus.md) — evolve · spore · compress · hardware
- [Conventions](Conventions.md) — workspace norms, docs, readiness

## Install

```bash
git clone https://github.com/denster32/aegis.git
cd aegis
./install.sh
aegis --version
grok login   # or: aegis login
aegis auth status
```

## Quick start

```bash
aegis -p "Reply with exactly: pong"
aegis --yolo --effort low -p "Create hello.txt with hi"
aegis readiness
aegis factory
aegis missions new "add a /health endpoint"
aegis missions run <id>
```

## In-repo docs

Feature matrix, Nexus, learning, missions, stress, release, and security live under `docs/` and `SECURITY.md`. This wiki is sourced from `docs/wiki/` and synced to the GitHub Wiki.
