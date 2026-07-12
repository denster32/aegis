# Automated QA

Install QA skills and run a structured QA pass against the current project (or a base branch).

```bash
aegis install-qa
aegis qa
aegis qa --base main
```

## Outputs

- Skills: `.aegis/skills/qa/` (and related)
- Reports: `.aegis/qa/reports/<timestamp>.md`
- Optional evidence under `.aegis/qa/evidence/` (gitignored patterns may apply)

## CI

Template workflow: [`.github/workflows/aegis-qa.yml`](../.github/workflows/aegis-qa.yml).

QA findings can feed project memory / lessons on subsequent agent runs.

Related: [review.md](./review.md), [learning.md](./learning.md).
