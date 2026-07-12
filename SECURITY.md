# Security

## Reporting

Open a private security advisory or contact the maintainer. Do not file public issues that include tokens.

## Secrets

- Never commit `~/.grok/auth.json`, `~/.local/share/aegis/auth.json`, or API keys.
- Project memory redacts common secret patterns; still avoid pasting tokens into prompts.
- Prefer Grok OAuth over long-lived console keys when possible.

## Tool safety

Default permission mode prompts for shell. Use `--yolo` only in trusted sandboxes.
