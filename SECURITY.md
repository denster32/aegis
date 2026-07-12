# Security

## Reporting

Open a private security advisory or contact the maintainer. Do not file public issues that include tokens.

## Secrets

- Never commit `~/.grok/auth.json`, `~/.local/share/aegis/auth.json`, or API keys.
- Prefer Grok OAuth over long-lived console keys when possible.
- Project memory **redacts common patterns** in `MEMORY.md`, skills, and JSONL lessons/failures (Bearer, `xai-…`, GitHub tokens, JWT-shaped strings, key=value secrets). Still avoid pasting tokens into prompts.

## Credential order

1. `AEGIS_ACCESS_TOKEN` / `XAI_ACCESS_TOKEN`
2. `~/.local/share/aegis/auth.json`
3. `~/.grok/auth.json`
4. `XAI_API_KEY` / `SPACEXAI_API_KEY`

## Tool safety

| Mode | When | Behavior |
|------|------|----------|
| **Prompt** | Interactive REPL (default) | Shell and outside-cwd FS need approval; fail-closed if no TTY ask |
| **Yolo** | `--yolo`, and auto for `-p` / `plan` / `mission` / `missions *` | All tools auto-approved — **trusted workspaces only** |
| **Sandbox (Deny)** | `--sandbox` (global; also `sandbox = true` in config) | Shell **fully denied**; FS **workspace-only** with **no** outside-cwd approval escape; overrides YOLO / auto-yolo |

### `--sandbox`

Multi-tenant-style hardening for shared or untrusted agent runs:

- Sets `PermissionMode::Deny` for the agent and mission workers.
- **`bash` is always denied** (no allowlist) — use read/write/edit/grep/git tools only.
- **`read_file` / `write_file` / `edit_file`** must stay under `--cwd` (canonicalized). Outside paths are hard-denied even if `--yolo` or `-p` auto-yolo would otherwise approve.
- **`--sandbox` wins over `--yolo` and auto-yolo** for `-p` / plan / mission.
- REPL `/yolo` is blocked while sandbox is active.
- `web_fetch` blocks localhost, link-local/metadata hosts, private/CGNAT/doc IPv4, unique-local/link-local IPv6, and IPv4-mapped private addresses (literal hosts only — not a full DNS rebinding shield).

Example:

```bash
aegis --sandbox --cwd /tmp/tenant-a -p "Summarize README.md"
```

Still not a full multi-tenant OS isolation layer (no seccomp/containers). Pair with OS-level isolation for hostile tenants.

### Without sandbox

- `read_file` / `write_file` / `edit_file` are **workspace-bound** unless approved (or YOLO).
- `bash` can still run arbitrary commands under YOLO — treat as full process privileges.
- Checkpoints store a stash SHA when possible; restore is **best-effort stash re-apply**, not a hard reset.

## What Aegis is not

Without `--sandbox`, Aegis is not multi-tenant isolation. Run only on machines and trees you trust, with prompts you trust. With `--sandbox`, shell and FS escape hatches are closed at the tool layer, but network and process isolation still depend on the host.
