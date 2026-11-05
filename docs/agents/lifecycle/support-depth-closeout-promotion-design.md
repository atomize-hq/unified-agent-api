# Support depth: isolated closeout, promotion and integration (B.5, B.6)

Status: CP-P1 to CP-P6 approved by the maintainer on 2026-11-05, after two independent reviews.
For CP-P3 the maintainer chose option A; option B is kept below, marked rejected, for the record.
The Annex A.1/A.2 amendment that option A needs is a separate docs PR; this document does not edit
the contract.
Source baseline: `feat/depth-isolated-ma4-route` `f8fcdc8` (PR #323). It assumes PR #324 (B.4
part 2: MA4 writes a reporting E from authenticated isolated evidence) lands first.
This is roadmap items B.5 and B.6 of [MP-P1](support-depth-machinery-proof-rescope.md), on the
synthetic subject of the [synthetic subject design](support-depth-synthetic-subject-design.md).
The [depth contract](../../specs/support-depth-contract.md) (Annex A rows MA5, MA6 and IN1,
A.5, D6, D7, "Path enablement"), the [evidence contract](../../specs/support-depth-evidence-v1.md)
and the approved [isolated route design](support-depth-isolated-route-design.md) stay normative.
This document changes no code, registry, workflow, host setting or credential, and creates no
background job.

## Outcome

After B.5 and B.6, the existing public commands carry the synthetic subject (`depth_fixture`
`0.2.0`, `x86_64-unknown-linux-gnu`, `wrapper_only`) from a delivered reporting E to an accepted,
promoted version, in process and under the cfg(test) route:

1. audit and an exact-generation stand-down marker on the integration parent;
2. **MA5**, through each of `close-agent-maintenance` and `prepare-agent-closeout --write`;
3. **IN1** (`manifest-validate --tip`) admits one resulting closeout candidate before merge, and
   validates it again after the push;
4. **MA6**, `manifest-promote`, as a separate change. The A.5 replay forks from the admitted
   closeout, before any promotion;
5. **IN1** admits the promotion candidate before merge and validates it after the push.

These steps are proof plan steps 4 to 7 ([proof plan](support-depth-isolated-proof-plan.md)), run
in process. The native merge queue, the protected host and the credential stay phase C. In
production, `isolated_maintenance::route` returns `None` (`LIVE_PROFILE = None`,
`isolated_maintenance/mod.rs:38`). Every route keeps its production refusal and
`production_results` stays empty. Nothing below names Codex or any other agent: the subject is
whatever compiled `scope::Subject` the installed profile names.

## Inventory: what exists and what is still production-bound

### Owning cores (shared by MA5, MA6 and IN1)

- `depth_acceptance/manual.rs` is the closeout predicate. `plan` (`:42`) always uses
  `Production` (`:29-41`). `Production` takes bindings from
  `support_depth::current_closeout_bindings` and evidence from
  `depth_evidence::production_results`, which returns no verdicts (`depth_evidence/results.rs:33-38`).
  `acceptance` (`:143`) checks:
  - the exact current request;
  - passed preflight and the commit binding;
  - the generation marker on the tip (`:186-197`);
  - `admitted` (`:220`).
  `admitted` refuses nonempty `acceptance_work` (`:250`) and requires each due obligation to be
  `verified` by the host's verdicts (`:270`). `integration_tip` (`:208-219`) reads the local
  `origin/staging` ref.
  `plan_with` admits only the closeout's own outputs: the closeout, `HANDOFF.md`, the remediation
  log and lifecycle state (`output_paths`, `:318-326`). Any publication output is refused as
  "unowned" (`:63-67`).
- `depth_acceptance/promotion.rs` is the promotion predicate. It has the same `Production` shape
  (`:17-29`), using `current_promotion_bindings`. `acceptance` (`:116-195`) requires:
  - a retained closeout entry;
  - `manual::admitted`;
  - marker retirement in the same proposal (`:150-180`).
  `integration_tip` (`:197-208`) is local too.
- `depth_acceptance/integration.rs` is IN1. `check` (`:12-13`) always passes `manual::Production`
  and `promotion::Production`; `admit` (`:108`) reuses both predicates (`:198`, `:216`).
  Attribution (`:181-195`) treats only the lifecycle closeout kinds as closeout effects. Every other
  effect, including a newly qualified publication row (`publication.rs::detect`, `:154-164`),
  counts as promotion.
- Publication consistency: `publication::validate_current` (`publication.rs:196-211`) requires
  the committed support rows to equal the derived ones, including depth facts
  (`support_matrix/consistency.rs:388`). The derivation marks a promise qualified as soon as the
  record lists any entry made for what it states now (`support_matrix/depth.rs:188-193`, `:378`).
  The support-matrix spec says the same (`support-matrix.md:360-363`): a closeout entry alone
  qualifies a promise whose obligations are verified.
- Evidence evaluation needs a Git workspace. `verdict.rs::evaluate` (`:101`) opens
  `GitInputs::new(workspace)`, which refuses anything but a Git top level (`git_inputs.rs:24-29`).
  On #324's branch, `isolated_record_results` re-resolves `isolated_maintenance::evidence_host(root)`
  from the root it is given (`verdict/github/mod.rs:622`). It does not borrow a caller's context.
  The verdict seal covers the acceptance entries (`depth_evidence/results.rs:6-28`), so verdicts
  computed before an entry is appended read `unverified` for the post-entry record.
- `support_depth/closeout.rs::current_bindings` reparses the registry with `parse_unenforced`
  from Git inputs. It is not production-bound, but it never compares that registry with the
  route's scoped registry.
- `agent_maintenance/closeout_proposal.rs::Proposal`:
  - It already has `isolated: Option<Context>` (`:31-34`), `capture_isolated` (`:43-54`) and a
    scratch `isolated_view` (`:75-99`). Both are dead code: only the unused
    `Context::proposal` (`isolated_maintenance/mod.rs:304-307`) reaches them.
  - `plan` and `apply` (`:111`, `:225`) always admit with `plan_manual_closeout`, which is
    production. The context only rechecks freshness and never selects the evidence host.

### MA5 entrypoints

- `close-agent-maintenance`:
  - `closeout.rs::run_in_workspace` calls `require_stand_down`. That goes through
    `stand_down::run`, whose registry is already routed (`stand_down.rs:208`).
  - `closeout/write.rs::write_closeout_outputs` uses `Proposal::capture` (`:82`), not the isolated
    capture. `plan_closeout_mutations` does too (`:39`).
- `prepare-agent-closeout --write`:
  - `prepare_closeout.rs:107-111` uses `Proposal::capture`.
  - CI preflight is read from the fixed canonical repository: `closeout/evidence.rs:46`, `:123`.
- Production `AgentRegistry::load` calls still on the MA5 path. Each one refuses a
  `support_depth` table:
  - `closeout/write.rs:103` (lifecycle update);
  - `prepare_closeout.rs:185`;
  - `closeout/findings.rs:163`;
  - `closeout/support_audit_truth.rs:276` (reached from `validate.rs:111`);
  - `drift/mod.rs:173` and `drift/shared.rs:50` (reached from `validate.rs:256` and
    `findings.rs:106`);
  - `agent_lifecycle/maintenance_adoption.rs:172` (reached from `validate_in_workspace`,
    `closeout/write.rs:140`).

### MA6 entrypoint

- `manifest_promotion.rs::build_plan`:
  - It uses `Proposal::capture` (`:103`).
  - It loads the **production** registry from an unversioned scratch copy (`:108`).
  - It renders support JSON, Markdown and runtime data (`:226`), plus the capability matrix
    (`:236`), and runs `manifest_validate::run` on the scratch root (`:253-264`).
  - Each of those reloads the registry through `isolated_maintenance::registry`
    (`support_matrix/derive.rs:264`, `capability_publication.rs:68`,
    `manifest_validate/depth.rs:63`).
  - With a route installed, the scratch tree cannot resolve one (it has no `.git`; see
    `Workspace::read`, `mod.rs:361-437`), so each reload errors. Without a route, each reload
    refuses depth. This is the known "cannot run in a scratch root" gap.
- `require_revision` (`:69-99`) compares HEAD and the local `origin/staging`, not an authenticated
  tip.
- `crate::manifest_version_metadata::plan` (`:162`) is the internal planner. Only its public entry
  refuses depth (`manifest_version_metadata.rs:203`), so this is not a gap.
- Support publication still evaluates verdicts with `production_results`
  (`support_matrix/derive.rs:400`). So an isolated promotion would publish `unverified`.
- `parity-promote.yml`:
  - It plans the validation matrix (acquisition plan already routed, `manifest_acquisition.rs:153`).
  - It runs the declared validation per target, then `manifest-promote --write` (`:289-292`), then
    `manifest-validate --root` (`:300`).
  - It opens the promotion PR with `AUTOMATION_TOKEN`.
- Neither MA5 writer renders publication. `closeout/write.rs::closeout_files` (`:62-74`) proposes
  only the closeout, `HANDOFF.md` and the remediation log (plus lifecycle state, `:86-88`).
  `prepare_closeout.rs::write_and_verify` (`:285-289`) proposes the closeout alone.

### IN1 entrypoint and host

- `manifest_validate/depth/merge.rs::validate` calls `check_integration` repository-wide (`:44`).
  It then loads the **production** registry with `.ok()` (`:47`). With a depth table, that yields
  `None`, and the subject root's records report `DEPTH_RECORD_UNANCHORED`.
- `ci.yml` runs `--tip` validation only for the fixed roots `codex`, `claude_code` and `opencode`
  (`:345-390`). Its parent comes from `scripts/check_staging_integration.py tip`: the PR base or
  the merge-group base. `depth-tip.yml` loops over the same three (`:34-36`) after the push. It
  passes `github.event.before` as `--tip` (`:26-35`), so there the candidate is the new protected
  tip and the parent is the previous one. It checks out shallowly (`:18-20`, `--depth=1` at `:33`).
- The preflight and manifest-validation jobs in `ci.yml` do not depend on the `rust-tests`
  producer job. The route design requires evidence consumers to be scheduled after the terminal
  producer through reviewed `needs` edges (`support-depth-isolated-route-design.md:154-161`). Contract A.4 says all-root admission does not replace
  per-root validation of a root that holds a depth record.
- Route resolution (`isolated_maintenance/mod.rs:259-286`, `host.rs:91-182`) requires:
  - a clean, non-shallow, unshared checkout whose `origin` is one of two spellings (`mod.rs:409-423`);
  - an authenticated `/usr/bin/gh` read of both repositories (`host.rs:24-35`);
  - a HEAD that descends from the protected tip and exists on the host (`host.rs:96-109`).

## Gaps and recommended approach

Each gap has one recommendation. "Isolated host" means a value that only a resolved route can
make. It carries no serializable token and no caller-built success.

### G1 — the owning predicates always use the production host (MA5, MA6, IN1)

Recommendation: add a private `depth_acceptance::Authority<'a>` with two variants,
`Production` and `Isolated(&'a isolated_maintenance::Context)`. Pass it into
`plan_manual_closeout`, `plan_promotion` and `check_integration`. The `Isolated` host implements
the existing `manual::Host` trait (`manual.rs`) in a new `depth_acceptance/isolated.rs`:

- `bindings` = `current_closeout_bindings` or `current_promotion_bindings`, plus a check that the
  registry they parse equals `Context::registry(root)`.
- `evidence` = `Context::evidence_host(root)` (`mod.rs:308-319`), passed to a new host-taking
  evaluation entry in `depth_evidence` that borrows that `EvidenceHost`, as #324's
  `isolated_assembly` already does (`verdict/github/mod.rs` on #324). It must not call
  `evaluate_isolated_record`, which re-resolves authority from its root (`:622` on #324). The
  verdict is re-authenticated for the exact record state on every call. E labels are never
  trusted, and an earlier verdict set is never resealed. This runs on the caller's Git workspace
  only; scratch evaluation is G7's separate question.
- A new `tip` method returns the authenticated protected tip. It refuses unless the local
  `origin/staging` equals that tip. This replaces `manual.rs:208` and `promotion.rs:197` for the
  isolated host only.

`Proposal` (`closeout_proposal.rs`) forwards `self.isolated.as_ref()` as the authority. A caller
cannot pick the `Isolated` variant without a `Context`, and only `Context::resolve` makes one.
`Production` behavior stays byte-identical.

### G2 — MA5 writers never capture the isolated proposal

Recommendation: in `closeout/write.rs` (`:39`, `:82`) and `prepare_closeout.rs` (`:107`), call
`isolated_maintenance::route(root)`. When it returns `Some`, use a new `Route::proposal(self, root)`,
which wraps the existing `Context::proposal`. When it returns `None`, use `Proposal::capture` as
today. An installed route that fails to resolve refuses, as at `mod.rs:85-94`; it never falls back.

### G3 — production registry loads on the MA5 path

Recommendation: replace each load listed in the MA5 inventory with `isolated_maintenance::registry`,
the B.3 part 2b pattern (`mod.rs:100-111`). It yields the production load, with its refusals,
whenever no route is installed. The proposal's own recheck at apply (`closeout_proposal.rs:240-243`)
stays the binding authority.

These readers are shared with `check-agent-drift`, onboarding and lifecycle validation. Production
behavior is unchanged because the routed function is the production load when there is no route.

### G4 — closeout preflight is read from the canonical repository

Recommendation (CP-P4): under a resolved route, `prepare-agent-closeout` resolves the preflight
runs from the authenticated isolated repository (`EvidenceHost::repository`, `mod.rs:221-224`).
It never uses `origin` or a flag. Without a route, the canonical `EVIDENCE_REPO` stays. The
existing run selection and whole-workflow terminality rules are unchanged. CP-P4 was approved
2026-11-05, so both MA5 writers are exercised.

### G5 — MA6 renders in a scratch tree that cannot resolve the route

Recommendation (CP-P2): MA6 resolves the route once, in the caller, and captures it with
`Route::capture` (`mod.rs:55-61`). It builds the proposal through `capture_isolated`, and the
scratch tree through the existing `Proposal::isolated_view` (`closeout_proposal.rs:75-99`). That
view requires an empty destination, records its identity and refuses a `.git`.

A new `CapturedRegistry::bind_view(&IsolatedView)` binds the captured registry to that scratch
root, for this thread, until the guard drops. It reuses the `CANDIDATE` thread-local and
`candidate_registry` lookup (`mod.rs:162-182`). Binding requires that:

- the view's proposal still verifies against the caller;
- the destination identity is unchanged;
- the scratch registry bytes equal the captured registry.

The renderers and `manifest_validate::run` then get the scoped registry through their existing
`isolated_maintenance::registry` call, unchanged. The scratch tree never resolves authority itself
(evidence contract, "Isolated maintenance binding foundation").

`build_plan` replaces `AgentRegistry::load(scratch)` (`manifest_promotion.rs:108`) with the
captured registry. Without a route, `build_plan` is unchanged.

Amended by the maintainer 2026-11-05: the scratch view also borrows the caller's Git inputs (read-only, pinned to the caller commit, no authority) for bound evidence; established by B.6a's positive scratch test.

Publication still renders production verdicts until B.7. Although `production_results` reads no
Git input, `bound_evidence` reads Git-backed evidence identities when execution evidence references
exist; `manifest_validate::run` also reaches this read through publication consistency. The binding
supplies the caller's pinned objects while retaining regular-file, byte and mode checks in scratch,
and rechecks the caller and scratch identities after binding. Without a bound view, this read is
unchanged. The binding gives no evidence authority. Scratch evaluation of authenticated evidence
needs the separate view-aware evaluator in G7 (CP-P3).

`isolated_maintenance/mod.rs` is 549 code lines on this base, and PR #324 adds about 229. Put the
binding in a new `isolated_maintenance/view.rs` to stay under 700.

### G6 — MA6 binds to unauthenticated local refs

Recommendation: under a route, `require_revision` (`manifest_promotion.rs:69-99`) also requires
the validation commit to equal the authenticated protected tip. `PromotionPlan::apply`
(`:43-55`) rechecks the captured route before delivery, as MA4 does
(`CapturedRegistry::recheck`, `mod.rs:132-134`). The promotion predicate gets the isolated host
from G1.

### G7 — publication renders production verdicts

Swapping the evaluator in `support_matrix/derive.rs:400` alone **breaks the closeout chain**,
because of the publication-consistency facts in the inventory:

1. The closeout entry qualifies the promise (`support_matrix/depth.rs:188-193`, `:378`;
   `support-matrix.md:360-363`).
2. Neither MA5 writer renders publication, and `manual.rs:63-67` would refuse it if one did.
3. IN1 rejects the stale rows (`publication.rs:196-211`).
4. IN1 would attribute a rendered qualification to promotion, not to closeout
   (`integration.rs:181-195`).

Qualified output also needs authenticated evidence for the **post-entry** record. The seal covers
entries (`results.rs:6-28`), so that evaluation must run after the entry is appended. It must also
run where publication is rendered, which for MA6 is the scratch view. The evaluator cannot run
there (`verdict.rs:101`, `git_inputs.rs:24-29`, and `evidence_host(root)` on #324).

CP-P3 is therefore a package, not one switch. Both semantics below are consistent with some
current text, and each needs a normative amendment. **The maintainer chose option A on
2026-11-05.** Option B was rejected and is kept here for the record:

| | **A — closeout publishes qualification (chosen)** | **B — qualification waits for promotion (rejected)** |
| --- | --- | --- |
| Rule | Keep the spec rule: any current entry qualifies. The MA5 proposal becomes one coherent batch: admit the entry, freshly evaluate the post-entry record, render support JSON, Markdown and runtime data. | Amend the support-matrix spec (`:360-363`) and record invariant 4 so that only a current `Promotion` entry qualifies. MA5 publication stays unchanged and unqualified. |
| Code changes | `manual.rs` admits publication outputs for the closeout's own agent and version only. IN1 attribution (`integration.rs:181-195`) counts a qualification effect as closeout when the candidate adds a matching `MaintenanceCloseout` entry and no promotion effect; otherwise it stays promotion. Both MA5 writers render publication through that proposal. | `support_matrix/depth.rs` derivation, plus a reordered MA6 (below); no MA5 or IN1 attribution change. |
| Normative change | Annex A.1 (MA5 outputs) and A.2 (publication writers) gain MA5. | Support-matrix spec and record invariant 4. |
| Evaluation in scratch | Needed for MA6 and for any MA5 rendering in a view. Add the private view-aware evaluator described below. | Same, for MA6. |
| Risk | Widens MA5's write envelope to aggregate publication, which touches every agent's rows. | Changes a published meaning for every depth-enrolled agent, not just this proof. |

**MA6 ordering (approved with option A).** Today MA6 renders publication first
(`manifest_promotion.rs:226-240`), then plans admission (`:269`), and admission appends the
`Promotion` entry (`promotion.rs:95-98`). Under rejected option B, the rendered row would stay unqualified, while
IN1 derives qualification from the delivered post-entry record and rejects the mismatch. MA6
therefore runs in this order:
1. admit and append its own entry;
2. build the proposed post-entry view;
3. evaluate that exact record freshly, with the view-aware evaluator;
4. render support, runtime and capability publication from that view;
5. admit the rendered outputs again, and deliver everything as one batch.

Under the chosen option A, the promotion entry does not change qualification. The same order
still keeps verdicts sealed to the delivered record (`results.rs:6-28`), and it was approved with
option A. The owning files are
`manifest_promotion.rs` and `depth_acceptance/promotion.rs`.

**View-aware evaluator (approved with option A).** This is a private evaluation path that:
- borrows the caller's authenticated `Context` and `EvidenceHost`, never resolving one from the
  scratch view;
- reads proposed bytes from the `IsolatedView` source;
- reads immutable provenance (commits, trees, blobs) from the caller's Git object store through
  the caller's `GitInputs`;
- checks dependency currentness against the proposed view;
- evaluates the exact post-entry record.

Its freshness check is the view's `verify_current` plus `Context::recheck` before and after
evaluation. It may write nothing, and its verdicts are sealed to the evaluated record only. It
needs changes in `depth_evidence/` (`verdict.rs`, `verdict/github/mod.rs`, `currentness.rs`) and
`git_inputs.rs`. Those are outside this packet's write scope, and the other worker owns
`depth_evidence/` today, so these slices must be scheduled after #324 and coordinated with that
owner. Before any MA6 wiring relies on it, a positive scratch-publication test must show a verified
row rendered in scratch.

**Before B.7 lands:** publication keeps `production_results` and stays `unverified`. The
closeout and promotion chains then hold publication consistency without any publication change.
Chain tests run before B.7 establish only that chain. B.7b reruns both complete MA5→IN1 chains and
the MA6→IN1 chain with option A on; the earlier proofs do not carry over. The proof is not complete
until B.7 lands; a run before it records "qualified publication not exercised".

### G8 — IN1 uses production hosts and the production registry

Recommendation:

- `check_integration` resolves `isolated_maintenance::route(workspace)` before root narrowing.
  `Some` selects `Authority::Isolated`; an error refuses.
- `merge.rs:47` uses `isolated_maintenance::registry`, with the same production/isolated error
  split as `manifest_validate/depth.rs:63-73`.
- IN1 has two uses: admission before merge (`ci.yml`) and observation after the push
  (`depth-tip.yml`). A single "parent equals the current tip" rule would reject every successful
  post-push readback. So the binding (CP-P1) is chosen from authenticated facts, never from a flag,
  an event name or another caller assertion. With T the authenticated protected tip, H the
  checkout HEAD and P the supplied parent:
  - **Admission:** P = T, H ≠ T and H descends from P. This covers merge-group and PR
    candidates. A merge-group entry stacked on another entry has P ≠ T and refuses; the queue
    rebuilds it. A PR merge ref built on an older base also refuses.
  - **Observation:** H = T and P is the first parent of H. That parent relation is read from Git
    objects reachable from the authenticated T, not from the event payload. The full admission
    then runs for the P → H transition, as D7 requires. Observation detects a bad transition; it
    cannot undo the push (Annex A.3).
  - Anything else, including H = P, refuses.

  Whether P must be the *first* parent of H depends on the queue's merge method. With a merge
  commit, a squash or a rebase, the base is H's first parent. A multi-commit rebase would put
  several commits between P and H, so observation would need "P is an ancestor of H on the
  first-parent chain". CP-P1 approved single first-parent only; widening it needs a separate
  maintainer decision.

A route resolved in the integration checkout grants no final permission by itself. The
existing all-root generation, closeout and promotion checks decide.

### G9 — CI and host wiring of the isolated repository (phase C)

Recommendation (CP-P5): B.5 and B.6 change no workflow. Record these as phase C prerequisites,
each delivered by a **reviewed change to the existing canonical workflows**:

- **Consumer ordering.** Trace every evidence consumer that runs under the route: the preflight
  run selection, every `manifest-validate --tip` step, support publication checks and
  `make preflight`/support-matrix. Schedule each after the terminal `rust-tests` producer through
  minimal explicit `needs` edges. The producer must never depend on its own admission. A failed
  producer must stay observable rather than silently skipping its consumers. The 17-job result
  policy and the terminal gate stay unchanged (`support-depth-isolated-route-design.md:142-161`).
- **Per-root validation.** `--tip` validation of every root that holds a depth record, in the
  merge-queue and post-push jobs. The fixed three-root loops (`ci.yml:345-390`,
  `depth-tip.yml:34-36`) do not cover the subject root, and contract A.4 says all-root admission
  does not substitute for that.
- **History, credentials and origin.** Full-history checkouts in every job that resolves the
  route, `depth-tip.yml` included (the route refuses shallow history, `mod.rs:415-420`). A
  least-privilege read credential reference for the route's host observations of both
  repositories (`host.rs:24-35`). An `origin` spelling that `Workspace::read` accepts
  (`mod.rs:409-414`).

Workflow source belongs to the independently approved protected source (route design,
`:112-118`). So the isolated repository runs the canonical bytes, and these become canonical
changes, not isolated-only edits. For example, the root loops derive from the registry instead of
a fixed list. These changes must keep ordinary admission semantics where no depth record exists:
the same checks still decide, under the same result policy and terminal gate. They are not free,
though. New `needs` edges serialize ordinary jobs behind `rust-tests`, and full-history checkouts
cost time in every run, depth or not. So each change needs a scheduling and latency review of
ordinary CI before it lands. Claiming no latency impact is ruled out (route design `:159-160`).
Letting the isolated
workflows diverge from canonical would be an exception to that source boundary, and it would need
an explicit contract amendment. This design does not propose one.

### G10 — the A.5 replay with a single-version subject

`scope.rs:112-119` refuses any retained record other than the subject's version.
Recommendation (CP-P6): the replay opens an ordinary, non-depth `0.3.0` maintenance generation.
The declaration still names only `0.2.0`, so that generation has no record. The isolated scope is
not widened.

The replay **forks from the committed, IN1-admitted MA5 state, before any MA6**. On that fork:
1. The `0.3.0` generation replaces the working request.
2. Its own ordinary closeout replaces the closeout file.
3. MA6 for `0.2.0` must then create the **first** `Promotion` entry and move the pointers, from
   the retained closeout entry alone.

This proves A.5. Repeating MA6 after a promotion would only exercise an idempotent re-run whose
entry already exists. A sibling fork from the same checkpoint changes wrapper code that a `0.2.0`
result depends on, and MA6 must refuse there.

## Approved decisions

| Id | Decision | Reason / boundary |
| --- | --- | --- |
| CP-P1 — isolated owning host and IN1 binding | **Approved by the maintainer 2026-11-05.** MA5, MA6 and IN1 admit through one private `Authority::Isolated(&Context)`. Evidence is re-authenticated for the exact record state through a host-taking entry that borrows the caller's `EvidenceHost`. The local integration tip must equal the authenticated protected tip T. IN1 picks its binding from authenticated facts only: **admission** (P = T, H descends from P, H ≠ T) or **observation** (H = T, P is H's single first parent read from Git); anything else refuses. | Applies the approved IR-P1 threading to the owning cores without a second predicate, and keeps D7's post-push readback (proof plan step 7). Admission serializes depth candidates in the queue, which is conservative. Widening observation to a multi-commit first-parent chain needs separate approval. Production stays `Production`. Revised after review: the unconditional parent-equals-tip rule is withdrawn. |
| CP-P2 — promotion scratch registry binding | **Approved by the maintainer 2026-11-05.** MA6's scratch view inherits the caller's captured scoped registry through an `IsolatedView`-attested, thread-bound guard. It never resolves itself and gives no evidence authority. Amended by the maintainer 2026-11-05: the scratch view also borrows the caller's Git inputs (read-only, pinned to the caller commit, no authority) for bound evidence; established by B.6a's positive scratch test. | Enough for MA6 until B.7 switches publication to isolated verdicts. Publication and validation need Git-backed bound-evidence identities even with production verdicts; the borrowed objects preserve the evidence identity checks. Extends the MA4 candidate binding to a non-Git view, an authority-boundary change. Scratch evidence evaluation is CP-P3's view-aware evaluator, not this row. Revised after review. |
| CP-P3 — qualified publication (package) | **Approved by the maintainer 2026-11-05: option A chosen.** **A (chosen):** a closeout publishes qualification, through one MA5 batch (entry, fresh post-entry evaluation, rendering) and closeout attribution at IN1, with Annex A.1/A.2 amended in a separate docs PR. **B (rejected, kept for the record):** only a current `Promotion` entry qualifies, with the support-matrix spec and record invariant 4 amended. Option A also needs the private view-aware evaluator (borrowed context, proposed bytes, caller Git objects, post-entry record, no writes), and MA6 reordered to entry → post-entry view → fresh evaluation → rendering. | The two options are compared side by side in G7. An evaluator-only switch breaks the closeout chain at IN1 (`publication.rs:196-211`, `integration.rs:181-195`). Until B.7 lands: production verdicts and `unverified`, recorded as not exercised. B.7b reruns every chain test with option A on. Revised after review. |
| CP-P4 — closeout preflight repository | **Approved by the maintainer 2026-11-05.** Under a route, `prepare-agent-closeout` reads preflight from the authenticated isolated repository (from the context, never `origin` or a flag). The existing run selection and whole-workflow terminality rules stay. | The packet's CI runs live there. Both reviewers agreed. |
| CP-P5 — CI prerequisites (phase C) | **Approved by the maintainer 2026-11-05.** The B.5 and B.6 PRs change no workflow. Phase C delivers reviewed changes to the **canonical** workflows: route consumers ordered after the terminal `rust-tests` producer with failure-preserving `needs` edges; registry-derived per-root `--tip` loops; full history in every routed checkout; an accepted `origin`; a least-privilege read credential reference. The 17-job result policy and the terminal gate are unchanged. | Workflow source is part of the independently approved protected source (route design `:112-118`, `:154-161`). Isolated-only divergence would need an explicit contract amendment; none is proposed. Revised after review. |
| CP-P6 — A.5 replay shape | **Approved by the maintainer 2026-11-05.** Replay with an ordinary (non-depth) later-version generation, forked from the admitted MA5 state **before the first MA6**. MA6 must then create its first `Promotion` entry from retained closeout authority. A sibling fork with dependency-invalidating changes must refuse. | `scope.rs:112-119` admits only the subject's version. Both reviewers agree; forking before the first MA6 is per their notes. |

These are existing requirements, not new decisions: maintainer classification, empty carried
`acceptance_work` (no corroborator), the exact generation marker on the integration parent,
separate closeout and promotion, owning admission at apply, all-root IN1, the native merge queue,
and separate per-agent production authorization (contract, "Path enablement" item 5).

## Risks

- **Carried work.** If the audit for the real `0.2.0` O leaves required uplift rows,
  `manual.rs:250` refuses closeout. That is correct: stop and bring a corroborator design. Never
  clear the work (the IR-D1 checkpoint, route design).
- **Evidence currentness at promotion.** The promotion commit differs from the packet commit.
  Verification depends on the evaluator's reuse and dependency rules (evidence contract, "Read-only
  verdict evaluation"). If pointer, metadata or publication files fall inside the dependency set, E
  goes stale, and MA6 must refuse rather than reseal.
- **Host read volume.** Each routed registry load re-observes the host. MA5 makes several, so the
  test fakes must count reads (`Fake.2`, `isolated_maintenance/tests/fixture.rs`). The live host
  may hit rate limits, so resolve once per command where the code allows.
- **PR-event IN1.** A PR merge ref built on an older base does not descend from a moved staging
  tip, so the route refuses. The merge queue rebuilds; required PR checks may need a re-run.
- **Thread-local binding.** The guard must drop on every exit path, and renderers must not spawn
  threads that read the registry. Spawned threads see the production load and refuse. They fail
  closed, but noisily.
- **LOC.** `isolated_maintenance/mod.rs` and `tests.rs` (704 lines) are near the cap. New code goes
  in new submodules.
- **Green gates.** The subprocess green-gate route gap from B.4 also affects the closeout's
  `preflight_passed`. In process it stays stubbed, as in B.4.

## Tests

All tests are credential-free, use `Fixture::install()` and the fake host, and live in new files
under `crates/xtask/src/isolated_maintenance/tests/`. They do not touch `ma4_route.rs`.

**Positive chain** (one connected test per MA5 alternative, from the `ma4_evidence` end state of
PR #324):

1. MA1 → acquire → re-freeze → MA4. E is delivered and the audit is clean.
2. Commit the exact-generation marker to the fake protected tip.
3. MA5:
   - `close-agent-maintenance`: the closeout is written, and the record gains exactly one current
     `MaintenanceCloseout` entry.
   - `prepare-agent-closeout --write`: the same, with the frozen-fetcher preflight from the
     isolated repository.
4. IN1 admission (`check_integration` with the route, P = T) admits the closeout candidate. The
   fake host then advances T to that candidate, and IN1 observation (H = T, P = H's first parent)
   passes for the same transition.
5. Checkpoint "admitted MA5". From it, run MA6 `build_plan` and `apply` with the captured route.
   Expect one coherent proposal:
   - pointers, metadata and `current.json`;
   - support, runtime and capability outputs;
   - one `Promotion` entry;
   - the marker retired.
6. IN1 admission and post-push observation both pass for the promotion candidate.
7. A.5 replay, forked from the "admitted MA5" checkpoint **before any MA6** (CP-P6):
   - an ordinary `0.3.0` generation replaces the request, then its ordinary closeout replaces the
     closeout file;
   - MA6 for `0.2.0` creates its **first** `Promotion` entry and moves the pointers;
   - IN1 admits it.

   A sibling fork from the same checkpoint changes wrapper code that a `0.2.0` result depends on,
   and MA6 refuses there.

Before B.7, publication stays `unverified` throughout, and IN1 publication consistency holds
unchanged. B.7b reruns steps 3 to 7 with option A on, and adds:
- a qualified row appears only through the MA5 batch, in the same candidate as its closeout entry;
- IN1 attributes that qualification to closeout. A qualification effect without a matching new
  `MaintenanceCloseout` entry, or one for another agent or version, refuses;
- the first promotion's rendered rows equal what IN1 derives from the delivered post-entry record
  (the row is already qualified, and stays so);
- a positive scratch-publication test with the view-aware evaluator.

**Refusals** (each one preserves caller state byte for byte):

- Missing marker, a candidate-only marker, or a marker for another generation.
- A stale request digest, or a changed P/O/E.
- Evidence becomes unverified because of:
  - a nonterminal or failed attempt;
  - revoked adoption;
  - a changed population.
- Nonempty `acceptance_work`, or a `classification_required` unit.
- An unsupported debt edit.
- A local `origin/staging` that differs from the authenticated tip.
- Identity drift between plan and apply:
  - route, tip or host identity drift;
  - a forged or copied context;
  - a scratch view with `.git`, a non-empty destination or a changed identity;
  - a registry byte change in scratch.
- IN1 binding refusals:
  - admission with P ≠ T (including a stacked merge-group base);
  - observation where P is not H's first parent, or H ≠ T;
  - H = P;
  - a forged transition: an event-supplied parent that the Git history reachable from T does not
    support.
- The proof plan's IN1 cases:
  - forged entries;
  - a promotion with no admitted retained closeout;
  - another root's record removed.
- MA6 with a moved validation commit or tip.
- MA6 after a later generation changed the wrapper code an earlier result depended on (expected:
  refuse).

**Production controls:** with no route installed, every touched entrypoint keeps its current
refusal on a depth workspace and its ordinary behavior otherwise. The existing suites run
unchanged: `tests/agent_maintenance_closeout/`, `tests/depth_promotion.rs`,
`depth_acceptance/manual/tests*` and `tests/c0_spec_validate/depth_records/merge*`.

## Implementation slices

Each slice is one reviewed PR into `staging`, stacked in this order after #324. No slice edits
`.github/workflows/` (phase C, CP-P5), `agent_maintenance/execute/` or
`isolated_maintenance/tests/ma4_route.rs`. Slices that touch `depth_evidence/` are marked; the
concurrent worker owns that directory today, so they wait for #324 and are coordinated with that
owner.

| Slice | Content | Write scope |
| --- | --- | --- |
| B.5a — owning authority (CP-P1, host part) | `Authority` and the isolated `Host`, the tip binding, `Proposal` forwarding, `Route::proposal`, and a host-taking evaluation entry that borrows the caller's `EvidenceHost`. No public route changes yet. Unit tests on the predicates with the fake host, plus production byte-identity controls. | `depth_acceptance/{mod,manual,promotion,integration}.rs`, new `depth_acceptance/isolated.rs`, `agent_maintenance/closeout_proposal.rs`, `isolated_maintenance/mod.rs` (accessors only), **`depth_evidence/{results.rs,verdict/github/mod.rs}` (entry point only, coordinated)**, new `isolated_maintenance/tests/authority.rs` |
| B.5b — both MA5 writers (G2–G4; CP-P4) | Route capture in both writers, routed registry loads, the isolated preflight repository (CP-P4). Positive chain to closeout for each writer, plus MA5 refusals. Publication unchanged. | `agent_maintenance/closeout/{write,findings,support_audit_truth,validate,evidence}.rs`, `agent_maintenance/prepare_closeout.rs`, `agent_maintenance/drift/{mod,shared}.rs`, `agent_lifecycle/maintenance_adoption.rs`, new `isolated_maintenance/tests/ma5_closeout.rs`, the B.5 notes in `support-depth-synthetic-subject-design.md` |
| B.5c — IN1 binding (G8; CP-P1, IN1 part) | Route-selected authority in `check_integration`; admission and observation chosen from authenticated facts; the routed registry in `merge.rs`. Closeout-candidate admission and post-push observation, plus the binding refusals. | `depth_acceptance/integration.rs` (and a new `depth_acceptance/integration/binding.rs` if needed), `manifest_validate/depth/merge.rs`, new `isolated_maintenance/tests/in1_integration.rs` |
| B.6a — scratch registry binding (CP-P2) | `CapturedRegistry::bind_view` over `IsolatedView`; removes the `dead_code` allowances that this slice now uses. Binding positive and refusal tests, plus a positive scratch render and `manifest_validate::run` with production verdicts (before B.7). | new `isolated_maintenance/view.rs`, `isolated_maintenance/mod.rs` (module line), `agent_maintenance/closeout_proposal.rs`, new `isolated_maintenance/tests/view.rs` |
| B.6b — MA6 through the route (G5, G6, G10) | Captured route, isolated proposal and scratch binding in `build_plan`, plus the tip and apply rechecks. Positive promotion with IN1 admission and observation, the A.5 replay forked before the first MA6 (CP-P6), MA6 refusals. | `manifest_promotion.rs`, new `isolated_maintenance/tests/ma6_promotion.rs`, the B.6 notes in `support-depth-synthetic-subject-design.md` |
| B.7a — view-aware evaluator (CP-P3) | Private evaluation over an `IsolatedView`: borrowed context and host, proposed bytes, caller Git objects, dependency currentness against the view, post-entry record, no writes. Positive scratch-publication test, plus refusals for a scratch `.git`, a moved caller and self-resolution. | **`depth_evidence/{verdict.rs,verdict/github/mod.rs,currentness.rs}` (coordinated)**, `git_inputs.rs`, `isolated_maintenance/view.rs`, new `isolated_maintenance/tests/view_evidence.rs` |
| B.7b — owning qualified publication (CP-P3, option A) | Prerequisite: the separate docs PR that amends contract Annex A.1 (MA5 outputs) and A.2 (publication writers) has landed; this slice does not edit the contract. Then: MA5's coherent batch (entry → fresh post-entry evaluation → rendering) in both writers; `manual.rs` admitting publication rows for the closeout's own agent and version only; closeout attribution in `integration.rs` (a qualification effect counts as closeout only with a matching new `MaintenanceCloseout` entry and no promotion effect); MA6 reordered to entry → post-entry view → fresh evaluation → rendering (G7, "MA6 ordering"); the isolated evaluator in `support_matrix/derive.rs` inside a route. Every chain test of B.5b, B.5c and B.6b reruns with option A on. | `depth_acceptance/{manual,integration,publication,promotion}.rs`, `agent_maintenance/closeout/write.rs`, `agent_maintenance/prepare_closeout.rs`, `manifest_promotion.rs`, `support_matrix/derive.rs`, new `isolated_maintenance/tests/publication.rs` and the chain test files above |

Phase C (CP-P5) carries the workflow and host prerequisites and the real native-queue run. Every
slice runs the required gates. B.7 is required for the complete proof, not optional. Approving this
design enables no path and installs no profile.
