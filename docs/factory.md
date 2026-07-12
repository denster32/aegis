# Software Factory

Local SDLC **coverage map** — what stages this repo is set up for (not a remote Factory product).

```bash
aegis factory
aegis factory --json
```

## Stages (typical)

| Stage | Signals |
|-------|---------|
| Triage | Issues, readiness, memory |
| Code-gen | Agent tools, Missions, skills |
| Validate | Tests, QA, CI |
| Review | `aegis review`, PR workflow |
| Document | Wiki generate/refresh |
| Release | Changelog, tags, install |
| Monitor | Metrics, automations, dream |

Artifacts: `.aegis/factory/status.json`.

Related: [readiness.md](./readiness.md), [missions.md](./missions.md).
