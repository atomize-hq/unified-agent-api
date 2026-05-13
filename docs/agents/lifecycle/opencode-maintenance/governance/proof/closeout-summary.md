# Closeout Summary

- Request: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- Request sha256: `f8fa17dc42ca05bf3ec09e7f01423240234db0fdf2553a45e39b98b90c71f570`
- Final run id: `20260512T235319Z`
- Closeout artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`
- Closeout command: `cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json`
- Command result: succeeded
- Recorded at: `2026-05-13T00:20:45Z`
- Commit captured in closeout: `dc809949`

## Resolved Findings

- `registry_manifest_drift`: refreshed `cli_manifests/opencode/artifacts.lock.json`, the `1.14.47` snapshots, the `1.14.47` reports, and `cli_manifests/opencode/versions/1.14.47.json`
- `support_publication_drift`: refreshed `cli_manifests/support_matrix/current.json` and `docs/specs/unified-agent-api/support-matrix.md`

## Deferred Findings

- None. `check-agent-drift --agent opencode` is clean on the proof-bearing tree.
