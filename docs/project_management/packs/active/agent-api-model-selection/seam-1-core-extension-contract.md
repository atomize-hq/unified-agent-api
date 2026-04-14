---
seam_id: SEAM-1
seam_slug: core-extension-contract
type: integration
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - canonical owner spec or registry entry changes for agent_api.config.model.v1
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
  planned_location: S3
  status: passed
open_remediations: []
---

# SEAM-1 - Core extension key contract

- **Name**: Core extension key contract
- **Type**: integration
- **Goal / user value**: Pin one stable universal request contract for model selection so callers can rely on a single
  key without wrapper-defined model registries or backend-specific branching.
- **Status**:
  - canonical owner-spec text is already landed in `docs/specs/unified-agent-api/extensions-spec.md`
  - remaining work is limited to ADR-0020 sync, drift verification across related universal specs, and any resulting
    canonical-doc clarification patches
- **Contract registry cross-refs**: MS-C01, MS-C02, MS-C03, MS-C04 (see `threading.md`)
- **Scope**
  - In:
    - normative definition of `agent_api.config.model.v1`
    - trim-before-validate semantics
    - absence semantics
    - boundary between pre-spawn validation and backend-owned runtime rejection
    - safe/redacted runtime error posture and terminal event requirement
  - Out:
    - implementation details of specific backend argv builders
    - any universal model catalog or alias scheme
    - standardizing `--fallback-model` or other secondary knobs
- **Primary interfaces (contracts)**
  - Inputs:
    - `AgentWrapperRunRequest.extensions["agent_api.config.model.v1"]`
    - canonical spec docs under `docs/specs/unified-agent-api/`
  - Outputs:
    - verification that the pinned schema and semantics in `docs/specs/unified-agent-api/extensions-spec.md` remain
      the source of truth
    - clarified error/run-lifecycle language in related universal specs only if the verification pass finds drift
- **Key invariants / rules**:
  - R0 capability gating runs before shape validation.
  - Effective value is the trimmed string.
  - Trimmed UTF-8 byte length is `1..=128`.
  - pre-spawn `InvalidRequest` failures use the exact safe template `invalid agent_api.config.model.v1`
    and MUST NOT echo the raw model id.
  - Absence preserves backend default behavior.
  - Runtime "unknown/unavailable/unauthorized model" outcomes remain backend-owned errors.
- **Dependencies**
  - Blocks:
    - SEAM-2
    - SEAM-3
    - SEAM-4
    - SEAM-5
  - Blocked by:
    - none
- **Touch surface**:
  - `docs/specs/unified-agent-api/extensions-spec.md`
  - `docs/specs/unified-agent-api/capabilities-schema-spec.md`
  - `docs/specs/unified-agent-api/contract.md`
  - `docs/specs/unified-agent-api/run-protocol-spec.md`
  - `docs/adr/0020-unified-agent-api-model-selection.md`
- **Verification**:
  - the SEAM-1 owner MUST run a repeatable drift check across:
    - `docs/specs/unified-agent-api/extensions-spec.md` (`### agent_api.config.model.v1`)
    - `docs/specs/unified-agent-api/capabilities-schema-spec.md` (`agent_api.config.model.v1`)
    - generic inherited baselines from:
      - `docs/specs/unified-agent-api/contract.md` (`AgentWrapperError`, `AgentWrapperBackend::run`, and
        `AgentWrapperEventKind::Error` / `AgentWrapperEvent.message`)
      - `docs/specs/unified-agent-api/run-protocol-spec.md` (`Capability validation timing` and
        `Error event emission for post-spawn unsupported operations (backend fault)`)
    - `docs/adr/0020-unified-agent-api-model-selection.md` sections
      `Canonical authority + sync workflow`, `Decision (draft)`, `Validation and error model`,
      `Backend mapping`, and `Capability advertising`
    - this pack's `README.md`, `scope_brief.md`, and `threading.md` restatements for SEAM-1-owned rules
  - the pass MUST confirm the compared sources agree on:
    - capability id + `agent_api.config.*` bucket placement
    - trim-before-validate behavior and trimmed byte bound `1..=128`
    - absence semantics
    - exact pre-spawn InvalidRequest template `invalid agent_api.config.model.v1`
    - backend-owned runtime rejection posture, including the inherited run-protocol rule for a terminal
      `AgentWrapperEventKind::Error` when a post-spawn failure occurs while the stream remains open
    - built-in backend mapping boundaries and capability-advertising posture
  - `no unresolved canonical-doc delta` means zero open mismatches remain after that comparison. If a mismatch is
    found, the owner MUST update the canonical specs first, then sync ADR-0020 and this pack in the same change before
    re-running the pass.
  - if a new mismatch appears after a passing run, SEAM-1 reverts to blocked status immediately and downstream seams
    MUST NOT claim the gate is satisfied again until a newer passing record replaces the stale one.
  - downstream implementation seams can reference a single canonical contract registry only after the latest recorded
    pass is `pass: no unresolved canonical-doc delta`
- **Verification record**:
  - the latest pass/fail result for this seam MUST be appended under `## Verification record` in this file
  - each record MUST include:
    - verification date
    - verifier name/role
    - compared sources
    - result (`pass: no unresolved canonical-doc delta` or `fail: canonical-doc delta opened`)
    - synchronization reference for the verified change set:
      - before the synchronized change set is committed or opened as a PR, record the base `git HEAD` plus an explicit
        note that the verification applies to the current working-tree delta
      - once a commit or PR exists, replace the provisional working-tree reference with that commit/PR reference
- **Risks / unknowns**
  - Risk:
    - drift between ADR rationale and owner-spec normative wording
  - De-risk plan:
    - doc-first reconciliation pass before backend code changes; if conflicts surface, resolve in the owner spec and update the ADR/body hash together
- **Rollout / safety**:
  - land contract text before enabling backend advertising
  - downstream seams may proceed once the SEAM-1 verification pass records `pass: no unresolved canonical-doc delta`
  - downstream seams MUST cite the latest passing entry in `## Verification record` when claiming the gate is
    satisfied, using the recorded synchronization reference exactly as written

## Verification record

- 2026-03-13 (UTC) - `pass: no unresolved canonical-doc delta`
  - Verifier: concrete-remediator (packet run directory `.codex/audit-trio-remediator/20260313T144946-514966Z/`)
  - Compared sources:
    - `docs/specs/unified-agent-api/extensions-spec.md` (`### agent_api.config.model.v1`)
    - `docs/specs/unified-agent-api/capabilities-schema-spec.md` (`agent_api.config.model.v1`)
    - `docs/specs/unified-agent-api/contract.md` (`AgentWrapperError`, `AgentWrapperBackend::run`, and
      `AgentWrapperEventKind::Error` / `AgentWrapperEvent.message`)
    - `docs/specs/unified-agent-api/run-protocol-spec.md` (`Capability validation timing` and the terminal
      `AgentWrapperEventKind::Error` rule for post-spawn failures)
    - `docs/adr/0020-unified-agent-api-model-selection.md` sections `Canonical authority + sync workflow`,
      `Decision (draft)`, `Validation and error model`, `Backend mapping`, and `Capability advertising`
    - this pack's `README.md`, `scope_brief.md`, and `threading.md` restatements for SEAM-1-owned rules
  - Synchronization reference: commit 4255d85.
  - Publication note: canonical alignment is the normative approval criterion for this pack.

- 2026-04-01 (UTC) - `pass: no unresolved canonical-doc delta`
  - Verifier: Codex seam-execution
  - Compared sources:
    - `docs/specs/universal-agent-api/extensions-spec.md` (`### agent_api.config.model.v1 (string)`)
    - `docs/specs/universal-agent-api/capabilities-schema-spec.md` (`- agent_api.config.model.v1:`)
    - `docs/specs/universal-agent-api/contract.md` (`AgentWrapperError` and `Stable payload rules for core event kinds (v1, normative)`)
    - `docs/specs/universal-agent-api/run-protocol-spec.md` (`## Capability validation timing` and `Error event emission for post-spawn unsupported operations (backend fault)`)
    - `docs/adr/0020-universal-agent-api-model-selection.md` (`## Canonical authority + sync workflow`, `### Decision (draft)`, `### Validation and error model`, `### Backend mapping`, and `### Capability advertising`)
    - this pack:
      - `README.md` (`## Canonical authority + sync workflow` and `## Canonical contracts (source of truth)`)
      - `scope_brief.md` (`## Required invariants (must not regress)` and `## Pinned execution decisions`)
      - `threading.md` (`## Contract registry` and `## Pinned decisions / resolved threads`)
  - Synchronization reference: commit 34b0ee9.
