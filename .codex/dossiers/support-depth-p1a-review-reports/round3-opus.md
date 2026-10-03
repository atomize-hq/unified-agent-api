# round3-opus: reviewer report

Lane: Opus 5.5 xhigh, adversarial. Candidate: `dd14fb32`. Scope: remediation follow-up.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

I found four P2 findings and no P1 findings, so dd14fb32 is not CLEAN. All four come from the f495a750→dd14fb32 delta. They concern the B1, B2 and B4 objectives. B3, B5 and B6 are fixed.

Spec path: `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-aa4a1d7527a751ad2/docs/specs/support-depth-contract.md`

## Findings

### F1 (P2): Freezing whole coverage reports blocks adding parity exclusions
- **Lines:** 460 ("coverage reports included", "content identity"), 471–472 ("removed or replaced"). These conflict with line 52 ("`parity_exclusions` … Unchanged").
- **Contradicted source:** `crates/xtask/src/manifest_validate/versions.rs:279-301` checks the reports of every version whose status is `reported`, `validated` or `supported` against the current `RULES.json`. `crates/xtask/src/manifest_validate/report_invariants.rs:79-120` raises `REPORT_MISSING_INCLUDES_EXCLUDED` when an older report lists a newly excluded unit as missing. `crates/xtask/src/manifest_report.rs:73-77` applies the exclusions only when a report is regenerated. Separately, `.github/workflows/parity-acquire.yml:550,558` rewrites the per-root `wrapper_coverage.json` on every acquisition and then validates every version.
- **Failure scenario:**
  1. codex v1 is depth-enrolled, accepted and published. Support rows exist for every version (`cli_manifests/support_matrix/current.json` has rows from 0.61.0 to 0.156.1), so v1's row keeps deriving from its acceptance indefinitely.
  2. A maintainer adds a TUI parity exclusion for a unit that v1's report lists as missing.
  3. `manifest-validate`, which also runs inside every later acquisition, fails on `reports/v1/`.
  4. The only fix is to regenerate v1's reports, which line 471 forbids.
  5. Result: exclusions touching any accepted version cannot be added, and later acquisitions stay red.
- **Transitive reading:** if `wrapper_coverage.json` counts as evidence the reports derive from, v2's acquisition is forbidden outright. The repo already separates "version-agnostic" files from version-scoped evidence (`crates/xtask/src/agent_maintenance/closeout/findings.rs:174-176`). The contract does not.
- **Origin:** introduced by the delta. The earlier text protected only "presence" and only against removal.
- **Remediation:** keep B2, but scope the protected identity to the values that belong to the tuple, not whole file bytes. For example, allow a coverage report to be regenerated when every entry belonging to a depth scope tuple is unchanged. Also exclude version-agnostic per-root inputs from "evidence".

### F2 (P2): Nothing checks the new evidence protection
- **Lines:** 471–472 versus the admission predicate (491–507) and the contradiction definition (531–533).
- **Violated objective:** B2, and B4's statement that validation compares gated state with the depth record.
- **Failure scenario:**
  1. Replacing or removing evidence is not on the acceptance list, so it is a reporting effect. Reporting effects need only items 1–2, and item 2 constrains only "what it writes", not the evidence the write displaces.
  2. The contradiction list is closed ("A contradiction is …") and does not include changed evidence identity.
  3. So a hand edit, or a reporting route, that rewrites `reports/v1/*` or a test-outcome record while leaving the depth record and published values alone passes both depth admission and repository validation. Rule 7 exists precisely to catch hand edits.
- **Origin:** introduced by the delta (the closed contradiction definition is new).
- **Remediation:** add to the contradiction list "evidence whose presence or content identity differs from what an accepted depth record binds while a published value derives from that acceptance". Also make line 471 a refusal condition of depth admission.

### F3 (P2): After working files are replaced, resolution has nothing complete to read
- **Lines:** 88–91 (the depth record holds only P, O and E "references"), 436–439 ("afterwards it comes from the depth record"). These interact with:
  - 390–392: consumers require all four bindings.
  - 440–442: "deleted bindings are errors".
  - 502: item 4 requires P, O and E to be "present".
  - 694: Unresolved refuses all depth-gated effects.
- **Violated objective:** B4, that the depth record is the durable resolution record.
- **Failure scenario:**
  1. v2's maintenance packet merges and replaces `maintenance-request.toml`, which held v1's P (its enrollment selectors) and O.
  2. `parity-promote` for v2 moves the pointer off v1. "Later versions" calls this a reporting effect for v1, which still needs v1's enrollment to resolve.
  3. Resolution falls to v1's depth record, which holds only a P reference.
  4. Strict reading: the binding is deleted, v1 is Unresolved, the effect is refused, and v2 can never be promoted. Lenient reading: resolution depends on git history (the Event's `request_commit`), which the contract never authorizes. CI checkouts may also be shallow.
  5. Either way, item 4 can never again hold for v1, so a revert back to v1 is permanently refused.
- **Origin:** an accepted root cause that is only partly fixed.
- **Remediation:** require the depth record to carry the resolved depth-enrollment selection for its tuples, plus the P and O content identities needed for currency. State that replacing a working file whose content the depth record has captured is not a deleted binding.

### F4 (P2): The merge-route rule contradicts the serialization rule
- **Lines:** 563–565 say a fast-forward-only push or exact-merge re-check "satisfies this rule". Both are tip compare-and-refuse. This conflicts with:
  - 555–558: depth admission must be established while ownership is held.
  - 571–572: "Ownership excludes every other member of the domain".
  - 578–579: "This revision requires serialization … Per-writer compare-and-refuse MAY replace it later by amendment."
- **Failure scenario:** every command route reaches `staging` through a pull request (`parity-promote.yml:368`, `agent-maintenance-open-pr.yml:263,371`). The rules therefore conflict for the one gate path-enablement item 3 depends on.
  - Strict reading: GitHub's require-up-to-date setting and a fast-forward-only push are not serialization, and enablement is blocked.
  - Lenient reading: compare-and-refuse is already allowed, so the 578 bullet is wrong, and it invites the same argument for routes that do not merge.
- **Origin:** introduced by the delta.
- **Remediation:** state that a tip-conditional integration step is that step's serialization point for items 1–3. Scope lines 578–579 to gated sequences other than the integration step.

### Non-blocking (P3)
- **Line 562:** "Results obtained while the candidate was prepared" can be read as obligation results (line 142), which would force re-verification at merge. Use "Depth admission established while …".
- **Line 693:** the classification_required block on acceptance appears only in the outcome table. Predicate item 5 and classification rule 4 (execution readiness only) do not name it. Add it to the predicate.
- **Lines 531–533:** "that enrollment's depth record" is unqualified, although line 82 says the term is always written with its qualifier, and there is one record per target. The spec also never says when a depth record must first exist. Before O is frozen, and on the docs-only lane, no record can satisfy Bindings rule 8.
- **Deferred:** the admission checker runs from the candidate's own tree. This predates the delta and sits inside the trust boundary stated at line 329.

## Objective status

| Objective | Status |
| --- | --- |
| B1 | Partially fixed (F4) |
| B2 | Partially fixed (F1 too broad, F2 unchecked) |
| B3 | Fixed. The list covers what `close-agent-maintenance`, `prepare-agent-closeout`, `close-proving-run`, `parity-promote.yml`, `refresh-publication`, `prepare-publication` and `historical-lifecycle-backfill` write. |
| B4 | Partially fixed (F3) |
| B5 | Fixed. An acceptance effect cannot go through for classification_required or missing evidence (line 693 plus item 5); see the P3 note. |
| B6 | Fixed (P3 wording note) |

Areas examined: the full delta; the definitions of terms, depth-gated outputs and acceptance effects, the admission predicate, complete mediation, "Later versions", final depth admission and publication. Against the code I checked closeout `write.rs`, `prepare_closeout.rs`, `close_proving_run.rs`, `prepare_proving_run_closeout.rs`, `publication_refresh.rs`, `prepare_publication.rs`, `historical_lifecycle_backfill.rs`, `runtime_follow_on/lifecycle.rs`, `capability_publication.rs` and `manifest_validate/{versions,report_invariants,current}.rs`. I also checked the workflows `parity-promote.yml`, `parity-acquire.yml` and `agent-maintenance-open-pr.yml`, and the history of commit `211d3520`.
