# Charter — Onboarding new CLI agent wrapper crates + `agent_api` backends

Status: Normative  
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

Maintenance request note:
- the canonical automated maintenance packet contract lives in `docs/specs/maintenance-request-contract-v1.md`
- maintainer-authored maintenance requests remain valid under the legacy request format
- automated release-watch maintenance requests use `artifact_version = "2"` and `trigger_kind = "upstream_release_detected"`
- automated release-watch requests MUST carry a `[detected_release]` table and MUST freeze `requested_control_plane_actions = ["packet_doc_refresh"]`
- automated release-watch requests for enrolled maintenance MUST also carry
  `[support_surface_audit]`, and that block is the only valid packet-owned source of truth for:
  - newly discovered non-TUI surface
  - preexisting non-TUI support debt
  - allowed deferrals
  - `required_uplifts_this_run[]`

Approval maintenance note:
- the committed `approved-agent.toml` artifact MUST carry frozen `descriptor.maintenance` truth in exactly one mode: `release_watch_enrolled` or `explicitly_deferred`
- `release_watch_enrolled` requires committed registry `maintenance.release_watch` truth for the same agent
- `explicitly_deferred` forbids committed registry `maintenance.release_watch` truth for the same agent
- the Claude Code approval with SHA-256 `9bd2dc39909499598791fcdd84a2c0960f68d70f167e05fc98264b5099397fc3` predates this requirement. Its 2026-02-12 approval and proving-run closeout MUST remain historical evidence, without a backdated maintenance settlement. The sole compatibility path for its closed baseline is the current-dated `maintenance-readiness-adoption.json` in the same governance directory. The adoption MUST bind that exact approval, the original proving-run closeout with SHA-256 `f11ea2a8dea65b4266135536ab30efac63614e3c4caadcb6a73e3730f97026b8`, the original publication packet with SHA-256 `bfbf90819b93c58eef1e44c3c2430c5525c1f3ffa2a0df847a48cde348cbea3c`, the current normalized registry release-watch hash, and a later `recorded_at`. Lifecycle `maintenance_readiness_settled` requires the adoption path and SHA-256, both validated against the current registry. No other missing-settlement baseline receives this exception.

Maintenance posture note:
- a newly onboarded agent MAY start with partial non-TUI support
- once an agent is enrolled in automated release-watch maintenance, non-TUI support MUST ratchet
  upward over time rather than treating deliberate unsupported posture as steady state
- TUI-only surface remains excluded from the automated maintenance ratchet
- published non-TUI blockers MUST live in
  `docs/specs/unified-agent-api/non-tui-support-debt.md`, not only in support-matrix caveat prose

## Goals

- Make adding “CLI Agent X” a deterministic process:
  - predictable contract surfaces
  - predictable validation and fake-binary evidence
  - predictable capability + extension declaration rules
- Keep the universal event envelope small and stable while allowing backend-specific expansion.
- Prevent cross-document contradictions by having exactly one owner doc per contract surface.

## Normative references

- `docs/specs/agent-registry-contract.md`
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/support-depth-contract.md`
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
- `agent_api.tools.mcp.list.v1`
- `agent_api.tools.mcp.get.v1`
- `agent_api.tools.mcp.add.v1`
- `agent_api.tools.mcp.remove.v1`

The first four are the core ids for running, events and non-interactive execution. The four MCP
management ids are standard ids that `docs/specs/unified-agent-api/capabilities-schema-spec.md`
defines, and a backend advertises each only where its target supports it and, for `add` and
`remove`, where `allow_mcp_write` is set. Under default settings they can therefore be advertised
by fewer than two backends, or by none. The list above is the one `capability-matrix-audit`
enforces.

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
   - regenerate the library-only validated-runtime projection at `crates/agent_api/src/runtime_support_data.rs` alongside the published support surfaces when support publication is enabled
   - own publication output writes, the required green gate, and rollback if a publication write or gate step fails
   - keep the required publication command inventory fixed to:
     - `cargo run -p xtask -- support-matrix --check`
     - `cargo run -p xtask -- capability-matrix --check`
     - `cargo run -p xtask -- capability-matrix-audit`
     - `make preflight`
   - on success, commit lifecycle stage `published` in `lifecycle-state.json` and record packet continuity there while leaving `publication-ready.json` as the pre-refresh handoff packet
   - the next command template after refresh remains `prepare-proving-run-closeout --approval <path> --write`
8) Run `prepare-proving-run-closeout --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --write` after publication refresh succeeds:
   - write the canonical closeout artifact only at `docs/agents/lifecycle/<prefix>/governance/proving-run-closeout.json`
   - materialize that closeout artifact with `state = prepared`
   - keep lifecycle stage `published` until final closeout succeeds
   - prepare the generated onboarding packet in preview phase `closeout_prepared`
   - hand bounded human edits on the prepared closeout artifact to the maintainer before final closeout
9) Complete bounded human edits in `docs/agents/lifecycle/<prefix>/governance/proving-run-closeout.json`, then run `close-proving-run --approval <path> --closeout docs/agents/lifecycle/<prefix>/governance/proving-run-closeout.json`:
   - the committed closeout artifact must remain on the canonical path above
   - closeout states are exactly `prepared` and `closed`
   - prepared packet surfaces must not present the proving run as closed
   - the machine-owned closeout settlement surface is `maintenance_settlement`
   - `maintenance_settlement.mode` MUST equal `release_watch_enrolled` or `explicitly_deferred`
   - every `maintenance_settlement` MUST carry `approval_section_sha256`
   - `release_watch_enrolled` requires `maintenance_settlement.release_watch_sha256` and forbids `maintenance_settlement.deferral_sha256`
   - `explicitly_deferred` requires `maintenance_settlement.deferral_sha256` and forbids `maintenance_settlement.release_watch_sha256`
   - successful proving-run closeout records `maintenance_readiness_settled` before the lifecycle advances to `closed_baseline`
   - the pre-settlement Claude Code baseline described above may retain its original evidence list only while it keeps its exact historical identity. A current-dated adoption adds `maintenance_readiness_settled` without changing the old approval, proving-run closeout, or publication packet. The lifecycle record points to the adoption artifact and retains the original publication packet hash.
10) Ensure required CI workflows pass (see below).

Publication handoff rule:
- `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/publication-ready.json` is the only committed publication handoff packet
- once `publication-ready.json` exists, its `runtime_evidence_paths` become the only frozen committed authority for runtime evidence; sibling `.uaa-temp` runs are never authoritative by sort order
- `publication_ready` means that committed handoff packet exists and refresh is the next required command; it is not a second steady-state publication meaning
- after publication refresh, the required post-publication flow is `refresh-publication -> prepare-proving-run-closeout -> bounded human edits -> close-proving-run`
- `prepare-proving-run-closeout` consumes committed `published` state on the normal path and writes the canonical closeout artifact in `state = prepared`
- `close-proving-run` is the final transition that consumes the prepared closeout artifact and records `state = closed`
- any remaining `publication_ready` acceptance is limited to narrow transitional compatibility for legacy/manual records
- scratch runtime `handoff.json` files remain run evidence only

Runtime evidence repair rule:
- `repair-runtime-evidence --write` may repoint `active_runtime_evidence_run_id` while leaving lifecycle stage unchanged
- that selector change is a lifecycle mutation and must update lifecycle provenance fields (`current_owner_command`, `last_transition_at`, `last_transition_by`)
- repair must be transactional across the canonical repair bundle and lifecycle state: on failure, neither authoritative surface may change

## Support depth

The [support-depth contract](support-depth-contract.md) owns the rules for support depth: depth
enrollment, the bindings Event, P, O and E, the depth record, depth admission, and which changes
are acceptance effects. This section states only what this charter owns for the create lane: what
an approval states for a depth enrollment and how it binds the registry, where P and O are frozen
and which committed artifact states their identities, how the surface O is frozen from is
acquired and admitted, the write-run boundary of a depth-enrolled generation, which create-lane
commands make acceptance effects, and what the lane has no route for. It uses
that contract's terms and redefines none of them.

While the support-depth contract is a Draft this section binds nothing.

Rules 5 to 9 and 12 to 14 adopt the maintainer-approved decisions OF-P1 to OF-P9 of the
[onboarding freeze decisions](../agents/lifecycle/support-depth-onboarding-freeze-decisions.md),
with OF-P3 option (A), OF-P4 option (B) and OF-P9 publication check (i). Each sentence that
adopts one, or the bullet or paragraph that holds it, cites its row. They define the create-lane depth path. They enable no path, and no
create-lane command implements them yet: [Present behavior](#present-behavior) says what holds
today.

Rules:

1. The support-depth contract defines a create-lane generation and its Event. The approval artifact
   of that Event is the committed `approved-agent.toml` of the agent's onboarding pack. In this
   section a generation is on the create lane from its approval until its proving-run closeout is
   recorded as `closed`.
2. A depth enrollment of a new agent on the onboarding path is declared in the registry's
   `support_depth` table with `lifecycle_path = "onboarding"`, as the
   [registry contract](agent-registry-contract.md#support-depth) states. `onboard-agent` appends
   the agent's entry without that table. A maintainer adds the table afterwards, as release-watch
   enrollment is added, in a change that leaves `onboarding` in `enabled_paths`.
3. An approval artifact that approves a depth enrollment states the one exact upstream version it
   approves it for. It states nothing else of the selection, and it declares nothing: the registry
   table is the declaration. While a generation is on the create lane the two bind by that version.
   A create-lane command resolves the generation's depth enrollment only where the registry
   declares, on the onboarding path, a depth enrollment of the version the approval states. Where
   the approval states a version and the registry declares none for it on the onboarding path, the
   generation's depth enrollment is unresolved, which the support-depth contract treats as an
   error. An approval that states no version approves no depth enrollment, and a declaration on the
   onboarding path of a version the agent's approval does not state opens no create-lane
   generation. Such a declaration is still a declaration and its version is depth-enrolled, as it
   is under the registry contract's rule 8 for a declaration whose path is not enabled: nothing of
   it is frozen on the create lane, and no create-lane command makes an acceptance effect for its
   depth scope tuples. A declaration that a maintainer moves to the maintenance path after the
   generation's proving-run closeout is recorded as `closed` is outside this rule.

   The approval states the version in a root table:

   ```toml
   [support_depth]
   version = "1.2.3"
   ```

   `version` is one exact upstream version, `MAJOR.MINOR.PATCH` with no prerelease and no build
   metadata, in the form the registry contract's `version` rule requires. The table holds no
   other key, and an approval without it states no version. The table states what the approval
   approves and is not authority: the registry declaration is, as the support-depth contract's
   depth enrollment rule 7 requires. `artifact_version` stays `"1"`: the
   table is optional, and a loader of the approval artifact MUST reject a root key it does not
   know, and a `support_depth` key anywhere but the root, so that a misspelled root table, or
   one placed below the root, is refused rather than read as an approval that states no version. This is the executable schema revision of this charter that the support-depth
   contract's path enablement requires for the onboarding path.
4. Before P is frozen, the agent's manifest root MUST hold version metadata for the approved exact
   upstream version, a `RULES.json` under which the declared targets meet the registry contract's
   `targets` rule, and a validator spec that adopts Annex B of the support-depth contract. A
   placeholder version is not an exact upstream version. This revision does not name the command
   that writes these files.
5. `runtime-follow-on --dry-run` is the create lane's freeze. The first dry run of a generation at
   which rule 3 resolves and rule 4 holds freezes P from the registry declaration and writes the
   version's depth record, or continues it where one exists. A dry run of a generation whose
   approval states a version MUST refuse where rule 3 does not resolve or rule 4 does not hold. It
   MUST NOT prepare the run as though the generation were not depth-enrolled. A later dry run of
   the same generation is a re-freeze, and the support-depth contract's Bindings say what it may
   change. No other create-lane command freezes or re-freezes P or O (OF-P1). The dry run writes
   the depth record and the lifecycle object of rule 7 in the same invocation, through the
   lifecycle writer (OF-P1, OF-P2).

   Both modes of `runtime-follow-on` require the lifecycle stage `enrolled`, and a dry run of a
   depth-enrolled generation at `runtime_integrated` MUST refuse (OF-P5). A dry run after a write
   run that failed, with the stage still `enrolled`, is a re-freeze. It is admitted only where
   rule 14 admits the surface it finds, only where rule 8 ensured that the failed run left no
   executor output in the caller tree, and only with P unchanged: it MUST preserve P and MUST
   refuse where P has changed, as freeze rule 3 of the
   [maintenance request contract](maintenance-request-contract-v1.md#freezes) requires of a
   re-freeze (OF-P5). A dry run that leaves O not yet produced MUST say so, and MUST say that a
   write run that passes then closes the generation's window to freeze O (OF-P5). A generation that
   reaches `runtime_integrated` with O not yet produced cannot reach acceptance for depth-enrolled
   scope. Work continues only in a new generation, which needs a new approval artifact, as the
   support-depth contract's Bindings rule 9 states (OF-P5).
6. A dry run freezes O where the acquired surface of the approved version is already in the
   manifest root. It MUST freeze O only from a surface that was acquired before the dry run, and
   MUST NOT freeze it from the output of a write run, of this generation or of an earlier one,
   because the support-depth contract lets the executor change no part of O. Where the surface is
   absent the dry run leaves O not yet produced, and the generation cannot reach acceptance for
   depth-enrolled scope until a dry run has frozen O. Rule 12 names how the surface is acquired,
   and rule 14 says what admits it as acquired (OF-P3, OF-P4).

   The acquired inputs of an onboarding O are exactly the inputs the support-depth contract's
   shared derivation of O binds, and the dry run derives O through that same derivation (OF-P6):
   - `RULES.json` and `SCHEMA.json` of the manifest root;
   - `snapshots/<version>/union.json` and each target shard under `snapshots/<version>/`;
   - `wrapper_coverage.json`;
   - `reports/<version>/coverage.*.json`.

   The approval artifact, version metadata, the validator spec, the lifecycle state and run files
   are not acquired inputs (OF-P6). A dry run freezes O only from a union whose `complete` is
   `true` for `union.expected_targets`. Where the union is present and not complete, the dry run
   MUST refuse and MUST name the missing targets (OF-P7). Whether a union that is not complete may
   freeze O for exactly the declared targets is not decided, and until it is the dry run refuses.
7. The depth record states P and O of a create-lane generation. The second committed statement of
   their identities is a root object `support_depth` in the onboarding pack's
   `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/lifecycle-state.json` (OF-P2). A
   run's input contract is not one: [Present behavior](#present-behavior) says why. The object has
   the shape of a maintenance request's `[support_depth]` table:

   ```json
   "support_depth": {
     "policy_identity": "<sha256 of P>",
     "obligations_identity": "<sha256 of O>"
   }
   ```

   It holds exactly `policy_identity` and, once O is frozen, `obligations_identity`, each as the
   request contract's field rules and rule 3 state them, and nothing else (OF-P2). It always states
   what the version's depth record states, and its states are those of the record (OF-P2):
   - **No record, no object.** P is not yet frozen. With the declaration and approval in place
     this is the case "declared, no record yet": insufficient depth, with no acceptance authority,
     and not an error. Declaring the depth enrollment and acquiring the surface happen in this
     state.
   - **Record without object, or object without record.** A deleted binding, which is an error. A
     missing object is never read as "not depth-enrolled".
   - **The record's Event names an earlier approval.** A new generation that has not frozen yet,
     which is insufficient depth. The object keeps stating the earlier generation until this
     generation's first dry run replaces it.

   Writer: the dry run of rule 5, in the same invocation that writes or continues the record.
   Replacer: a re-freeze dry run of the same generation, which keeps `policy_identity`, or the first
   dry run of a later generation, which continues the record. Every other lifecycle writer,
   `onboard-agent`, `runtime-follow-on --write`, `repair-runtime-evidence`, `prepare-publication`,
   `refresh-publication`, `prepare-proving-run-closeout`, `close-proving-run` and
   `historical-lifecycle-backfill` included, MUST preserve the object byte for byte (OF-P2).
   Repository validation compares the record with the object as it compares a record with a
   maintenance request's table, with the onboarding Event in place of the request's (OF-P2). The
   record's P identity MUST equal `policy_identity`, its O identity MUST equal
   `obligations_identity`, and its O MUST be shown as not yet produced exactly while
   `obligations_identity` is absent. A record whose Event names an earlier approval is read as the
   third state above.
8. `runtime-follow-on --write` and `repair-runtime-evidence` make reporting effects only. Each MUST
   NOT list an acceptance entry and MUST NOT change what the depth record states under P or O.
   Adapter code that a write run lands can advertise a capability. On its branch that code is a
   candidate, and the advertising it adds is admitted only with the acceptance entry that
   publication lists under rule 9.

   For a depth-enrolled generation (OF-P4, OF-P8, OF-P9):
   - A write run's allowed paths do not include `<manifest_root>/snapshots/**`, and a write run
     that changes it fails the boundary check (OF-P4).
   - The run's input contract carries an optional `support_depth` object holding `version`, taken
     from the approval's `[support_depth]` table and equal to the record's selection version, and
     `policy_identity` and an optional `obligations_identity` copied from the lifecycle object of
     rule 7. The object is present exactly for depth-enrolled generations, and adding it bumps the
     input contract's `workflow_version` (OF-P8). It is a cross-check and never authority. A write
     run MUST refuse before execution where its values differ from the lifecycle object or the
     record, and MUST refuse after execution where either has changed. `repair-runtime-evidence`
     copies the identities from the lifecycle object and never computes them (OF-P8). An input
     contract that lacks the object is refused for a depth-enrolled generation (OF-P8).
   - A write run executes in a disposable candidate copy, not in the caller workspace, in these
     phases (OF-P9):
     1. *Trusted preparation, in the caller.* Load and check the dry-run packet, and record the
        write attempt in the caller's lifecycle state. The attempt state stays outside candidate
        delivery.
     2. *Before-state, in the controller.* Capture the before-tree in the controller's memory
        immediately before execution. The serialized `baseline` in the input contract is
        reporting only.
     3. *Execution, in the candidate.* Run files fall in three classes. Immutable inputs: the
        input contract and prompt, the lifecycle state, the depth record, the approval and the
        acquisition statement of rule 13. Executor outputs: `handoff.json` and the allowed source
        paths. Controller outputs: stdout, stderr, the execution evidence, the validation report,
        the run status, `written-paths.json` and the summary. An executor change to an immutable
        input rejects the run.
     4. *Observation, in the controller.* Observe the candidate and run the create-lane boundary
        and manifest-split checks against the in-memory before-tree, before any controller output
        is written, so that controller outputs never count as executor changes.
     5. *Finalization.* There is no early exit between execution and finalization. Controller
        outputs are written as diagnostics. A failed or interrupted run delivers only the
        before-tree plus diagnostics, so the caller tree keeps no executor output. A passing run is
        delivered through checked expected-before mutations, and delivery refuses where the caller
        tree changed.
   - `written-paths.json` is written by the controller from its own observation and is reporting
     only. The lifecycle state written when the run passes records the SHA-256 of
     `written-paths.json`, outside the object of rule 7, and publication MUST refuse a
     `written-paths.json` whose SHA-256 does not match it (OF-P9, check (i)).

   The candidate copy is not an operating-system sandbox, the same limit as Annex A route MA4 of
   the support-depth contract (OF-P9). A write run of a generation that is not depth-enrolled is
   unchanged by this rule (OF-P9).
9. `refresh-publication --write` and `close-proving-run` are the create-lane commands that make
   acceptance effects. The support-depth contract's list of acceptance effects and its Annex A.2
   say which effects those are. They include the capability advertising and the qualified or
   depth-qualified results that publication adds for a new agent, the lifecycle stages `published`
   and `closed_baseline`, and a proving-run closeout recorded as `closed` with its evidence id.
   That contract requires each command to establish depth admission and to list an acceptance
   entry in the same change. For a depth-enrolled generation, the commands of Annex A routes ON3 to
   ON5 MUST refuse where an acquired input of rule 6 differs from the O the record states, as the
   maintenance relay refuses a changed frozen input. A regenerated `wrapper_coverage.json` is such
   a change (OF-P6). Under rule 5 no re-freeze is then left to the generation (OF-P5, OF-P6).
10. The create lane has no packet freeze, because no automation replaces a create-lane branch. An
    onboarding generation is committed from the time the integration branch holds its depth
    record, as the support-depth contract's depth enrollment rule 7 states.
11. The create lane sets no pointer and records no `validated` or `supported` version status, and
    this charter adds no route for either. For a version whose depth enrollment the onboarding path
    owns, those acceptance effects are not available on the create lane. They become available
    after the proving-run closeout is recorded as `closed`, when a maintainer moves the version's
    declaration to the maintenance path under the registry contract's rules and a maintenance
    generation of the version reaches promotion.
    [Multi-target parity acquisition](#multi-target-parity-acquisition-when-a-new-agent-joins-it)
    says what the agent needs in order to be on that path.
12. The surface a dry run freezes O from is acquired by the support-depth contract's input-only
    writers, run directly: the agent's snapshot command, then `manifest-union`, the
    wrapper-coverage generator and `manifest-report` (OF-P3). This holds whether the approval
    states `release_watch_enrolled` or `explicitly_deferred`. A depth enrollment on the onboarding
    path does not require `maintenance.release_watch`, and the agent is not enrolled in maintenance
    automation for it (OF-P3). The `manifest-acquisition-plan` preview stays gated on
    release-watch and is used only when the agent is enrolled (OF-P3). No new command acquires the
    surface, `parity-acquire.yml` is not used for it, and the acquired outputs reach the integration
    branch in a reviewed change before the dry run (OF-P3). These writers grant no acceptance
    authority.
13. The acquisition statement says which acquired inputs a maintainer acquired. It is committed at
    `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/depth-acquisition/<version>.toml`,
    beside the lifecycle state, and is modeled on the maintenance stand-down markers (OF-P4). It
    states the agent, the exact upstream version, and for each acquired input its
    repository-relative path, its SHA-256 and the command that produced it (OF-P4). A maintainer
    writes it, by a change to the integration branch (OF-P4). It takes effect only from the
    integration branch, and never from the onboarding branch that uses it: a statement carried on
    that branch does not count before it merges (OF-P4). Marker rule 5 of the maintenance request
    contract governs stand-down markers only, and the create lane has no packet freeze (rule 10),
    so this constraint is adopted by OF-P4 and not inherited. A wrong statement is a maintainer
    error that integration admission cannot detect (OF-P4).
14. A dry run admits a surface, and freezes O from it, only where both of these hold (OF-P4):
    - **Integrity.** Every acquired input of rule 6 meets the bound-input rule the shared
      derivation reads by: a regular, staged, unconflicted file whose working bytes equal its staged
      bytes. Each is byte-identical to the same path at an integration ref that the caller fetched
      and named, as a caller names the ref it reads stand-down markers from. On an isolated route
      that ref is the route's protected tip. Where the tree differs from the ref, the dry run MUST
      refuse and MUST name the paths. This equality is a currentness check only and says nothing of
      where the bytes came from.
    - **Provenance.** The acquisition statement of rule 13 for the version is present at that
      named ref, and every acquired input equals both the statement and the ref. An acquired input
      the statement does not cover, an input whose bytes differ from the statement, and a statement
      entry with no matching input each make the dry run refuse. Neither the order nor the time of
      commits is consulted (OF-P4).

    Without a fetched integration ref the dry run cannot freeze O, though it can still freeze P
    (OF-P4). The executor's protected paths include the acquisition statement (OF-P4, OF-P9).
    `written-paths.json` is never read to decide where a surface came from (OF-P4, OF-P9).

The support-depth contract's minimum machinery rule requires the failure a new field or mechanism
prevents to be named. Without the version in the approval, a declaration added to the registry
after onboarding began would attach a depth enrollment to a generation whose approval never covered
it, and the depth record's Event would name an approval that says nothing of depth. Without the
lifecycle object of rule 7, a record rewritten together with its identities could not be told from
the record that was frozen, because no second committed statement of them exists on the create
lane (OF-P2). Without the acquisition statement of rule 13, an earlier generation's executor output
already merged to the integration branch could not be told from an acquired surface: byte equality
and the order of commits cannot distinguish them (OF-P4). Without the input contract's
`support_depth` object, a write run prepared under an earlier freeze could not be detected
(OF-P8). Without the recorded SHA-256 of `written-paths.json`, publication would trust a file the
executor can write (OF-P9).

### Present behavior

Only the approval loader's reading of rule 3 is implemented. Rules 5 to 9 and 12 to 14 state
the create-lane depth path that the
[onboarding freeze decisions](../agents/lifecycle/support-depth-onboarding-freeze-decisions.md#implementation-slices)
plan in later slices; none of it is installed. Every create-lane entry still refuses a depth
workspace (Annex A routes ON1 to ON5 of the support-depth contract), production installs no
profile, and the onboarding path is not enabled. Naming the freeze point and the identity
artifact meets only the charter's part of the support-depth contract's Path enablement for the
onboarding path. Items 1, 3, 4 and 5 there, the isolated new-entry run and the maintainer's
authorization for each agent are still required. Source references are to `staging` at
`f61534be` unless a bullet names another revision. Today:

- An approval artifact carries `approval_commit` and `approval_recorded_at`
  (`crates/xtask/src/approval_artifact.rs:246-249`). The approval loader reads the
  `[support_depth]` table of rule 3 and refuses an unknown root key; that is not at `f61534be`.
  No committed approval carries the table, no command writes it, and no create-lane command acts
  on the version it states; `onboard-agent` refuses an approval that carries it (below).
- `onboard-agent --write` scaffolds the manifest root with a bootstrap `current.json`, a
  `RULES.json`, `SCHEMA.json`, `VERSION_METADATA_SCHEMA.json`, a validator spec, root version
  pointers set to `none` and an empty `wrapper_coverage.json`
  (`crates/xtask/src/onboard_agent/preview.rs`, `crates/xtask/src/onboard_agent/preview/governing.rs`),
  and lists every path it writes in the pack's `governance/written-paths.json`. The validator spec
  does not adopt Annex B and `RULES.json` names no depth record path, so rule 4 does not hold for a
  freshly onboarded root. It writes no version metadata for any version: it refuses an approval
  whose `[support_depth]` table states a version, because no surface of that version has been
  acquired, and it refuses a descriptor with more than one canonical target, because no policy
  names the required target. The `aider` and `gemini_cli` roots, onboarded before the scaffold,
  have no `RULES.json`. Their validator specs were written by hand; each states the shared depth
  checks and says that its root adopts no Annex B record until a `RULES.json` names the record
  path. The `aider` root's only version is the placeholder `0.0.0`.
- `runtime-follow-on --dry-run` writes a frozen prompt and input contract under
  `docs/agents/.uaa-temp/runtime-follow-on/runs/<run_id>/`, and `--write` requires the run id of a
  prepared dry run (`crates/xtask/src/runtime_follow_on.rs:165-169,235-238`). Both modes require
  the lifecycle stage `enrolled`, and a write run that passes sets `runtime_integrated`
  (`crates/xtask/src/runtime_follow_on/lifecycle.rs:86-106`), so no dry run is admitted after a
  write run has succeeded. A write run that fails leaves the stage `enrolled`, recording a side
  state where its validation fails (`crates/xtask/src/runtime_follow_on/lifecycle.rs:141-182`), and
  the command restores nothing, so a later dry run is admitted with the failed run's output in
  place.
- A write run may change `snapshots/` and `supplement/` under the manifest root and is rejected
  when it changes anything else there, `reports/` and version metadata included
  (`crates/xtask/src/runtime_follow_on/codex_exec.rs:150-163`,
  `crates/xtask/src/runtime_follow_on.rs:455-482`). The check runs after the executor has written.
  The write run is the only create-lane command that writes `snapshots/`. The input-only writers of
  rule 12 also write it, but no command reads an acquisition statement, so no dry run admits a
  surface under rule 14 yet. A write run lists the paths it changed in its run
  directory's `written-paths.json` (`crates/xtask/src/runtime_follow_on.rs:196-221`). No dry run
  reads that file, it is ignored by git unless added as runtime evidence, an executor can rewrite
  it, and an interrupted run writes none, so nothing a dry run reads says which command wrote a
  surface. The baseline a write run is compared with is taken before the dry run writes its own
  files (`crates/xtask/src/runtime_follow_on.rs:314-318`), so a depth record written by the dry run
  would count as a change of the write run.
- A run's input contract carries the approval artifact's path and SHA-256 and no depth identity
  (`crates/xtask/src/runtime_follow_on/models.rs:8-30`). Run directories are ignored by git, apart
  from four historical ones, and reach a commit only when added as runtime evidence. The runs
  root is inside a write run's allowed paths and outside the comparison
  (`crates/xtask/src/runtime_follow_on/codex_exec.rs:48-54`,
  `crates/xtask/src/runtime_follow_on.rs:314-318`), so an executor can rewrite the file
  unnoticed. `repair-runtime-evidence` writes a different input contract without a dry run.
- No create-lane command writes `reports/<version>/`, version metadata or a pointer, and none
  reads a depth enrollment.
- The following references are to `f8fcdc8`, the baseline of the onboarding freeze decisions. The
  typed lifecycle state has no `support_depth` object and no field for the SHA-256 of
  `written-paths.json`, and it does not reject unknown fields
  (`crates/xtask/src/agent_lifecycle.rs:301-302`), so a lifecycle writer that predates the object
  would drop it. No reader of an acquisition statement exists.
- At `f8fcdc8` a write run executes in the caller workspace, and its boundary check compares the
  tree with the `baseline` read back from the executor-writable input contract
  (`crates/xtask/src/runtime_follow_on.rs:180-181,407-408`). The candidate execution of rule 8
  exists only for Annex A route MA4, in `crates/xtask/src/agent_maintenance/execute/`.
- `manifest-version-metadata` refuses a depth manifest, so whether version metadata for rule 4
  is written before the declaration or only by a later writer is not decided.
- Two questions stay open, and the behavior above refuses meanwhile. Whether a union that is not
  complete may freeze O for exactly the declared targets: rule 6 refuses it. Whether publication
  may regenerate `wrapper_coverage.json` for a depth-enrolled generation: rule 9 refuses the
  changed input.

## Multi-target parity acquisition (when a new agent joins it)

Onboarding captures a new agent's baseline surface from a **single host**, which is correct for
proving the wrapper but can only ever produce a partial, `complete:false` union. Entering
multi-target parity acquisition is a separate, explicit step.

Entry rule:
- an agent enters multi-target acquisition when it is enrolled in `maintenance.release_watch`
  **and** its `cli_manifests/<agent>/RULES.json` carries an `acquisition` block
- no separate registry field gates this; see `docs/specs/agent-registry-contract.md`
- until both hold, the agent stays on the docs-only maintenance path and nothing about its
  existing behavior changes
- this entry rule governs multi-target acquisition only; acquiring the surface a create-lane dry
  run freezes O from needs no `maintenance.release_watch` ([Support depth](#support-depth) rule
  12, OF-P3)

Adding the `acquisition` block is what routes a newly onboarded agent onto the **same** reusable
lane every other agent uses (`.github/workflows/parity-acquire.yml`) rather than a bespoke
per-agent workflow. The block must declare, for every target in `union.expected_targets`:
- the release source (`github_releases` or `npm`) and its source-specific configuration
- each target's runner, download coordinates, archive shape, and installed binary path
- the agent's snapshot command and its binary flag
- the per-target validation commands promotion must pass

Two consistency rules are enforced by `xtask manifest-acquisition-plan` and MUST hold:
- `acquisition.targets` and `union.expected_targets` describe exactly the same target set
- `union.required_target` is present in `acquisition.targets`

A new agent also needs a snapshot adapter (`<agent>-snapshot`) before acquisition can run, since
capturing a CLI's help surface requires executing it natively on a matching-OS runner. Preview the
whole resolved plan before wiring CI:

```sh
cargo run -p xtask -- manifest-acquisition-plan --agent <agent_id> --version <semver>
```

Promotion remains maintainer-gated for every agent, at every tier.

## CI expectations (must stay green)

The following workflows are expected to remain green for onboarding work:
- `.github/workflows/ci.yml`
  - `cargo test --workspace --all-targets`
  - `cargo test -p agent_api --all-features`
  - public API type leak guard for `agent_api`
- Smoke workflows for feature packs (when present), e.g.:
  - `.github/workflows/unified-agent-api-smoke.yml`
  - `.github/workflows/claude-code-live-stream-json-smoke.yml`
