# Modules

Core library lives in `crates/aegis-core`. The CLI (`crates/aegis`) is a thin front-end. Nexus adds evolution, spore, and hardware crates.

## Core stack

| Area | Responsibility |
|------|----------------|
| **agent** | Main agent loop, tool use, streaming |
| **learn** | Mid-run self-heal, after-run reflect, `.aegis/` updates |
| **missions** | Factory-inspired plan → Mission Control → execute → validate |
| **auth** | Grok OAuth (`~/.grok/auth.json`) |
| **xai** | xAI / Grok API client |
| **tools** | read/write/edit, bash, glob, grep, git, web, memory + capability map |
| **memory** | Project memory + neural summary |
| **swarm** | DAG workers for parallel feature work |
| **evolution** | Mutation genes + fitness scoring |
| **spore** | Viral pack / unpack / vaccinate |
| **hardware** | Host probe + throttle policy |
| **mcp** | MCP integration |
| **store** | Persistence helpers |
| **context** | Context assembly (+ nexus inject) |

## Platform modules

Sibling source files under `crates/aegis-core/src/`:

| Module | File (approx.) | Role |
|--------|----------------|------|
| Dream | `dream.rs` | Nightly deep self-improve cycle |
| Factory | `factory.rs` | SDLC coverage map |
| QA | `qa.rs` | Automated QA (+ `.github/workflows/aegis-qa.yml`) |
| Readiness | `readiness_v2.rs` | L1–L5 readiness scoring |
| Review | `review.rs` | PR code review hooks |
| Wiki | `wiki.rs` | AutoWiki-style documentation |
| Automations | `automations.rs` | Scheduled / event automations |
| UI | `ui.rs` | Monochrome terminal chrome |

## Missions flow

```text
plan → Mission Control → features → validate → reflect
```

## Learning artifacts

| Path | Content |
|------|---------|
| `.aegis/MEMORY.md` | Stack, commands, conventions |
| `.aegis/LESSONS.jsonl` | Durable lessons |
| `.aegis/FAILURES.jsonl` | Heal patterns |
| `.aegis/SKILLS/` | Reusable playbooks |
| `.aegis/nexus/` | Neural summary, evolution runs, capability map |

## Related

- [Home](Home.md) · [Architecture](Architecture.md) · [Commands](Commands.md) · [Nexus](Nexus.md) · [Conventions](Conventions.md) · [Nexus](Nexus.md)
