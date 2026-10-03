# P1a dual review — lead adjudication (2026-10-02)

- Candidate reviewed: `e60664e2` (`docs/specs/support-depth-contract.md`, Draft), base `2797b8d9`.
- Lanes: `opus-xhigh-adversarial-reviewer` (claude-opus-5-5, xhigh) and `sonnet-max-defect-reviewer`
  (claude-sonnet-5-5, max). Maintainer override of CLAUDE.md: no Codex; lead remediates.
- Results: Opus not CLEAN (3 P1, 10 P2, 7 P3). Sonnet not CLEAN (0 P1, 8 P2, 13 P3 groups).
- Method: the lead re-read the cited source for every factual claim before deciding. All factual
  claims checked held (retain deletes `reports/<v>`; promotion lands by PR to `staging`;
  per-lane concurrency groups; nightly regeneration with fresh `request_recorded_at`; closeout
  requires a marker; MCP isolation is SHOULD with a Claude `.mcp.json` caveat; no normative owner for
  runtime profiles; version statuses `reported/validated/supported`; existing "admission gate" term;
  `parity_exclusions` in RULES.json; onboarding `approval_commit`/`approval_recorded_at`).

## Decisions

Duplicates across lanes are merged under one id. `L` marks the lead's own route-pass observations.

| Id | Finding | Decision | Rationale and fix |
| --- | --- | --- | --- |
| A1 (O-P1-1, L-2) | Depth evidence carrier (coverage reports) is not a gated output; `manifest-retain` deletes it | Accept | Add depth results and their evidence, existence included, to the gated outputs; forbid removing evidence a published output relies on. |
| A2 (O-P1-2, S-P2-4) | Reporting effects gated by authority and currency; truthful downgrade refused | Accept | Reporting needs only resolution and truthful content; stale results are written `unverified`; authority, currency and due-work items apply to acceptance effects only. Unresolved tuples are errors reported by validation. |
| A3 (O-P1-3, L-1) | Final admission ends at the working-tree write; effects land at PR merge | Accept | A merge into the integration branch is a route; the gated operation ends at the merge; admission including currency must hold for the merge result. |
| A4 (O-P2-1, S-P2-1) | Serialization ownership scope undefined; per-lane locks satisfy the text | Accept | Define one serialization domain over overlapping outputs, tuples and admission inputs; define "invalidating change"; "demonstrated" means an Annex A entry plus a conformance test. |
| A5 (O-P2-2, S-P2-3 part) | "Result for a depth scope tuple" undefined; markers over-included | Accept | Per-output table of the values that belong to a tuple; marker retirement only, declaration excluded. |
| A6 (O-P2-3, S-P2-3) | Later non-enrolled version replacing an enrolled pointer deadlocks | Accept | New Supersession rule: reporting effect for the superseded tuple; new version admitted under its own rules; superseded results, acceptance and closeout records never rewritten. |
| A7 (S-P2-2) | Effect kinds not total; status flips and stage commits unnamed | Accept | Enumerate acceptance effects; everything else is reporting. |
| A8 (O-P2-4) | Capability gating leaks to capabilities the enrollment does not claim | Accept | Gate only capabilities a depth enrollment claims; table says admission is added for them. |
| A9 (O-P2-5, S-P3) | Generation = run conflicts with nightly regeneration and version-keyed enrollment | Accept | Generation is one packet generation per Event; depth enrollment keyed to agent, path, exact version and covers every generation of it. |
| A10 (O-P2-6, S-P2-7) | O re-freeze authorizer unnamed | Accept | Only the existing acquisition and preparation path freezes or re-freezes O; it cannot change agent, version, targets or enrollment; never the executor. |
| A11 (O-P2-7) | Classification authority unassigned | Accept | Maintainer decides classification (P); independent adequacy review decides boundary evidence; executor neither. |
| A12 (O-P2-8, S-P3) | Debt rules vs name-level inventory; "obligation coverage" field; retirement paths | Accept | Debt rows are name-level and disposition no depth obligation unless the debt contract is amended; retirement follows the request contract; renewal keyed to `blocker_class`; define frozen delegation and baseline. |
| A13 (O-P2-9, S-P3) | Validation backstop keyed on output's own enrollment; hand edits | Accept | Validation compares every depth enrollment's gated state with its records whatever produced the change. |
| A14 (O-P2-10, S-P2-5) | "Validated reuse binding" undefined; section reference ambiguous | Accept | Define the reuse binding in the contract; pre-enrollment evidence only through it. |
| A15 (S-P2-6) | "Does not change" owners misattributed | Accept | Correct owners; runtime profiles have no normative owner; promotion is the charter's; drop the single-owner claim. |
| A16 (S-P2-8) | EXCLUDED rule contradicts the research example | Accept | "MUST NOT extend to a headless mode of the same command or to the command as a whole." |
| A17 (O-P3, S-P3) | Term collisions: "protected", "admission", "obligation", "template", "enrolled", unqualified enrollment, `O` dimension vs binding, "candidate", "selector", lowercase "may" | Accept | Rename to "depth-gated" and "depth admission"; Terms notes for obligation and template; dimension rows without ids and template obligation ids `OUT*`/`LC*`; qualified selectors; RFC keyword fix. |
| A18 (O-P3, S-P3) | BASE wrongly includes O and L; template cells lack ids; LAUNCH/MANAGED lack owned-I/O | Accept | BASE = R1, V1, G1, E1, A1, X1–X4; obligation table `OUT1–4`, `LC1–4`, `FX1`, `S1`, `N1`, `B1`, `MCP1`, `EX1`. |
| A19 (O-P3, S-P3) | Onboarding Event; lanes without O; reprepare path for stood-down packets | Accept | Onboarding Event = `approval_commit`/`approval_recorded_at`; lanes without O cannot reach acceptance; continuation is a new generation. |
| A20 (O-P3) | Path enablement inside P invalidates other paths | Accept | Enabling another path does not change P. |
| A21 (O-P3, S-P3) | Onboarding proof not isolated; additive gate stricter than research; refusal scope; "existing" approval authority | Accept | Isolated onboarding proof; additive waits for "proven and landed"; refusal limited to depth-enrolled tuples; "existing, explicitly bounded". |
| A22 (O-P3, S-P3) | MCP template turns SHOULD into MUST; rule 5 not tied to exposed flows nor materialized | Accept | `MCP1` defers to the MCP spec with its caveats; rule 5 covers every exposed flow and the mapping is published with depth results. |
| A23 (S-P3) | Final admission leftovers: restoration precedence, "reads as accepted", "a caller" | Accept | Restoration that would overwrite unowned state does not run; interruption leaves no uncovered acceptance effect; route-scoped wording. |
| A24 (S-P3) | Evidence nits: build graph term, producer list, favorable run | Accept | "full resolved build graph"; producer recognized by existing trust rules; no silent favorable-run selection. |
| A25 (S-P3) | Overrides can erase promises, no expiry | Accept | Add limits and an expiry or review condition. |
| A26 (S-P3) | Shared evaluator not established; Scope/Purpose omissions | Accept | One shared implementation for both paths; Purpose and Scope updated. |
| A27 (S-P3) | Conformance table diverged from rules | Accept | Replace with one sentence pointing to the proposal's acceptance cases as non-normative guidance. |
| A28 (S-P3) | EXCLUDED vs `parity_exclusions` (second ledger risk) | Accept | No second exclusion list; mode exclusion never removes a unit from name-level accounting. |
| A29 (S-P3) | "approved request"; "each applicable obligation"; enrolled meanings list | Accept | Wording fixes. |
| R1 (O-P3, S-P3) | MCP spec is a Draft | Reject | The approved capabilities spec already names it as owner of MCP semantics; citing the owner document is correct. |

## Next

Remediate in one revision, re-run both lanes in remediation-follow-up mode on the new candidate,
then final once-over. Re-review is required because A2–A7 change the safety rules themselves.

## Round 2 (remediation-follow-up on `f495a750`)

Results: Opus not CLEAN (3 P2, 4 P3, 1 deferred). Sonnet not CLEAN (2 P2, 11 P3 groups). Every P2
was introduced by the round-1 delta. Lead verified: per-agent request path
(`agent_lifecycle.rs` `maintenance_request_path`), per-root closeout path (`contract_policy.rs`),
closeout replaced across versions at one path (`afcfed2b` over `93e771d7`), `MaintenanceCloseout`
has no open/closed state, `close-agent-maintenance` edits lifecycle evidence and the `Drifted` side
state, `RuntimeProfile` serializes `feature_rich`.

| Id | Finding | Decision | Fix |
| --- | --- | --- | --- |
| B1 (O2-P2-1, S2-P3-2) | Merge-route ownership spans the PR lifetime; read-overlap puts every route in one domain | Accept | The gated sequence of a merge route is the integration step alone; branch writes prepare a candidate; a fast-forward-only push or exact-merge re-check satisfies it; domains from write-write and write-read pairs only. |
| B2 (O2-P2-2) | In-place evidence overwrite is not gated | Accept | Evidence content identity is a tuple value; evidence an accepted result derives from may be neither removed nor replaced while a published value derives from it. |
| B3 (O2-P2-3, S2-P3-10) | "Records a closeout as closed" fits no maintenance closeout; lifecycle row onboarding-only | Accept | Acceptance by effect: creating or changing a maintenance closeout, proving-run `closed`, lifecycle stage, closeout evidence entry or cleared drift; lifecycle row path-neutral; runtime projection in the list. |
| B4 (S2-P2-1) | Per-root request and closeout are working files replaced by the next version | Accept, option (a) | Define the per-version depth record (results, P/O/E references, mappings, acceptance identities) stored with the depth results; resolution falls back to it; a later version's replacement of working files is a reporting effect; validation compares against the depth record and defines contradiction. |
| B5 (S2-P2-2) | "Unresolved" refusal swallows `classification_required` and missing evidence | Accept | Refusal only for depth enrollment resolution failures; classification required is published and blocks readiness and acceptance; missing evidence is `unverified`. |
| B6 (O2-P3, S2-P3) | Currency wording at the merge; root pointer and `current.json`; "Supersession" collision; undefined records and contradiction; unqualified selectors; B1 "route"; "operation" in gated sequence; "acceptance output/scope"; P/O/E forward reference; `M` in the set formula; debt reason vs `blocker_class` and no depth deferral mechanism; override lapse; mode exclusion in P; pre-enrollment evidence; generation vs `--from-request`; "partial"; "outstanding"; onboarding O freeze and continuation; `feature_rich`; roots without RULES.json; redundant partial-retirement sentence | Accept | Wording and definition fixes as listed; onboarding O freeze point deferred to the charter amendment; pre-enrollment evidence stated inadmissible. |
| D1 (O2-deferred) | `manifest-retain` deletes a pending (not accepted) version's evidence | Defer | Results stay truthful (`unverified`). Trigger: the Annex A entry for the retain route and its P2 enforcement. |

## Round 3 (remediation-follow-up on `dd14fb32`)

Results: Opus not CLEAN (4 P2, 3 P3; B3, B5, B6 fixed). Sonnet not CLEAN (5 P2, 6 P3; B1, B5 fixed).
All P2s are delta-introduced or partial root causes. Lead verified: `manifest-validate` re-checks
reports of every reported/validated/supported version against current `RULES.json`
(`manifest_validate/versions.rs`), exclusions apply only at report generation, acquisition rewrites
per-root `wrapper_coverage.json`; one lifecycle record per agent; `runtime-follow-on` sets
`runtime_integrated` and `prepare-publication` sets `publication_ready`; the charter defines no
onboarding O freeze point.

| Id | Finding | Decision | Fix |
| --- | --- | --- | --- |
| C1 (O3-F1) | Whole-file evidence freeze blocks adding parity exclusions | Accept | Protect evidence entries that belong to the tuple; regeneration that leaves them unchanged is permitted; version-agnostic inputs are dependencies, not evidence. |
| C2 (O3-F2, S3-F4) | Evidence protection is not a refusal condition nor a contradiction | Accept | Predicate item for all effects; contradiction clause for changed accepted evidence. |
| C3 (O3-F3) | Depth record holds only references; resolution has nothing complete after displacement | Accept | Record carries the resolved selection and P/O content identities; replacing captured working files is not a deleted binding. |
| C4 (O3-F4, S3-O1) | Tip compare-and-refuse at merge contradicts serialize-first | Accept | The integration step's tip-conditional ref update is its serialization; D13 governs the other gated sequences. Consistent with D13, not a reversal. |
| C5 (S3-F1) | Lifecycle stage and per-agent record make routine execution and other versions' tuples acceptance effects | Accept, modified | Acceptance stage advances limited to `published`/`closed_baseline`; the lifecycle row belongs only to the writing route's generation's tuples. No refusal clause for other tuples: their acceptance lives in their depth records. |
| C6 (S3-F2, O3-P3) | Depth record onset undefined | Accept, modified | Record written no later than O's freeze; the missing-record contradiction applies only once O is frozen; resolution before a record exists comes from registry authority plus the generation's frozen request or approval. Rejected part: refusing displacement of a record-less tuple (nothing is depth-gated before O's freeze; same-version regeneration continues the enrollment). |
| C7 (S3-F3) | Flat pre-enrollment ban contradicts cross-version reuse | Accept | Evidence admissible only if produced under some depth enrollment's P and O; another version only through a reuse binding under a compatibility rule. |
| C8 (S3-F5) | Contract says the charter defines the onboarding O freeze; it does not | Accept | State the charter must be amended to define it; onboarding enablement prerequisite. |
| C9 (O3-P3, S3-O2–O6) | Wording and definitions | Accept | "Established earlier" wording; `classification_required` in the predicate; qualified "depth records"; no-O scope as insufficient depth; missing evidence and contradictory runs `unverified`; working-file location; depth record is not an inventory or ledger; debt intro order. |
| D2 (O3-deferred) | Admission checker runs from the candidate's own tree | Defer | Pre-existing, inside the stated trust boundary. Trigger: P2 design of the merge-time check. |

## Round 4 (remediation-follow-up on `5ff62ab8`) — loop paused for the maintainer

Results: Opus not CLEAN (2 P2, 5 P3). Sonnet not CLEAN (2 P2, 7 P3). No P1. All P2s are
delta-introduced. Opus: C1, C3, C4, C5, C7, C8, C9 fixed; C2 partial; C6 regressed. Sonnet: C1–C9
fixed on their face, with two regressions.

| Id | Finding | Decision | Fix (applied in `d78cf535`) |
| --- | --- | --- | --- |
| E1 (O4-F1) | A reporting effect can narrow a depth record and defeat evidence protection | Accept, refined | A record is never removed and a listed acceptance never removed or altered (predicate item 3, all effects). What a record states changes only as the binding concerned permits, and every such change shows in the P, O or E identity it states. An acceptance names the identities it accepted and supports a published qualified value only while the record still states them. Validation compares a proposed branch's record with the integration branch's copy. Refinement over the proposed "acceptance effect": a truthful re-verification must stay possible without closeout authority, so the control is identity-bound support rather than authority. |
| E2 (S4-F1) | "Evidence entry" undefined and undecidable: coverage reports list only gaps, so a covered unit has no entry; collides with lifecycle evidence entries; the closeout would count as evidence; the record holds no per-entry identities | Accept | The term is gone. The depth record is one committed file per version under `reports/<version>/` and binds each `verified` result to its evidence by content identity. Coverage reports and `wrapper_coverage.json` are not depth-gated outputs. Changing bound evidence is not prohibited: the result is published `unverified`. Lifecycle uses renamed "evidence id". |
| E3 (O4-F2) | A depth-enrolled version whose O never freezes has no record and is silently un-enrolled when a later packet replaces the working files | Accept | The record is first written when the first generation freezes P (the request's first freeze on the maintenance lane; charter-defined on the create lane) and extended as O, E and acceptances follow. A frozen P for depth-enrolled scope with no record is a contradiction. Resolution comes from the record. Bindings rule 8 now requires the bindings established so far. |
| E4 (S4-F2) | Lifecycle row attributed by actor; merges, hand edits and multi-agent backfill cannot be attributed | Accept, refined | State-based instead of "same change": a `published` stage reports the publication packet the lifecycle record names, `closed_baseline` and the proving-run id the closeout it names, the maintenance id and absent drift the agent's maintenance closeout record. A stage or closeout id whose reported record the revision does not hold is a contradiction. Refinement: `refresh-publication` does not rewrite the packet, so "same change" would misfire when prepare and refresh merge separately. Lead verified all five agents' lifecycle records resolve under this rule today. |

P3 observations folded into the same commit: Event stated in the record; later generation of the
same version continues the record; "deleted binding" pointer moved to Depth enrollment rule 8 and
worded as identities; mis-scoped missing-record check; tip-conditional ref update made a MUST and
tied to items 1 to 3; admissibility limited to evidence that verifies an obligation of a production
depth enrollment; "under investigation" replaced by "unresolved"; the charter prerequisite stated
once (Path enablement) and widened to the P freeze point.

| D3 (O4-P3) | Acceptance for a version whose working files were replaced has only the record's identities for P and O | Defer | Fails closed or admits from the record; no depth claim is forged either way. Trigger: Annex A's promotion entry, where V+1 opening between V's closeout and V's promotion is reachable. Pointer left in the route-facts dossier. |

Lead-verified facts for this round: `manifest-retain` removes `reports/<v>` wholesale for versions
outside the keep set and no workflow calls it (`manifest_retain.rs:97-126`); report-presence checks
look for coverage files, not the directory (`audit_status/evidence.rs:34-51`);
`refresh-publication` sets `published` and the packet reference and clears the closeout baseline
(`publication_refresh.rs:439-467`); `close-proving-run` sets `closed_baseline`, both references and
clears drift (`close_proving_run.rs:447-474`).

Trend of distinct P2s by round: about 13 (plus 3 P1), 5, 8, 4. E2 identifies the root cause behind
most second-order findings since round 2.

## Round 5 (bounded delta review of `5ff62ab8..d78cf535`)

Maintainer instruction 2026-10-03: land the fix, then narrow reviews of only the changes in it.
Both lanes dispatched on the delta alone. Results recorded below when they return.

### Opus lane (returned first): NOT CLEAN, 2 P2, both delta-introduced, no P1

E1 fixed as to compliance, E2 fixed, E3 fixed, E4 fixed. Lead verified: nightly re-dispatch of an
unchanged version is real (`uaa-0048`, PRs #211 and #208 carried only the previous night's commits);
`MaintenanceCloseout` names `request_ref` and `request_sha256` and no version
(`closeout/types.rs:71-85`).

| Id | Finding | Decision | Proposed fix (not yet applied) |
| --- | --- | --- | --- |
| F1 (O5-F1) | The "same P, O and E identities, nothing else may change" contradiction fires when a later generation restates only the Event: a P-only record on staging plus a nightly re-dispatch with unchanged policy. Validation fails until the maintainer changes P. | Accept. The lead's self-check missed the case where O and E are absent in both copies. | Drop the same-identity comparison. Replace it with: the record's Event, P and O are those of the frozen request or approval the same revision holds for its version; when the revision holds none, what the record states does not change against the tip it replaces. State that a later generation's opening leaves O, E and results not yet established. |
| F2 (O5-F2) | Nothing checks that a stated identity is the identity of what it names, so two merges launder a rebinding: restate E, drop `qualified`, then restate the old E with the new evidence and republish `qualified` under the genuine acceptance. Variant: drop a tuple and restate P. | Accept | Each listed acceptance names the content identity of everything the record stated when it was granted, the acceptance list aside. That identity is recomputable from the record, so it survives replaced working files. A qualified claim is supported only while the record's stated content has that identity. The selection variant is closed by the request-agreement rule above. |

P3 (accept unless noted): comparison base is "the merge result against the tip it replaces";
bound evidence is committed content named by path and identity (restores what the base text
assumed); "a published value" narrowed to qualified and depth-qualified; the generation of a
reported record is found through the request it names or, once replaced, the depth record that
lists the acceptance it granted; the packet and closeout references count as lifecycle values;
"items 1 to 3" disambiguated; "unresolved" collides with the Unresolved outcome; the admissibility
tail sentence gets the production qualifier; the create-lane P freeze must fall where the manifest
root and exact version exist; record versus the "status ledger" prohibition worded honestly.
Noted, no action: coverage reports join the dependency class whose change no contradiction detects
(pre-existing class, lazy invalidation); drift absence attributed to the previous closeout while a
newer generation is open; a hand-listed acceptance that nothing granted (pre-existing, inside the
review trust boundary, and an acceptance effect at the integration step).

### Sonnet lane: NOT CLEAN, 2 P2, no P1

E1 to E4 all fixed. Its F1 is the same defect as Opus F1 (Event restatement misfires the
same-identity comparison). Lead verified: the publication packet names `approval_artifact_path`
and sha, the proving-run closeout names `approval_ref` and sha, the maintenance closeout names
`request_ref` and sha.

| Id | Finding | Decision | Proposed fix (not yet applied to the candidate) |
| --- | --- | --- | --- |
| F1 (S5-F1 = O5-F1) | See Opus F1. | Accept | As Opus F1. |
| F3 (S5-F2) | "The acceptance that granted it" cannot be decided for pointer, status, marker, closeout, lifecycle and advertising effects: an entry does not name the effects it covers. "Supports a published value" has two readings, one of which rolls back pointers after an evidence refresh. The bullet applies to "effects" on the integration branch, where there is no diff. | Accept | Contradictions are split into those decided in any revision and those decided by a merge result against the tip it would replace. An acceptance effect in a merge result needs an acceptance entry naming the content identity of what the record states there; no effect-to-entry link is needed. Support is limited to qualified and depth-qualified. |

P3 (accepted): "agents concerned" defined; published results must equal stated results, with
`verified` to `unverified` the one exception, and a selected tuple may not be published as not
depth-enrolled; "the first generation that freezes P"; "acceptance entry" replaces the overloaded
"acceptance" and "granted"; lifecycle evidence ids named literally; "packet or closeout" instead of
bare "record"; Bindings rule 8 reworded without "established"; "contradict each other" instead of
"unresolved"; resolution sentence shortened.

### Round-five summary and state

Distinct P2s: 3, all delta-introduced, all in the record-integrity and validation text: O5-F1/S5-F1,
O5-F2, S5-F2. Trend by round: about 13 (plus 3 P1), 5, 8, 4, 3. Both lanes confirm E1 to E4 fixed.

- Reviewed candidate: `d78cf535` on `docs/support-depth-p1-contracts` (unchanged).
- Unreviewed proposal for F1 to F3 and the accepted P3s: `2a956def` on
  `docs/support-depth-p1-contracts-r5-proposal` (one file, 88 insertions, 69 deletions against
  `d78cf535`). The candidate worktree is checked out on this branch.
- Per the stopping rule agreed with the maintainer, no further review round was dispatched. The
  maintainer decides: fast-forward the candidate to the proposal and run one more bounded review of
  `d78cf535..2a956def`, or take the draft for review.
- Lead observation for that decision: rounds three to five found defects only in the mechanical
  validation backstop for hand edits (Complete mediation rule 7 and the record's integrity rules).
  If another round is not clean, an alternative is to keep rule 7 as a principle in this contract
  and move its enumerated contradictions to the manifest-root validator spec, where they are
  written beside the record's schema.

### After round five (2026-10-03)

The maintainer asked why the validation rule is written in prose instead of code. The lead's
recommendation changed: do not run another prose round on Complete mediation rule 7; cut it back
to invariants, carry the ten enumerated checks and the reviewers' attack scenarios forward as the
P2 test list, and run one bounded review of that smaller change. The maintainer has not answered.
The session then stopped for a machine switch; the state is in
`.codex/handoffs/2026-10-03-101515-support-depth-p1a-machine-switch.md`, and the verbatim reviewer
reports for all five rounds are in `support-depth-p1a-review-reports/`.
