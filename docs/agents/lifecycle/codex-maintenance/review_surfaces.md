<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Review surfaces

Some paths below contain guarded lifecycle command names. An agent executing this packet inside a relay session must not invoke those commands.

## Writable surfaces

- `docs/agents/lifecycle/codex-maintenance/**`
- `crates/codex/**`
- `crates/agent_api/**`
- `cli_manifests/codex/artifacts.lock.json`
- `cli_manifests/codex/snapshots/0.157.0/**`
- `cli_manifests/codex/reports/0.157.0/**`
- `cli_manifests/codex/versions/0.157.0.json`
- `cli_manifests/codex/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `crates/agent_api/src/runtime_support_data.rs`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`
- `docs/specs/codex-wrapper-coverage-scenarios-v1.md`

## Read-only inputs

- `docs/agents/lifecycle/codex-maintenance/OPS_PLAYBOOK.md`
- `docs/agents/lifecycle/codex-maintenance/CI_WORKFLOWS_PLAN.md`
- `docs/agents/lifecycle/codex-maintenance/governance/execute-agent-maintenance-prompt.md`
- `.github/workflows/agent-maintenance-open-pr.yml`

## Support debt baseline

- `docs/specs/unified-agent-api/non-tui-support-debt.md`
