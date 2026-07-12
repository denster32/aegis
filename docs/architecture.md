# Architecture

```
┌──────────────── CLI (aegis) ────────────────┐
│ agent · missions · dream · factory · wiki   │
│ review · qa · automation · memory · auth    │
└─────────────────────┬───────────────────────┘
                      │
     ┌────────────────┼────────────────┐
     ▼                ▼                ▼
 aegis-auth      aegis-core       aegis-swarm
     │                │                │
     │     ┌──────────┼──────────┐     │
     │     ▼          ▼          ▼     │
     │  memory     tools      context  │
     │     │          │          │     │
     └─────┴──── aegis-xai ──────┴─────┘
                      │
                 api.x.ai (Responses)
                      │
                 aegis-store (SQLite)
```

| Crate | Responsibility |
|-------|----------------|
| **aegis** | Clap CLI, wiring, install helpers |
| **aegis-auth** | Grok OAuth device flow, `~/.grok/auth.json`, refresh |
| **aegis-core** | Agent loop, heal/reflect, missions, dream, factory, readiness, wiki, QA, review, hooks, checkpoints |
| **aegis-xai** | Responses client: tools, reasoning, cache key, streaming |
| **aegis-tools** | Local coding tools + path locks + permissions |
| **aegis-memory** | `.aegis/` files, lessons, failures, redaction |
| **aegis-swarm** | DAG scheduling, Mission Control types |
| **aegis-context** | Workspace pack for bootstrap context |
| **aegis-store** | Sessions, usage, todos (SQLite) |
| **aegis-mcp** | MCP bridge (optional) |

## Hot path

1. Resolve credentials (env → Aegis auth → Grok auth → API key)
2. Pack workspace + inject memory lessons
3. Responses loop: model → local tools (parallel) → tool outputs → model
4. Self-heal on tool errors when learning enabled
5. Reflect into `.aegis/` at end of turn/session

## Performance note

Wall time is dominated by **API latency + tokens**, not the Rust CLI. The binary wins on startup (~2 ms), memory, and concurrent tool execution vs large JS bundles. See [xai-capabilities.md](./xai-capabilities.md).
