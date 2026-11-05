# Support depth: isolated closeout, promotion and integration (B.5, B.6)

Status: design for review. CP-P1 to CP-P6 below are **awaiting maintainer approval**.
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
3. **IN1** (`manifest-validate --tip`) admits one resulting closeout candidate;
4. **MA6**, `manifest-promote`, as a separate change, including the A.5 replay;
5. **IN1** admits the promotion candidate.

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
- `depth_acceptance/promotion.rs` is the promotion predicate. It has the same `Production` shape
  (`:17-29`), using `current_promotion_bindings`. `acceptance` (`:116-195`) requires:
  - a retained closeout entry;
  - `manual::admitted`;
  - marker retirement in the same proposal (`:150-180`).
  `integration_tip` (`:197-208`) is local too.
- `depth_acceptance/integration.rs` is IN1. `check` (`:12-13`) always passes `manual::Production`
  and `promotion::Production`; `admit` (`:108`) reuses both predicates (`:198`, `:216`).
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

### IN1 entrypoint and host

- `manifest_validate/depth/merge.rs::validate` calls `check_integration` repository-wide (`:44`).
  It then loads the **production** registry with `.ok()` (`:47`). With a depth table, that yields
  `None`, and the subject root's records report `DEPTH_RECORD_UNANCHORED`.
- `ci.yml` runs `--tip` validation only for the fixed roots `codex`, `claude_code` and `opencode`
  (`:345-390`). `depth-tip.yml` loops over the same three (`:34-36`) and checks out shallowly
  (`:18-20`, `--depth=1` at `:33`). Contract A.4 says all-root admission does not replace
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
- `evidence` = `Context::evidence_host(root)` (`mod.rs:308-319`) and then
  `depth_evidence::evaluate_isolated_record`, which PR #324 extends to the synthetic profile. It
  is re-authenticated for this exact record state on every call. E labels are never trusted, and
  an earlier verdict set is never resealed.
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
It never uses `origin` or a flag. Without a route, the canonical `EVIDENCE_REPO` stays. Until
CP-P4 is approved, the conservative option is used: under a route, `prepare-agent-closeout`
refuses, and only `close-agent-maintenance` (with a maintainer-written closeout) is exercised.
That leaves "both MA5 alternatives" unmet.

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

`isolated_maintenance/mod.rs` is 549 code lines on this base, and PR #324 adds about 229. Put the
binding in a new `isolated_maintenance/view.rs` to stay under 700.

### G6 — MA6 binds to unauthenticated local refs

Recommendation: under a route, `require_revision` (`manifest_promotion.rs:69-99`) also requires
the validation commit to equal the authenticated protected tip. `PromotionPlan::apply`
(`:43-55`) rechecks the captured route before delivery, as MA4 does
(`CapturedRegistry::recheck`, `mod.rs:132-134`). The promotion predicate gets the isolated host
from G1.

### G7 — publication renders production verdicts

Recommendation (CP-P3): inside a resolved route, `support_matrix/derive.rs:400` uses the same
isolated evaluator as G1. Otherwise it uses `production_results`. This keeps one reporting truth
for render and for IN1's `publication::validate_current` and `qualifications`
(`integration.rs`, `admit`).

Until CP-P3 is approved, the conservative option holds: publication stays `unverified`. Promotion
still moves pointers, metadata and `current.json`, under its own entry, but the proof run must
record "qualified publication not exercised". Qualified output still needs an owning entry and
current final admission; reporting alone never creates qualification (route design, "Real packet
evidence and reporting").

### G8 — IN1 uses production hosts and the production registry

Recommendation:

- `check_integration` resolves `isolated_maintenance::route(workspace)` before root narrowing.
  `Some` selects `Authority::Isolated`; an error refuses.
- `merge.rs:47` uses `isolated_maintenance::registry`, with the same production/isolated error
  split as `manifest_validate/depth.rs:63-73`.
- Under an installed route, IN1 requires the supplied parent to equal the authenticated protected
  tip (CP-P1). That is conservative: a merge-group entry stacked on another entry refuses, and the
  queue rebuilds it.

A route resolved in the integration checkout grants no final permission by itself. The
existing all-root generation, closeout and promotion checks decide.

### G9 — CI and host wiring of the isolated repository (phase C)

Recommendation (CP-P5): no change to canonical workflows in B.5 or B.6. Record the needed host
changes as phase C prerequisites:

- `--tip` per-root validation of the subject root in the merge-queue and post-push jobs;
- full-history checkouts in `depth-tip.yml`, since the route refuses shallow history;
- a least-privilege read credential for route observations of both repositories in the
  merge-queue, closeout and promote jobs;
- an `origin` spelling that `Workspace::read` accepts.

Whether the isolated repository may carry workflow bytes that differ from canonical depends on the
approved producer closure. That is CP-P5's open question.

### G10 — the A.5 replay with a single-version subject

`scope.rs:112-119` refuses any retained record other than the subject's version.
Recommendation (CP-P6): the replay opens an ordinary, non-depth `0.3.0` maintenance generation.
The declaration still names only `0.2.0`, so that generation has no record. It replaces the working
request (and later the closeout); then MA6 promotes `0.2.0` from its retained record and entries.
The isolated scope is not widened.

## Decisions awaiting approval

| Id | Proposed decision (awaiting approval) | Reason / boundary |
| --- | --- | --- |
| CP-P1 — isolated owning host | MA5, MA6 and IN1 admit through one private `Authority::Isolated(&Context)`. Evidence is the re-authenticated isolated verdict for the exact record state. The integration tip is the authenticated protected tip, and the local `origin/staging` must equal it. At IN1 under an installed route, the supplied parent must equal that tip. | Applies the approved IR-P1 threading to the owning cores without a second predicate. The tip-equality rule serializes depth candidates in the queue; that is conservative. Production stays `Production`. |
| CP-P2 — promotion scratch binding | MA6's scratch view inherits the caller's captured scoped registry through an `IsolatedView`-attested, thread-bound guard. It never resolves itself. | Removes the "workspace-wide regeneration cannot run in scratch" gap without changing any renderer. It extends the MA4 candidate binding to a non-Git view, which is an authority-boundary change, so it needs approval. |
| CP-P3 — isolated publication verdicts | Inside a resolved route only, support publication renders verdicts from the isolated evaluator. | Without it, the chain cannot show qualified output. Until approved: `unverified`, and the claim is recorded as not exercised. |
| CP-P4 — closeout preflight repository | Under a route, `prepare-agent-closeout` reads preflight from the authenticated isolated repository. | The packet's CI runs live there. Until approved, `prepare-agent-closeout` refuses under a route. |
| CP-P5 — isolated CI wiring | The B.5 and B.6 PRs make no canonical workflow change. Per-root `--tip` validation of the subject root, full-history checkouts, an `origin` spelling the route accepts and a read credential become phase C host prerequisites. | The canonical loops are fixed to three roots (`ci.yml:345-390`, `depth-tip.yml:34-36`). Open: may the isolated workflows differ from canonical, or must canonical gain a registry-derived loop? |
| CP-P6 — A.5 replay shape | Replay with an ordinary (non-depth) later-version generation. The isolated scope is not widened. | `scope.rs:112-119` admits only the subject's version. |

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
4. IN1 (`check_integration` with the route) admits the closeout candidate against the parent.
5. Advance the fake tip to that candidate. Then run MA6 `build_plan` and `apply` with the
   captured route. Expect one coherent proposal:
   - pointers, metadata and `current.json`;
   - support, runtime and capability outputs;
   - one `Promotion` entry;
   - the marker retired.
6. IN1 admits the promotion candidate.
7. A.5 replay: a `0.3.0` ordinary generation replaces the request and closeout, then MA6 for
   `0.2.0` passes and IN1 admits.

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
- An IN1 parent that is not the protected tip, plus the IN1 cases of the proof plan:
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
`.github/workflows/`, `depth_evidence/`, `agent_maintenance/execute/` or
`isolated_maintenance/tests/ma4_route.rs`.

| Slice | Content | Write scope |
| --- | --- | --- |
| B.5a — owning authority (CP-P1) | `Authority` and the isolated `Host`, the tip binding, `Proposal` forwarding, `Route::proposal`. No public route changes yet. Unit tests on the predicates with the fake host, plus production byte-identity controls. | `depth_acceptance/{mod,manual,promotion,integration}.rs`, new `depth_acceptance/isolated.rs`, `agent_maintenance/closeout_proposal.rs`, `isolated_maintenance/mod.rs` (accessors only), new `isolated_maintenance/tests/authority.rs` |
| B.5b — both MA5 writers (G2–G4; CP-P4) | Route capture in both writers, routed registry loads, the isolated preflight repository (or a refusal until CP-P4 is approved). Positive chain to closeout for each writer, plus MA5 refusals. | `agent_maintenance/closeout/{write,findings,support_audit_truth,validate,evidence}.rs`, `agent_maintenance/prepare_closeout.rs`, `agent_maintenance/drift/{mod,shared}.rs`, `agent_lifecycle/maintenance_adoption.rs`, new `isolated_maintenance/tests/ma5_closeout.rs`, the B.5 notes in `support-depth-synthetic-subject-design.md` |
| B.5c — IN1 for closeout (G8) | Route-selected authority in `check_integration`, the routed registry in `merge.rs`, parent equals protected tip. Closeout-candidate admission and the IN1 refusals. | `depth_acceptance/integration.rs`, `manifest_validate/depth/merge.rs`, new `isolated_maintenance/tests/in1_integration.rs` |
| B.6a — scratch binding (CP-P2) | `CapturedRegistry::bind_view` over `IsolatedView`; removes the `dead_code` allowances that this slice now uses. Binding positive and refusal tests. | new `isolated_maintenance/view.rs`, `isolated_maintenance/mod.rs` (module line), `agent_maintenance/closeout_proposal.rs`, new `isolated_maintenance/tests/view.rs` |
| B.6b — MA6 through the route (G5, G6, G10) | Captured route, isolated proposal and scratch binding in `build_plan`, plus the tip and apply rechecks. Positive promotion and IN1 admission, the A.5 replay (CP-P6), MA6 refusals. | `manifest_promotion.rs`, new `isolated_maintenance/tests/ma6_promotion.rs`, the B.6 notes in `support-depth-synthetic-subject-design.md` |
| B.6c — publication verdicts (CP-P3; only if approved) | The isolated evaluator in support derivation inside a route. Tests that the qualified row appears only after the owning entry, and that IN1 publication consistency holds. | `support_matrix/derive.rs`, new `isolated_maintenance/tests/publication.rs` |

Phase C (CP-P5) carries the workflow and host prerequisites and the real native-queue run. Every
slice runs the required gates. Approving this design enables no path and installs no profile.
