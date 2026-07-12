# Architecture

Aegis is a **Cargo workspace** with a thin CLI and a library core.

```
aegis → core (agent, learn, missions)
     → auth · xai · tools · memory · swarm · mcp · store · context
```

## Crates

| Crate | Role |
|-------|------|
| `crates/aegis` | CLI entrypoint (`aegis`, flags, subcommands) |
| `crates/aegis-core` | Library: agent loop, tools, learning, Missions, platform modules |

## Data flow

1. **CLI** parses flags (`-p`, `--yolo`, `--effort`, `--cwd`, …) and routes to core.
2. **Auth** reuses Grok OAuth (`grok login` / `~/.grok/auth.json`).
3. **Agent** calls **xAI** with tools (read/write/edit, bash, glob, grep, git, web, memory).
4. **Learning** updates `.aegis/` mid-run (self-heal) and after-run (reflect).
5. **Missions / Swarm** plan → Mission Control → DAG workers → validate → reflect.

## Platform modules

Sibling modules under `crates/aegis-core/src/`:

- `dream` — nightly deep self-improve
- `factory` — SDLC coverage map
- `qa` — automated QA
- `readiness_v2` — L1–L5 readiness scoring
- `review` — PR code review hooks
- `wiki` — AutoWiki-style docs
- `automations` — scheduled/event automations

## Project memory

Per-project state lives in `.aegis/`:

- `MEMORY.md` — stack, commands, conventions
- `LESSONS.jsonl` / `FAILURES.jsonl` — durable lessons & heal patterns
- `SKILLS/` — playbooks Missions reuse

## Related

- [Home](Home.md) · [Modules](Modules.md) · [Commands](Commands.md) · [Conventions](Conventions.md)
- Upstream docs: `docs/architecture.md`, `docs/learning.md`, `docs/missions.md`
