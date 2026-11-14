# Producer freeze attempts: unresolved policy question

Round-2 slice-5 investigation, starting at `45895c94`. This records a blocker, not a
contract amendment or permission to filter the population.

## Existing maintenance rule

Maintenance and onboarding use the same discovery and approval path:

- `crates/xtask/src/depth_evidence/verdict/github/population.rs:134` reads each
  authenticated repository head's profile record. Matching Event/P/O makes it
  applicable before outcomes or artifacts; there is no stage or source-closure
  exclusion. Thus a maintenance MA2 request-freeze run with matching Event/P/O is
  applicable, even though MA4 later changes implementation.
- `crates/xtask/src/depth_evidence/verdict/github/profile.rs:83` includes the whole
  `crates` tree in the protected-staging closure. At line 101, exact Git object
  comparisons reject a different earlier tree. The earlier run is not excluded;
  discovery refuses the generation. Retries do not change those source bytes.
- There is an important fixture boundary: `depth_ci/profile.rs:52` selects the
  maintenance record under `crates/xtask/tests/fixtures/depth-synthetic/`, whereas
  onboarding selects the actual ON2 report at line 53. A missing profile fixture
  is skipped by `population.rs:137`; a mismatching Event/P/O is skipped at line
  141. The connected maintenance setup in
  `isolated_maintenance/tests/view_eval.rs:60` commits the MA2 freeze, then copies
  its record to the producer fixture at lines 89–94. Only afterward does
  `depth_evidence/verdict/github/tests/host.rs:11` install/commit the producer and
  serve its one run. It never serves CI on the genuine MA2 checkpoint. That
  fixture therefore supplies neither a historical-freeze positive nor a
  source-closure exclusion rule for matching heads. Absence of the profile
  fixture can exclude an earlier maintenance head; closure mismatch cannot.
- `docs/specs/support-depth-evidence-v1.md:203`: "A complete independently
  discovered population includes all applicable runs and attempts, including
  missing or nonterminal ones."
- The same contract, lines 317–322: "A canonical-repository head with the same
  frozen Event/P/O is applicable before artifact inspection" and "Every attempt
  from 1 through the authenticated attempt count is included."
- Lines 280–283 require the complete collector/helper/dependency closure and
  workflow to match approved protected staging. Lines 304–310 explicitly include
  `crates/`, current indexed bytes/modes and actual built/workflow implementation,
  and conservatively refuse changes. This is not historical per-source approval.
- The onboarding amendment at lines 652–655 retains initial implementation and
  later publication/proving attempts, all retries and attempts without witnesses;
  changed source/dependencies cannot qualify. It supplies no freeze exception.

Consequently maintenance supplies no precedent that reconciles earlier changed
source with a later verified population. The contract decides applicability and
refusal today, but does not authorize a satisfiable historical-freeze rule.

## Exact question requiring a maintainer decision

How can a genuine pre-implementation freeze attempt participate in a subsequently
verified Event/P/O generation when its complete producer/subject source closure
differs from protected staging and the current evaluated source?

Options and required contract changes:

1. **Applicable and satisfiable.** Define independent approval of historical
   producer/subject closures, actual witnesses and substantive cases at the
   pre-implementation source, and adoption/currentness across those changed
   closures. Amend the exact single-staging closure requirement (280–283,
   304–310), and specify how an absent implementation can satisfy every due
   obligation without invented execution or hidden failures. An earlier approved
   hash alone does not meet currentness or substantive adequacy.
2. **Outside the execution population because the source closure differs.** Amend
   the Event/P/O-only applicability rule (203–207, 315–325, 652–655) to define a
   distinct, independently authenticated readiness boundary. Merely ignoring
   *any* changed closure would also hide implementation regressions; arbitrary
   source differences cannot be a safe exclusion rule.
3. **Applicable but only a reporting checkpoint until implementation readiness.**
   Keep complete discovery and expose freeze attempts, while explicitly defining
   which independently authenticated pre-implementation checkpoints require no
   execution receipt and cannot contribute success. Amend complete-population
   evaluation/required-witness rules and define the immutable readiness transition;
   all implementation/publication/proving attempts and retries still require full
   authentication. A lifecycle stage label alone is insufficient authority.

Recommendation: option 3, with a shared maintenance/onboarding readiness rule
approved before implementation. It preserves complete visibility, acknowledges
that the requested subject has not yet been implemented, and avoids weakening
source approval or fabricating successful freeze evidence. Defining that boundary
is outside slice 5's approved producer binding; no option is installed here.

## Executable boundary

`onboarding_genuine_freeze_attempt_remains_applicable_and_blocks_later_proof`
uses the actual `on2-freeze` tag from the public ON2 connected chain, verifies its
matching frozen inputs, and observes closure refusal even after increasing the
attempt count. It creates no witness for that source. The older twelve-attempt
positive uses an enrolled-state projection on the final producer closure, not the
genuine freeze head; it does not resolve this blocker.

## Round-2 implementation and validation

P2 is implemented in `crates/xtask/src/depth_evidence/verdict/github/`:
`mod.rs` registers `progression.rs`; `population.rs` observes each applicable
onboarding host head and distinct actual checkout. `progression.rs` authenticates
per-source lifecycle inputs, orders sources by Git ancestry (independent of API
listing order and retry time), and invokes the same
`runtime_follow_on::depth::check_progression` that IN1 uses. Sources are checked
once per unique commit per discovery. There is no new stage ordering predicate.
Maintenance behavior, receipt schema, cases, obligations, CI and production
parsing are unchanged.

`tests/onboarding.rs` registers `tests/onboarding/history.rs`. The connected
`onboarding_producer_refuses_descendant_lifecycle_rollback_in_either_api_order`
restores the exact genuine freeze state after implementation, retains approval,
Event/P/O and implementation files, proves both states independently valid and
the producer closure identical, and requires progression refusal in both host
listing orders. It failed before the fix because discovery authenticated the
rollback. The genuine-freeze characterization also checks historical progression
in both observation orders, including a repeated freeze observation after the
implementation; this does not bypass closure authentication.

Other changed files: `docs/specs/support-depth-contract.md` (Annex A forward,
backward and conformance rows) and this investigation. Existing onboarding tests
retain eleven-obligation, cross-Event, all-attempt/adoption/source/currentness,
failure-preservation and CP-P5 invocation-graph coverage. The twelve-attempt
positive's genuine-freeze limitation is now explicit.

Commands and measured results in this sandbox:

| Validation | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass, final run 6.69 s |
| `make loc-check`; `git diff --check` | Pass |
| `cargo test -p xtask --lib depth_ci` | 6 passed, 7.91 s including compilation |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 59 passed, 142.21 s including compilation/build-lock wait; test execution 126.86 s |
| `cargo test -p xtask` with all nine top-level `--test depth_*` targets | All pass, 32.31 s total |
| `cargo test -p xtask --lib depth_evidence` | 91 passed, 12 environment refusals, 1 ignored; 186.14 s including compilation; test execution 178.57 s |

All seven onboarding tests, including both additions and the twelve-attempt case,
passed in the final evidence run. That entire evidence run stays below round 1's
five-minute target. The twelve-attempt case retains fresh artifact-validator calls
for job mutations, fresh adoption resolution and full rediscovery after changing
failure evidence; the fix introduces no authentication reuse across mutations.

Ten staged-channel tests cannot establish their Unix socket in this sandbox
(`Operation not permitted`, including the forged-channel test whose child cannot
bind). Two native-context tests independently reproduce `ForeignLoader` because
`/tmp/.codex` and `/tmp/.agents` are injected into their ancestor tree; those tests
explicitly create their fixtures under `/tmp`. Neither restriction is changed or
worked around here. The lead must run those tests unrestricted.

P1 remains deliberately unimplemented pending the decision above. Phase C still
must prove real protected CI/needs ordering (including the recursive workspace
test leg), independent adoption, terminal producers before failure-preserving
consumers/same-run IN1, and the terminal workflow/gate before conditional MERGE.
No real host call, live profile, production enablement, commit or push was made.

## Round-3 history correction

The round-2 source map compared only commits discovered through CI. A rollback could
therefore authenticate when its higher-stage ancestor had no workflow run. The
round-3 regression reproduced that acceptance with only the rollback descendant in
`RUNS`, while retaining independent adoption.

`progression.rs` now walks every raw parent edge reachable from each applicable
host head and actual checkout. It reads commit headers with the hardened Git
runner's `cat-file commit`, then lifecycle, record and approval blobs from pinned
`Source` views. Raw headers retain parents hidden by shallow or graft metadata.
Missing commit/tree/blob objects refuse through local reads. History traversal
performs no object acquisition. The complete graph is visited without Git's
path-based history simplification. Identical record/state/approval/registry tree
entries share their authenticated projection, and validated commit histories are
cached within one discovery of one Event/P/O generation.

The lifecycle interval begins at the unique first ancestor whose record matches
that Event/P/O, normally the ON2 freeze. Every parent/child transition thereafter
uses IN1's `check_progression`. Every merge parent carrying this generation
participates, regardless of parent order. A staging parent with no earlier state
of this generation contributes no transition. Thus both a staging merge into a PR
branch and GitHub's test merge receive the same checks. An earlier freeze observed
after implementation remains an ancestor observation with its own forward history.
An intervening rollback remains invalid even if a later commit restores the higher
stage. Binding disappearance/change after that interval begins, multiple independent
first matching states, and different generation starts among observed sources
refuse as ambiguous histories.

This follows the evidence contract's onboarding producer binding and producer
discovery requirements that actual approval Event/P/O bind each source and that
all applicable heads/attempts participate. The support-depth contract's Generation
definition names one create-lane run under one approval as one generation. The
onboarding charter's Support depth rules 5 and 7 close the freeze window after a
passing write and preserve the lifecycle object. The support-depth contract's
Complete mediation rule 7 covers every proposed merge result. Together with the
existing IN1 progression predicate, these require ancestry-based checks at merges.
A first-parent interpretation could discard the PR head of a test merge or a
higher-stage staging parent. The implementation checks all parent edges and
requires one connected frozen generation, refusing ambiguous lineage.

The producer-only reconstructed freeze fixture previously descended from the real
implementation checkpoint. Its new parent is the genuine `on2-freeze`, with the
final-source projection replayed there before the implementation lifecycle projection.
The genuine implementation is retained as `on2-implementation` in the disposable
fixture. This preserves the positive's disclosed final-closure projection while
making its history forward-only. The actual freeze's earlier closure still receives
the separate refusal characterization.

The regression now covers the rollback-only run population, both API listing
orders, a subsequently restored higher stage, valid staging merges, a rollback
reachable through the second merge parent, the actual PR-test-checkout path,
interrupted bindings, independently rooted generations and missing parent objects
hidden by shallow metadata. Historical freeze observations remain progression-valid.
The freeze-checkpoint exception remains deferred to its separate contract amendment.

### Round-3 validation

The changed files are `github/progression.rs`, `github/tests/fixture.rs`,
`github/tests/onboarding/history.rs`, this investigation, and the forward,
backward and conformance rows in `docs/specs/support-depth-contract.md` Annex A.
The fixture replay is the only setup adjustment beyond history validation and
its regressions. The initial rollback-only regression failed because discovery
returned authenticated evidence. The five history tests subsequently passed,
and all ten onboarding tests passed in the full evidence run.

| Command | Result and elapsed time |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass, 14.78 s including build-lock wait |
| `cargo test -p xtask --lib depth_evidence::verdict::github::tests::onboarding::history -- --nocapture` | 5 passed, 34.09 s including compilation, 23.78 s test execution |
| `cargo test -p xtask --lib depth_evidence` | 94 passed, 12 environment failures, 1 ignored, 93.69 s total, 87.41 s test execution |
| `cargo test -p xtask --lib depth_ci` | 6 passed, 7.78 s including compilation |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 59 passed, 115.36 s including compilation/build-lock wait, 109.98 s test execution |
| `cargo test -p xtask --test depth_ci_archive --test depth_evidence --test depth_evidence_payloads --test depth_fixture_snapshot --test depth_native_evidence --test depth_promotion --test depth_record --test depth_refusal_routes --test depth_synthetic_governing` | All nine targets passed, 27.07 s including compilation |
| `make loc-check`, `make hygiene`, `git diff --check` | Pass |

The full evidence run stays within round 1's five-minute budget. Its twelve
failures match the round-2 environment failures above. Ten staged-channel tests
cannot establish their Unix socket, including the forged-channel child's failure
to bind. The other two fail with `ForeignLoader` in the `/tmp` native-context
fixtures. These require the lead's unrestricted validation environment.

Phase C still owes independently authenticated adoption of the reviewed source,
real protected CI producer/consumer ordering with the recursive workspace-test
leg and same-run IN1, and terminal workflow/checks/gate before conditional MERGE.
P1 remains deferred. The genuine historical-freeze attempt still receives exact
closure refusal. Repository edits remain uncommitted in this worktree.
