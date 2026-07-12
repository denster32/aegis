# Architecture

```
┌──────────────── CLI (aegis) ────────────────┐
│ agent · missions · dream · factory · wiki   │
│ review · qa · automation · memory · auth    │
└─────────────────────┬───────────────────────┘
                      │
     ┌────────────────┼────────────────┬──────────────┐
     ▼                ▼                ▼              ▼
 aegis-auth      aegis-core       aegis-swarm    aegis-mcp
     │                │                │              │
     │     ┌──────────┼──────────┐     │              │
     │     ▼          ▼          ▼     ▼              ▼
     │  memory     tools      context store         tools
     │     │          │          │     │
     └─────┴──── aegis-xai ──────┴─────┘
                      │
                 api.x.ai (Responses)
```

| Crate | Responsibility |
|-------|----------------|
| **aegis** | Clap CLI, wiring, install helpers |
| **aegis-auth** | Grok OAuth device flow, auth files, refresh |
| **aegis-core** | Agent loop, heal/reflect, missions, dream, factory, readiness, wiki, QA, review, hooks, checkpoints, UI |
| **aegis-xai** | Responses client: tools, reasoning, cache key, streaming |
| **aegis-tools** | Local coding tools + path locks + permissions |
| **aegis-memory** | `.aegis/` files, lessons, failures, redaction |
| **aegis-swarm** | DAG scheduling, Mission Control types |
| **aegis-context** | Workspace pack for bootstrap context |
| **aegis-store** | Sessions, usage, todos (SQLite) |
| **aegis-mcp** | MCP bridge (optional; tools only) |

## Edges (intentional)

- `tools → memory` for `memory_read` / `memory_write`
- `context → memory` for inject
- `swarm → store` for mission task rows
- **No cycles.** tools/memory/xai/auth do not depend on core.

## Hot path

1. Resolve credentials (env → Aegis auth → Grok auth → API key)
2. Pack workspace + inject memory lessons
3. Responses loop: model → local tools (parallel) → tool outputs → model  
   (`store=true` for tool chains; `reasoning` only on grok-4/3)
4. Self-heal on tool errors when learning enabled
5. Reflect into `.aegis/` at end of turn/session

## Performance note

Wall time is dominated by **API latency + tokens**, not the Rust CLI.
