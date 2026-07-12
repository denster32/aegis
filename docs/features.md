# Feature matrix

Verified against CLI + crates as of **0.5.1**.

## Agent

| Feature | Status | Entry |
|---------|--------|--------|
| One-shot / REPL agent | ✅ | `aegis` · `-p` |
| Grok OAuth (`~/.grok/auth.json`) | ✅ | `aegis auth status` · `login` |
| Responses API + tools | ✅ | read/write/edit, bash, glob, grep, git, web, memory, vision |
| `reasoning.effort` (grok-4*) | ✅ | `--effort low\|medium\|high` |
| `prompt_cache_key` | ✅ | per session |
| Server tools (web_search, code_execution, x_search) | ✅ | config / defaults |
| Parallel tool calls + path locks | ✅ | agent loop |
| Self-heal + reflect | ✅ | project learning (disable: `--no-learn`) |
| Structured plan | ✅ | `aegis plan` |
| Swarm DAG mission | ✅ | `aegis mission` (workers omit reasoning on code-fast) |
| Factory Missions | ✅ | `aegis missions new\|list\|status\|run` |
| Sessions + cost | ✅ | `aegis session` · REPL `/cost` |
| Checkpoints | ✅ | `aegis checkpoint` |
| Vision | ✅ | `aegis vision` |
| MCP tools (optional) | ✅ | `.aegis/config.toml` mcp_servers |

## Platform

| Feature | Status | Entry |
|---------|--------|--------|
| Readiness L1–L5 | ✅ | `aegis readiness` |
| Software Factory map | ✅ | `aegis factory` |
| Nightly dream | ✅ | `aegis dream` · `dream install` |
| Wiki | ✅ | `aegis wiki generate\|refresh` |
| QA | ✅ | `aegis install-qa` · `aegis qa` |
| Code review | ✅ | `aegis review --diff\|--pr` |
| Automations | ✅ | `aegis automation *` |
| Install helpers | ✅ | `install-code-review` · `install-wiki-refresh` |

## UI language

Monochrome SpaceX / xAI terminal chrome — `aegis_core::ui`:

- White primary · dim secondary · red only for errors
- Uppercase section headers + thin rules
- Geometric marks: `●` ok · `×` fail · `·` idle · `▸` active
- REPL prompt `›`

## Not productized (future)

- Multi-agent model routing (`grok-4.20-multi-agent`)
- Image/video generation products
- Public CI green on private Actions (runner quota)
