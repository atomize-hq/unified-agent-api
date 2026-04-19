---
seam_id: SEAM-1
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: threaded-seams/seam-1-wrapper-crate-and-manifest-foundation/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - ../../opencode-cli-onboarding/governance/seam-1-closeout.md
    - ../../opencode-cli-onboarding/governance/seam-2-closeout.md
  required_threads:
    - THR-04
    - THR-05
  stale_triggers:
    - OpenCode CLI event-shape drift on the canonical run surface
    - accepted control drift off `opencode run --format json`
    - manifest inventory or deterministic replay posture drift
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-1 Wrapper crate and manifest foundation

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-1-wrapper-crate-and-manifest-foundation/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - `3e2f1ee` `SEAM-1: complete slice-00-workspace-and-manifest-contract-baselines`
  - `af8bcbf` `SEAM-1: complete slice-1-wrapper-crate-runtime-and-fixture-foundation`
  - `dd99c7e` `SEAM-1: complete slice-2-manifest-root-artifacts-and-validator-scope`
  - `4b86656` `SEAM-1: complete slice-3-deterministic-evidence-and-downstream-handoff`
  - `cargo test -p unified-agent-api-opencode`
  - `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
- **Contracts published or changed**:
  - `C-01` wrapper-owned runtime boundary published through landed `crates/opencode/**`
  - `C-02` manifest-root inventory and validator boundary published through landed
    `cli_manifests/opencode/**`
- **Threads published / advanced**:
  - `THR-05` publishes the closeout-backed handoff from `SEAM-1` into `SEAM-2` and `SEAM-3`
- **Review-surface delta**:
  - deterministic fake-binary, fixture, transcript, offline-parser, and root-validation evidence
    are the default proof path for `SEAM-1`
  - live provider-backed smoke remains basis-lock or stale-trigger evidence only
  - helper surfaces remain deferred and fail closed outside the canonical `run --format json`
    wrapper boundary
  - manifest support remains separate from backend support and from UAA unified support
- **Planned-vs-landed delta**:
  - `SEAM-1` landed a bootstrap OpenCode manifest root that preserves promotion pointers as `none`
    while still carrying committed `1.4.11` Linux validation evidence and `1.4.9` darwin-arm64
    snapshot evidence
  - root-validation support landed without widening into generic future-agent validator
    scaffolding
  - deterministic downstream handoff rules were recorded directly in `threading.md` and this
    closeout instead of being left implicit in seam-local planning prose
- **Downstream stale triggers raised**:
  - canonical run-surface or accepted-control drift
  - wrapper parser, event, completion, or redaction drift
  - manifest-root inventory, pointer/update-rule, wrapper-coverage, or validator drift
  - deterministic proof posture weakening back toward live-smoke dependence
- **Remediation disposition**:
  - none
- **Promotion blockers**:
  - none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
