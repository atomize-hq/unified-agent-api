# OpenCode CLI Onboarding (Pre-Implementation Triads)

Pre-implementation planning pack for onboarding `OpenCode` as the first real third CLI agent in
this repo.

Status:
- packet closeout is recorded
- `opencode run --format json` is the frozen presumptive v1 wrapper surface
- this directory is now a repo-standard triad scaffold for C0-C3

Source docs:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/extensions-spec.md`

Start here:
- `docs/project_management/next/opencode-cli-onboarding/plan.md`
- `docs/project_management/next/opencode-cli-onboarding/tasks.json`
- `docs/project_management/next/opencode-cli-onboarding/session_log.md`

Execution order:
- C0: packet-closeout confirmation + fixture strategy
- C1: `crates/opencode/` + `cli_manifests/opencode/` planning
- C2: `crates/agent_api/` planning bounded by the wrapper contract
- C3: UAA promotion review after wrapper/backend scope is concrete

Pack boundaries:
- This pack is docs/planning only.
- Until a downstream execution pack is created, edits stay inside
  `docs/project_management/next/opencode-cli-onboarding/`.
- The charter and `docs/specs/**` remain authoritative for contract semantics.
