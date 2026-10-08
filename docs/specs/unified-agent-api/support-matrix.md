# Support Matrix Spec — Unified Agent API

Status: Approved
Approved (UTC): 2026-04-15
Canonical location: `docs/specs/unified-agent-api/support-matrix.md`

This document is the authoritative contract for support publication semantics in the Unified Agent API spec set.

Normative language: this spec uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Purpose

This spec defines the meaning of published support truth for the Unified Agent API documentation layer.
It separates support publication from capability advertising so the two concerns do not drift together.

## Support layers

Support publication MUST distinguish the following four layers:

- `manifest support`: what the committed CLI manifest evidence says about a target or version.
- `backend support`: what the agent-specific wrapper/backend crate can safely support based on its implementation and manifest inputs.
- `UAA unified support`: what the Unified Agent API can claim as a deterministic cross-agent support statement.
- `passthrough visibility`: backend-specific surface area that remains visible but is not promoted into unified support.

These layers MUST NOT be conflated with workflow status fields, pointer files, or generated overview artifacts.

For onboarding and publication purposes, backend support is crate-first:

- a new agent MAY have committed manifest evidence before any wrapper crate exists
- manifest evidence alone MUST NOT be promoted into backend support
- backend support MUST remain `unsupported` until crate-owned wrapper evidence exists for that target

## Publication targets

The phase-1 publication targets are:

- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`

The JSON artifact is the machine-readable publication surface.
This Markdown document is the normative human-readable projection.

Both publication targets MUST describe the same support model.
If they disagree, the repository is in an invalid publication state.

## Support debt inventory

Published support truth for enrolled automated-maintenance agents MUST stay aligned with:

- `docs/specs/unified-agent-api/non-tui-support-debt.md`

That inventory is the machine-checkable baseline for temporary non-TUI blockers and preexisting
unsupported surface. Publication rules:

- support publication MUST NOT normalize deliberate non-TUI unsupported posture as a permanent
  steady-state note
- any remaining enrolled non-TUI caveat MUST point to a concrete debt-inventory row or a concrete
  external blocker that would also qualify for the maintenance-request deferral taxonomy
- `evidence_notes` MAY explain partial support, but MUST NOT become a shadow debt ledger

## Target-first primacy

Support truth MUST be target-scoped first.
Per-target rows are the primary publication unit.
Per-version summaries, if present elsewhere in the repository, MUST be treated as projections derived from those rows.

Support publication MUST preserve these distinctions:

- a target can be supported even when another target is not.
- a version summary MUST NOT collapse partial target support into a version-global claim.
- pointer state MAY inform publication, but pointer state alone is not support truth.

## Shared support row model

Both publication targets MUST be derived in a single pass from one shared support row model.
Each published row MUST describe exactly one `(agent, version, target)` tuple.
Publication MUST contain exactly one row for every `(agent, version, target)` tuple implied by the committed root set, each root's `current.json.expected_targets`, and each root's committed `versions/*.json` metadata.

The publication row set is exact:

- omitting an implied tuple is invalid publication state
- duplicating a tuple is invalid publication state
- publishing a tuple not implied by committed manifest metadata is invalid publication state

The canonical row field order is:

- `agent`
- `version`
- `target`
- `manifest_support`
- `backend_support`
- `uaa_support`
- `pointer_promotion`
- `evidence_notes`

These fields have the following meanings:

- `agent`: the manifest-root identifier whose committed evidence produced the row.
- `version`: the semantic version associated with the committed version metadata and reports.
- `target`: the root-native target identifier for the published row.
- `manifest_support`: the support state claimed by committed manifest evidence for that target.
- `backend_support`: the backend support state derived from implementation and manifest inputs for that target.
- `uaa_support`: the Unified Agent API support state published for that target after applying the support semantics in this spec.
- `pointer_promotion`: the pointer-derived promotion posture relevant to the row's target.
- `evidence_notes`: deterministic notes that explain intentional partial support or other non-contradictory caveats grounded in committed evidence.

For phase 1, `backend_support` is specifically the support posture proven by the agent-specific wrapper crate and the committed wrapper-derived reports under `cli_manifests/<agent>/reports/**`.
A manifest root without wrapper-derived report evidence MUST publish `backend_support` as `unsupported`.

The shared row model MUST preserve target-scoped truth even when multiple rows share the same version.
Publication code MUST NOT collapse multiple target rows into one version-global support claim.

The canonical row ordering MUST be deterministic:

- rows MUST sort by `agent` ascending
- within an agent, rows MUST sort by `target` ascending
- within an `(agent, target)` grouping, rows MUST sort by semantic `version` descending

`evidence_notes` rules are:

- notes MUST be derived from committed evidence and MUST NOT be speculative
- notes MAY explain intentional partial support, passthrough visibility, or other non-contradictory caveats visible in committed evidence
- notes MUST NOT redefine `validated` or `supported`, and MUST NOT stand in for contradiction policy
- for enrolled automated-maintenance agents, notes MUST NOT be the only place a non-TUI blocker is
  recorded; the blocker MUST also exist in `non-tui-support-debt.md`
- when no such caveat exists, publication MUST treat `evidence_notes` as empty rather than inventing explanatory prose

## Markdown projection boundary

`docs/specs/unified-agent-api/support-matrix.md` is the human-readable Markdown projection of the shared support row model.
It MUST render the same support truth as `cli_manifests/support_matrix/current.json`.

The Markdown projection MAY:

- group rows for readability
- add stable headings and table structure
- restate this spec's already-approved semantics

The Markdown projection MUST NOT:

- derive support truth independently from manifest roots
- introduce row fields or support states that are absent from the shared row model
- collapse target-scoped rows into version-global claims
- become a second mutable support ledger separate from the JSON artifact

## Authority rules

Published support rows MUST be derived from committed repository evidence.

For this spec set:

- manifest evidence is authoritative for manifest support.
- backend implementation evidence is authoritative for backend support.
- Unified Agent API publication text is authoritative for unified support semantics.
- passthrough visibility MUST remain explicit when a backend exposes behavior that is not part of unified support.

For phase 1, backend implementation evidence means wrapper-crate-owned evidence materialized into committed coverage reports.
Manifest metadata by itself MUST NOT be treated as backend support evidence.

The following MUST remain separate from published support truth:

- `validated` and `supported` status fields in version metadata
- generated capability inventory
- runtime backend capability checks

## Neutral root intake

The support-matrix pipeline MUST consume committed evidence from each agent root through one neutral root-intake contract.

For phase 1, that intake contract MUST be limited to these evidence categories under `cli_manifests/<agent>/`:

- `versions/*.json` version metadata
- `pointers/latest_supported/*.txt` and `pointers/latest_validated/*.txt`
- `current.json`
- `reports/**`

This intake contract MUST remain shape-driven rather than agent-name-driven:

- the pipeline MUST reason about root-local evidence categories and paths, not special-case Codex or Claude by name inside shared intake logic.
- the contract MAY preserve root-native target identifiers as loaded evidence; later derivation decides how publication rows compare or project them.
- the contract MUST NOT introduce a second support evidence store outside the committed manifest roots.

This intake contract governs evidence loading only. It MUST NOT change publication targets, support-layer meanings, or the distinction between `validated` and `supported`.

## Validated versus supported

`validated` and `supported` are distinct workflow states.

- `validated` means a version passed the validation matrix and is promotion-grade for the version pointer flow.
- `supported` means wrapper coverage satisfies the stronger support policy for the version and target surface.

A version MAY be manifest-supported before it is backend-supported.
A version MAY be backend-supported before it is promoted into UAA unified support.

The repository MUST NOT treat `validated` as equivalent to `supported`.
The repository MUST NOT treat workflow status as a published support row.
The repository MUST NOT use workflow status as a substitute for target-scoped support evidence.

## Separation from the capability matrix

The support matrix MUST remain separate from `docs/specs/unified-agent-api/capability-matrix.md`.

- The capability matrix documents backend capability advertising.
- The support matrix documents published support truth.
- The two artifacts MAY share source evidence, but they MUST NOT share meaning.
- A change to one artifact MUST NOT be assumed to update the other.

If a reader needs backend capability coverage, they SHOULD use the capability matrix.
If a reader needs published support truth, they MUST use the support matrix.

## Support depth

The [support-depth contract](../support-depth-contract.md) owns the rules for support depth: what a
depth record states, when a result is published as `verified` or as `unverified`, when a promise is
qualified and a capability depth-qualified, and which changes to a published row need depth
admission. This section states only what this spec owns: which rows carry depth facts, where those
facts are read from, what such a row states, how its `uaa_support` is derived, and the revision of
the JSON artifact that carries them. It uses that contract's terms and redefines none of them.

While the support-depth contract is a Draft this section binds nothing.

Rules:

1. A row is **depth-enrolled** when, in the tree being published, either of these holds: the
   depth record of the row's agent and version covers an invocation mode on the row's target, or
   the agent's registry entry declares a depth enrollment of the row's version that lists the
   row's target. Every other row is a **row without depth facts**. It keeps the fields, the
   meanings and the derivation that the sections above give it, and it carries no marker.
2. Whether a row is depth-enrolled is read as rule 1 states. Every other depth fact is read from
   depth records. A depth record is committed evidence under `reports/**`, which
   [Neutral root intake](#neutral-root-intake) already lists, and this section adds no evidence
   category. Publication MUST NOT decide a result, a qualification or a capability's depth
   qualification from the generated capability inventory, from a backend's advertised capability
   set, from a registry declaration, from a working file or from the lifecycle record. A
   capability id reaches a row only through the capability mappings of the row's depth record.
   Where the support-depth contract has a published result depend on evidence the record binds,
   publication reads that evidence to establish its content identity and for nothing else. Such a
   file may lie outside the intake categories. Reading it for its content identity loads no
   support evidence, and it is the one read this section makes outside those categories.
3. A depth record of a version that has committed version metadata is invalid publication state
   when it cannot be read, states a schema revision other than the current one or does not follow
   Annex B of the support-depth contract. Publication MUST fail. It MUST NOT publish that
   version's rows as rows without depth facts.
4. For a version that has committed version metadata, a target that the version's depth record
   covers, or that a registry declaration of the version lists, implies a row for that agent,
   version and target, whether or not `current.json.expected_targets` still lists the target.
5. In the next revision of the JSON artifact the row model includes, for a depth-enrolled row and
   its target:
   - each promise of the record's selection that serves a covered mode on the target, with every
     obligation of that promise and the obligation's published result;
   - for each such promise, whether it is published as qualified, and each outcome that applies to
     it among those the support-depth contract's Publication section requires reports to keep
     distinguishable. More than one of them can apply to a promise, and it can be that none does.
     The qualified forwarding outcome is stated only for a promise that is published as qualified;
   - each covered mode that the record's policy excludes, as a mode exclusion with the published
     result of its obligation; and
   - each capability that the record's mappings claim, and whether it is published as
     depth-qualified for the target.

   The support-depth contract's record invariants and depth admission predicate decide each of
   these values, and this spec adds no case to them. The JSON artifact and the Markdown projection
   both state every obligation's result. That contract says what may not stand in for them. A
   depth-enrolled row for which no record states anything yet, because no generation has frozen
   its policy for the row's target, states that it is depth-enrolled and that its obligations are
   not yet frozen, which that contract reports as insufficient depth.
6. For a depth-enrolled row whose record claims at least one capability, `uaa_support` is derived
   as the sections above derive it, with one further condition: the row is `supported` only when,
   in addition, every capability the record claims is published as depth-qualified for the row's
   target. Where that derivation yields `supported` and the condition does not hold, the row is
   `partial`. Depth facts never raise `uaa_support` above what that derivation yields, and they
   lower no state other than `supported`. A capability the record does not claim is not assessed,
   and the state says nothing about it. Every other depth-enrolled row derives `uaa_support` as a
   row without depth facts does, and every other field of every row keeps its derivation.
7. A change that makes a depth-enrolled row `supported` because a capability comes to be published
   as depth-qualified is part of that publication. The support-depth contract makes it an
   acceptance effect and governs it. This spec adds no condition of its own.
8. `evidence_notes` MUST NOT carry a depth fact, and authorized debt stays where
   [Support debt inventory](#support-debt-inventory) puts it. Neither is a second place for the
   facts of rule 5.
9. `schema_version` 1 of the JSON artifact is the row model of the sections above. The facts of
   rule 5 and the derivation of rule 6 belong to the next revision of the artifact. That revision
   is an executable schema revision of this spec, among those the support-depth contract's path
   enablement requires to be adopted, and this revision of the spec does not define its fields.
   In it, the exact row set and the row model are those of the sections above as rules 4 and 5
   extend them, and a row without depth facts has exactly the fields and the values it has in
   revision 1. A reader of the artifact MUST require the revision it supports, and it MUST NOT
   read an artifact of another revision as one that holds no depth-enrolled row.
10. [Separation from the capability matrix](#separation-from-the-capability-matrix) holds for
    depth-enrolled rows as well. A row's capability-level fact says whether a capability is
    depth-qualified. It never says that a backend advertises the capability, and the capability
    matrix never says that a capability is depth-qualified. A change to one artifact is still not
    assumed to update the other.

The support-depth contract's minimum machinery rule requires the failure a new field or mechanism
prevents to be named. Without the row facts of rule 5, the `supported` state of a depth-enrolled
row would not say which capabilities were assessed, and that contract requires a qualified subset
to be named by its exact scope. Without the condition of rule 6, a row would stay `supported` on
name coverage alone while a capability its depth enrollment claims is unverified. Without rule 4,
removing a target from `expected_targets` would remove a depth-enrolled row, and with it the
published results of that target. Without the revision of rule 9, a reader written for revision 1
would take a `uaa_support` that depends on depth facts for one derived from manifest and backend
support alone.

### Present behavior

None of this section is implemented. Source references are to `staging` at `f61534be`. Today:

- No manifest root holds a depth record, so no row is depth-enrolled.
- `uaa_support` is derived from `manifest_support`, `backend_support` and whether the row has
  evidence notes (`crates/xtask/src/support_matrix/derive.rs:704-723`). The derivation reads no
  capability.
- Root intake opens a version's coverage reports by exact file name
  (`crates/xtask/src/support_matrix/derive.rs:575-600`). It would not open `depth-record.json`.
  It reads pointer files only for the targets `expected_targets` lists
  (`crates/xtask/src/support_matrix/derive.rs:471`).
- The generator loads the agent registry to find the manifest roots it publishes
  (`crates/xtask/src/support_matrix/derive.rs:258-269`) and reads nothing else from it.
- The JSON artifact is written with `schema_version` 1
  (`crates/xtask/src/support_matrix/publication.rs:20-31`). Inside a workspace,
  `manifest-validate` requires that value (`crates/xtask/src/manifest_validate.rs:635-652`).
- `uaa_support` has three readers besides the Markdown projection. The consistency check behind
  `support-matrix --check` and `manifest-validate` compares it with the derived state
  (`crates/xtask/src/support_matrix/consistency.rs:362-366`). The `markdown_support_claim`
  governance check compares a claimed `uaa_support` with the currently derived rows
  (`crates/xtask/src/agent_maintenance/drift/governance.rs:436-520`). The drift check's
  publication comparison parses the JSON artifact without checking `schema_version`, compares whole
  rows with the derived ones, and compares the published Markdown with its own copy of the row
  rendering (`crates/xtask/src/agent_maintenance/drift/publication.rs:211-277`,
  `crates/xtask/src/agent_maintenance/drift/shared.rs:134-162`). For a depth-enrolled row each of
  them would read the state that rule 6 yields.

## Verification checklist

Before downstream work consumes this contract, reviewers MUST confirm:

- the canonical support publication targets are named exactly once and without ambiguity.
- the four support layers have distinct meanings and no overlap with workflow metadata.
- target-scoped rows are described as primary and per-version summaries as derived projections.
- the neutral root-intake contract is limited to committed root evidence and does not introduce agent-name-specific loading semantics.
- backend support is explicitly crate-first and cannot be claimed from manifest metadata alone.
- `validated` is not treated as equivalent to `supported`.
- the support matrix is explicitly separate from the capability matrix.
- the spec is sufficient for downstream implementation without reopening authority or output-path decisions.

## Change control

Any future update to support publication semantics MUST update this spec first.
If the publication model changes, the README index and any dependent publication docs MUST be updated in the same change.

## Published support matrix

This section is generated by `cargo run -p xtask -- support-matrix`.
Do not edit this section by hand.

<!-- support-matrix-published:start -->
### `aider`

| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |
|---|---|---|---|---|---|---|---|
| `aider` | `0.0.0` | `darwin-arm64` | `supported` | `unsupported` | `unsupported` | `latest_supported_and_validated` | — |

### `claude_code`

| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |
|---|---|---|---|---|---|---|---|
| `claude_code` | `2.1.274` | `darwin-arm64` | `supported` | `partial` | `partial` | `latest_supported_and_validated` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `claude_code` | `2.1.29` | `darwin-arm64` | `supported` | `supported` | `supported` | `none` | — |
| `claude_code` | `2.1.274` | `linux-x64` | `supported` | `partial` | `partial` | `latest_supported_and_validated` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `claude_code` | `2.1.29` | `linux-x64` | `supported` | `supported` | `supported` | `none` | — |
| `claude_code` | `2.1.274` | `win32-x64` | `supported` | `partial` | `partial` | `latest_supported_and_validated` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `claude_code` | `2.1.29` | `win32-x64` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support |

### `codex`

| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |
|---|---|---|---|---|---|---|---|
| `codex` | `0.156.1` | `aarch64-apple-darwin` | `supported` | `partial` | `partial` | `latest_supported_and_validated` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.155.0` | `aarch64-apple-darwin` | `unsupported` | `unsupported` | `unsupported` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.144.6` | `aarch64-apple-darwin` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.129.0` | `aarch64-apple-darwin` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.125.0` | `aarch64-apple-darwin` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.97.0` | `aarch64-apple-darwin` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.92.0` | `aarch64-apple-darwin` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.91.0` | `aarch64-apple-darwin` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.61.0` | `aarch64-apple-darwin` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.156.1` | `aarch64-unknown-linux-musl` | `supported` | `partial` | `partial` | `latest_supported_and_validated` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.155.0` | `aarch64-unknown-linux-musl` | `unsupported` | `unsupported` | `unsupported` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.144.6` | `aarch64-unknown-linux-musl` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.129.0` | `aarch64-unknown-linux-musl` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.125.0` | `aarch64-unknown-linux-musl` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.97.0` | `aarch64-unknown-linux-musl` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.92.0` | `aarch64-unknown-linux-musl` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.91.0` | `aarch64-unknown-linux-musl` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.61.0` | `aarch64-unknown-linux-musl` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.156.1` | `x86_64-pc-windows-msvc` | `supported` | `partial` | `partial` | `latest_supported_and_validated` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.155.0` | `x86_64-pc-windows-msvc` | `unsupported` | `unsupported` | `unsupported` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.144.6` | `x86_64-pc-windows-msvc` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.129.0` | `x86_64-pc-windows-msvc` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.125.0` | `x86_64-pc-windows-msvc` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.97.0` | `x86_64-pc-windows-msvc` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.92.0` | `x86_64-pc-windows-msvc` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.91.0` | `x86_64-pc-windows-msvc` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.61.0` | `x86_64-pc-windows-msvc` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `codex` | `0.156.1` | `x86_64-unknown-linux-musl` | `supported` | `partial` | `partial` | `latest_supported_and_validated` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.155.0` | `x86_64-unknown-linux-musl` | `unsupported` | `unsupported` | `unsupported` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.144.6` | `x86_64-unknown-linux-musl` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes passthrough surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.129.0` | `x86_64-unknown-linux-musl` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.125.0` | `x86_64-unknown-linux-musl` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.97.0` | `x86_64-unknown-linux-musl` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.92.0` | `x86_64-unknown-linux-musl` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.91.0` | `x86_64-unknown-linux-musl` | `unsupported` | `unsupported` | `unsupported` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |
| `codex` | `0.61.0` | `x86_64-unknown-linux-musl` | `supported` | `partial` | `partial` | `none` | backend report includes intentionally unsupported surface outside unified support; backend report includes backend-only surface outside unified support |

### `gemini_cli`

| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |
|---|---|---|---|---|---|---|---|
| `gemini_cli` | `0.38.2` | `darwin-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |

### `opencode`

| agent | version | target | manifest_support | backend_support | uaa_support | pointer_promotion | evidence_notes |
|---|---|---|---|---|---|---|---|
| `opencode` | `1.14.47` | `darwin-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.4.11` | `darwin-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | current root snapshot omits this target |
| `opencode` | `1.4.9` | `darwin-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.14.47` | `darwin-x64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.4.11` | `darwin-x64` | `unsupported` | `unsupported` | `unsupported` | `none` | current root snapshot omits this target |
| `opencode` | `1.4.9` | `darwin-x64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.14.47` | `linux-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.4.11` | `linux-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | current root snapshot omits this target |
| `opencode` | `1.4.9` | `linux-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.14.47` | `linux-x64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.4.11` | `linux-x64` | `supported` | `unsupported` | `unsupported` | `latest_validated` | — |
| `opencode` | `1.4.9` | `linux-x64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.14.47` | `win32-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.4.11` | `win32-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | current root snapshot omits this target |
| `opencode` | `1.4.9` | `win32-arm64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.14.47` | `win32-x64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
| `opencode` | `1.4.11` | `win32-x64` | `unsupported` | `unsupported` | `unsupported` | `none` | current root snapshot omits this target |
| `opencode` | `1.4.9` | `win32-x64` | `unsupported` | `unsupported` | `unsupported` | `none` | — |
<!-- support-matrix-published:end -->
