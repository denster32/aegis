# Architecture

Aegis is a **Rust Cargo workspace**: a thin CLI binary plus library crates that implement the agent, tools, learning, and platform features.

See also [Home](Home.md), [Modules](Modules.md), [Commands](Commands.md).

## High-level layout

```
aegis (CLI)
  ├── aegis-core         agent · missions · dream · factory · ui
  ├── aegis-auth         Grok OAuth
  ├── aegis-xai          Responses API
  ├── aegis-tools        coding tools + locks + capability map
  ├── aegis-memory       .aegis/ learning · neural summary
  ├── aegis-swarm        DAG + Mission Control
  ├── aegis-evolution    mutation genes + fitness
  ├── aegis-spore        viral pack / vaccinate
  ├── aegis-hardware     host probe + throttle policy
  ├── aegis-context      workspace pack
  ├── aegis-store        SQLite sessions
  └── aegis-mcp          optional MCP
```

## Workspace crates

| Crate | Role |
|-------|------|
| `crates/aegis` | CLI entrypoint (`aegis` binary) |
| `crates/aegis-core` | Agent loop, Missions, Dream, factory, QA, readiness, review, wiki, automations, TUI |
| `aegis-auth` | Reuses `grok login` / `~/.grok/auth.json` |
| `aegis-xai` | Grok 4.5 Responses API (`reasoning.effort`, `prompt_cache_key`, server tools) |
| `aegis-tools` | read / write / edit / bash / glob / grep / git / web / memory / vision |
| `aegis-memory` | Project learning under `.aegis/` |
| `aegis-swarm` | Mission DAG + Mission Control |
| `aegis-evolution` | Evolve genes + fitness |
| `aegis-spore` | Spore pack / vaccinate |
| `aegis-hardware` | Host probe + throttle |
| `aegis-context` | Workspace context pack |
| `aegis-store` | SQLite session store |
| `aegis-mcp` | Optional MCP integration |

## Runtime flow

1. **CLI** parses flags (`--effort`, `--yolo`, `--sandbox`, `--cwd`, `--session`, …) and routes subcommands.
2. **Auth** loads Grok OAuth credentials.
3. **Agent loop** (core) calls **xAI** with tools from **aegis-tools**.
4. **Memory** reads/writes durable lessons in `.aegis/`; optional self-heal mid-run.
5. **Missions / swarm** plan → Mission Control → execute → validate.
6. **Platform** surfaces (readiness, factory, dream, wiki, QA, review, automations) live as modules under `aegis-core`.
7. **Nexus** layers (evolve, spore, compress, hardware, capability map) extend the organism model.

## Binary profile

- ~16 MB Rust CLI, ~2 ms cold start, no Node runtime
- Monochrome TUI (SpaceX / xAI style)

## Related docs in-repo

- `docs/architecture.md` — crate detail
- `docs/nexus.md` — living immune system
- `docs/learning.md` — memory & heal
- `docs/missions.md` — Factory Missions
- `SECURITY.md` — threat model
