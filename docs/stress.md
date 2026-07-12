# Stress testing Aegis

Live multi-phase harness against Grok OAuth.

```bash
./scripts/stress_test.sh
# shorter (skip S15–S20):
STRESS_LONG=0 ./scripts/stress_test.sh
```

Requires `aegis auth status` OK. Unsets `XAI_API_KEY` so a spent console key cannot override subscription OAuth.

## Phases

| ID | Scenario | Severity if fail |
|----|----------|------------------|
| S0 | CLI / auth / readiness / factory / memory / automation | P0 |
| S1 | Cold create Rust crate + tests | P1 |
| S2 | Multi-step edit | P1 |
| S3 | Induced compile failure + self-heal | **P0** |
| S4 | Multi-file writes | P2 |
| S5 | Swarm mission | P2 |
| S6 | Dream | P2 |
| S7 | Wiki generate | P2 |
| S8 | QA | P2 |
| S9 | Review `--diff` | P2 |
| S10 | Concurrent agents (2) | **P0** |
| S11 | Grep/edit fidelity (`sub`) | P1 |
| S12 | Short turn / no hang | P1 |
| S13 | Learning artifacts under `.aegis/` | P2 |
| S14 | Checkpoint create | P3 |
| S15 | Multi-module expansion (`math.rs`) | P1 *(long)* |
| S16 | Structured plan | P2 *(long)* |
| S17 | Missions product new/status | P2 *(long)* |
| S18 | Second induced heal | **P0** *(long)* |
| S19 | Triple concurrent writes | **P0** *(long)* |
| S20 | Final `cargo test` green | P1 *(long)* |

Results: [stress-report.md](./stress-report.md).

## Notes

- Wall clock is mostly **API-bound** (often 10–25+ minutes for LONG=1).
- Soft passes: mission may leave files without perfect exit codes; review may write artifacts only.
- Always reinstall before a release stress: `./install.sh`
