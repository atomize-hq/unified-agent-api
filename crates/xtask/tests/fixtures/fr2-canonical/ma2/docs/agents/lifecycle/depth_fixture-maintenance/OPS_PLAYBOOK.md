<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Ops playbook

This packet-owned playbook freezes operator context for `depth_fixture` target `0.2.0`.

The recovery packet-regeneration command is a maintainer action run from outside a relay session. An agent executing this packet inside a relay session must not run it.

- request artifact: `docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml`
- basis ref: `cli_manifests/depth_fixture/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- branch linkage: `automation/depth_fixture-maintenance-0.2.0`
- canonical handoff: `docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md`
- recovery packet regeneration: `cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml --write`
