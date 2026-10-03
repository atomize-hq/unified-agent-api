# Handoff: support-depth P1a contract, machine switch

Created 2026-10-03 10:15 America/New_York by the lead Claude Code session (Opus 5.5).
Continues `.codex/handoffs/2026-10-02-184000-support-depth-p1a-dual-review-dispatch.md`.

## Bottom line

- The contract `docs/specs/support-depth-contract.md` is a Draft that has been through five review
  rounds. No P1 remains. Three P2s are open against the reviewed candidate, all in one rule.
- A fix for those three exists as an **unreviewed** proposal.
- One decision is waiting on the maintainer (see "The open decision").
- Nothing is merged and nothing is on `staging`. Everything travels on one temporary branch:
  `tmp/support-depth-handoff-2026-10-03`. Never merge it; delete it once pulled.

## What is where

| Commit | Recreate as branch | What it is | Review state |
| --- | --- | --- | --- |
| `2797b8d9` | `docs/support-depth-policy-proposal` | P0 research package: `docs/agents/lifecycle/support-depth-policy-proposal.md`, `support-depth-policy-research-decisions.md`, backlog `uaa-0072` | Pro consult returned ADJUST with one line, applied. Maintainer has not read it yet. |
| `d78cf535` | `docs/support-depth-p1-contracts` | The contract draft, **reviewed candidate** (771 lines, `Status: Draft`) | Rounds 1 to 5. Three open P2s. |
| `2a956def` | `docs/support-depth-p1-contracts-r5-proposal` | Proposed fix for the round-five findings (88 insertions, 69 deletions against `d78cf535`) | **Unreviewed.** |
| tip of the tmp branch | do not recreate | Adds the local records below and two reviewer agent definitions | Snapshot only. |

All three are ancestors of the tmp branch, which is based on `staging` at `f61534be`.

Local records carried in the snapshot commit (normally gitignored under `.codex/`):

- `.codex/dossiers/support-depth-p1-dispositions-2026-10-02.md`: decisions D1 to D14, Pro
  refinements, deferrals with triggers.
- `.codex/dossiers/support-depth-p1a-review-adjudication-2026-10-02.md`: every finding from rounds
  1 to 5 with its decision and fix. Start here for the review history.
- `.codex/dossiers/support-depth-annex-a-route-facts-2026-10-02.md`: first-pass writer inventory
  for Annex A, plus the changes after round four. Read its last section before writing Annex A.
- `.codex/dossiers/support-depth-p1a-review-reports/`: the ten reviewer reports verbatim, and the
  two round-five packets as dispatched (reuse them as the bounded-review template).
- `.codex/dossiers/support-depth-policy-decisions-2026-09-28.md`,
  `.codex/guidance/2026-10-02-support-depth-p1-dispositions-approach-review.md` and the two
  2026-10-02 handoffs: background.
- `.claude/agents/opus-xhigh-adversarial-reviewer.md` and
  `.claude/agents/sonnet-max-defect-reviewer.md`: the two review lanes. Untracked on `staging`;
  whether to commit them for real is the maintainer's call.

## Resume on the new machine

```bash
git fetch origin tmp/support-depth-handoff-2026-10-03
git branch docs/support-depth-policy-proposal 2797b8d9
git branch docs/support-depth-p1-contracts d78cf535
git branch docs/support-depth-p1-contracts-r5-proposal 2a956def
git restore --source=origin/tmp/support-depth-handoff-2026-10-03 --worktree -- \
  .codex \
  .claude/agents/opus-xhigh-adversarial-reviewer.md \
  .claude/agents/sonnet-max-defect-reviewer.md
```

Then:

1. Restart the Claude desktop session. A running session does not load new `.claude/agents/*.md`
   files, so the two lanes are not dispatchable until a restart.
2. Edit the contract in a separate worktree checked out on the candidate branch. Reviewers read by
   commit in their own worktrees, so the candidate must be committed locally before dispatch.
3. Once everything is restored, delete the temporary branch. The repository is public and the
   `.codex` records are normally gitignored:

```bash
git push origin --delete tmp/support-depth-handoff-2026-10-03
```

## Working arrangement (maintainer decisions for this effort)

- No Codex. The lead authors and remediates. Review is two lanes: Opus 5.5 at `xhigh`
  (adversarial) and Sonnet 5.5 at `max` (defects and conformance). This overrides `CLAUDE.md`.
- Reviews are now narrow: only the delta of the last fix, not the whole document.
- Stopping rule: one fix, one confirming round, then the results go to the maintainer. The lead
  does not start another round on its own.
- Amending `docs/specs/**` is approved for the P1 contract inventory, rules only. No evaluator
  code, no runtime work, no depth enrollment, no pilot.
- Nothing is pushed or opened as a PR under a real branch name without the maintainer.

## Decisions already made

D1 to D7 and D9 are accepted as the research companion recommends. D8 (pilot version and target,
the new onboarding agent) is deferred to the enrollment checkpoints. The dispositions dossier has
the detail. The four that shape P1:

- **D10:** amend the onboarding charter's promotion allowlist from four entries to the eight the
  audit code already reads.
- **D11:** the maintenance path may be enabled before the onboarding path is proven, provided
  every route that can affect acceptance outputs enforces depth admission or refuses.
- **D12:** the first depth enrollment covers one exact upstream version. No successor rule yet.
- **D13:** serialize the affected writes first; per-writer compare-and-refuse is a later
  optimization.
- **D14 (option A, reaffirmed):** `uaa_support` in the support matrix is re-derived from capability
  plus depth truth for depth-enrolled generations, under a schema version bump. This amends the
  support-matrix spec's deliberate separation from capability advertising, so capability linkage
  must travel through depth facts under the manifest root. The `markdown_support_claim` drift
  checks also read `uaa_support`.

## Review history

| Round | Candidate | Scope | Distinct P2s | Outcome |
| --- | --- | --- | --- | --- |
| 1 | `e60664e2` | Full | about 13, plus 3 P1 | Fixed in `f495a750` |
| 2 | `f495a750` | Follow-up | 5 | Fixed in `dd14fb32` |
| 3 | `dd14fb32` | Follow-up | 8 | Fixed in `5ff62ab8` |
| 4 | `5ff62ab8` | Follow-up | 4 | Fixed in `d78cf535` |
| 5 | `d78cf535` | Bounded delta | 3 | Proposal `2a956def`, unreviewed |

Rounds three to five found defects in one place only: Complete mediation rule 7 (the validation
backstop for hand edits) and the depth record's integrity rules. That rule grew from 49 words after
round one to 266 words and 8 enumerated checks in the reviewed candidate, and 350 words and 10
checks in the proposal.

## Open findings against `d78cf535`

All three were introduced by the round-four fix. Both lanes confirmed the four round-four findings
are fixed.

1. **Event restatement misfires** (both lanes). The rule "nothing a record states may change while
   its P, O and E identities are unchanged" fires when a later generation of the same version
   restates only the Event. A nightly re-dispatch with unchanged policy on a record that has only P
   would fail validation. Nightly re-dispatch of unchanged versions is real (`uaa-0048`).
2. **Stated identities are never checked** (Opus). Two merges launder a rebinding: change the
   evidence and restate E, then restore the old E and republish `qualified` under the genuine
   acceptance. A variant drops a tuple and restates P.
3. **"The acceptance that granted it" is undecidable** (Sonnet). An acceptance entry does not name
   the effects it covers, and "supports a published value" can be read as requiring pointer
   rollback after an evidence refresh.

What `2a956def` does about them: each acceptance entry names the content identity of what the
record stated when it was made, which is recomputable from the record. The record's Event, P and O
must be those of the generation whose frozen request or approval the same revision holds, and
without one the record does not change. Contradictions are split into those decided in any
revision and those decided by a merge result against the tip it would replace.

## The open decision

The maintainer asked why this rule is written in prose instead of code. The answer given:

- By plan, P1 writes rules and P2 writes code, and that is right for policy.
- This rule stopped being policy and became an algorithm in English. Prose cannot be run, so the
  reviewers have been executing it by hand and finding test-level bugs.

The lead's recommendation changed as a result, and **the maintainer has not answered yet**:

1. Do not run another prose round on this rule.
2. Cut Complete mediation rule 7 back to the invariants: a depth record is never removed, a
   published claim never exceeds its record, nothing counts as accepted without an acceptance
   behind it.
3. Carry the ten enumerated checks and the reviewers' attack scenarios forward as the P2 test
   list. Their natural home is each manifest root's validator spec, which `manifest-validate`
   implements, written beside the record's schema.
4. Run one bounded review of that smaller change.

Alternatives offered: one more bounded round on `2a956def` as it stands, or the maintainer takes
the draft for review at `d78cf535` or with the proposal applied. Writing the validator now was
advised against because it needs the record's schema and the request-contract changes that carry P
and O, and neither exists yet.

If the answer is "make the cut": branch from `d78cf535`, keep from `2a956def` everything outside
rule 7 that both lanes asked for (the acceptance entry term, committed evidence named by path, the
lifecycle attribution wording, the smaller wording fixes), reduce rule 7 to invariants with a
pointer to Path enablement item 4, move the enumerated checks into a test list in the Annex A
route-facts dossier or a new P2 note, commit locally, and dispatch both lanes with the round-five
packet as the template.

## After the contract converges

1. Write Annex A, the route inventory: outputs backward to every writer, entrypoints forward to
   their effects, reconciled. The contract says it cannot be approved without it.
2. Dependent amendments, each a later commit: agent registry contract; maintenance request
   contract (the four bindings, frozen requirements); onboarding charter (D10 allowlist, and where
   P and O are frozen on the create lane); support-matrix contract (D14); capability documents;
   each manifest root's schema, rules and validator spec, including the depth record's file name
   and schema.
3. The maintainer reviews the P0 documents and the contract. Adoption is the maintainer changing
   the contract's status to `Normative`.

## Deferrals, with where each will be hit

- **D1:** `manifest-retain` deletes `reports/<version>/` wholesale for versions outside the keep
  set, which would delete a depth record. The contract now forbids removing one. Trigger: the
  retain entry in Annex A.
- **D2:** the admission checker runs from the candidate's own tree. Trigger: P2 design of the
  merge-time check.
- **D3:** a later version's request can replace the working files between a version's closeout and
  its promotion, leaving promotion only the record to check against. Trigger: the promotion entry
  in Annex A.
- The companion's section 10 verification script asserts pre-revision hashes. Fix on next edit.

## Operational notes

- Review packets must give the commit and tell the lane to run `git switch --detach <sha>` in its
  own worktree. A worktree-isolated lane cannot use `git -C` or `cd` into another worktree, so
  never name a lead worktree path in a packet.
- New agent definitions need a session restart before they can be dispatched.
- `~/.chatgpt-pro-browser/browser wait-done` can hang long after a Pro response has finished. A
  0-byte result file is the tell. Extract directly with `browser extract --out <path>` to a
  different path.
- Documentation changes are not gated in CI. A push to a `tmp/` branch triggers no workflow.

## Not carried over

- Claude Code memory under `~/.claude/projects/.../memory/` is machine-local. The two notes written
  this session are repeated under "Operational notes".
- The session transcript and scratchpad. The reviewer reports and packets were extracted from the
  transcript into the reports directory, so nothing in them is lost.
- The September 28 and 29 Pro captures under `.codex/guidance/` (about 1 MB). They fed the P0
  documents, which are committed.
- Reviewer worktrees under `.claude/worktrees/`. They are disposable.
