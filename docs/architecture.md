# Architecture

```
┌──────────────── CLI (aegis) ────────────────┐
│ agent · missions · nexus · evolve · spore   │
│ dream · factory · hardware · compress       │
└─────────────────────┬───────────────────────┘
                      │
     ┌────────────────┼────────────────┬──────────────────┐
     ▼                ▼                ▼                  ▼
 aegis-auth      aegis-core       aegis-swarm      aegis-evolution
     │                │                │                  │
     │     ┌──────────┼──────────┐     │         aegis-spore
     │     ▼          ▼          ▼     ▼         aegis-hardware
     │  memory     tools      context store
     │     │          │          │     │
     └─────┴──── aegis-xai ──────┴─────┘
                      │
                 api.x.ai (Responses)
                      │
                   aegis-mcp (optional)
```

| Crate | Responsibility |
|-------|----------------|
| **aegis** | Clap CLI, wiring, install helpers |
| **aegis-auth** | Grok OAuth device flow, auth files, refresh |
| **aegis-core** | Agent loop, heal/reflect, missions, dream, factory, readiness, wiki, QA, review, UI |
| **aegis-xai** | Responses client: tools, reasoning, cache key, streaming |
| **aegis-tools** | Local coding tools + path locks + capability map |
| **aegis-memory** | `.aegis/` files, lessons, failures, redaction, neural summary |
| **aegis-swarm** | DAG scheduling, Mission Control types |
| **aegis-evolution** | Mutation genes, Grok propose, local fitness |
| **aegis-spore** | Viral pack / unpack / vaccinate |
| **aegis-hardware** | Host probe + throttle policy |
| **aegis-context** | Workspace pack (+ nexus summary inject) |
| **aegis-store** | Sessions, usage, todos (SQLite) |
| **aegis-mcp** | MCP bridge (optional) |

## Edges (intentional)

- `tools → memory` for `memory_read` / `memory_write`
- `context → memory` for inject + neural summary
- `swarm → store` for mission task rows
- `spore → memory` for redaction
- `evolution → xai` for gene proposals
- **No cycles.**

## Hot path

1. Resolve credentials (env → Aegis auth → Grok auth → API key)
2. Pack workspace + inject memory + neural summary
3. Responses loop: model → local tools (parallel) → tool outputs → model
4. Self-heal on tool errors when learning enabled
5. Reflect / compress into `.aegis/`

## Nexus layout

```
.aegis/nexus/
  neural-summary.json
  capability-map.json
  evolution/<run-id>.json
  spore-out/   # optional pack target
```

See [nexus.md](./nexus.md).
