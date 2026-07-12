# Aegis Nexus

**Living digital immune system** — agents as cells, sandbox + redact as membrane, Grok 4.5 as brain stem.

## Metaphor → code

| Biology | Aegis |
|---------|--------|
| Cell | Agent / swarm worker |
| Memory | `.aegis/` lessons, failures, dream |
| Neural summary | `.aegis/nexus/neural-summary.json` |
| Mutation | `aegis evolve` (genes from Grok 4.5) |
| Fitness | readiness + cargo + lessons |
| Spore | `aegis spore pack|unpack` |
| Membrane | `--sandbox` + redact |
| Host symbiosis | `aegis hardware probe|policy` |

## CLI

```bash
aegis nexus status          # organism overview
aegis evolve propose "…"    # Grok mutation genes
aegis evolve run <id>       # score genes locally
aegis evolve status         # list evolution runs
aegis compress              # neural summary (local or Grok)
aegis spore pack --out DIR
aegis spore unpack DIR
aegis spore vaccinate DIR   # unpack + sandbox default
aegis hardware probe
aegis hardware policy       # heuristic throttle policy
```

## Layout

```
.aegis/nexus/
  neural-summary.json
  evolution/<run-id>.json
  capability-map.json   # optional snapshot
```

## Safety

- Evolution micro-eval uses sandbox Deny when configured.
- Spores always redact secrets; foreign hosts default `sandbox=true`.
- Not OS multi-tenant isolation — tool-layer membrane only.

## Crates

- `aegis-evolution` — genes, fitness, runs
- `aegis-spore` — pack / unpack / vaccinate
- `aegis-hardware` — host probe + throttle policy
