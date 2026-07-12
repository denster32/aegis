# Architecture

```
┌──────────── CLI (aegis) ────────────┐
│ auth · memory · missions · agent    │
└───────────────┬─────────────────────┘
                │
    ┌───────────┼───────────┬─────────────┐
    ▼           ▼           ▼             ▼
 aegis-auth  aegis-core  aegis-memory  aegis-swarm
    │           │           │             │
    └───────────┴─────┬─────┴─────────────┘
                      ▼
                 aegis-xai → api.x.ai
                      │
              aegis-tools · aegis-mcp
```

- **aegis-auth** — Grok OAuth + API key  
- **aegis-core** — agent loop, heal, reflect, missions  
- **aegis-memory** — project learning files  
- **aegis-swarm** — DAGs + Mission Control types  
- **aegis-xai** — Responses API  
