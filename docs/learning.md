# Project learning & self-improvement

Aegis gets better at **each repository** every time it runs.

## Layout

```
.aegis/
  MEMORY.md        # stack, commands, gotchas, conventions
  LESSONS.jsonl    # durable lessons
  FAILURES.jsonl   # error fingerprints → fixes
  SKILLS/          # playbooks
  runs/            # per-run summaries
  missions/        # Factory-style mission state
  metrics.json
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
aegis --no-learn -p "..."   # disable learning
```

## Tools

- `memory_read` / `memory_write` — agent can record lessons mid-run
