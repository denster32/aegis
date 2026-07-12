# Commands

CLI reference for the `aegis` binary. Cross-links: [Home](Home.md) · [Architecture](Architecture.md) · [Modules](Modules.md) · [Conventions](Conventions.md).

## Install & auth

```bash
git clone https://github.com/denster32/aegis.git
cd aegis
./install.sh
aegis --version

grok login          # or: aegis login
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

## Subcommands

| Command | Purpose |
|---------|---------|
| `aegis` / `-p` | REPL / one-shot prompt |
| `aegis plan` | Structured plan |
| `aegis mission` | Swarm DAG |
| `aegis missions *` | Factory Missions (new / run / …) |
| `aegis memory *` | Project learning |
| `aegis readiness` | L1–L5 readiness |
| `aegis factory` | SDLC coverage map |
| `aegis dream` | Nightly self-improve |
| `aegis wiki *` | AutoWiki |
| `aegis smoke` | Live smoke harness (no agent loop) |
| `aegis stress` | Live stress S0–S27 (no agent loop) |
| `aegis review` | PR / local diff |
| `aegis qa` | Automated QA |
| `aegis automation *` | Schedules / events |
| `aegis checkpoint *` | Git checkpoints |
| `aegis vision` | Image describe |
| `aegis auth` / `login` | OAuth |
| `aegis nexus status` | Nexus organism overview |
| `aegis evolve *` | Mutation genes + fitness |
| `aegis spore *` | Viral pack / vaccinate |
| `aegis compress` | Neural summary |
| `aegis hardware *` | Host probe / throttle |

## Global flags

| Flag | Meaning |
|------|---------|
| `--effort low\|medium\|high` | Grok reasoning effort |
| `--yolo` | Less confirmation / aggressive tool use |
| `--sandbox` | Sandboxed execution |
| `--cwd` | Working directory |
| `--session` | Session id |
| `--no-learn` | Disable project learning writes |
| `--stream` | Stream responses |
| `-v` | Verbose |
| `-m` | Model override |

## Dev / CI validation

```bash
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace
aegis smoke
aegis stress
```

Prefer **aegis CLI entrypoints** over ad-hoc scripts when available. If the agent REPL denies bash (prompt mode), type `/yolo` or run `aegis stress` outside the loop.

## Smoke & stress

- `aegis smoke` — live smoke (no agent loop)
- `aegis stress` — stress S0–S27 (no agent loop)
- Scripts: `./scripts/live_smoke.sh` · `./scripts/stress_test.sh`

See `docs/stress.md` and `docs/RELEASE.md` for release verification.
