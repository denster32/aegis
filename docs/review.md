# Code Review

Agent-assisted review of a GitHub PR or the local uncommitted / branch diff.

```bash
aegis review --diff
aegis review --diff --depth shallow
aegis review --pr 12 --depth deep
aegis install-code-review     # install GH workflow helper
```

## Depth

| Depth | Use |
|-------|-----|
| `shallow` | Fast pass, high-signal only |
| `deep` | Broader review (more tokens / latency) |

## Outputs

Reports under `.aegis/reviews/` when the command writes artifacts.

Requires Grok OAuth (or API key). For `--pr`, GitHub CLI / token access may be needed depending on environment.

Related: [qa.md](./qa.md), [SECURITY.md](../SECURITY.md).
