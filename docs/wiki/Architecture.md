# Architecture

Aegis is a **Cargo workspace**: thin CLI + library crates. Nexus extends the core with evolution, spores, and hardware probes.

```
aegis → core (agent, learn, missions, platform)
     → auth · xai · tools · memory · swarm · mcp · store · context
     → evolution · spore · hardware
```

## Crates

| Crate | Role |
|-------|------|
| `aegis` | CLI entrypoint |
| `aegis-core` | Agent loop, learning, Missions, dream/factory/QA/wiki/UI |
| `aegis-auth` | Grok OAuth |
| `aegis-xai` | Responses API |
| `aegis-tools` | Local tools + sandbox membrane + capability map |
| `aegis-memory` | Project memory + neural summary |
| `aegis-swarm` | DAG + Mission Control |
| `aegis-evolution` | Mutation genes + fitness |
| `aegis-spore` | Viral pack / vaccinate |
| `aegis-hardware` | Host probe + throttle policy |
| `aegis-context` | Workspace pack |
| `aegis-store` | SQLite sessions |
| `aegis-mcp` | Optional MCP |

## Data flow

1. **CLI** parses flags (`-p`, `--yolo`, `--sandbox`, `--effort`, …) and routes to core / Nexus.
2. **Auth** reuses Grok OAuth (`grok login` / `~/.grok/auth.json`).
3. **Agent** calls **xAI** with tools (read/write/edit, bash, glob, grep, git, web, memory).
4. **Learning** updates `.aegis/` mid-run (self-heal) and after-run (reflect / compress).
5. **Missions / Swarm** plan → Mission Control → DAG workers → validate → reflect.
6. **Nexus** mutates (evolve), packs spores, probes host, injects neural summary.

## Platform modules

Under `crates/aegis-core/src/`: dream, factory, qa, readiness_v2, review, wiki, automations, ui.

## Project memory

Per-project state in `.aegis/` (MEMORY, LESSONS, FAILURES, SKILLS, missions, dreams, nexus/).

See [docs/architecture.md](../architecture.md) · [docs/nexus.md](../nexus.md).
