# Charter — Onboarding new CLI agent wrapper crates + `agent_api` backends

Status: Draft  
Date (UTC): 2026-02-20  
Owner(s): atomize-hq wrappers team

This charter defines the canonical rules for onboarding new CLI agent support in this repo.

It is designed to keep the system **orthogonal**:
- wrapper crates can evolve independently, and
- the universal facade (`agent_api`) can onboard new backends mechanically with minimal drift.

Procedure note:
- this charter is normative and defines repo requirements
- the shipped operator workflow lives in `docs/cli-agent-onboarding-factory-operator-guide.md`
- if the charter and an operator step summary ever diverge, the charter and `docs/specs/**` own the contract truth

## Goals

- Make adding “CLI Agent X” a deterministic process:
  - predictable contract surfaces
  - predictable validation and fake-binary evidence
  - predictable capability + extension declaration rules
- Keep the universal event envelope small and stable while allowing backend-specific expansion.
- Prevent cross-document contradictions by having exactly one owner doc per contract surface.

## Normative references

- `docs/specs/agent-registry-contract.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/extensions-spec.md`

## Non-Goals

- Forcing semantic parity across all agents (capabilities differ; the API must represent that).
- Turning planning docs into a CI gate (planning artifacts are for humans and execution triads).

## Canonical architecture layers

### 1) Wrapper crates (per CLI agent)

Example: `crates/codex/`, `crates/claude_code/`.

A wrapper crate SHOULD provide:
- A deterministic spawn surface (builder + request types).
- A typed streaming surface:
  - `events: Stream<Item = Result<TypedEvent, ParseError>>`
  - `completion: Future<Output = Result<Completion, Error>>`
- A pure parsing API for offline JSONL/stream parsing (testable without spawning).
- A fake-binary or fixture strategy for cross-platform tests when the real CLI is unavailable.

A wrapper crate MUST:
- avoid leaking secrets by default (no raw-line echoing as “errors” in library APIs unless explicitly opted in),
- support deterministic disabling of interactive prompting when the upstream CLI supports it,
- document its CLI parity surface (what flags and flows are covered by the wrapper).

### 2) Universal facade (`crates/agent_api/`)

`agent_api` is the stable, agent-agnostic surface:
- `AgentWrapperRunRequest` (prompt/working_dir/timeout/env/extensions)
- `AgentWrapperEvent` (stable envelope + optional JSON `data`)
- `AgentWrapperCompletion` (exit status + optional `final_text`)
- capability gating for optional features

The universal API MUST:
- be safe by default:
  - bounded payloads
  - redacted error messages
  - no raw backend line leakage in v1
- preserve protocol invariants:
  - “completion finality” (DR-0012): completion resolves only once the event stream is final/dropped
- keep backend types out of the public API (guarded in CI).

## Capabilities + extensions rules (canonical)

### Capability ids

Rules are owned by:
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`

### Capability promotion rule

To keep the universal facade orthogonal, any new `agent_api.*` capability id (except the allowlist
below) is only considered “promoted” once it is supported by **≥2 lifecycle-eligible agent
backends in capability publication truth**.

This is CI-enforced by:
- regenerating and diff-checking `docs/specs/unified-agent-api/capability-matrix.md`
  via `cargo run -p xtask -- capability-matrix`, and
- running `cargo run -p xtask -- capability-matrix-audit`.

Allowlist (may be supported by fewer than 2 backends):
- `agent_api.run`
- `agent_api.events`
- `agent_api.events.live`
- `agent_api.exec.non_interactive`

### Extension keys

Core extension key registry + ownership rules are owned by:
- `docs/specs/unified-agent-api/extensions-spec.md`

Required invariants:
- Every supported extension key MUST be advertised in `AgentWrapperCapabilities.ids`.
- Backends MUST fail-closed on unknown extension keys before spawning.
- Every extension key MUST have exactly one authoritative owner document:
  - `agent_api.*` keys are owned by the universal registry
  - `backend.<agent_kind>.*` keys are owned by the backend’s contract/spec docs

## Streaming event mapping rubric (recommended buckets)

To keep onboarding orthogonal, treat backend output as mapping into these buckets:

- **TextOutput**: assistant text (snapshots and deltas)
- **ToolCall**: tool use intent/start (command execution, file ops, MCP tool call, web search, etc.)
- **ToolResult**: tool result/finish (where a backend provides a stable “result” event)
- **Status**: lifecycle markers (thread/turn start/complete, progress)
- **Error**: redacted backend errors (transport, parse, normalize, tool failures)
- **Unknown**: parseable but unmapped events (safe placeholder)

Rules of thumb:
- Prefer “best-effort parity” rather than forcing identical payload schemas.
- Use `data` only for stable, bounded, redacted payloads; never for raw backend lines.

## Non-interactive + sandbox posture (canonical)

Backends should be automation-safe by default, and hosts should be able to override explicitly per run.

Core key:
- `agent_api.exec.non_interactive` (owned by `extensions-spec.md`)

Backend-specific exec-policy knobs (pattern):
- `backend.<agent_kind>.exec.*` keys (owned by backend contract/spec docs)

## Onboarding checklist (new CLI agent)

Canonical lifecycle record:
- `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/lifecycle-state.json`
- this file owns committed lifecycle stage, support tier, evidence satisfaction, and next-command truth for create mode
- at `lifecycle_stage = runtime_integrated`, this file also owns `active_runtime_evidence_run_id`, the only canonical selector for the authoritative runtime-evidence run under `docs/agents/.uaa-temp/runtime-follow-on/runs/<run_id>/`
- generated packet docs and handoff prose are evidence, not lifecycle authority
- maintenance comparisons must anchor to the committed lifecycle record rather than reconstructing state from scattered packet artifacts

1) Run `onboard-agent --write` to enroll the control-plane surfaces:
   - registry entry
   - docs pack
   - manifest root
   - workspace/release touchpoints
   - `onboard-agent` does not create the wrapper crate
2) Run `scaffold-wrapper-crate --agent <agent> --write` to create the wrapper crate shell at the registry-owned `crate_path` under `crates/`:
   - initial crate layout and Cargo metadata
   - initial publishability metadata owned by the scaffold, including crate-local `README.md`, `LICENSE-APACHE`, `LICENSE-MIT`, and `readme = "README.md"`
   - hyphenated crate directories are supported; the scaffold derives `[lib].name` from the final `crate_path` component by normalizing `-` to `_`
   - if the normalized basename contains anything outside ASCII `[A-Za-z0-9_]+`, validation fails before scaffold output is written
3) Implement backend/runtime details in the wrapper crate and `agent_api` backend adapter:
   - builder + request types
   - streaming typed events + completion
   - offline parser API
   - fixtures/fake binary strategy
   - map typed events → universal envelope
   - enforce redaction + bounds
   - preserve completion gating (DR-0012)
   - advertise capabilities + extension keys
4) Add wrapper coverage manifest (or equivalent) proving which CLI flags/flows are supported.
5) Add C2-style tests in `agent_api` that do not require a real CLI:
   - “live event before completion”
   - redaction (no raw line leakage)
   - exec-policy default behavior (non-interactive) and override levers if applicable
6) Run `prepare-publication --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --write` after committed runtime evidence exists:
   - validate approval SHA continuity, implementation-summary completeness, capability publication continuity, and the exact runtime-evidence bundle selected by `active_runtime_evidence_run_id`
   - write only `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/publication-ready.json`
   - advance the committed lifecycle record to `publication_ready`, the pre-refresh-only stage in the canonical path `publication_ready -> published -> closed_baseline`
   - clear `active_runtime_evidence_run_id` as part of that stage transition
   - the next command template remains `refresh-publication --approval <path> --write`
7) `refresh-publication --approval <path> --check|--write` is the only publication consumer command; run `refresh-publication --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --write` to consume the committed handoff packet:
   - refresh publication outputs from the committed handoff packet
   - own publication output writes, the required green gate, and rollback if a publication write or gate step fails
   - keep the required publication command inventory fixed to:
     - `cargo run -p xtask -- support-matrix --check`
     - `cargo run -p xtask -- capability-matrix --check`
     - `cargo run -p xtask -- capability-matrix-audit`
     - `make preflight`
   - on success, commit lifecycle stage `published` in `lifecycle-state.json` and record packet continuity there while leaving `publication-ready.json` as the pre-refresh handoff packet
   - the next command template after refresh remains `close-proving-run --approval <path> --closeout docs/agents/lifecycle/<prefix>/governance/proving-run-closeout.json`
8) Ensure required CI workflows pass (see below).

Publication handoff rule:
- `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/publication-ready.json` is the only committed publication handoff packet
- once `publication-ready.json` exists, its `runtime_evidence_paths` become the only frozen committed authority for runtime evidence; sibling `.uaa-temp` runs are never authoritative by sort order
- `publication_ready` means that committed handoff packet exists and refresh is the next required command; it is not a second steady-state publication meaning
- `close-proving-run` consumes committed `published` state on the normal path
- any remaining `publication_ready` acceptance is limited to narrow transitional compatibility for legacy/manual records
- scratch runtime `handoff.json` files remain run evidence only

Runtime evidence repair rule:
- `repair-runtime-evidence --write` may repoint `active_runtime_evidence_run_id` while leaving lifecycle stage unchanged
- that selector change is a lifecycle mutation and must update lifecycle provenance fields (`current_owner_command`, `last_transition_at`, `last_transition_by`)
- repair must be transactional across the canonical repair bundle and lifecycle state: on failure, neither authoritative surface may change

## CI expectations (must stay green)

The following workflows are expected to remain green for onboarding work:
- `.github/workflows/ci.yml`
  - `cargo test --workspace --all-targets`
  - `cargo test -p agent_api --all-features`
  - public API type leak guard for `agent_api`
- Smoke workflows for feature packs (when present), e.g.:
  - `.github/workflows/unified-agent-api-smoke.yml`
  - `.github/workflows/claude-code-live-stream-json-smoke.yml`
