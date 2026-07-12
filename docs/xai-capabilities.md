# Grok 4.5 / xAI utilization in Aegis

See also: [Grok 4.5 docs](https://docs.x.ai/developers/grok-4-5)

## What we use (via OAuth or API key)

| Feature | How Aegis uses it |
|---------|-------------------|
| **OAuth JWT** | `~/.grok/auth.json` + refresh → Bearer on `api.x.ai` |
| **Responses API** | Primary agent transport |
| **`grok-4.5`** | Default model |
| **Function calling** | Local coding tools (read/write/bash/…) |
| **Parallel tools** | `parallel_tool_calls: true` |
| **`previous_response_id`** | Multi-turn tool loops |
| **`reasoning.effort`** | low / medium / high from `--effort` + tool-step policy |
| **`prompt_cache_key`** | Session id (sticky cache routing) |
| **Server tools** | Optional `web_search`, `x_search`, `code_execution` |
| **Structured JSON** | Plans, missions, dream, review |
| **Vision** | `vision_describe` + image inputs |

## `--effort` mapping (now correct)

| Flag | Model | Default reasoning | Tool-step reasoning |
|------|-------|-------------------|---------------------|
| high | grok-4.5 | high | medium |
| medium | grok-4.5 | medium | **low** |
| low | grok-4.5 | **low** | low |

xAI docs: tool-heavy agent loops should prefer **low** reasoning for latency; planning stays higher.

## Config (`.aegis/config.toml`)

```toml
reasoning_effort = "high"
tool_reasoning_effort = "low"
web_search = true
x_search = false
code_execution = true
```

## Still optional / future

- API-native context compaction (local compact remains)
- Multi-agent model (`grok-4.20-multi-agent`)
- Image/video generation products
