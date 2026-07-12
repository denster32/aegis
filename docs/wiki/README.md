# Wiki source (synced to GitHub Wiki)

These Markdown pages are the **source of truth** for the GitHub Wiki at
https://github.com/denster32/aegis/wiki

## Publish

```bash
./scripts/publish_wiki.sh
```

Or run the **Publish Wiki** GitHub Action (`workflow_dispatch` / push to `docs/wiki/**`).

### One-time bootstrap

GitHub does not create `*.wiki.git` until the first page exists **or** a successful
first push with a token that can write wikis.

1. Preferred: open https://github.com/denster32/aegis/wiki while signed in as a
   collaborator, click **Create the first page**, save a stub Home, then re-run
   `./scripts/publish_wiki.sh`.
2. Or set a **classic** PAT (`repo` scope) as `WIKI_TOKEN` (Actions secret) /
   `GH_TOKEN` env — fine-grained PATs often cannot access wiki git.

Pages: Home, Commands, Architecture, Modules, Nexus, Conventions, `_Sidebar`.

Current version labels in Home / Sidebar: **0.8.0**.
