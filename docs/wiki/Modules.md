# Modules

Feature modules and crates that make up the agent platform. See [Architecture](Architecture.md) for crate boundaries and [Commands](Commands.md) for CLI entrypoints.

## Core platform (`aegis-core`)

Sibling modules under `crates/aegis-core/src/`:

| Module | Role |
|--------|------|
| **Agent / UI** | REPL, one-shot (`-p`), monochrome TUI, session chrome |
| **Missions** | Factory Missions: plan → Mission Control → execute → validate |
| **Dream** | Nightly self-improve cycle (`dream.rs`) |
| **Factory** | SDLC coverage map |
| **Readiness** | L1–L5 readiness (`readiness_v2`) |
| **QA** | Automated QA (`qa.rs`; CI via `.github/workflows/aegis-qa.yml`) |
| **Review** | PR / local diff review; `aegis install-code-review` |
| **Wiki** | AutoWiki generation / sync |
| **Automations** | Schedules and event-driven runs (`automations.rs`) |

## Learning & memory

- **`aegis-memory`**: project learning, reflect, neural summary
- Durable state under **`.aegis/`** (memory, readiness, agent docs)
- Flags: `--no-learn` to disable writes; self-heal mid-run when enabled

## Tools & API

| Piece | Role |
|-------|------|
| **aegis-tools** | read, write, edit, bash, glob, grep, git, web, memory, vision + locks + capability map |
| **aegis-xai** | Grok 4.5 Responses API |
| **aegis-auth** | OAuth (`grok login` / `~/.grok/auth.json`) |
| **aegis-context** | Workspace pack for prompts |
| **aegis-store** | SQLite sessions |
| **aegis-mcp** | Optional MCP |

## Swarm & Missions

- **`aegis-swarm`**: DAG execution + Mission Control
- CLI: `aegis mission`, `aegis missions new|run|…`, `aegis plan`

## Nexus (organism layer)

| Module | Role |
|--------|------|
| **aegis-evolution** | Mutation genes + fitness (`aegis evolve *`) |
| **aegis-spore** | Viral pack / vaccinate (`aegis spore *`) |
| **compress** | Neural summary (`aegis compress`) |
| **aegis-hardware** | Host probe + throttle (`aegis hardware *`) |
| **nexus status** | Organism overview (`aegis nexus status`) |

Details: `docs/nexus.md`.

## Harness (no agent loop)

- **Smoke** — `aegis smoke` / `./scripts/live_smoke.sh`
- **Stress** — `aegis stress` S0–S27 / `./scripts/stress_test.sh`

## Gotchas

- Readiness may report high (e.g. L5/95%) but stay blocked until **code-review automation** is installed.
- Keep dream, factory, qa, readiness, review, wiki, automations, `.aegis/`, and QA workflow **tracked in git** so they are not lost as untracked work.
