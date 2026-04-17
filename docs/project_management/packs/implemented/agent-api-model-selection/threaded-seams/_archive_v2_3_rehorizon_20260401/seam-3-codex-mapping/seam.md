---
seam_id: SEAM-3
seam_slug: codex-mapping
status: decomposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_seam_brief: ../../seam-3-codex-mapping.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - Codex builder/argv ordering contract changes
    - Codex fork transport gains model selection support
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S4
  status: pending
open_remediations: []
---
# SEAM-3 - Codex backend mapping

## Seam Brief (Restated)

- **Goal / value**: map `agent_api.config.model.v1` into Codex exec/resume `--model <trimmed-id>` behavior through the existing builder/argv path while preserving safe fork rejection and safe runtime rejection translation.
- **Type**: capability
- **Scope**
  - In:
    - consume the typed model selection output from SEAM-2 (C-09)
    - exec/resume `--model <trimmed-id>` mapping and ordering
    - fork pre-handle safe rejection when model override is requested
    - safe runtime rejection translation (completion + terminal Error event parity)
  - Out:
    - capability advertising ownership (SEAM-2)
    - any model catalog validation
- **Touch surface**:
  - `crates/agent_api/src/backends/codex/{backend.rs,harness.rs,exec.rs,fork.rs}`
  - `crates/codex/src/builder/mod.rs`
  - Codex spec docs under `docs/specs/`
- **Verification**:
  - exactly-one `--model` emission on exec/resume
  - no `--model` when absent
  - fork pre-handle rejection with pinned message
  - runtime rejection scenario coverage (stream-open parity)
- **Basis posture**:
  - Currentness: provisional (blocked on `THR-02` publishing)
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`, `THR-02`
  - Stale triggers: Codex ordering contract changes; fork transport changes
- **Threading constraints**
  - Upstream blockers: `THR-01`, `THR-02`
  - Downstream blocked seams: `SEAM-5`
  - Contracts produced: `C-06`
  - Contracts consumed: `C-02`, `C-04`, `C-09`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Expected contracts to publish**: `C-06` (and any touched Codex spec updates)
- **Expected threads to publish / advance**: `THR-04`
- **Likely downstream stale triggers**:
  - fork transport gains model support (fork rejection contract changes)
  - ordering contracts change in Codex CLI builder
- **Expected closeout evidence**:
  - links to mapping PR/commit
  - links to tests proving fork rejection and runtime rejection parity

## Slice index

- `S1` -> `slice-1-exec-resume-model-handoff.md`
- `S2` -> `slice-2-fork-model-rejection.md`
- `S3` -> `slice-3-runtime-rejection-conformance.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`

