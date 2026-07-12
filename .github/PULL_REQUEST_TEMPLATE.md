## Summary

<!-- What and why (1–3 bullets). -->

## Risk

- [ ] Low (docs / tests / pure UI)
- [ ] Medium (agent / tools / CLI behavior)
- [ ] High (auth, sandbox, permissions, network)

## Test plan

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test --workspace --locked`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] (optional) `./scripts/live_smoke.sh`
- [ ] (optional) `STRESS_LONG=1 ./scripts/stress_test.sh` for agent-path changes

## Notes

<!-- Breaking changes, migration, follow-ups. -->
