---
seam_id: SEAM-4
seam_slug: claude-code-mapping
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-4-claude-code-mapping.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-2-closeout.md
    - ../../governance/seam-3-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - Claude argv ordering contract changes
    - new universal keys touch fallback-model semantics
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
# SEAM-4 - Claude Code backend mapping (Activated)

## Seam brief (source of truth)

- See `../../seam-4-claude-code-mapping.md`.

## Promotion basis

- Upstream seam exit: `../../governance/seam-3-closeout.md` (seam-exit gate passed; promotion readiness ready).
- Required threads: `THR-01`, `THR-02` are published per `../../threading.md`.

## Next planning step

- Execute `slice-*.md` sequentially (S1..S4), then complete the dedicated `seam-exit-gate` slice.
- `S1` → `slice-1-model-handoff.md`: adopt SEAM-2's normalized model helper in Claude request/build wiring and emit exactly one `--model <trimmed-id>` only when the typed handoff is `Some`.
- `S2` → `slice-2-print-session-argv-conformance.md`: pin argv ordering across print/session flows and prove this universal key never maps to `--fallback-model`.
- `S3` → `slice-3-runtime-rejection-conformance.md`: harden runtime rejection/error-event translation and update the Claude contract/test surfaces that SEAM-5 will rely on.

## Threading Alignment (mandatory)

- **Contracts produced (owned)**:
  - `MS-C07`: Claude mapping contract. Claude Code consumes the effective trimmed model id and emits exactly one `--model <trimmed-id>` through the existing print request / argv path, before any `--add-dir` group, session-selector flags, or `--fallback-model`; this key carries no fallback-model or secondary override semantics.
    - Canonical locations: `threading.md`, `docs/specs/claude-code-session-mapping-contract.md`
    - Produced by: `S1` (typed handoff adoption), `S2` (print/session mapping + fallback exclusion), `S3` (runtime/error conformance + contract publication)
- **Contracts consumed**:
  - `MS-C02`: absence semantics owned by SEAM-1.
    - Consumed by: `S1.T1` and `S2.T1` so missing `agent_api.config.model.v1` never synthesizes `.model(...)` or `--model`.
  - `MS-C04`: backend-owned runtime rejection contract owned by SEAM-1.
    - Consumed by: `S3.T1` and `S3.T2` to translate runtime model rejection into safe backend errors and one terminal `Error` event when applicable.
  - `MS-C05`: built-in advertising contract owned by SEAM-2.
    - Consumed by: `S1`/`S2` as a reachability assumption only; Claude mapping is valid only once every exposed print/session flow is deterministic after SEAM-2 lands.
  - `MS-C09`: shared model-normalizer handoff owned by SEAM-2.
    - Consumed by: `S1.T1` and `S2.T1`; SEAM-4 must consume only the typed `Option<String>` output and must not re-parse raw extensions.
- **Dependency edges honored**:
  - `SEAM-1 gates SEAM-4`: this plan assumes the canonical semantics in `docs/specs/unified-agent-api/extensions-spec.md` are already pinned and only implements Claude-side conformance.
  - `SEAM-2 blocks SEAM-4`: `S1` and `S2` explicitly depend on the shared helper output from `crates/agent_api/src/backend_harness/normalize.rs`; no task in this seam adds a second parser.
  - `SEAM-4 blocks SEAM-5B`: `S1`/`S2`/`S3` together provide the final Claude mapping and safe runtime-error behavior that SEAM-5B must assert.
- **Parallelization notes**:
  - What can proceed now: draft spec/test cases and fake-Claude scenario hooks in `S3`; prep localized Claude harness/module changes once SEAM-2's helper signature is stable.
  - What must wait: landing `S1`/`S2` requires SEAM-2's shared helper; SEAM-5B must wait for all three slices because the tests pin both mapping and backend-error behavior.
