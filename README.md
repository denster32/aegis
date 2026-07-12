<p align="center">
  <img src="assets/logo.svg" alt="Aegis" width="120" />
</p>

# Aegis

**Sovereign, Grok-native coding agent in Rust** — tools, Missions, and project learning that improves every run.

[![CI](https://github.com/denster32/aegis/actions/workflows/ci.yml/badge.svg)](https://github.com/denster32/aegis/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-skyblue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.96+-orange.svg)](https://www.rust-lang.org)

<p align="center">
  <img src="assets/banner.svg" alt="banner" width="100%" />
</p>

## Why Aegis

| Capability | What you get |
|------------|----------------|
| **Grok OAuth** | Reuses `grok login` / `~/.grok/auth.json` |
| **Coding tools** | read/write/edit, bash, glob, grep, git, web, memory |
| **Learning loop** | Self-heal mid-run · reflect after-run · `.aegis/` memory |
| **Missions** | Factory-inspired plan → Mission Control → execute → validate |
| **Swarm** | DAG workers for parallel features |

<p align="center">
  <img src="assets/learning-loop.svg" alt="learning loop" width="720" />
</p>

## Install

```bash
git clone https://github.com/denster32/aegis.git
cd aegis
./install.sh
# or: cargo install --path crates/aegis --force
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
aegis memory show
aegis readiness
aegis missions new "add a /health endpoint"
aegis missions status
aegis missions run <id>
```

### Learning

Every run on a project updates `.aegis/`:

- `MEMORY.md` — stack, commands, conventions  
- `LESSONS.jsonl` / `FAILURES.jsonl` — durable lessons & heal patterns  
- `SKILLS/` — playbooks Missions can reuse  

See [docs/learning.md](docs/learning.md).

### Missions (Factory-inspired)

```text
plan → Mission Control → features → validate → reflect
```

<p align="center">
  <img src="assets/mission-control.svg" alt="Mission Control" width="640" />
</p>

See [docs/missions.md](docs/missions.md) · inspired by [Factory Missions](https://docs.factory.ai/features/missions/overview).

## Architecture

See [docs/architecture.md](docs/architecture.md).

```
aegis → core (agent, learn, missions)
      → auth · xai · tools · memory · swarm · mcp · store · context
```

## CLI map

| Command | Purpose |
|---------|---------|
| `aegis` / `-p` | REPL / one-shot agent |
| `aegis plan` | Structured implementation plan |
| `aegis mission` | Quick swarm DAG mission |
| `aegis missions *` | Full Missions product |
| `aegis memory *` | Project learning |
| `aegis readiness` | Agent readiness score |
| `aegis auth *` / `login` | OAuth |

Flags: `--effort low|medium|high`, `--yolo`, `--cwd`, `--session`, `--no-learn`, `--stream`, `-v`

## Live smoke

```bash
./scripts/live_smoke.sh
```

## License

MIT — see [LICENSE](LICENSE).
