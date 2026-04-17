# C1 Spec - `crates/opencode/` + `cli_manifests/opencode/` Planning

Source docs:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- `docs/project_management/next/opencode-cli-onboarding/C0-spec.md`

## Decisions (no ambiguity)

- C1 starts only after `C0-integ` locks the canonical runtime surface.
- C1 is still planning-only. It must not edit `crates/opencode/` or `cli_manifests/opencode/`.
- C1 owns two downstream targets:
  - `crates/opencode/`
  - `cli_manifests/opencode/`
- C1 must produce a bounded implementation plan for:
  - spawn/request surface
  - typed events / completion surface
  - offline parsing or fixture-backed strategy
  - fake-binary or maintainer-smoke strategy
  - manifest-root artifact inventory and update rules
- C1 must not plan `crates/agent_api/` beyond the exact inputs that C2 needs.

## Task Breakdown (no ambiguity)

- `C1-code`:
  - define the wrapper crate contract and manifest-root artifact plan based on the C0 runtime lock
- `C1-test`:
  - define the fixture, fake-binary, snapshot, and maintainer validation obligations for the wrapper
    and manifest root
- `C1-integ`:
  - reconcile the wrapper scope and validation scope into one execution-ready C1 packet for
    downstream implementation

## Scope

- wrapper crate boundaries for `crates/opencode/`
- manifest-root expectations for `cli_manifests/opencode/`
- platform/auth posture and reproducibility constraints
- separation between wrapper-owned behavior and future `agent_api` adapter work

## Acceptance Criteria

- the pack defines the intended `crates/opencode/` surface in terms of spawn, stream, completion,
  parsing, and redaction boundaries
- the pack defines the intended `cli_manifests/opencode/` artifact inventory and ownership model
- the pack names how automated fixtures and maintainer smoke complement each other
- C2 receives explicit adapter inputs without reopening C0 runtime selection

## Out of Scope

- editing downstream wrapper or manifest files
- defining `agent_api` event mappings or capability promotion
- changing the runtime surface chosen in C0 unless C0 is formally reopened
