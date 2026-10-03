# round2-sonnet: reviewer report

Lane: Sonnet 5.5 max, defects and conformance. Candidate: `f495a750`. Scope: remediation follow-up.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

**Candidate f495a750 (base e60664e2): not CLEAN. Two P2 findings, both introduced by the delta. No P1.**

All paths below are under `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-ac1e725107a1569e0/`. "Contract" means `docs/specs/support-depth-contract.md`.

## Findings

### P2-1: The contract protects per-root maintenance request and closeout files as durable per-generation records, but the next version's packet replaces them

**Lines:** 454, 457-458, 424-426, 503-504, 514-517, 524-525.

**Objectives hit:** A6 (a later non-enrolled version must not deadlock) is only partly fixed. A5 (values per output) is mis-modelled for the closeout row.

**What the contract says:**
- Row 454 makes "The record of any generation of the tuple's version" a tuple value.
- Rule 503-504: "Not matching a depth enrollment is never permission to change a depth-enrolled tuple's results, acceptance records or closeout records."
- Rule 524-525: a superseded tuple's "results, acceptance records and closeout records are not rewritten."
- Rule 424-426 resolves an enrollment from "the frozen request ... for that generation".
- Rule 514-517 validates against "admission and acceptance records".

**What the repo does:**
- The request has one path per agent, with no version in it (`crates/xtask/src/agent_lifecycle.rs:613-616`).
- The closeout has one path per maintenance root (`crates/xtask/src/agent_maintenance/contract_policy.rs:160`). The request contract says so too: `maintenance-request-contract-v1.md:380`, "`closeout_path` | MUST be the repo-relative closeout artifact for this maintenance root."
- `close-agent-maintenance` rewrites that file in place (`crates/xtask/src/agent_maintenance/closeout/write.rs:35`, `plan_create_or_replace`).
- History confirms it. Commit afcfed2b (codex 0.156.1) replaced the 0.144.6 closeout at the same path (`request_sha256` 27f3c079… became 92961ae8…).

**Consequence.** After the first depth-enrolled version V is accepted, the ordinary next step replaces V's request and closeout on the integration branch. That step is the nightly packet, `close-agent-maintenance` and promotion for V'. The contract gives two readings, and an implementer cannot decide between them.
- **Strict reading.** The replacement changes an enrolled tuple's closeout record by a non-matching version (503-504), so the successor's packet merge or closeout is refused. V's frozen request no longer exists, so V's enrollment no longer resolves. The Supersession reporting effect at V' promotion then fails predicate item 1 and lands in "Unresolved ... refused" (670). The agent's maintenance lane stalls.
- **Permissive reading.** V's acceptance record is destroyed while the pointer still names V. Rule 514-517 then reports a contradiction on the integration branch between the V' packet merge and V' promotion. Evidence stops counting as "acceptance is recorded" (457-458), so `manifest-retain` is no longer blocked from deleting V's evidence.

Supersession (519-528) covers only pointer, row and runtime-support moves.

**Origin.** Delta. e60664e2 said only "overwrite a depth-enrolled result" and "pointer or row". The closeout-record value, the extension of rule 2, Supersession rule 1 and the "frozen request" wording are new. A6's root cause (non-matching successor work touching an enrolled tuple) remains unfixed for these routes.

**Remediation direction.** Choose one and say so.
- (a) Name the per-version carrier that already persists as the acceptance record and the resolution source after the generation ends. That carrier is E plus depth results under `reports/<version>/`. Treat per-root `maintenance-request.toml` and `maintenance-closeout.json` as working files. Add "replaces the per-root request or closeout of an earlier version" to the Supersession trigger as a reporting effect, exempt from rules 2 and 7.
- (b) Make per-version request and closeout paths a dependent amendment required by Path enablement item 1.

### P2-2: The new refusal in the "Unresolved" row contradicts classification rule 4, Proof rule 5 and the admission predicate

**Lines:** 670 against 192-194, 343-344, 475-491, 447, 139, 309, 272-275.

**Objective hit:** A2 (truthful pending or failed results stay publishable) is partly undone.

**The new clause.** Row 670: "Depth enrollment, classification or evidence could not be resolved. Validation reports it as an error, and depth-gated effects for the tuple are refused until it is resolved." In e60664e2 the row said only "This is an error and is never reported as not depth-enrolled", with no refusal.

**Contradictions:**
- **Rule 4.** A unit with "no resolvable template" is `classification_required`, and "Acquisition MAY record it" (192-194). That is the same condition as "classification could not be resolved". Acquisition's report write is a depth-gated reporting effect on the tuple (447), so the new clause refuses it.
- **Missing evidence.** Missing evidence is `unverified` (139, 309, 272-275). Proof rule 5 and predicate items 1-2 require truthful `unverified` or pending results to stay publishable. Row 670 treats "evidence could not be resolved" as a refusal.
- **Source.** `docs/agents/lifecycle/support-depth-policy-proposal.md:151`: "Required-support classification gaps block execution readiness and remain visible in acquired reports." and "Invalid/ambiguous policy fails validation."

**Consequence.** Every new unclassified flag or mode on a depth-enrolled operation is the normal nightly discovery case. Under the literal text it either blocks the acquisition write that would show the gap to the maintainer, who alone can resolve it (197-200), or it fails validation on an otherwise valid report. Deleted or unresolvable evidence cannot be rewritten as `unverified`. Under the other reading the row does nothing for classification or evidence. Implementers cannot tell which applies.

**Origin.** Delta.

**Remediation direction.**
- Limit the refusal to depth-enrollment resolution failures (predicate item 1, Depth enrollment rule 8).
- State that `classification_required` and missing or unresolvable evidence are recordable and publishable as pending or `unverified`.
- State that these states block only execution readiness and acceptance effects.

## Deferred observations (P3, non-blocking)

1. **Undecidable MUSTs.** Rule 514-517 uses "admission and acceptance records", "contradiction" and "candidate", none of them defined, and no rule makes a route record admission. Lines 457-458 ("acceptance is recorded", "relies on") and 564-565 ("did not cover") have the same problem. Define the records (E-derived results plus closeout identity) and "contradiction".
2. **Ownership lifetime for PR-borne writes.** Lines 535-538 require ownership "before its first write", but 550-553 end the operation at the merge. Read literally, ownership spans the PR's life with no release rule for an abandoned PR. State that the merge is the first write for PR-borne effects.
3. **"Outstanding" is not an outcome.** Line 378 reports O-less scope "as outstanding", which is not one of the six outcomes at 663-670. Rule 8 (379-381) calls the same case an error.
4. **Residual term collisions (A17).**
   - Unqualified "enrollment selector(s)" at 428 and 431, against 81.
   - "route" in B1 (162) against the defined Route (94).
   - "operation" in "gated/interrupted operation" (532-564) against 61.
   - "supersession" in 355 and 399 against the new section at 519.
   - "acceptance output" (5, 28) and "acceptance scope" (355, 368) against the defined "depth-gated output".
   - P, O and E first used at 263 and 312, before they are defined at 346-357.
5. **Obligation-set formula omits `M`.** Lines 180-181 do not include `M`, though qualification includes it (129, 223-225, 233).
6. **Debt and override gaps.**
   - "reason" (399) against `blocker_class` (396).
   - "any deferral of a depth obligation" (399) against 389-391. No mechanism disposes a depth obligation, so "validly dispositioned" at 487-488 is empty for them.
   - Override "expiry or review condition" (205) has no stated consequence.
7. **Mode exclusion storage.** P holds "overrides and exclusions" (355), while 110, 202 and 668 forbid a second exclusion list. Say a mode exclusion is a P classification entry.
8. **Pre-enrollment evidence.** Lines 317-318 admit it only through a reuse binding, which needs an unchanged "obligation's definition" (312-313). With no earlier P or O to point to, it is inadmissible. Say so if that is the intent.
9. **Generation wording.** Lines 77-78 say any "maintenance regeneration ... opens a new generation". The request contract (`maintenance-request-contract-v1.md:82-83`) says a `--from-request` re-freeze "completes the same generation". Qualify the term as a regeneration that records a new Event.
10. **Acceptance list gaps.**
    - The runtime-support projection (452) is not in the acceptance list (460-469).
    - "records a closeout as closed" has no state field to test for maintenance closeouts. `crates/xtask/src/agent_maintenance/closeout/types.rs:74-84` has none. Only proving-run closeouts have `prepared` and `closed` (charter:221).
    - "Partial" in Proof rule 5 (343) is ambiguous against "publishes a promise as qualified" (468).
11. **Onboarding lanes.** O's freeze point for onboarding is undefined; 373-378 cover only maintenance lanes. "Continuation is a new generation" (382-385) would need a new approval artifact (354).

## Factual statements checked against sources

| Statement | Source | Result |
| --- | --- | --- |
| `approval_commit`, `approval_recorded_at` (354) | `crates/xtask/src/approval_artifact.rs:246-249` | Confirmed |
| Debt renewal keyed to `blocker_class`; three fields change in place (396) | `non-tui-support-debt.md:44-61` | Confirmed |
| Retirement conditions (397) | `maintenance-request-contract-v1.md:289-295`, `221-223` | Confirmed. "Partial satisfaction keeps the row" has no request-contract source (proposal:131) |
| Freeze points and lanes (373-378) | `maintenance-request-contract-v1.md:59-68` (`uaa-0063`) | Confirmed |
| Version statuses and per-target outcomes (463) | `cli_manifests/codex/VERSION_METADATA_SCHEMA.json` | Confirmed: `snapshotted\|reported\|validated\|supported`, `validation.passed_targets` |
| `runtime_family` | `runtime-support-contract.md:21,40,100` | The mapping runtime family = agent id is only in `crates/xtask/src/support_matrix/derive.rs:407`, not in the contract |
| Existing admission gate (88) | `crates/xtask/src/agent_maintenance/closeout.rs:60,155-174` | Confirmed in code. The only prose source is the Draft lifecycle spec's `uaa-0039` row |
| `parity_exclusions` (52) | `RULES.json` in codex, claude_code, opencode | Confirmed. The aider and gemini_cli roots have no `RULES.json` |
| Runtime profiles, "no normative owner" (51) | No spec under `docs/specs` defines them | Confirmed. `RuntimeProfile` is at `agent_lifecycle.rs:145-149` and serialises as `feature_rich`; the `feature-rich` spelling belongs to `RequestedTier` |
| Lifecycle record owner (50) | Charter:164-169 | Confirmed |
| Integration branch is `staging` (93) | `crates/xtask/src/agent_maintenance/stand_down.rs:71`, `AGENTS.md:40` | Confirmed |
| Nightly re-dispatch opens a new Event (77) | `.github/workflows/agent-maintenance-open-pr.yml:162-163` | Confirmed |
| Neutral root intake carrier (650) | `support-matrix.md:170-175` | Confirmed. Only `reports/**` fits |
| Reusing the MCP spec in `MCP1` (163) | | Not re-litigated (R1) |

All internal anchors and relative links resolve. Numbered lists are complete. No stale "protected" or "final admission" terms remain.

## Accepted objectives A1-A29

| ID | Status | Text at f495a750 / note |
| --- | --- | --- |
| A1 | Fixed | 447, 457-458 |
| A2 | Partially fixed | 140, 478-480, 490 fix it; 670 undoes it (P2-2) |
| A3 | Fixed | 94-95, 514-517, 550-553, 594-596 |
| A4 | Fixed | 542-558 (P3-2) |
| A5 | Fixed | 440-471; the closeout row 454 is defective (P2-1) |
| A6 | Partially fixed | 519-528 cover pointer, row and runtime-support only (P2-1) |
| A7 | Fixed | 460-471, 490 |
| A8 | Fixed | 49, 229-235, 449 |
| A9 | Fixed | 76-83, 406-410 |
| A10 | Fixed | 356, 361-362 |
| A11 | Fixed | 197-200 |
| A12 | Fixed | 387-399 (P3-6) |
| A13 | Fixed as text | 514-517 (P3-1) |
| A14 | Fixed | 311-316 |
| A15 | Fixed | 42-55 |
| A16 | Fixed | 164, 178, 181, 201-203 |
| A17 | Partially fixed | P3-4 |
| A18 | Fixed | 131-132, 147-181 |
| A19 | Fixed | 354, 373-378, 382-385 (P3-11) |
| A20 | Fixed | 368-370 |
| A21 | Fixed | 603-616, 630-631 |
| A22 | Fixed | 163, 229-233, 650-653 |
| A23 | Fixed | 560-565 |
| A24 | Fixed | 262-287 |
| A25 | Fixed | 204-207 (P3-6) |
| A26 | Fixed | 32-33, 624-627 |
| A27 | Fixed | 672-677 |
| A28 | Fixed | 108-110, 201-203, 668 (P3-7) |
| A29 | No defect found | Wording nits are in the P3 list |

**Areas examined.**
- Delta against e60664e2.
- Every factual claim added or changed by the delta.
- Contradictions among the admission predicate, Supersession, Complete mediation, the acceptance-effect list and Proof rule 5.
- The 'does not change' table against its sources.
- Shared output layout against the depth-gated table.

No cargo builds or test suites were run.
