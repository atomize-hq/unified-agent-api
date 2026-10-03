# round3-sonnet: reviewer report

Lane: Sonnet 5.5 max, defects and conformance. Candidate: `dd14fb32`. Scope: remediation follow-up.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

Candidate `dd14fb32` (base `f495a750`) is **not CLEAN**. There is no P1. There are five P2 findings, all introduced by the f495a750→dd14fb32 delta.

Contract: `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-acbdf1ea111416778/docs/specs/support-depth-contract.md`. Line numbers are at `dd14fb32`.

## Findings

### F1 (P2): the lifecycle-record row and the acceptance bullets are unbounded in two ways
- **Lines:** 469 (row) and 480-482 (bullets), interacting with 500-504, 514-518, 524-525, 539-547 and 347.
- **Text:**
  - Line 469: "Its stage, side states and evidence entries for the tuple's agent".
  - Line 481: "advances the lifecycle record's stage, adds a closeout evidence entry or clears a drift side state".
  - The base named only "advances the lifecycle record to `published` or `closed_baseline`" and limited the row to onboarding-path enrollments.
- **Source facts:**
  - There is one `lifecycle-state.json` per agent (`~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-acbdf1ea111416778/crates/xtask/src/agent_lifecycle.rs:302-328`).
  - `runtime-follow-on --write` sets `RuntimeIntegrated` as the last step of the implementation run (`.../crates/xtask/src/runtime_follow_on/lifecycle.rs:99-106`).
  - `prepare-publication` sets `PublicationReady` (`.../crates/xtask/src/prepare_publication.rs:139`).
  - `close-agent-maintenance` clears `Drifted` and adds `MaintenanceCloseoutWritten` in that same per-agent record (`.../crates/xtask/src/agent_maintenance/closeout/write.rs:83-101`).
- **Consequence (a), any stage advance:**
  - Every advance is now an acceptance effect, including the ones executor-run commands perform.
  - Predicate items 4-5 (lines 500-504) require present, current P/O/E and all due obligations satisfied. Those cannot hold for the advance out of `enrolled`, because the independent adequacy review needs the implementation that the same command writes.
  - This contradicts line 347, "Depth enrollment MAY begin with implementation and evidence work outstanding", and the proposal at `.../docs/agents/lifecycle/support-depth-policy-proposal.md:227`.
- **Consequence (b), agent-wide membership:**
  - Rule 1 (lines 514-518) says the touched set is determined by the values a write changes. So every lifecycle write touches every tuple of the agent.
  - A maintenance closeout for any version is then an acceptance effect for every earlier tuple.
  - For a tuple whose enrollment is on the onboarding path, line 524 forbids it outright: "A route MUST NOT produce an acceptance effect on a tuple whose depth enrollment belongs to another lifecycle path". Lines 627-630 contemplate both paths enabled together.
  - For an earlier same-path tuple, predicate items 3-5 demand authority and satisfied obligations for a version the closeout is not for.
  - Later versions (539-547) lists only pointer, runtime-support record, row and working files, so it does not exempt lifecycle writes. This contradicts line 423-425 (a later version inherits nothing) and line 545-547 ("otherwise the existing lifecycle rules, which this contract leaves unchanged").
  - If the exemption is stretched to cover lifecycle writes, a later version could change `drifted` or closeout-evidence state that an earlier tuple's acceptance relies on, without that tuple's admission.
- **Class:** regression caused by the B3 remediation.
- **Remediation:**
  - Limit the stage bullet to advances to a stage whose minimum evidence includes a publication or closeout entry. Those are `published` and `closed_baseline` in `required_evidence_for_stage`. Keep the evidence-entry and drift bullets by effect.
  - Say that a lifecycle-record write belongs to the tuples of the generation (agent, path, version) of the writing route. For the agent's other tuples it is a reporting effect, unless it regresses or removes what their recorded acceptance relies on.

### F2 (P2): when a depth record first exists is undefined, so validation clause (c) and "Later versions" conflict
- **Lines:** 88-91, 436-439, 530-535, 537-547, with the freeze context at 383-389.
- **Text:**
  - Line 533: "or a depth-enrolled version without a depth record" is an unconditional contradiction.
  - Line 438-439: "afterwards it comes from the depth record".
  - Nothing says when the record is first written.
- **Consequence 1:**
  - A generation is depth-enrolled from its frozen request or approval, but the record holds P/O/E references and results that do not exist until the second freeze or execution. Line 384-385 says the opening placeholder stays non-executable.
  - Branch validation (line 530-531, every branch proposed for merge) therefore fails every depth-enrolled packet PR at open, or an implementer invents an exemption.
- **Consequence 2:**
  - Later versions rule 1 and the nightly re-dispatch can replace the working files of a depth-enrolled generation that never produced a record.
  - Resolution then has no source, which line 440-442 says must be an error, never "not depth-enrolled". Nothing refuses the displacement.
- **Class:** delta-introduced (B4 remediation).
- **Remediation:**
  - State the onset, for example "written with the generation's first depth-gated effect and no later than O's freeze". Say that clause (c) applies only from then.
  - Add to Later versions that a displacing effect for a tuple with no depth record is refused.

### F3 (P2): the flat pre-enrollment rule contradicts the compatibility and reuse text
- **Lines:** 327-328 against 311-313, 321-326 and 418-425.
- **Text:**
  - Line 327-328: "Evidence produced before the depth enrollment it would serve, a successful closeout included, is not admissible."
  - Line 311-313 lets a compatibility rule extend evidence to other versions.
  - Line 324 lets a reuse binding rely on a version "covered by an approved compatibility rule".
- **Consequence:**
  - Enrollment is per exact version, and a later version inherits nothing. Evidence from V1 reused for V2 is therefore always produced before V2's enrollment.
  - The compatibility rule and the cross-version arm of the reuse binding become dead text, and an implementer cannot tell which rule governs.
  - The research package scopes the ban to pre-adoption evidence and keeps cross-version reuse as an optional approved rule (proposal lines 107 and 109).
- **Class:** delta-introduced contradiction (B6 pre-enrollment item).
- **Remediation:**
  - Scope the ban to the tuple's own version. For example: "Evidence for a tuple's version produced before that version's depth enrollment is not admissible; evidence from another version is admissible only through a reuse binding under a compatibility rule."
  - Alternatively, drop the cross-version arms for this revision.

### F4 (P2): evidence identity is a tuple value, but "contradiction" has no matching clause
- **Lines:** 460, 471-472, 530-535.
- **Text:**
  - Lines 460 and 471-472 make evidence presence and content identity a tuple value, and bar removing or replacing it.
  - Line 532-533: "A contradiction is a published value the depth record does not support, an acceptance effect the depth record does not record, or a depth-enrolled version without a depth record."
- **Consequence:**
  - Evidence removed or swapped by a hand edit or unsupported tooling is none of the three. The depth record is untouched and still supports the published values.
  - Validation therefore need not fail, although line 534-535 promises failure "whatever produced the change ... a hand edit". The MUST NOT at 471-472 binds only routes.
- **Class:** an accepted root cause (B2, enforced through B4 validation) remains partly unfixed.
- **Remediation:** add one clause, for example "or evidence whose presence or content identity differs from what the depth record records for an accepted entry".

### F5 (P2): line 388-389 attributes to the onboarding charter a definition it does not contain
- **Text:** "On the onboarding path, the onboarding charter defines where O is frozen."
- **Source:**
  - `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-acbdf1ea111416778/docs/specs/cli-agent-onboarding-charter.md`: `grep -i 'freeze|frozen'` hits only lines 22, 31 and 234. Those cover `requested_control_plane_actions`, `descriptor.maintenance` and runtime-evidence paths.
  - The checklist (lines 171-230) has no input or obligation freeze step.
  - The only description of an onboarding freeze is the non-normative proposal line 145 ("frozen runtime requirements").
- **Consequence:**
  - The sentence reads as a delegation to text that does not exist.
  - Onboarding acceptance depends on O (predicate item 4, Bindings rule 8).
  - The onboarding enablement conditions (lines 627-630) do not require the charter amendment, so that path can be enabled and proven against an undefined freeze point. The failure is closed but untestable.
- **Class:** delta-introduced. B6 said to leave the point to the charter; the text asserts the charter already has it.
- **Remediation:** reword to "the charter does not yet define this and MUST be amended to define it", and add that amendment as an onboarding enablement prerequisite.

## Non-blocking observations (deferred, P3)

- **O1.** Line 565 says an integration step that refuses when the tip moved "satisfies this rule". Lines 578-579 say compare-and-refuse may replace serialization only "later by amendment". Say that a ref-update refusal is the integration branch's serialization, not the deferred per-writer compare-and-refuse.
- **O2.** The block on acceptance effects for `classification_required` appears only in the table cell at line 693. Predicate items 4-5 (lines 500-504) do not mention it.
- **O3.** Line 387-388 reports a generation with no frozen O as "insufficient depth". Row 689 defines that outcome as a `failed` or `unverified` obligation, but with no O there are no obligations.
- **O4.** "Missing evidence is `unverified`" and the contradictory-run case (lines 295-297) have no explicit result mapping. B5 relies on inference.
- **O5.** Wording nits:
  - "per-root" (line 92) reads as manifest root, but those files live under `docs/agents/lifecycle/<agent>-maintenance/governance/`.
  - "stored with the depth results" (line 90) is circular.
  - Line 403 merges two sentences into one long line.
  - "pending" in Proof rule 5 (line 353) is not a result value.
  - Lines 401-404 read "No row dispositions ... until the debt contract is amended", which is absolute and then conditional.
- **O6.** The depth record is now the resolution source after displacement (lines 438-439). Lines 116-118 prohibit a "second ... depth enrollment inventory or mutable status ledger". Say the record is neither.

## B1-B6 status

| Obj | Status | Evidence |
| --- | --- | --- |
| B1 | Fixed | Lines 560-565 give the merge-route integration step alone, branch writes as a candidate, and the fast-forward-only or exact-merge exception. Lines 569-573 limit domains to write-write and write-read overlap. See O1. |
| B2 | Partially fixed | Line 460 makes the evidence identity a tuple value, and 471-472 bar removal or replacement. Validation does not enforce it (F4). |
| B3 | Partially fixed | Lines 469 and 474-487 are by-effect and path-neutral, but over-broad (F1). |
| B4 | Partially fixed | Depth record (88-91), working files (92-93), resolution (436-439), contradiction (530-535) and Later versions (537-547) are all defined. The record's onset is undefined (F2), and evidence mismatch is not a contradiction (F4). |
| B5 | Fixed | Lines 693-694 and 200-208 separate "Unresolved" (enrollment only) from "Classification required". The `unverified` mapping is implicit (O4). |
| B6 | Partially fixed | Eleven of thirteen items check out. F3 (pre-enrollment evidence) and F5 (onboarding O freeze) fail. |

B6 items that check out:
- merge-time currency wording (no "currency" remains), `latest_validated.txt` and `current.json` tuple values, terms qualified, the P/O/E forward reference at line 87;
- `M` in the obligation-set formula (188-189), debt fields (408, 411), override lapse (215-216), mode exclusion as a P entry (209-212);
- generation vs `--from-request` (76-79), "partial" removed, `feature_rich` (line 51), and the new approval artifact for onboarding continuation (396-397).

## Facts checked against source (no defect)

- **Working-file names:** `maintenance-request.toml` path is built by `maintenance_request_path` (`.../crates/xtask/src/agent_lifecycle.rs:613`). `maintenance-closeout.json` sits beside it in `claude_code-maintenance/governance`. Both are per maintenance root.
- **`--from-request`:** it preserves `request_recorded_at` and `request_commit` (`.../crates/xtask/src/agent_maintenance/prepare/from_request.rs`; `.../docs/specs/maintenance-request-contract-v1.md:82-83`). Each dispatch of `agent-maintenance-open-pr.yml` writes a fresh pair (lines 162-163), and the release-watch cron runs nightly (`agent-maintenance-release-watch.yml:5`).
- **Debt fields:** `current_reason` and `blocker_class` exist, and renewal is limited to the three fields while `blocker_class` still holds (`.../docs/specs/unified-agent-api/non-tui-support-debt.md:23-24, 54-57`).
- **`feature_rich`:** it is the serde snake_case spelling in `agent_lifecycle.rs:143-149` and in the committed `lifecycle-state.json`. The kebab form is only the CLI spelling (`.../crates/xtask/src/runtime_follow_on.rs:113`).
- **Pointers and `current.json`:**
  - Root `latest_validated.txt` must equal the required-target pointer (`.../crates/xtask/src/manifest_validate/pointers.rs:292-312`).
  - `current.json.expected_targets` exists and drives support rows (`.../docs/specs/unified-agent-api/support-matrix.md:77`).
- **Lifecycle record:** `SideState::Drifted` and the closeout evidence ids exist as the contract describes.
- **`parity_exclusions`:** the "where the root has them" qualifier is right. The aider and gemini_cli roots have no `RULES.json` or validator spec.
- **Links:** every internal anchor and relative link resolves.
