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
| **Yolo** | `--yolo`, and auto for `-p` / `plan` / `mission` / `missions *` | All tools auto-approved — **trusted sandboxes only** |
| **Deny** | (internal / future) | Block shell and outside writes |

- `read_file` / `write_file` / `edit_file` are **workspace-bound** unless approved (or YOLO).
- `bash` can still run arbitrary commands under YOLO — treat as full process privileges.
- Checkpoints store a stash SHA when possible; restore is **best-effort stash re-apply**, not a hard reset.

## What Aegis is not

Not a multi-tenant sandbox. Run only on machines and trees you trust, with prompts you trust.
