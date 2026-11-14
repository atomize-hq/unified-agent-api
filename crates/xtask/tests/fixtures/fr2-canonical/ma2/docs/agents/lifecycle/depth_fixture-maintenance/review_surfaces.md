<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Review surfaces

Some paths below contain guarded lifecycle command names. An agent executing this packet inside a relay session must not invoke those commands.

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

## Read-only inputs

- `docs/agents/lifecycle/depth_fixture-maintenance/OPS_PLAYBOOK.md`
- `docs/agents/lifecycle/depth_fixture-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/depth_fixture-maintenance/governance/execute-agent-maintenance-prompt.md`
- `.github/workflows/agent-maintenance-open-pr.yml`

## Support debt baseline

- `docs/specs/unified-agent-api/non-tui-support-debt.md`
