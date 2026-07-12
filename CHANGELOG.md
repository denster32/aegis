# Changelog

## 0.8.0 — 2026-07-12

### Bulletproof reliability

- **Heal credit fix** — after self-heal guidance, the next successful tool call increments `heal_successes` once (mixed ok/err batches allowed)
- **Hard-fail stress** — no soft-pass on mission/review/checkpoint; S3 requires `heal_successes >= 1`
- **Stress S24–S27** — sandbox bash deny, path-lock concurrent edit, SSRF probe, CLI help surfaces
- **Unit coverage** — learn runtime, checkpoint create/list/restore ambiguity, expanded UI tokens

### UI

- All CLI boards route through `aegis_core::ui` (`primary` / `dim` / `row` / `empty` / `hint`)
- Density pass: consistent marks, fixed-width keys, monochrome only

### GitHub / OSS traffic

- Issue templates (bug + feature), config.yml, richer PR template
- Dependabot (cargo + actions), CODEOWNERS, release workflow (linux binary + SHA256)
- CI optional macOS job on main / workflow_dispatch
- Contributor Covenant CoC, expanded CONTRIBUTING + SECURITY supported versions

## 0.7.0 — 2026-07-12

### Aegis Nexus (living immune system MVP)

- **`aegis-evolution`** — genes, Grok propose, local fitness, run persistence
- **`aegis-spore`** — pack / unpack / vaccinate with redaction membrane
- **`aegis-hardware`** — host probe + throttle policy heuristics
- **Neural summary** — `aegis compress` → `.aegis/nexus/neural-summary.json` (injected into context)
- **Capability map** — registry self-description snapshot
- CLI: `nexus`, `evolve`, `spore`, `compress`, `hardware`
- Docs: [docs/nexus.md](docs/nexus.md)

## 0.6.0 — 2026-07-12

### Sandbox (multi-tenant-style tool isolation)

- Global **`--sandbox`** → `PermissionMode::Deny` (wins over YOLO / auto-yolo)
- Bash fully denied; read/write/edit workspace-only (no outside-cwd escape)
- Hardened `web_fetch` private IP / host blocking
- Workers inherit sandbox from boss missions
- Documented in SECURITY.md + features.md

### Tests

- Unit suite **13 → ~95** covering store, auth, tools, xai types, core config/factory/readiness, swarm DAG, memory redact

### CI

- Core CI: rust-cache, `fmt`, `test --locked`, `clippy -D warnings --locked`, `build --release --locked`
- QA workflow: unit tests fail the job; live QA only if `XAI_API_KEY` secret present
- `workflow_dispatch` on CI and QA

## 0.5.2 — 2026-07-12

### Final release polish

- Monochrome SpaceX / xAI brand kit (logo, banner, learning loop, mission control, dream)
- Regenerated grayscale PNG/OG assets; removed legacy colorful/orphan media
- CLI visual system finalized (`aegis_core::ui`)
- README centered mark + monochrome badges
- Project audit + security hardening carried from 0.5.1
- Screenshots: brand kit, CLI surfaces, README preview

### Hygiene

- Confirmed no leftover cyan/colorful CLI brand refs or dead silence stubs
- Confirmed `failures_tests_extra` removed; factory Document stage sees `docs/wiki`
- `.gitignore`: secrets/runtime ignored; skills/automations seeds remain trackable

### Docs

- [docs/features.md](docs/features.md) · [docs/audit-2026-07-12.md](docs/audit-2026-07-12.md) · [docs/RELEASE.md](docs/RELEASE.md)

## 0.5.1 — 2026-07-12

### Fixed

- **Compile break** in agent tool-loop: restore `had_tools_last` for reasoning policy
- **Swarm workers:** omit `reasoning.effort` for models that reject it (`grok-code-fast-1` HTTP 400)
- Align workspace version with release tags; `cargo fmt` / clippy clean for CI
- UTF-8-safe truncation in agent/learn (no mid-char panic)
- Shared `model_supports_reasoning` for wiki/dream/review/reflect
- Tool loops force `store=true` for `previous_response_id` chains
- `read_file` workspace-bound; `approve()` fail-closed without ask
- Redact secrets on JSONL lessons/failures
- Checkpoint stash stored as commit SHA; ambiguous id rejected
- QA workflow installs binary and can fail honestly
- Mission id short-slice bounds; live_smoke unsets console API key

### Docs

- Prime-time README (full CLI map, Grok 4.5, docs index)
- Expand platform docs; stress harness documentation
- Changelog catch-up for 0.4.x / 0.5.0
- Architecture diagram matches crate graph; SECURITY.md permission matrix
- Full audit + hardening pass (multi-agent)

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
