# round1-opus: reviewer report

Lane: Opus 5.5 xhigh, adversarial. Candidate: `e60664e2`. Scope: full.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

**Verdict: not CLEAN.** I found 3 P1, 10 P2 and 7 P3 findings in candidate `e60664e2`. `git diff --stat 2797b8d9 e60664e2` confirms one new file, 580 lines. Line numbers below refer to `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-ab920e30031e7fb73/docs/specs/support-depth-contract.md`.

## P1

**P1-1. The place where depth results are stored is not a protected output (lines 374–384 and 536–538).**
- **Requirement:** acceptance criterion 3, and the refinement that admission keys on effects so nothing overwrites a depth-enrolled result.
- **Problem:** Publication requires depth results to be committed "in an evidence category the support matrix's neutral root intake already reads". `support-matrix.md` limits that intake to four categories:
  - `versions/*.json` and `pointers/*`, which are protected
  - `current.json` and `reports/**`, which are not
- The proposal recommends `reports/<version>/coverage.<target>.json`, which is one of the unprotected ones.
- **Failure:** V1 is depth-enrolled and still has due work, so its status is `reported`. `cargo run -p xtask -- manifest-retain --root cli_manifests/<agent> --apply` keeps only pointer versions and the last N `validated`/`supported` versions. It runs `remove_dir_all` on `reports/V1` (`crates/xtask/src/manifest_retain.rs:64,125-126`). Re-running acquisition for V1 rewrites the same reports.
- Neither write is a protected effect, so no admission, mediation or final-admission rule applies. The next support-matrix regeneration either fails forever as Unresolved or publishes from the destroyed evidence.
- **Fix:** add the committed depth results and their manifest-root evidence to the protected outputs, or require Publication to use a category that is already protected.

**P1-2. The admission rules can refuse a truthful downgrade, so a stale "qualified" stays published (lines 395–400 and 418–420, against 285–286, 253–257 and 299).**
- **Refinement weakened:** "truthful pending or failed assessments must stay publishable." Proof rule 5 says publishing such an assessment "MUST remain possible."
- **Problem:** Admission predicate items 2 and 3 apply to reporting effects as well as acceptance effects:
  - item 2 requires the route to hold authority "under the lifecycle path that owns its depth enrollment"
  - item 3 requires P, O and E to be "current"
- **Failure:** V1 is accepted and published as qualified. A later unrelated PR changes a crate in V1's dependency closure, which invalidates V1's result (lines 253–257).
  - The PR author regenerates the support matrix. `make preflight` runs `support-matrix --check`, so they must. That would change V1's row, which is a protected reporting effect.
  - The command holds no maintenance-path authority for V1. Only "Execution" may change E (line 299), and the generation is closed, so E can never become current again.
  - The truthful downgrade is refused. Either CI wedges, or the generator never recomputes and the false "qualified" persists.
- **Fix:** for reporting effects, require only items 1 and 4. Require that stale or unbound results publish as `unverified` or Unresolved. Keep items 2, 3 and 5 for acceptance effects.

**P1-3. Final admission ends at the working-tree write, but these effects take effect when the PR merges (lines 433–446, 453–455 and 427–429).**
- **Refinement weakened:** "serialization narrows but does not remove the need for a final freshness check (stale admission, changed authority or inputs ... survive it)."
- **Problem:** Line 446 says a check made inside ownership "is the final check and need not be repeated". But:
  - `parity-promote.yml` writes pointers and retires the marker, then opens a PR to `staging` with `create-pull-request` (lines 368–379 of that workflow). The open-PR lane works the same way.
  - The things that invalidate an admission are not protected operations, so serialization never excludes them: a P re-freeze, a debt-inventory edit, a registry edit, a dependency change.
- **Failure:**
  - At T1 the promotion is admitted inside ownership and its PR opens.
  - At T2 a maintainer merges a P re-freeze that adds obligations.
  - At T3 the promotion PR merges.
  - The stale acceptance is now on `staging`. Rule 7's validation checks only missing bindings or contradiction of scope, not currency.
- `stand_down.rs` already documents this hazard ("re-read base fresh immediately before mutating").
- **Fix:** say that an effect landed by PR completes at merge into the protected branch, and require the predicate, including currency, to be re-established against the merge result. Otherwise such routes must refuse.

## P2

**P2-1. "Serialization ownership" has no defined scope (lines 436, 451–455).**
- **Requirement:** D13 serializes "affected publication writes", and acceptance criterion 4.
- **Problem:** "Hold serialization ownership of the protected operation" never says what it must be serialized against: other routes touching the same outputs or tuples, or the writers of the inputs the admission relies on. "Demonstrated for each route" has no reference set.
- **Failure:** the existing concurrency groups are per lane:
  - `parity-promote-<agent>-<version>`
  - `agent-maintenance-<branch>`
  - `parity-acquire-<agent>-<version>-<ref>`
  - local `xtask` runs have none.

  An implementer can cite these as ownership. Then promotion of V1, promotion of V2 and `refresh-publication` all run concurrently against the same outputs.
- **Fix:** define the domain as every route whose outputs, touched tuples or admission inputs overlap.

**P2-2. A "protected effect" depends on an undefined "result for a depth scope tuple" (lines 72–73, 374–391).**
- **Problem:** pointers, version metadata, markers, closeout records and lifecycle stages carry no per-obligation results.
- **Evadable reading:**
  - Moving a pointer from enrolled V1 to V2 "alters no result", so no admission is needed.
  - This contradicts mediation rule 1 at lines 415–416.
  - It is exactly the request-version keying the refinement forbids.
- **Over-inclusive reading:** writing the marker becomes protected.
  - Stand-down markers are hand-committed by maintainers.
  - `close-agent-maintenance` requires one on `origin/staging` before closeout (`closeout.rs` `require_stand_down`).
  - Gating that maintainer protection on admission inverts it. Rule 7 would then reject every marker for an enrolled generation as "depth-enrolled output that lacks its bindings", which blocks closeout. That contradicts line 46.
- **Fix:** for each listed output, define which part keyed to a tuple counts as its "result". Make marker retirement, not declaration, the protected marker effect.

**P2-3. A later, non-enrolled version replacing an enrolled pointer is undecidable (lines 340–345, 349–356, 397–404, 413–416).**
- **Decision involved:** D12, one exact generation and no successor rule.
- **Problem:** mediation rule 1 says promoting V2 touches V1's tuple, so the predicate is evaluated for V1:
  - Item 3 asks for P, O and E "for the candidate being written". V2 can never have them under D12, so every later promotion is refused.
  - Item 5 gates V2 on V1's due work. After any invalidating change to V1's dependencies, no route can refresh V1's E (see P1-2), so the pointer freezes permanently.
- Both outcomes contradict Depth enrollment rule 5 (the remainder keeps every existing publication duty) and rule 6 (MUST NOT narrow the release-watch ratchet).
- **Fix:** state the predicate for an effect that replaces another tuple's state. For example, the superseded tuple gets a reporting check, the new version is admitted under its own rules, and the enrolled generation's own acceptance records cannot be rewritten.

**P2-4. The "does not change" table is untrue for capability advertising (lines 43, 177–178, 378, 390).**
- **Requirement:** acceptance criterion 6, and rule 4 ("Advertising stays governed by the capability documents and the charter's promotion rule").
- **Problem:** capability publication "for an agent that has a depth-enrolled mapped operation" is protected, and "adds capability advertising" is an acceptance effect. So advertising any capability for that agent requires the enrolled operation's due depth work to be satisfied, even an unrelated capability such as `session.handle.v1`.
- **Consequence:** that changes capability-publication truth and therefore the charter's two-backend promotion outcomes.
- **Fix:** limit the protected capability output to the capabilities the enrollment claims (rule 5's set), and record in the table that admission is added for them.

**P2-5. "Generation" means a run, which conflicts with D12's version-keyed enrollment (lines 65–66, 296, 321–324, 340–343).**
- **Problem:** the nightly re-dispatch regenerates an unprotected packet "with a fresh `request_recorded_at`" (`stand_down.rs`). That is either:
  - a new generation, so the single enrolled generation ends after one night, which is the D12 version-scoped enrollment shedding its obligations; or
  - a changed Event inside one generation, which line 296 forbids.
- Supersession and the reprepare path ("fresh bindings", line 324) have the same ambiguity.
- **Fix:** key depth enrollment to (agent, path, exact version). State that regeneration, supersession and reprepare of the same version stay inside it, and say what happens to Event.

**P2-6. The O row does not say who authorizes a re-freeze (line 298).**
- **Refinement:** transition rules "must say what may change, who authorizes it and what it invalidates."
- **Problem:** "afterwards only explicit re-freeze" names no authorizer and no limits. The P row names the maintainer, but this row is silent.
- **Failure:** an executor re-runs `prepare-agent-maintenance --from-request` after a change to the acquired inputs. That changes the concrete obligation set without maintainer authority.
- **Fix:** name the authority and state that a re-freeze cannot change version, targets or selection.

**P2-7. Nobody is assigned the authority to resolve `classification_required` or to approve inherited forwarding (lines 146–157).**
- **Problem:** rule 5 lets a unit inherit forwarding "only when evidence shows it stays inside the approved effects boundary". Rule 3's question, whether a flag "alters effects", is the same judgment. Neither says who decides. Adequacy review covers `verified` results, not classification.
- **Failure:** an executor submits evidence that a new flag on a DIAG operation is harmless. It inherits DIAG forwarding and avoids ADMIN obligations.
- **Fix:** resolution requires the maintainer (a change to P) or the independent adequacy authority, never the executor.

**P2-8. The debt rules do not match the name-level debt inventory (lines 41, 278–280, 326–336).**
- **Mapping is undefined:** "Authorized debt MAY disposition due work" never says how a name-identity row maps to per-operation obligations. The research's limit ("name-level historical debt cannot automatically defer every new semantic requirement") was dropped. One row covering one flag could be read as dispositioning a whole operation's obligations.
- **Field does not exist:** "obligation coverage" (line 333) is not a debt-row field. Readers "MUST reject unknown row bullet keys" (request contract line 247), yet the table says debt rows are unchanged.
- **Closeout path blocked:** request-contract invariant 6 resolves an `excluded_by_rules` unmatched row by retiring it. Executor retirement here requires evidence covering every obligation or upstream removal, and there is no maintainer retirement row. The transition fails closed, which blocks closeout.
- **Fix:** state the mapping, drop or route the "obligation coverage" wording through the debt contract, and add the existing retirement paths.

**P2-9. The validation backstop is keyed on the written output's own enrollment (lines 426–429, 470–471).**
- **Research text weakened:** "Compare against authoritative required scope, not the edited candidate's claim."
- **Problem:** rule 7 rejects only "depth-enrolled output" that lacks bindings or contradicts scope.
- **Failure:** a hand edit, a route declared unsupported, or a new route added after enablement replaces the V1 pointer and row with V2 content. The result is not depth-enrolled output, so validation passes. Annex A's "MUST update before that route may perform a protected effect" has no enforcement once a path is enabled.
- **Fix:** validation must check every authoritative enrollment's protected state against its admission and acceptance record.

**P2-10. "Validated reuse binding" is undefined (lines 229, 258–259).**
- **Requirement:** acceptance criterion 4.
- **Problem:** this term is the only way evidence can cross a P/O boundary, and it is never defined. Also, "every rule in this section" is ambiguous between the subsection and the whole Evidence section. Under the whole-section reading, evidence produced before enrollment can never name the P/O "it was produced under".
- **Fix:** define what the reuse binding must establish (the research's E reuse binding: unchanged requirement, dependencies, exact version and target or an approved compatibility rule), and fix the section reference.

## P3 (clarity, non-blocking)

- **Term rules:**
  - Line 349 "unenrolled remainder" and line 89 "second enrollment inventory" break the contract's own always-qualify rule (lines 67–69).
  - "Protected effect" collides with the stand-down meaning of a "protected" generation (`acquisition-maintenance-lifecycle-spec.md:417-418`, `stand_down.rs`).
- **Output dimension (line 102):** O offers no owned-I/O or handle form, which LAUNCH and MANAGED need (research O4). "R1 through X4" (line 94) quietly includes O and L because of table order.
- **Event for onboarding (line 296):** Event is defined only by maintenance-request fields; an onboarding generation has no `request_commit`.
- **Path enablement in P (lines 297, 310–311):** P includes path enablement, so turning on onboarding may be read as changing maintenance P and invalidating it. The research says it "need not change an unchanged maintenance P".
- **Onboarding proof (lines 492–499):** the proof is not required to be isolated, unlike item 2 for maintenance.
- **Approval authority (line 517):** "explicitly bounded approval authority" should say "existing", or it reads as allowing a new approval step.
- **MCP:**
  - Line 136: "Isolated effective home" turns the MCP spec's SHOULD (line 362) into an obligation, and conflicts with its pinned Claude caveat that project `.mcp.json` is not isolated.
  - Rule 5 (lines 179–182): the "complete set" of operations behind a capability is not tied to the adapter's exposed flows. It should also be materialized under the manifest root so the support-matrix intake restriction still holds.

**Areas examined:**
- all sections of the candidate
- both research documents
- the nine controlling specs
- context-only check of the lifecycle spec, for stand-down and uaa-0063
- workflows: `parity-promote.yml`, `parity-acquire.yml`, `agent-maintenance-open-pr.yml`
- `crates/xtask/src/` modules: `manifest_retain.rs`, `manifest_validate/fix_mode.rs`, `agent_maintenance/{stand_down.rs,closeout.rs,contract_policy.rs,support_audit.rs}`
- the Makefile preflight gates
