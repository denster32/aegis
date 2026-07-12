# Aegis Wiki

**Sovereign, Grok-native coding agent in Rust** — tools, Missions, and project learning that improves every run.

| | |
|--|--|
| **License** | MIT |
| **Rust** | 1.96+ |
| **Repo** | [denster32/aegis](https://github.com/denster32/aegis) |

## Why Aegis

| Capability | What you get |
|------------|----------------|
| **Grok OAuth** | Reuses `grok login` / `~/.grok/auth.json` |
| **Coding tools** | read/write/edit, bash, glob, grep, git, web, memory |
| **Learning loop** | Self-heal mid-run · reflect after-run · `.aegis/` memory |
| **Missions** | Plan → Mission Control → execute → validate |
| **Swarm** | DAG workers for parallel features |

## Quick start

```bash
./install.sh   # or: cargo install --path crates/aegis --force
grok login     # or: aegis login
aegis -p "Reply with exactly: pong"
aegis readiness
```

## Wiki pages

- [Architecture](Architecture.md) — crates, agent loop, data flow
- [Modules](Modules.md) — core library modules and platform features
- [Commands](Commands.md) — CLI map, flags, platform commands
- [Conventions](Conventions.md) — workspace layout, memory, process norms

## Learning on every run

Project state under `.aegis/`:

- `MEMORY.md` — stack, commands, conventions
- `LESSONS.jsonl` / `FAILURES.jsonl` — durable lessons & heal patterns
- `SKILLS/` — playbooks Missions can reuse

## Platform (Factory-inspired)

`dream` · `readiness` · `factory` · `wiki generate` · `review` · `qa` · `automation`

See also `docs/dream.md`, `docs/factory.md`, `docs/readiness.md`, `docs/missions.md`, `docs/learning.md`.
