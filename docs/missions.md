# Missions (Factory-inspired)

Structured multi-feature work: **plan → Mission Control → execute → validate → learn**.

Inspired by [Factory Missions](https://docs.factory.ai/features/missions/overview) (original Aegis design).

## Flow

1. `aegis missions new "goal"` — structured plan (features + milestones + skills)
2. `aegis missions list` / `aegis missions status [id]` — Mission Control board
3. `aegis missions run <id>` — execute features in dependency order, validate milestones
4. Project memory updated via reflection

## Quick swarm (lighter)

```bash
aegis mission --workers 2 "Add README and keep cargo test green"
```

Single-shot DAG swarm without full Mission Control persistence.

## Readiness

```bash
aegis readiness
```

Higher readiness (git, tests/manifest, `.aegis`, CI) improves long missions.

## Storage

```
.aegis/missions/<id>/{plan.json,state.json,progress.jsonl,handoffs/,evidence/}
```

Related: [readiness.md](./readiness.md), [learning.md](./learning.md).
