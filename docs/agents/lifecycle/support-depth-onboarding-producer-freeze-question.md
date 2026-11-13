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
