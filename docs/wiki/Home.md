# Aegis Wiki

**Aegis** is a sovereign Grok-native coding agent written in Rust.

## Start here

- [Commands](Commands.md) — full CLI map
- [Architecture](Architecture.md) — crate layout
- [Modules](Modules.md) — module notes
- [Conventions](Conventions.md) — project norms

## External docs (repo)

| Topic | Link |
|-------|------|
| Auth | [../auth.md](../auth.md) |
| Grok 4.5 | [../xai-capabilities.md](../xai-capabilities.md) |
| Learning | [../learning.md](../learning.md) |
| Missions | [../missions.md](../missions.md) |
| Stress | [../stress.md](../stress.md) |
| Changelog | [../../CHANGELOG.md](../../CHANGELOG.md) |

## Quick start

```bash
./install.sh
grok login          # or: aegis login
aegis auth status
aegis -p "Reply with exactly: pong"
```

Version target: **0.7.0**.
