<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Scope brief

This automated maintenance lane is limited to the frozen shared packet for `depth_fixture` and the declared writable surfaces below.

## Support uplift rule

Newly discovered non-TUI surface must land in this run unless the frozen packet records one allowed deferral. Preexisting non-TUI gaps remain valid only when the packet ties them to the committed debt inventory.

## Writable surfaces

- `docs/agents/lifecycle/depth_fixture-maintenance/**`
- `crates/depth_fixture/**`
- `crates/agent_api/**`
- `cli_manifests/depth_fixture/artifacts.lock.json`
- `cli_manifests/depth_fixture/snapshots/0.2.0/**`
- `cli_manifests/depth_fixture/reports/0.2.0/**`
- `cli_manifests/depth_fixture/versions/0.2.0.json`
- `cli_manifests/depth_fixture/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `crates/agent_api/src/runtime_support_data.rs`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`
