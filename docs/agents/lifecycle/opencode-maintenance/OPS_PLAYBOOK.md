<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Ops playbook

This packet-owned playbook freezes operator context for `opencode` target `1.18.31`.

The recovery packet-regeneration command is a maintainer action run from outside a relay session. An agent executing this packet inside a relay session must not run it.

- request artifact: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- basis ref: `cli_manifests/opencode/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- branch linkage: `automation/opencode-maintenance-1.18.31`
- canonical handoff: `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- recovery packet regeneration: `cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --write`
