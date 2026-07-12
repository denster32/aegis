# Commands

## Install & auth

```bash
git clone https://github.com/denster32/aegis.git
cd aegis
./install.sh
# or: cargo install --path crates/aegis --force

grok login          # or: aegis login
aegis auth status
```

## Agent

| Command | Purpose |
|---------|---------|
| `aegis` / `aegis -p "…"` | REPL / one-shot agent |
| `aegis plan` | Structured implementation plan |
| `aegis mission` | Quick swarm DAG mission |
| `aegis missions new "…"` | Create full Mission |
| `aegis missions status` | Mission status |
| `aegis missions run <id>` | Execute Mission |
| `aegis memory show` | Project learning |
| `aegis readiness` | L1–L5 agent readiness score |
| `aegis auth *` / `login` | OAuth |

### Common flags

`--effort low\|medium\|high` · `--yolo` · `--cwd` · `--session` · `--no-learn` · `--stream` · `-v`

### Examples

```bash
aegis -p "Reply with exactly: pong"
aegis --yolo -p "Create hello.txt with hi"
aegis memory show
aegis readiness
aegis missions new "add a /health endpoint"
aegis missions run <id>
```

## Platform

| Command | Purpose |
|---------|---------|
| `aegis dream` | Nightly deep self-improve |
| `aegis factory` | SDLC coverage map |
| `aegis wiki generate` | AutoWiki-style docs |
| `aegis review --pr N` | PR code review |
| `aegis install-qa` / `aegis qa` | Automated QA |
| `aegis automation list` | Scheduled/event automations |
| `aegis install-code-review` | Install code-review automation (readiness gap) |

## Dev / CI

```bash
cargo test
cargo build --workspace
./scripts/live_smoke.sh
```

QA workflow: `.github/workflows/aegis-qa.yml`.

Prefer **aegis CLI** entrypoints over ad-hoc scripts when available.

## Related

- [Home](Home.md) · [Architecture](Architecture.md) · [Modules](Modules.md) · [Conventions](Conventions.md)
