# Project learning & self-improvement

Aegis gets better at **each repository** every time it runs (unless `--no-learn`).

## Layout

```
.aegis/
  MEMORY.md         # stack, commands, gotchas, conventions
  LESSONS.jsonl     # durable lessons
  FAILURES.jsonl    # error fingerprints → fixes
  SKILLS/           # playbooks
  runs/             # per-run summaries (often gitignored)
  missions/         # Factory-style mission state
  dreams/           # dream journals
  qa/ reviews/      # platform artifacts
  metrics.json      # heal / usage-ish counters
  config.toml       # optional per-project overrides
```

## Loop

1. **Start** — inject top lessons + MEMORY into context  
2. **Self-heal** — on tool errors, apply known fix or diagnose (budgeted)  
3. **Reflect** — end of turn/session, Grok extracts lessons into `.aegis/`  

## CLI

```bash
aegis memory show
aegis memory dump
aegis memory clear
aegis --no-learn -p "..."   # disable learning for this run
```

## Tools

- `memory_read` / `memory_write` — agent can record lessons mid-run

## Redaction

Common secret patterns are redacted before write. Still avoid pasting tokens into prompts. See [SECURITY.md](../SECURITY.md).
