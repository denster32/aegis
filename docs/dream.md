# Nightly Dream

Deep offline self-improve for a project: consolidate memory, skills, wiki-oriented notes, and failure patterns.

## Usage

```bash
aegis dream                 # dry-ish journal
aegis dream --apply         # write consolidations into .aegis/
aegis dream install         # cron + automation entry (typically ~3am)
```

## Outputs

| Path | Content |
|------|---------|
| `.aegis/dreams/<timestamp>.md` | Human-readable journal |
| `.aegis/dreams/<timestamp>.json` | Structured summary |
| `.aegis/MEMORY.md` / lessons | Updated when `--apply` |

## When to use

- After a long coding day (batch reflect)
- Before a big Mission (refresh project knowledge)
- On a schedule via `aegis automation` / cron

Related: [learning.md](./learning.md), [automations.md](./automations.md).
