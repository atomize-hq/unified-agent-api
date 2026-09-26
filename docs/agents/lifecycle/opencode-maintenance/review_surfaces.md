<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Review surfaces

Some paths below contain guarded lifecycle command names. An agent executing this packet inside a relay session must not invoke those commands.

## Writable surfaces

- `docs/agents/lifecycle/opencode-maintenance/**`
- `crates/opencode/**`
- `crates/agent_api/**`
- `cli_manifests/opencode/artifacts.lock.json`
- `cli_manifests/opencode/snapshots/1.18.31/**`
- `cli_manifests/opencode/reports/1.18.31/**`
- `cli_manifests/opencode/versions/1.18.31.json`
- `cli_manifests/opencode/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `crates/agent_api/src/runtime_support_data.rs`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`

## Read-only inputs

- `docs/agents/lifecycle/opencode-maintenance/OPS_PLAYBOOK.md`
- `docs/agents/lifecycle/opencode-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md`
- `.github/workflows/agent-maintenance-open-pr.yml`

## Support debt baseline

- `docs/specs/unified-agent-api/non-tui-support-debt.md`
