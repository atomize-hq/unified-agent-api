---
seam_id: SEAM-2
seam_slug: backend-advertising-normalization
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-backend-advertising-normalization.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - shared helper signature or validation rules change after downstream mapping starts
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S4
  status: passed
open_remediations: []
---
# SEAM-2 - Backend advertising + normalization hook

## Seam Brief (Restated)

- **Goal / value**: ensure both built-in backends expose `agent_api.config.model.v1` consistently while enforcing a single raw-parse site and a single typed handoff (`Option<String>`) for the effective trimmed model id.
- **Type**: integration
- **Scope**
  - In:
    - implement one shared model-selection normalization helper in `crates/agent_api/src/backend_harness/normalize.rs`
    - keep R0 allowlist gating ahead of model parsing and preserve the pinned safe InvalidRequest message
    - carry the normalized `Option<String>` through `NormalizedRequest` so backend mapping seams consume typed output only
    - regenerate `docs/specs/unified-agent-api/capability-matrix.md` in the same change that flips built-in advertising
    - couple built-in backend `supported_extension_keys()` and `capabilities()` posture for `agent_api.config.model.v1`
  - Out:
    - backend-specific argv insertion details (SEAM-3 / SEAM-4)
    - runtime rejection translation details (SEAM-3 / SEAM-4)
- **Touch surface**:
  - `crates/agent_api/src/backend_harness/normalize.rs`
  - `crates/agent_api/src/backends/codex/backend.rs`
  - `crates/agent_api/src/backends/claude_code/backend.rs`
  - `crates/agent_api/src/backends/claude_code/mod.rs`
  - `crates/agent_api/src/backends/claude_code/harness.rs`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- **Verification**
  - one raw-parse site only (repo search for `agent_api.config.model.v1`)
  - harness unit tests prove absent / non-string / empty-after-trim / oversize-after-trim / trimmed-success cases and preserve the exact safe message `invalid agent_api.config.model.v1`
  - backend capability tests prove `supported_extension_keys()` and `capabilities()` do not drift for the model key
  - `cargo run -p xtask -- capability-matrix` updates the generated matrix in the same change as the advertising flip
- **Threading constraints**
  - Upstream blockers: `THR-01` (SEAM-1 gate record published)
  - Downstream blocked seams: `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Contracts produced: `C-05`, `C-08`, `C-09`
  - Contracts consumed: `C-01`, `C-03`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`
- Revalidation: SEAM-1's gate record is now published with stable commit references (THR-01); `gates.pre_exec.revalidation` has passed.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: SEAM-2 is the handoff point where "spec text" becomes "runnable typed contract + truthful capability publication", which downstream seams and promotion should be able to trust without re-deriving.
- **Expected contracts to publish**: `C-09`, `C-05`, `C-08`
- **Expected threads to publish / advance**: `THR-02`, `THR-03`
- **Likely downstream stale triggers**:
  - helper signature changes after SEAM-3/4 starts
  - advertising flip without matrix regeneration
- **Expected closeout evidence**:
  - links to merged diff (or PR) that flips advertising and adds the shared helper tests
  - recorded `rg` output showing no extra parse sites
  - recorded `xtask capability-matrix` output committed

## Slice index

- `S1` -> `slice-1-shared-model-normalizer.md`
- `S2` -> `slice-2-backend-exposure-gates.md`
- `S3` -> `slice-3-capability-publication-and-conformance.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Threading alignment (mandatory)

- **Contracts produced (owned)**:
  - `MS-C09 — Shared model-normalizer handoff`: produced by S1 in `crates/agent_api/src/backend_harness/normalize.rs` and `crates/agent_api/src/backend_harness/contract.rs`, yielding one typed `Option<String>` handoff on `NormalizedRequest`.
  - `MS-C05 — Built-in advertising contract`: wired by S2 across `supported_extension_keys()` and `capabilities()` so built-in backends expose `agent_api.config.model.v1` only when the flow set has one deterministic v1 outcome.
  - `MS-C08 — Capability-matrix publication handoff`: completed by S3 via `docs/specs/unified-agent-api/capability-matrix.md` regeneration in the same change as the advertising flip.
- **Contracts consumed**:
  - `MS-C01`: S1 uses the canonical key id and trim-first semantics.
  - `MS-C02`: S1 preserves `Ok(None)` absence behavior so downstream mapping omits `--model`.
  - `MS-C03`: S1 owns the pinned safe InvalidRequest behavior and byte bound.
  - `MS-C06` / `MS-C07`: S3 consumes the already-landed downstream mapping posture as readiness evidence for the final advertising flip; ownership remains with SEAM-3 / SEAM-4.
- **Dependency edges honored**:
  - `SEAM-1 blocks SEAM-2`: all slices assume the owner-spec semantics and safe InvalidRequest posture are already pinned.
  - `SEAM-2 blocks SEAM-3`: S1 must land before Codex mapping can consume a typed normalized model id instead of raw request parsing.
  - `SEAM-2 blocks SEAM-4`: S1 must land before Claude mapping can consume a typed normalized model id instead of raw request parsing.
  - `SEAM-2 blocks SEAM-5B`: S2/S3 provide the truthful capability posture and published matrix that backend/runtime regression tests must assert.
- **Parallelization notes**:
  - What can proceed now:
    - S1 can start immediately after SEAM-1 verification closes.
    - S2 can be prepared in parallel with late S1 review, as long as it does not introduce a second parser or split allowlist and advertising decisions.
  - What must wait:
    - The final advertising flip and matrix publication in S3 must wait for the integration change that already carries the deterministic mapping outcomes from SEAM-3 / SEAM-4.
    - SEAM-5B should wait for S2/S3 so its assertions target the final capability posture rather than an intermediate state.

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
