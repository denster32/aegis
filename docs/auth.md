# Authentication

Aegis resolves credentials in this order:

1. `AEGIS_ACCESS_TOKEN` / `XAI_ACCESS_TOKEN` (Bearer JWT)
2. `~/.local/share/aegis/auth.json` (Aegis-stored OAuth)
3. `~/.grok/auth.json` (Grok CLI / Grok Build OIDC) — **preferred daily path**
4. `XAI_API_KEY` (console key; may have separate billing)

## Commands

```bash
aegis auth status
aegis login                 # device OAuth when configured
aegis login --import-grok   # copy Grok CLI tokens into Aegis store
aegis logout                # clear Aegis store only (does not wipe ~/.grok)
```

## Tips

- If `grok login` already works, `aegis auth status` should show `source: grok-auth.json`.
- Live scripts (`live_smoke.sh`, `stress_test.sh`) **unset** `XAI_API_KEY` so a spent console key does not override OAuth.
- Never commit `auth.json`, `.env`, or tokens. See [SECURITY.md](../SECURITY.md).
