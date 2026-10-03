# Handoff: dispatch the P1a dual review (support-depth contract)

Created 2026-10-02 18:40 America/Indianapolis by the lead Claude Code session.
Continues `.codex/handoffs/2026-10-02-101256-support-depth-policy-claude-code.md`.

## Why this exists

The maintainer asked for a Codex-free dual review of the P1a candidate on two new lanes. The lane
definitions were created, but a running session does not load new `.claude/agents/*.md` files, so
the dispatch has to happen after a session restart. Everything needed is below.

## State

| Item | Where |
| --- | --- |
| P0 research package | commit `2797b8d9`, branch `docs/support-depth-policy-proposal` (local, unpushed) |
| P1a candidate | commit `e60664e2`, branch `docs/support-depth-p1-contracts` (local, unpushed), one new file `docs/specs/support-depth-contract.md`, `Status: Draft` |
| Candidate worktree | `~/.codex/worktrees/support-depth-proposal/unified-agent-api`, currently on `docs/support-depth-p1-contracts` |
| Dispositions D1–D14 | `.codex/dossiers/support-depth-p1-dispositions-2026-10-02.md` |
| Pro approach review (ADJUST, proceed) | `.codex/guidance/2026-10-02-support-depth-p1-dispositions-approach-review.md` |
| New lane definitions (uncommitted, primary checkout) | `.claude/agents/opus-xhigh-adversarial-reviewer.md`, `.claude/agents/sonnet-max-defect-reviewer.md` |

Working arrangement (maintainer override of CLAUDE.md for this effort): no Codex. The lead authors
and remediates. Review is two lanes: Opus 5.5 at `xhigh` (adversarial) and Sonnet 5.5 at `max`
(defects and conformance).

## Next steps

1. Confirm both lane names appear in the session's available agent types.
2. Dispatch both lanes concurrently with the packet below (shared part plus each lane's intent).
   Never name a lead worktree path in a lane prompt; lanes read by SHA in their own worktree.
3. Adjudicate per `.claude/skills/cross-review/SKILL.md`: read the source and cited contracts
   directly, mark each finding accepted, rejected or deferred with a rationale, remediate accepted
   findings in the candidate, and re-run both lanes if the remediation changes the reviewed risk.
4. Then write Annex A (route inventory: outputs backward to writers, entrypoints forward, reconciled)
   and the dependent amendments: registry, request, charter (D10 allowlist to eight), support-matrix
   (D14 option A), capability documents.

## Carry-forward notes

- D14 option A was reaffirmed after a correction: it amends the support-matrix spec's deliberate
  separation from capability advertising. Capability linkage must travel through depth facts under
  the manifest root so the support matrix never reads the capability inventory. The
  `markdown_support_claim` drift checks also read `uaa_support`.
- Lifecycle `support_tier` already has `first_class` (codex, claude_code); nothing assigns it and no
  spec defines what earns it. The contract leaves tiers alone.
- The companion's section 10 verification script asserts pre-revision hashes; fix on next edit.
- The new lane definitions are untracked in the primary checkout on `staging`. Committing them is
  the maintainer's call.

## Review packet — shared part (send to both lanes verbatim)

READ-ONLY REVIEW PACKET. A second, independent lane reviews the same candidate; do not coordinate
with it.

### How to read the candidate
You run in your own git worktree. First run, inside that worktree only:
    git switch --detach e60664e2
Then read files normally. Do not use `git -C`, do not `cd` to any other worktree or path, do not
edit, write, stage or commit anything. This is a documentation-only candidate: do not run cargo
builds or test suites. Allowed commands: read-only git (`show`, `log`, `diff`, `grep`), `grep`/`rg`,
`sed -n`, `wc`, and reading files.

### Revisions
- Base: 2797b8d9 (research package landed)
- Candidate: e60664e2
- Changed-file set: exactly one new file, `docs/specs/support-depth-contract.md` (580 lines,
  `Status: Draft`). Confirm with `git diff --stat 2797b8d9 e60664e2`.

### What the candidate is
This repository is a Rust workspace: per-agent CLI wrapper crates (`crates/codex`,
`crates/claude_code`, `crates/opencode`, ...), a unified facade crate `crates/agent_api`, and
lifecycle automation in `crates/xtask` (manifest acquisition, maintenance packets, onboarding,
support-matrix and capability-matrix publication, version-pointer promotion). `docs/specs/**` is
normative authority.

The candidate is the first piece of a normative-contract packet ("P1"). It converts a non-normative
research package into rules for a new "support-depth" policy: semantic support obligations added
beside the existing name-coverage machinery. It writes RULES ONLY. It is a Draft and says it binds
nothing until the maintainer changes its status.

### Sources the contract must be faithful to (all present in the tree at the candidate)
Research it converts (non-normative):
- `docs/agents/lifecycle/support-depth-policy-proposal.md`
- `docs/agents/lifecycle/support-depth-policy-research-decisions.md`

Controlling normative specs it must not contradict:
- `docs/specs/maintenance-request-contract-v1.md`
- `docs/specs/agent-registry-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/codex-wrapper-coverage-generator-contract.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/unified-agent-api/runtime-support-contract.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/non-tui-support-debt.md`
- `docs/specs/unified-agent-api/mcp-management-spec.md`
Context only (itself a Draft work plan):
`docs/specs/unified-agent-api/acquisition-maintenance-lifecycle-spec.md`
You may inspect source under `crates/xtask/src/**` and `crates/agent_api/src/**` to test factual
claims.

### Maintainer decisions the contract must encode (made 2026-10-02; not open for re-litigation)
- Two hard constraints: (1) greenfield — no legacy inventory, grandfathering, semantic
  carry-forward ledger or migration engine; (2) minimum necessary machinery — any new field,
  artifact, registry, schema or approval step must name a concrete reachable failure that existing
  machinery cannot prevent.
- D1–D7, D9: accepted as the research companion recommends (templates plus exact effect selectors;
  opaque MCP output preserved; four logical bindings carried by existing fields; reuse of existing
  tests/runners/CI with content-bound adequacy review; affected-crate dependency closure with exact
  version/target reuse by default; existing debt inventory stays authoritative; positive selection
  with an explicit unenrolled remainder and no enrollment subsystem; additive shared integration as
  derived candidates plus approved selection through existing packets).
- D8 (pilot version/target, new onboarding agent): deferred to later enrollment checkpoints; not an
  input to this contract.
- D10: the onboarding charter's four-entry promotion allowlist will be amended to the eight entries
  the audit code already reads. That amendment is NOT in this candidate.
- D11: the maintenance lifecycle path may be enabled before the onboarding path is proven,
  conditional on every route that can affect acceptance outputs either enforcing admission or
  refusing.
- D12: the first depth enrollment covers one exact generation (one frozen upstream version). No
  successor-generation rule yet.
- D13: serialize affected publication writes initially; per-writer compare-and-refuse is a later
  optimization.
- D14: `uaa_support` in the support matrix will be re-derived from capability plus depth truth for
  depth-enrolled generations under a schema version bump; historical rows keep their meaning. That
  amendment is NOT in this candidate; this contract only needs to state capability-level
  qualification and leave publication to the support-matrix contract.
- Refinements accepted from an external review: D11's condition is complete mediation (every
  supported route to a protected effect enforces the admission predicate with the authority correct
  for that route, or refuses; sharing a gate that checks the wrong lifecycle authority is
  insufficient); serialization narrows but does not remove the need for a final freshness check
  (stale admission, changed authority or inputs, and snapshot restoration acting as a writer all
  survive it); not matching an enrollment must never license overwriting an enrolled result
  (admission keys on effects, not the request's version); truthful pending or failed assessments
  must stay publishable; keeping depth facts out of the runtime-support payload does not exempt
  pointer changes from admission; binding transition rules must say what may change, who authorizes
  it and what it invalidates.
- Two deliberate terminology choices: "obligation template" replaces the research term "family"
  (which already means runtime family in the support-matrix code), and "depth enrollment" is always
  qualified (because "enrolled" already means release-watch enrollment and a lifecycle stage).

### Known and excluded — do not report these as findings
- Annex A (the route inventory) is absent; the contract states it cannot be approved without it.
- Amendments to the registry, request, charter, support-matrix and capability documents come in
  later commits.
- No runtime, schema or evaluator implementation exists or is claimed.

### Acceptance criteria
1. No rule contradicts a controlling normative spec listed above.
2. Every maintainer decision and accepted refinement that belongs in a common contract is encoded
   correctly; none is misstated or silently weakened.
3. The two safety rules — "Complete mediation" and "Final admission" — cannot be satisfied by an
   implementation that still permits the failures they exist to prevent.
4. Every MUST is decidable and enforceable as written; terms are defined and do not collide with
   existing repository terms.
5. Nothing requires machinery the constraints forbid, and nothing is stated as existing behavior
   that does not exist.
6. The "What this contract does not change" table is true: no later rule alters something that
   table says is untouched.

### Output
Findings first, most severe first. Severity: P1 = adopting the contract as written would be unsafe
or wrong (contradiction with a normative spec, a bypassable invariant, an authority error, a
misstated decision); P2 = material ambiguity or omission that would plausibly cause a wrong
implementation or a wrong dependent amendment; P3 = clarity only, non-blocking. For each finding
give: severity; `docs/specs/support-depth-contract.md` line number(s); the violated requirement or
the contradicted source with its path and quoted text; a concrete failure scenario or consequence;
and a narrowly scoped remediation direction. Do not redesign, do not widen scope, do not re-litigate
the decisions above. If no P1 or P2 remains, say `CLEAN`, name the candidate revision, and list the
areas you examined.

## Lane intent — `opus-xhigh-adversarial-reviewer` (append to the shared part)

Review intent for YOUR lane (adversarial). Try to break it. In particular:
- Construct concrete request sequences that comply with the text of "Protected effects",
  "Admission predicate", "Complete mediation" and "Final admission" yet still let a cross-path,
  non-matching, aggregate-regeneration, or snapshot-restoration write replace a depth-enrolled
  result, or let stale admission commit. Is "protected effect" (defined by whether the result
  differs for a depth scope tuple) circular or evadable? Is "serialization ownership" defined well
  enough that an implementer could not claim compliance with nothing?
- Look for authority leaks in "Bindings", "Debt operations" and "Depth enrollment": any path by
  which acquisition, execution, an executor, a pause, a deleted field or a later version gains or
  sheds obligations it should not.
- Check "Path enablement" for circularity or a way to enable the maintenance path with an unguarded
  onboarding or publication route.
- Check "Shared mapping and capabilities" against `capabilities-schema-spec.md`, the charter's
  promotion rule and `support-matrix.md` (which says the support matrix and capability matrix
  "MUST NOT share meaning" and restricts support-matrix inputs to manifest-root evidence).
- Check whether any rule quietly requires a new registry, ledger, inventory or approval step that
  the minimum-machinery constraint forbids.

## Lane intent — `sonnet-max-defect-reviewer` (append to the shared part)

Review intent for YOUR lane (defects and conformance). Work through the contract section by section
against the sources. In particular:
- For every row of the "What this contract does not change" table, open the named owner document
  and confirm (a) the contract's description of the existing rule is accurate, and (b) no rule
  elsewhere in the contract changes it.
- For every factual statement about existing behavior (coverage level names, name identity fields,
  debt row fields that a renewal may change, request freeze points, lifecycle stages, the meaning
  of `runtime_family`, where support-matrix intake is allowed to read), find the source text and
  confirm it. Quote the source where the contract misstates it.
- Compare the contract with the two research documents: list any rule the research requires of a
  common contract that the contract omits, and any rule in the contract that the research does not
  support or contradicts. Distinguish deliberate changes listed under "Maintainer decisions" from
  unexplained divergence.
- Check internal consistency: every term used is defined; every cross-reference and anchor
  resolves; no two rules conflict; the conformance table at the end matches the rules above it and
  adds no new requirement; numbered lists and tables are complete.
- Flag any MUST that an implementer could not decide or test, and any place where a safe
  simplification would remove words without changing meaning.
