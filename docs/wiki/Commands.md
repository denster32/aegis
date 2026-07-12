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
| `aegis missions new\|list\|status\|run` | Full Missions product |
| `aegis memory show\|dump\|clear` | Project learning |
| `aegis readiness` | L1–L5 agent readiness score |
| `aegis auth *` / `login` / `logout` | OAuth |
| `aegis checkpoint *` | Git checkpoint / restore |
| `aegis vision` | Describe an image |
| `aegis session *` | Session management |

### Common flags

`--effort low|medium|high` · `--yolo` · `--sandbox` · `--cwd` · `--session` · `--no-learn` · `--stream` · `-v` · `-m`

### Examples

```bash
aegis -p "Reply with exactly: pong"
aegis --yolo --effort low -p "Create hello.txt with hi"
aegis --sandbox -p "Summarize README only"
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
| `aegis review --pr N` / `--diff` | PR / local code review |
| `aegis install-qa` / `aegis qa` | Automated QA |
| `aegis automation list\|ensure\|run` | Scheduled/event automations |
| `aegis install-code-review` | Install code-review automation |
| `aegis install-wiki-refresh` | Install wiki-refresh workflow |

## Nexus

| Command | Purpose |
|---------|---------|
| `aegis nexus status` | Organism overview |
| `aegis evolve propose\|run\|status` | Mutation genes + fitness |
| `aegis spore pack\|unpack\|vaccinate` | Viral pack / import |
| `aegis compress [--local]` | Neural summary |
| `aegis hardware probe\|policy` | Host probe / throttle |

See [nexus.md](../nexus.md).

## Dev / CI

```bash
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace
./scripts/live_smoke.sh
./scripts/stress_test.sh
```

QA workflow: `.github/workflows/aegis-qa.yml`.

## Related

- [Home](Home.md) · [Architecture](Architecture.md) · [Modules](Modules.md) · [Conventions](Conventions.md)
