---
seam_id: SEAM-1
status: proposed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-1-wrapper-crate-and-manifest-foundation/slice-99-seam-exit-gate.md
  status: pending
  promotion_readiness: blocked
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
    landing: pending
    closeout: pending
open_remediations: []
---

# Closeout - SEAM-1 Wrapper crate and manifest foundation

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-1-wrapper-crate-and-manifest-foundation/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - wrapper-owned runtime surfaces under `crates/opencode/**`
  - deterministic wrapper proof under `crates/opencode/tests/**` and
    `crates/opencode/src/bin/fake_opencode_run_json.rs`
  - manifest-root evidence under `cli_manifests/opencode/**`
  - mechanical manifest validation via `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
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
  - pending final seam-exit review
- **Downstream stale triggers raised**:
  - canonical run-surface or accepted-control drift
  - wrapper parser, event, completion, or redaction drift
  - manifest-root inventory, pointer/update-rule, wrapper-coverage, or validator drift
  - deterministic proof posture weakening back toward live-smoke dependence
- **Remediation disposition**:
  - none
- **Promotion blockers**:
  - `SEAM-2` backend implementation and `SEAM-3` support publication remain unfinished
- **Promotion readiness**: blocked

## Post-exec gate disposition

- **Landing gate**: pending
- **Closeout gate**: pending
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
