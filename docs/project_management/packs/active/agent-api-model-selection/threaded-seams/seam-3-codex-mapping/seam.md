---
seam_id: SEAM-3
seam_slug: codex-mapping
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-codex-mapping.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - Codex builder/argv ordering contract changes
    - Codex fork transport gains model selection support
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
# SEAM-3 - Codex backend mapping (Activated)

## Seam brief (source of truth)

- See `../../seam-3-codex-mapping.md`.

## Promotion basis

- Upstream seam exit: `../../governance/seam-2-closeout.md` (seam-exit gate passed; promotion readiness ready).
- Required threads: `THR-01`, `THR-02` are published per `../../threading.md`.

## Next planning step

- Execute `slice-*.md` sequentially (S1..S4), then complete the dedicated `seam-exit-gate` slice.
- `S1` → `slice-1-exec-resume-model-handoff.md`: adopt SEAM-2's normalized model helper in Codex policy/harness wiring and map exec/resume flows through the existing builder argv path.
- `S2` → `slice-2-fork-model-rejection.md`: pin the no-transport fork rejection behavior so accepted model-selection inputs fail safely before any app-server request.
- `S3` → `slice-3-runtime-rejection-conformance.md`: harden runtime rejection/error-event translation and update the Codex contract/test surfaces that SEAM-5 will rely on.

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `MS-C06`: Codex mapping contract. Exec/resume consume the effective trimmed model id and emit exactly one `--model <trimmed-id>` through the existing builder/argv path; fork flows reject accepted model-selection inputs before any app-server request with the pinned safe backend message; the key carries no extra semantics beyond model selection itself.
    - Canonical locations: `threading.md`, `docs/specs/codex-streaming-exec-contract.md`, `docs/specs/codex-app-server-jsonrpc-contract.md`
    - Produced by: `S1` (exec/resume mapping), `S2` (fork rejection path), `S3` (runtime/error conformance + contract publication)
- **Contracts consumed**:
  - `MS-C02`: absence semantics owned by SEAM-1.
    - Consumed by: `S1.T1` and `S1.T2` so missing `agent_api.config.model.v1` never synthesizes `.model(...)` or `--model`.
  - `MS-C04`: backend-owned runtime rejection contract owned by SEAM-1.
    - Consumed by: `S3.T1` and `S3.T2` to translate runtime model rejection into safe backend errors and one terminal `Error` event when applicable.
  - `MS-C05`: built-in advertising contract owned by SEAM-2.
    - Consumed by: `S1`/`S2` as a reachability assumption only; Codex mapping is valid only once every exposed flow is deterministic after SEAM-2 lands.
  - `MS-C09`: shared model-normalizer handoff owned by SEAM-2.
    - Consumed by: `S1.T1` and `S2.T1`; SEAM-3 must consume only the typed `Option<String>` output and must not re-parse raw extensions.
- **Dependency edges honored**:
  - `SEAM-1 gates SEAM-3`: this plan assumes the canonical semantics in `docs/specs/unified-agent-api/extensions-spec.md` are already pinned and only implements Codex-side conformance.
  - `SEAM-2 blocks SEAM-3`: `S1` and `S2` explicitly depend on the shared helper output from `crates/agent_api/src/backend_harness/normalize.rs`; no task in this seam adds a second parser.
  - `SEAM-3 blocks SEAM-5B`: `S1`/`S2`/`S3` together provide the final Codex mapping and safe runtime-error behavior that SEAM-5B must assert.
- **Parallelization notes**:
  - What can proceed now: draft spec/test cases and stage doc updates in `S3`; prep localized Codex module changes once SEAM-2's helper signature is stable.
  - What must wait: landing `S1`/`S2` requires SEAM-2's shared helper; SEAM-5B must wait for all three slices because the tests pin both mapping and backend-error behavior.
