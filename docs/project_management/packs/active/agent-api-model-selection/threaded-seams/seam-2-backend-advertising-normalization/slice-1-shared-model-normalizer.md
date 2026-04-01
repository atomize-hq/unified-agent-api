---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
contracts_produced:
  - C-09
contracts_consumed:
  - C-01
  - C-03
open_remediations: []
candidate_subslices: []
---
### S1 - Shared model normalizer + typed handoff

- **User/system value**: creates exactly one raw-parse site and one typed, backend-neutral handoff (`Option<String>`) for model selection so downstream seams cannot drift or duplicate parsing.
- **Scope (in/out)**:
  - In:
    - shared helper parses `agent_api.config.model.v1` after R0 gating only
    - trim-before-validate; bounds `1..=128` (UTF-8 bytes) after trim
    - absence returns `Ok(None)`
    - invalid inputs return `Err(InvalidRequest { message: "invalid agent_api.config.model.v1" })` without echoing raw model ids
  - Out:
    - backend-specific argv ordering and mapping
- **Acceptance criteria**:
  - repo search shows only one raw-parse site
  - unit tests cover absent, non-string, whitespace-only, oversize-after-trim, and trimmed-success cases
  - typed output is plumbed to backend mapping seams without re-parsing
- **Dependencies**: `THR-01` (SEAM-1 canonical contract gate), `C-01`, `C-03`
- **Verification**:
  - `rg -n "agent_api\\.config\\.model\\.v1" crates/agent_api/src` shows the single permitted parse site
  - `cargo test -p agent_api` includes the new unit tests
- **Rollout/safety**: keep helper backend-neutral; do not add remote lookups or model catalogs.
- **Review surface refs**: `../../review_surfaces.md` (R1, R2)

#### S1.T1 - Implement shared helper contract in `normalize.rs`

- **Outcome**: a single function returns `Result<Option<String>, AgentWrapperError>` for the key.
- **Thread/contract refs**: `THR-02`, `C-09`, `C-03`
- **Acceptance criteria**: function matches the contract exactly; invalid message is the safe template; no raw id is echoed.

Checklist:
- Implement: helper + any shared constants
- Test: unit tests for all pinned cases
- Validate: `rg` confirms no new parse sites

#### S1.T2 - Plumb typed output to downstream mapping surfaces

- **Outcome**: backend harness/adapter surfaces expose `Option<String>` so SEAM-3/4 can consume it without touching raw JSON.
- **Thread/contract refs**: `THR-02`, `C-09`
- **Acceptance criteria**: no backend mapping code reads `request.extensions["agent_api.config.model.v1"]`.

Checklist:
- Implement: struct/plumbing changes (typed field)
- Test: compile + existing mapping tests still pass
- Validate: `rg` confirms downstream does not parse raw JSON
