Nightly Dream skill for Aegis:
- Read readiness, git status, and Project Memory first.
- Fill memory_stack, memory_commands, memory_gotchas, memory_conventions with durable facts only (no secrets).
- Extract new_lessons with kind/summary/detail/confidence; prefer process, readiness, architecture, and failure lessons.
- Propose small high-value improvements (docs, tests, skills, readiness) with priority 1–3; avoid large refactors.
- Emit skill_updates only when a reusable procedure should be remembered.
- Always return the required JSON schema fields; leave sections null only if truly unknown.
- After proposing install-code-review or commits, note them as next human/agent actions rather than performing unsafe side effects.