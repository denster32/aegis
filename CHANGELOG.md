# Changelog

## 0.5.1 — 2026-07-12

### Fixed

- **Compile break** in agent tool-loop: restore `had_tools_last` for reasoning policy
- **Swarm workers:** omit `reasoning.effort` for models that reject it (`grok-code-fast-1` HTTP 400)
- Align workspace version with release tags; `cargo fmt` / clippy clean for CI

### Docs

- Prime-time README (full CLI map, Grok 4.5, docs index)
- Expand platform docs; stress harness documentation
- Changelog catch-up for 0.4.x / 0.5.0

## 0.5.0 — 2026-07-12

### Added

- Wire **`reasoning.effort`** and **`prompt_cache_key`** on every Responses turn
- Optional server tools: `web_search`, `x_search`, `code_execution` (config + flags)
- Track cached / reasoning token usage details
- `--effort` correctly drives API reasoning (tool steps prefer low latency)
- [docs/xai-capabilities.md](docs/xai-capabilities.md)

## 0.4.0 — 2026-07-12

### Added

- Live multi-phase **stress harness** (`scripts/stress_test.sh`)
- Self-heal proven under induced compile failure
- Vision tools, git checkpoints, hooks
- Checkpoint mkdir hardening for `.aegis/checkpoints`

## 0.3.0 — 2026-07-12

### Added

- Nightly dream (`aegis dream`)
- Readiness v2 L1–L5 pillars
- Software Factory map
- Wiki, QA, code review, automations

## 0.2.0 — 2026-07-12

### Added

- **Project learning**: `.aegis/MEMORY.md`, LESSONS/FAILURES JSONL, self-heal, end-of-run reflection
- **Factory-style Missions**: `aegis missions new|list|status|run`, Mission Control board, readiness
- **memory_read / memory_write** tools
- Docs, CI, assets kit
- GitHub repository packaging

## 0.1.0 — 2026-07-11

- Initial Grok OAuth agent: tools, plan, swarm mission DAG, sessions
