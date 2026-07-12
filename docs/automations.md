# Automations

File-based scheduled / event automations for a project.

```bash
aegis automation ensure
aegis automation list
aegis automation run nightly-dream
aegis automation install-all
```

## Definitions

```
.aegis/automations/*.toml
```

Common entries:

- `nightly-dream.toml` — deep reflect / consolidate
- `wiki-refresh.toml` — keep wiki close to the tree

Install helpers also wire cron or GitHub workflows where appropriate (`aegis dream install`, `install-wiki-refresh`, etc.).

Related: [dream.md](./dream.md), [wiki.md](./wiki.md).
