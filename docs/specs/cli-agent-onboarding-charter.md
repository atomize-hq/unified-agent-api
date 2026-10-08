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
an approval states for a depth enrollment and how it binds the registry, where P and O are frozen,
which create-lane commands make acceptance effects, and what the lane has no route for. It uses
that contract's terms and redefines none of them.

While the support-depth contract is a Draft this section binds nothing.

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
   generation's proving-run closeout is recorded as `closed` is outside this rule. This revision
   does not define the field that carries the version. It belongs to an executable schema revision
   of this charter, adopted before the onboarding path is enabled.
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
   change.
6. A dry run freezes O where the acquired surface of the approved version is already in the
   manifest root. It MUST freeze O only from a surface that was acquired before the dry run, and
   MUST NOT freeze it from the output of a write run, of this generation or of an earlier one,
   because the support-depth contract lets the executor change no part of O. Where the surface is
   absent the dry run leaves O not yet produced, and the generation cannot reach acceptance for
   depth-enrolled scope until a dry run has frozen O. This revision does not name the command that
   acquires the surface ahead of the dry run, nor what marks a surface in the manifest root as
   acquired.
7. The depth record states P and O of a create-lane generation. This revision names no second
   committed artifact that states their identities, as a maintenance request's `[support_depth]`
   table does on the maintenance path. A run's input contract is not one:
   [Present behavior](#present-behavior) says why.
8. `runtime-follow-on --write` and `repair-runtime-evidence` make reporting effects only. Each MUST
   NOT list an acceptance entry and MUST NOT change what the depth record states under P or O.
   Adapter code that a write run lands can advertise a capability. On its branch that code is a
   candidate, and the advertising it adds is admitted only with the acceptance entry that
   publication lists under rule 9.
9. `refresh-publication --write` and `close-proving-run` are the create-lane commands that make
   acceptance effects. The support-depth contract's list of acceptance effects and its Annex A.2
   say which effects those are. They include the capability advertising and the qualified or
   depth-qualified results that publication adds for a new agent, the lifecycle stages `published`
   and `closed_baseline`, and a proving-run closeout recorded as `closed` with its evidence id.
   That contract requires each command to establish depth admission and to list an acceptance
   entry in the same change.
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

The support-depth contract's minimum machinery rule requires the failure a new field or mechanism
prevents to be named. Without the version in the approval, a declaration added to the registry
after onboarding began would attach a depth enrollment to a generation whose approval never covered
it, and the depth record's Event would name an approval that says nothing of depth.

### Present behavior

None of this section is implemented. Source references are to `staging` at `f61534be`. Today:

- An approval artifact carries `approval_commit` and `approval_recorded_at`
  (`crates/xtask/src/approval_artifact.rs:246-249`) and no upstream version.
- `onboard-agent --write` scaffolds the manifest root with a `current.json` and placeholder
  directories (`crates/xtask/src/onboard_agent/preview.rs:282-320`). It writes no version
  metadata, no `RULES.json` and no validator spec. The `aider` and `gemini_cli` roots have neither
  of the last two, and the `aider` root's only version is the placeholder `0.0.0`.
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
  The write run is the only create-lane command that writes `snapshots/`, so no create-lane command
  yields a surface that rule 6 admits. A write run lists the paths it changed in its run
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
