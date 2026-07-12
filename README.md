<p align="center">
  <img src="assets/logo.svg" alt="Aegis" width="120" />
</p>

# Aegis

**Sovereign, Grok-native coding agent in Rust** — tools, Missions, project learning, and a Factory-inspired SDLC surface.

[![CI](https://github.com/denster32/aegis/actions/workflows/ci.yml/badge.svg)](https://github.com/denster32/aegis/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-skyblue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.96+-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.5.1-blue.svg)](CHANGELOG.md)

<p align="center">
  <img src="assets/banner.svg" alt="banner" width="100%" />
</p>

## Why Aegis

| Capability | What you get |
|------------|----------------|
| **Grok OAuth** | Reuses `grok login` / `~/.grok/auth.json` (subscription path) |
| **Grok 4.5 knobs** | `reasoning.effort`, `prompt_cache_key`, optional server tools |
| **Coding tools** | read/write/edit, bash, glob, grep, git, web, memory, vision |
| **Learning loop** | Self-heal mid-run · reflect after-run · `.aegis/` memory |
| **Missions** | Factory-inspired plan → Mission Control → execute → validate |
| **Platform** | Dream, readiness L1–L5, factory map, wiki, QA, review, automations |
| **Standalone binary** | ~16 MB Rust CLI, ~2 ms cold start — no Node/Bun runtime |

<p align="center">
  <img src="assets/learning-loop.svg" alt="learning loop" width="720" />
</p>

## Install

```bash
git clone https://github.com/denster32/aegis.git
cd aegis
./install.sh
# or: cargo install --path crates/aegis --force
aegis --version   # aegis 0.5.1
```

Auth (if you already use Grok Build, skip login):

```bash
grok login          # or: aegis login
aegis auth status
```

## Quick start

```bash
aegis -p "Reply with exactly: pong"
aegis --yolo -p "Create hello.txt with hi"
aegis --effort low -p "quick tool-heavy fix"
aegis memory show
aegis readiness
aegis factory
aegis missions new "add a /health endpoint"
aegis missions status
aegis missions run <id>
```

## Docs

| Doc | Topic |
|-----|--------|
| [docs/features.md](docs/features.md) | Feature matrix (verified) |
| [docs/architecture.md](docs/architecture.md) | Crate layout |
| [docs/auth.md](docs/auth.md) | OAuth / API key precedence |
| [docs/xai-capabilities.md](docs/xai-capabilities.md) | Grok 4.5 feature matrix |
| [docs/learning.md](docs/learning.md) | Project memory & self-heal |
| [docs/missions.md](docs/missions.md) | Factory-style Missions |
| [docs/dream.md](docs/dream.md) | Nightly dream |
| [docs/readiness.md](docs/readiness.md) | L1–L5 readiness |
| [docs/factory.md](docs/factory.md) | Software Factory map |
| [docs/wiki.md](docs/wiki.md) · [docs/qa.md](docs/qa.md) · [docs/review.md](docs/review.md) | Wiki / QA / review |
| [docs/automations.md](docs/automations.md) | File-based automations |
| [docs/stress.md](docs/stress.md) | Stress harness |
| [docs/wiki/Home.md](docs/wiki/Home.md) | In-repo wiki home |

## Terminal UI

Monochrome **SpaceX / xAI** language: white primary, dim secondary, thin rules, geometric marks (`● · ▸ ×`). No rainbow chrome.

## Learning

Every run on a project can update `.aegis/`:

- `MEMORY.md` — stack, commands, conventions  
- `LESSONS.jsonl` / `FAILURES.jsonl` — durable lessons & heal patterns  
- `SKILLS/` — playbooks Missions can reuse  
- `dreams/`, `missions/`, `qa/`, `reviews/` — platform artifacts  

See [docs/learning.md](docs/learning.md).

## Missions (Factory-inspired)

```text
plan → Mission Control → features → validate → reflect
```

<p align="center">
  <img src="assets/mission-control.svg" alt="Mission Control" width="640" />
</p>

See [docs/missions.md](docs/missions.md) · inspired by [Factory Missions](https://docs.factory.ai/features/missions/overview).

## Architecture

```
aegis (CLI)
  ├── aegis-core     agent loop, heal, dream, missions, factory, review, wiki
  ├── aegis-auth     Grok OAuth + API key
  ├── aegis-xai      Responses API (reasoning, cache, server tools)
  ├── aegis-tools    coding tools + path locks
  ├── aegis-memory   project learning files
  ├── aegis-swarm    DAG workers + Mission Control types
  ├── aegis-context  workspace packing
  ├── aegis-store    SQLite sessions / usage
  └── aegis-mcp      MCP bridge (optional)
```

Details: [docs/architecture.md](docs/architecture.md).

## CLI map

| Command | Purpose |
|---------|---------|
| `aegis` / `-p` | REPL / one-shot agent |
| `aegis plan` | Structured implementation plan |
| `aegis mission` | Quick swarm DAG mission |
| `aegis missions *` | Full Missions product |
| `aegis memory *` | Project learning |
| `aegis readiness` | Agent readiness L1–L5 |
| `aegis factory` | SDLC coverage map |
| `aegis dream` | Nightly deep self-improve |
| `aegis wiki *` | AutoWiki-style docs |
| `aegis review` | PR or local diff review |
| `aegis qa` / `install-qa` | Automated QA |
| `aegis automation *` | Scheduled/event automations |
| `aegis checkpoint *` | Git checkpoint / restore |
| `aegis vision` | Image describe |
| `aegis session *` | Session management |
| `aegis auth *` / `login` | OAuth |

**Flags:** `--effort low|medium|high` · `--yolo` · `--cwd` · `--session` · `--no-learn` · `--stream` · `-v` · `-m`

## Grok 4.5 utilization

Aegis is built around the xAI Responses API:

- **OAuth JWT** from `~/.grok/auth.json` (preferred over spent console keys)
- **`reasoning.effort`** on every turn (`--effort` + tool-step policy)
- **`prompt_cache_key`** sticky per session
- Optional server tools: `web_search`, `code_execution`, `x_search`

Full matrix: [docs/xai-capabilities.md](docs/xai-capabilities.md).

## Smoke & stress

```bash
./scripts/live_smoke.sh          # short live path (OAuth)
./scripts/stress_test.sh         # multi-phase live stress (OAuth)
```

Latest results: [docs/stress-report.md](docs/stress-report.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Security: [SECURITY.md](SECURITY.md).

## License

MIT — see [LICENSE](LICENSE).
