# FR1 admission history implementation report

## Review round 7 repair

Status — implemented and validated on `feat/fr1-admission-history` over `1ce898d0`, subject to
four Unix-socket sandbox failures detailed below.
This section was started before code changes. Changes remain uncommitted.

`inputs_identity` removes `claims`, `decision` and `reasons` from canonical record JSON and
replaces the audit predecessor identities with the sorted unique set of predecessor commit and
transition-digest pairs. The transition digest hashes canonical `[old_state, new_state]` under
`support-depth-in1-transition-v1` followed by NUL. The input domain remains
`support-depth-in1-inputs-v1` followed by NUL. Checkpoint classification and substantive
repository/object/policy/generation/root/blob/observed-authority facts remain included. Raw
predecessor identities and emitting session claims remain authenticated in retained record bytes.
Equivalent predecessor retries have identical conflict keys regardless of representative selection.

Reconstruction authenticates and indexes every retained record before comparing reached decisions.
Every reached admission/refusal participates under its substantive key, including quarantined
records. The sole-reason incomplete-only refusal remains exempt. Missing, refused, quarantined,
non-ancestor or superseded predecessor references and inconsistent Bootstrap classification mark
a record history-deficient. Any observation-incomplete or history-deficient record is quarantined
from frontiers, readiness, progression and generation-history completion. Unresolved audit references
contribute no authenticated predecessor pair. Quarantine leaves retained bytes intact and allows
healthy retries and other candidates to reconstruct. Object/provenance integrity, history gaps,
bounds, duplicate identities and identical-key contradictions continue to refuse.

Two round-6 assertions used the broader secondary comparison that omitted checkpoint and all
predecessors. The round-7 substantive-key rule supersedes those assertions. Their tests now
verify quarantine/recovery for different substantive keys and refusal for contradictory reached
decisions over the same deficient key. The general history-integrity test likewise now requires quarantine for
missing predecessors and inconsistent classification, while preserving object/blob rejection.
The remaining round-4 through round-6 regressions retain their behavior. This interpretation was
raised during implementation and follows the round-7 substantive-key rule.

### Round 7 regressions and vectors

After correcting the new fixtures' protected-root binding, both required regressions failed
against inherited production logic. Retry reordering allowed a contradictory reached admission,
and concurrent parent admission returned `missing predecessor admissions or bootstrap anchoring`.
The repaired lineage tests pass. The real IN1 publication regression also passes through input
capture, changed-frontier detection, incomplete reached-admission retention, healthy complete
retry, independent sibling admission and a later complete contradiction refusing reconstruction.
Additional coverage checks missing-reference quarantine, exclusion from progression and generation
history, propagation to a referencing descendant, and continued object-integrity rejection.

Python `json`/`hashlib` independently generated the updated literal canonical input vector and
transition digest using compact sorted-key UTF-8 and NUL-terminated domains. Admission and
generation payloads/digests remain the existing literals.

| Vector | SHA-256 |
| --- | --- |
| Predecessor transition | `98e4ae1846fa06900dc1f159fee09db39fe3ddcaa8d3c57a8ae03988e07f1ad9` |
| Substantive evaluated inputs | `d4d8e94e1aae330ce60f2c13c2f2b1f5c3b3ff90f14d54b19479d6b81bd9a399` |

### Round 7 validation

Cargo uses `env -u TMPDIR`. Logs are under ignored `target/fr1-round7-validation/`.
The final requested test groups report 257 passing tests and four fixture failures. The two groups
with updated test-only projection assertions were rerun after those edits. The final depth and IN1
results below are from those reruns. Clippy initially caught one cloned-reference slice in a new
test, which was corrected before the successful final run.

| Command | Result | Test duration |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | — |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | — |
| `cargo test -p xtask --lib depth_acceptance` | 83 passed | 52.80 s |
| `cargo test -p xtask --lib in1_integration` | 35 passed, 4 sandbox failures | 236.44 s |
| `cargo test -p xtask --lib depth_evidence::verdict` | 78 passed | 131.54 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 61 passed | 365.21 s |
| `make loc-check` | Pass | — |
| `make hygiene` | Pass | — |
| `git diff --check` | Pass | — |

The four IN1 fixture failures are
`in1_closeout_candidate_admission_and_post_push_observation`,
`in1_prepared_closeout_candidate_admission_and_post_push_observation`,
`in1_reauthenticates_population_in_admission_and_post_push_observation` and
`in1_refuses_forged_closeout_entries_in_admission_and_observation`. Each fails before its
socket-dependent assertions with `cannot open the MA4 evidence channel: Operation not permitted
(os error 1)`. Those assertions remain unverified in this sandbox. Both round-6 post-capture
history-failure regressions, all socket-free round-4/5 observed-authority regressions, the new
retry-reordering and quarantine tests, and the real concurrent-parent recovery/contradiction
regression pass. The connected ON2 bootstrap admission/observation case also passes. All tracked
changes remain inside this worktree, uncommitted, with HEAD still `1ce898d0`.

## Review round 6 repair

Status — implemented and validated on `feat/fr1-admission-history` over `e0efb49b`, subject to
four Unix-socket sandbox failures detailed below. This section was started before code changes.
Changes remain uncommitted.

`Emitter::prepare` runs immediately after the authenticated parent binding and before the
owning policy predicates on both finite-bootstrap and enrolled routes. It resolves the complete
history frontier and all content-addressed record inputs. Retention clones this captured record
and fills only the decision/reasons, pinned normalized authority and independently authenticated
session association. It never reconstructs identity-bearing fields from fallible emission reads.
A failed retention-time history reconstruction preserves the reached admission/refusal, appends
`observation_incomplete`, publishes the pinned inputs and returns failure. Closing frontier changes
also fail without replacing the captured predecessors. The independent session event must equal
the invocation event obtained before policy evaluation.

The chosen history rule checks complete frontier coverage and Bootstrap classification for every
reached decision, including reached-incomplete admissions/refusals. Inconsistent reached admissions
refuse verification. A refusal with missing/superseded predecessors or inconsistent classification
is explicitly marked incomplete and non-authoritative in reconstructed `History::incomplete`.
It still compares against every reached admission over all remaining pinned facts, omitting exactly
`predecessors` and `checkpoint` from that secondary canonical comparison projection. Retained
bytes and the full evaluated-input identity stay intact.
Only incomplete-only refusals remain exempt from conflict comparisons. All incomplete records
remain ineligible for successful history/readiness. The evidence contract, freeze/readiness design
and Annex A IN1 row state these rules.

Validation caught a compatibility problem in an initial strict-rejection implementation. A refused
stacked candidate can precede a later successful admission for its ancestor, which supersedes the
old observed frontier. A focused regression reproduced `missing or inconsistent predecessor
admissions`. The final rule uses the packet's explicit incomplete/non-authoritative alternative,
preserves the older refusal and retains its remaining-facts contradiction check.

Initial input capture must succeed before the owning policy predicates can run and a complete
record can be published. Failed capture leaves the scheduler attempt incomplete and unverified.
Present malformed lifecycle/depth bytes now fail capture. Null states/generation, empty input/evidence maps
and lists, and empty predecessor lists represent successful lookups of actual absence. This
extends the repair beyond the reported history fallback to the state and depth parsing paths.

### Round 6 identity-field audit

`inputs_identity` still hashes the existing canonical record representation under the existing
domain. The following table covers every record field, including normalized/excluded fields.
Schema version 1, the canonical encoding and existing digest domains apply. Literal vectors
continue passing with the refactored input normalization.

| Field | Pinned source and failure handling |
| --- | --- |
| `schema_version` | Fixed schema constant `1` in the prepared record. |
| `kind` | Fixed `in1_admission` constant in the prepared record. |
| `repository` | Captured source-installed repository name/id/owner, independently checked by history and isolated-host binding. |
| `generation` | Typed parse of authenticated candidate depth-record blob before predicates. Null only when the path is absent. Read/parse errors abort capture. |
| `generation_sha256` | Derived from the pinned generation with the existing canonical generation domain. Errors abort capture. |
| `candidate` | Captured checkout commit with bounded, rehashed Git commit/tree reads before predicates. Errors abort capture. |
| `parent` | Authenticated binding parent with rehashed Git commit/tree reads before predicates. Errors abort capture. |
| `checkout` | Exact copy of the pinned candidate object. |
| `head_relationship` | Fixed `exact_head` invariant from authenticated binding. |
| `old_state` | Parent lifecycle path/blob/stage read and parsed before predicates. Null only for actual absence or a route with no applicable state path. Read/parse errors abort capture. |
| `new_state` | Candidate lifecycle path/blob/stage read and parsed before predicates, with the same absence/error rule as old state. |
| `checkpoint` | Bootstrap for an authenticated empty parent frontier, unclassified for an existing frontier, resolved before predicates. Retention never substitutes a classification on error. |
| `predecessors` | Sorted unique successful identities from authenticated complete parent history before predicates. A failed read aborts capture. Closing history errors preserve these identities. |
| `enrollment_anchor` | Source-installed baseline commit/tree authenticated by bounded history resolution before predicates. |
| `policy` | Source-installed implementation/workflow/job/app/closure snapshot. History authenticates its source against the protected installation anchor before predicates. |
| `protected_tip` | Captured authenticated host tip with rehashed commit/tree reads before predicates. |
| `canonical_tip` | Captured authenticated canonical host root, preserved in the prepared record. |
| `accepted_origin` | Deterministically derived from the pinned repository name. |
| `registry_blob` | Captured authenticated protected-registry blob from the isolated context, preserved before predicates. |
| `inputs` | Candidate authoritative paths and acquired/execution paths resolved by bounded, rehashed Git blob reads before predicates. An absent path is omitted only after a successful lookup. Any source read error aborts capture. |
| `observed_authority` | Normalized authenticated facts accumulate during evaluation and pin at the first reached owning decision. Closing/emission failures track incompleteness separately. |
| `producer_context` | Explicit FR1 reserved null constant. FR3 will supply its owning association. |
| `execution_evidence` | Sorted deduplicated identities projected from the pinned parsed depth record before predicates. Empty means the authenticated record has no execution references or is absent. |
| `claims.event` | Independent invocation event from the host before predicates. Session publication checks exact equality with that pinned event. |
| `claims.run`, `claims.attempt`, `claims.job`, `claims.check` | Independent session associations normalized to zero by `inputs_identity`, then authenticated through retained job/attempt/artifact facts. |
| `decision` | Normalized to admitted by `inputs_identity`. The separately pinned reached value remains a conflict side. |
| `reasons` | Cleared by `inputs_identity`. Incompleteness changes eligibility without changing the reached conflict identity. |

### Round 6 regressions

Both required post-capture tests failed against inherited code. The first-admission reproduction
retained Unclassified, and the existing-frontier reproduction retained empty predecessors.
The one-shot fake-host fault ignores every history read while observation capture is active and
fails the first reconstruction read inside `retain()`. Subsequent publication reads succeed.
Both repaired cases retain the original policy refusal with incompleteness, preserve the exact
frontier/checkpoint, and accept a healthy contradictory retry far enough to retain its reached
admission with identical normalized authority and evaluated-input identity. Independent history
then rejects `conflicting decisions over identical inputs`.

- `post_capture_history_failure_preserves_bootstrap_refusal_conflict` covers first-admission Bootstrap.
- `post_capture_history_failure_preserves_predecessors_and_refusal_conflict` covers an existing successful parent frontier.
- `missing_refusal_predecessors_cannot_hide_a_reached_conflict` detects the contradiction despite an omitted parent admission.
- `missing_refusal_bootstrap_is_incomplete_and_still_conflicts` marks a reached-incomplete first refusal with Unclassified incomplete and detects a later contradictory admission.
- `later_parent_admission_preserves_a_refusals_original_inputs` preserves the retained bytes while explicitly marking the superseded frontier incomplete.
- `failed_input_capture_cannot_reach_policy_or_publish_default_projections` supplies malformed authenticated lifecycle bytes and verifies policy remains unevaluated, with an incomplete attempt and unavailable historical authority.

### Round 6 validation

Cargo validation uses `env -u TMPDIR`. Logs are under ignored
`target/fr1-round6-validation/`. Groups ran concurrently. Durations below are test execution
times reported by Cargo, with build and lock waits excluded. Initial reproduction and diagnostic
logs are retained in that directory with the `initial-` prefix.

| Command | Result | Test duration |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | — |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | — |
| `cargo test -p xtask --lib depth_acceptance` | 79 passed | 106.46 s |
| `cargo test -p xtask --lib in1_integration` | 34 passed, 4 sandbox failures | 163.33 s |
| `cargo test -p xtask --lib depth_evidence::verdict` | 78 passed | 155.85 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 61 passed | 541.29 s |
| `make loc-check` | Pass | — |
| `make hygiene` | Pass | — |
| `git diff --check` | Pass | — |

The four IN1 failures are the connected cases
`in1_closeout_candidate_admission_and_post_push_observation`,
`in1_prepared_closeout_candidate_admission_and_post_push_observation`,
`in1_reauthenticates_population_in_admission_and_post_push_observation` and
`in1_refuses_forged_closeout_entries_in_admission_and_observation`. All fail during fixture
preparation with `cannot open the MA4 evidence channel: Operation not permitted (os error 1)`.
Their socket-dependent assertions remain unverified in this sandbox. Both required round-6
post-capture regressions and the failed-input-capture guard pass in the final IN1 group. All
existing socket-free producer-backed observed-authority regressions also pass. The final bootstrap
run has passed the connected ON2 admission/observation case that exposed the initial strict-rule
compatibility failure. Across the four requested test groups, 252 tests passed and four failed
in socket-dependent fixture preparation. Final diff, formatting, LOC and repository hygiene
checks pass. HEAD remains `e0efb49b` on the requested branch.

## Review round 5 repair

Status — implemented and validated on `feat/fr1-admission-history` over `c5b4c99f`, subject to
four Unix-socket sandbox failures detailed below.
This section was started before code changes. Changes remain uncommitted; nothing was pushed.

P1 preserves the owning decision before closing checks. `observations::after_decision` pins the
admitted/refused result and normalized substantive facts before the captured-context and binding
rechecks, including the finite bootstrap registry path. Nested closing boundaries cannot overwrite
that result. Failed closing rechecks record sticky incompleteness separately and preserve the
original refusal diagnostic alongside a closing error. Late availability facts cannot salt the
pinned identity. Before any substantive decision, unavailable reads retain their fixed availability
fact and the incomplete-only refusal behavior.

The emitter retains the reached decision and owning reason, appending `observation_incomplete`
when closing fails. Record validation permits exactly the matching ordered combinations
`[owning_predicates_satisfied, observation_incomplete]` on admitted and
`[owning_predicate_refused, observation_incomplete]` on refused, plus the existing incomplete-only
refused form. Only that sole-reason form is excluded from conflict comparisons. Every incomplete
record is excluded from readiness, successful predecessor frontiers, generation-history completion
and progression. The independent job association and fake publisher require failure for every
incomplete record, even a reached admission. Invocation success still requires closing checks and
publication to succeed.

P2 authenticates author login and current permission before fallible adoption parsing and source
or mapping processing. Unauthorized malformed adoption JSON and prose take the same ignored path;
neither truncates the authorized population or subsequent stability facts.

Both new regressions failed against inherited code: unauthorized malformed JSON returned
`invalid exact adequacy adoption`, and the policy refusal plus one-shot late flake retained only
`observation_incomplete`. Regression coverage now includes:

- `late_flake_preserves_reached_policy_refusal_and_conflicts_with_later_admission`: real producer
  reads, deliberate refusal and one-shot late isolated read failure, followed by an admission with
  the same authority/input identities. Independent history rejects their contradictory decisions.
- `unauthorized_malformed_adoption_cannot_salt_a_reached_conflict`: malformed unauthorized JSON
  admits, then an edit to prose accompanies a reached contradictory refusal. Authority and inputs
  remain identical, and history rejects the conflict. The ordinary-prose edit test remains.
- `late_isolated_host_flake_preserves_reached_admission_and_retry_recovers`: a reached admission
  retains both reasons, remains ineligible, and a complete retry preserves authority and recovers.
- Unit coverage pins decision-boundary facts, sticky incomplete-only failures, reached decisions
  with closing failures, reason combinations, failed job association, conflict participation and
  exclusion of incomplete admissions from successful predecessor authority.

The evidence contract, freeze/readiness design and Annex A IN1 row describe these rules. Schema
fields, canonical JSON and digest domains are unchanged. Python `json`/`hashlib` independently
recomputed the existing literal vectors and two newly permitted reason combinations, using sorted
keys, compact UTF-8 and NUL-terminated domains. The new vectors modify only decision/reasons of
`tests/vectors/admission.json`; their evaluated-input identity remains the existing literal.

| Vector | SHA-256 |
| --- | --- |
| Normalized observed authority (unchanged) | `a740cc927410b5d84653e5d424e74b60ad767838b47dc9c6659762f118ed442d` |
| Admission (unchanged) | `ab204698473c5cb9cba16156b4eafc8e9effca90a996d6478103f0c865b3d358` |
| Evaluated inputs (unchanged) | `79493c986ea198827b223508785188dc8b9a36204f911acc35086ff0ff73b5ec` |
| Generation (unchanged) | `141491450471d7ce945131a830e01fff76967b19d8034d7765d0088b946804f4` |
| Reached admission with incomplete closing | `320064f43916c987960b73578ececb81e17fe5922905ba728fa71d58174c154d` |
| Reached refusal with incomplete closing | `34e074e7ce7a02d2981ab2fc08b5e6404d3c723be041e2605031d00d0c517fea` |

### Round 5 validation

Cargo commands use `env -u TMPDIR`; logs and elapsed wall times are under ignored
`target/fr1-round5-validation/`. Groups ran concurrently, so wall times include contention and
build-lock waits. No sandbox restrictions were bypassed.

| Command | Result | Wall time |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | 1.89 s |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | 24.07 s |
| `cargo test -p xtask --lib depth_acceptance` | 76 passed | 101.55 s |
| `cargo test -p xtask --lib in1_integration` | 31 passed, 4 sandbox failures | 198.68 s |
| `cargo test -p xtask --lib depth_evidence::verdict` | 78 passed | 144.48 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 61 passed | 290.35 s |
| `make loc-check` | Pass | 0.10 s |
| `make hygiene` | Pass | Under 1 s |
| `git diff --check` | Pass | Under 1 s |

The four IN1 failures are the existing connected cases
`in1_closeout_candidate_admission_and_post_push_observation`,
`in1_prepared_closeout_candidate_admission_and_post_push_observation`,
`in1_reauthenticates_population_in_admission_and_post_push_observation` and
`in1_refuses_forged_closeout_entries_in_admission_and_observation`. Each fails during fixture
preparation with `cannot open the MA4 evidence channel: Operation not permitted (os error 1)`.
Their socket-dependent assertions remain unverified in this sandbox. All seven socket-free
producer-backed authority regressions pass in the final IN1 group, including both required
round-5 reproductions and the reached-admission late-flake recovery case. Across the requested
four test groups, 246 tests passed and four failed during socket-dependent fixture preparation.
The final diff and repository hygiene checks pass; HEAD remains `c5b4c99f`.

## Review round 4 repair

Status — implemented and validated on `feat/fr1-admission-history` over `7fb60ec8`, subject to
four Unix-socket sandbox failures detailed below. This section was started before code changes.
Changes remain uncommitted.

P1 is repaired at predicate ownership boundaries. The producer adapter records transport
availability while approved predicates project normalized repository/anchor, packet PR,
applicable Event/P/O workflow/run/attempt, producer job/required step, artifact binding and
witness-byte facts. Ignored out-of-scope runs contribute no content facts, and the closing
comparison records explicit discovery stability.
Canonical population sorting preserves multiplicity and removes incidental response order.
The stability recheck compares the same normalized workflow identity fields, allowing unrelated
IR-P3 workflow progress. Matching authorized adoptions commit author login/id, accepted permission,
endorsement identity, source/mapping and exact referenced body-byte digests. Ordinary comments,
unauthorized records, unrelated artifacts/jobs, ignored log text and unused metadata contribute
no content facts. Discovery records start and successful bounded exhaustion independently of
ignored-row counts or page boundaries. Relevant empty normalized arrays commit absence, including
an exhausted population with no authorized adoptions. Existing source/transport/population limits,
canonical encoding, domains and the 65,536-byte admission bound apply.

P2 is repaired by recording normalized isolated-host reads in `Context::recheck`, including the
closing captured-context and binding rechecks. Facts cover repository identity/protection/tips,
ancestry/candidate corroboration, governing registry and approved-scope canonical source inventory.
A transport failure is sticky across later successful reads. Producer object acquisition failures
and retained-history transport failures during predicate evaluation also mark the observation
incomplete. A bound invocation with any unavailable evaluation read
retains a refused decision with the sole bounded reason `observation_incomplete`. It cannot return
success or supply readiness. Record validation rejects combining that reason with admission or
other reasons. History excludes incomplete observations from both sides of conflict comparisons,
while reached contradictory decisions sharing identical normalized authority continue to refuse.
The binding-error early return now preserves incomplete refusals after an authenticated parent
has been captured. Capture ends before the emitter reconstructs history and publishes its record.

Implementation changes are in `depth_acceptance/admission/{observations,emitter,record,history,transport}.rs`,
`depth_evidence/verdict/github/{population,transport,artifact,adoption}.rs` and
`isolated_maintenance/host.rs`, relative to `crates/xtask/src/`.

Regression coverage includes the following cases.

- `ordinary_comment_edit_cannot_salt_a_reached_conflict` completes the same producer reads,
  edits only an ordinary PR comment and retains a deliberately contradictory reached refusal.
  Both authority and input identities remain identical, and history refuses the conflict.
- `reordered_populations_and_unrelated_workflow_progress_preserve_identity` reverses multi-row
  workflow/job/artifact populations and changes unrelated enclosing workflow state/metadata
  and an ignored out-of-scope run's head.
  Admission and normalized identities remain stable.
- `late_isolated_host_flake_is_retained_incomplete_and_retry_recovers` first retains an admission,
  completes the same producer reads, then injects one unavailable isolated read into the final
  captured recheck inside `with_integration_publication`. The later binding recheck succeeds.
  IN1 retains an ineligible incomplete refusal, reconstructs history and admits the next complete
  retry with the first admission's input identity.
- `incomplete_observation_is_never_eligible_or_a_conflict_side` supplies an incomplete refusal
  with exactly an admission's authority and input identity. History accepts retries around it,
  then still rejects a reached contradictory refusal. Invalid reason/decision combinations refuse.
- `history_transport_unavailability_is_sticky_while_reached_refusal_is_complete` covers
  unavailable JSON/log history reads followed by restored reads and a reached identity refusal.
- Observation unit tests cover fixed normalized facts, deduplication, canonical population order,
  multiplicity, explicit absence, sticky unavailability, arbitrary error-text exclusion and scope.
  Existing disturbed/restored producer and candidate-owned witness-copy regressions remain covered.
  The witness-copy test now checks repeat stability for each committed candidate, since normalized
  isolated-host corroboration includes the candidate commit. Its evidence identities remain stable.

The original raw-response implementation failed the ordinary-comment/workflow-metadata reproduction.
The repaired socket-free producer-backed regression group passes all six tests. During development,
a fake-host array-indexing error was corrected and the witness-copy expectation was updated for
isolated-host candidate corroboration. The LOC gate required placing the incomplete-history test
in the existing `tests/policy.rs` child module. Full validation also caught loss of the original
isolated-binding diagnostic when marking an observation incomplete. The returned error now
preserves that diagnostic, and its dedicated moved-tip regression passes.

Literal vectors were computed independently with Python `json` and `hashlib`, using sorted keys,
compact UTF-8 JSON and the terminating NUL in each domain. The normalized observation fixture
commits discovery start/completion, an empty run population and the digest of bytes `[1,2,3]`.

| Vector | SHA-256 |
| --- | --- |
| Normalized observed authority | `a740cc927410b5d84653e5d424e74b60ad767838b47dc9c6659762f118ed442d` |
| Admission with that authority | `ab204698473c5cb9cba16156b4eafc8e9effca90a996d6478103f0c865b3d358` |
| Evaluated inputs with that authority | `79493c986ea198827b223508785188dc8b9a36204f911acc35086ff0ff73b5ec` |

The evidence contract's `observed_authority` bullet and reason rules, freeze/readiness design and
Annex A IN1 row now specify normalized authoritative facts and the incomplete-observation rule.

### Round 4 validation

Cargo validation uses `env -u TMPDIR` to select the default writable temporary directory.
The groups ran concurrently, so runtimes include contention. Final runs include normalized
applicable populations, retained-history availability and preservation of binding diagnostics.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass, 16.93 s including build-lock wait |
| `cargo test -p xtask --lib depth_acceptance` | 72 passed, 97.82 s tests |
| `cargo test -p xtask --lib in1_integration` | 29 passed, 4 Unix-socket sandbox failures, 182.05 s tests |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 61 passed, 439.58 s tests |
| `cargo test -p xtask --lib depth_evidence::verdict` | 78 passed, 133.90 s tests |
| `make loc-check` | Pass |
| `make hygiene` | Pass |
| `git diff --check` | Pass |

The four sandbox failures are the connected IN1 tests
`in1_closeout_candidate_admission_and_post_push_observation`,
`in1_prepared_closeout_candidate_admission_and_post_push_observation`,
`in1_refuses_forged_closeout_entries_in_admission_and_observation` and
`in1_reauthenticates_population_in_admission_and_post_push_observation`.
All fail in fixture preparation with `cannot open the MA4 evidence channel: Operation not
permitted (os error 1)`. Their socket-dependent assertions remain unverified in this sandbox.
All five socket-free producer-backed authority regressions pass in the final IN1 group, including
ordinary-comment conflict, reordered/ignored population, late-flake recovery, disturbed/restored
producer authority and candidate witness-copy cases. The moved-tip diagnostic regression also passes.


## Previous round 3 observed host authority repair

Status — implemented and validated on `feat/fr1-admission-history` over `7624f1ec`, subject to
four Unix-socket sandbox failures detailed below. Changes are uncommitted for the lead.
The report was started before implementation.

The lead reproduced `in1_reauthenticates_population_in_admission_and_post_push_observation`.
The previous evaluated-input identity omitted independently authenticated host observations,
so a host-disturbed refusal and a host-restored admission collided over identical repository
inputs. Required `observed_authority` now binds the producer host facts consumed during this IN1
invocation into the canonical record and evaluated-input identity.

The approved producer API adapter records canonical JSON responses and complete binary response
digests for population, attempt, witness artifact, producer job/log and adoption/permission reads.
Empty responses commit observed absence. Transport failures commit a fixed `unavailable` state,
excluding arbitrary error text. Domain-separated per-read commitments are sorted and deduplicated,
then hashed under `support-depth-in1-observed-authority-v1` followed by NUL. Only its fixed-size
lowercase SHA-256 digest enters the record. Existing response/population bounds govern discovery.
Capture ends before history reconstruction/publication, keeping retries independent of their own
new artifacts. Record schema version 1, canonical UTF-8 JSON and the 65,536-byte bound apply.
The admission/input literal vectors and independent observation-domain vectors cover the addition.
The new capture lives in `crates/xtask/src/depth_acceptance/admission/observations.rs`, with scoped
wiring in `integration.rs` and emission in `admission/emitter.rs`. Producer authentication wraps
its existing host API in `depth_evidence/verdict/github/{population,transport}.rs` under the same
`crates/xtask/src` root.

Regression coverage includes the following cases.

- `depth_acceptance::admission::tests::observed_authority_distinguishes_decisions_but_unchanged_facts_still_conflict`
  authenticates distinct-fact decisions and refuses contradictory decisions sharing the same facts.
- `isolated_maintenance::tests::in1_integration::observed_authority::in1_retains_disturbed_and_restored_host_authority_without_a_conflict`
  drives real producer reads and real IN1 publication/history across two disturbed-attempt states
  and revoked adoption, restoring and admitting after every refusal through the socket-free
  assembler fixture already used by view-evaluation tests.
- `isolated_maintenance::tests::in1_integration::observed_authority::candidate_witness_copy_cannot_select_observed_authority_or_avoid_conflicts`
  commits candidate-owned altered/genuine copies of the actual hosted witness archive and a forged
  `admission.json`, checks stable authority digests, then requires conflict refusal for contradictory
  decisions with that authority and the same repository inputs.
- Observation tests pin canonical response/digest bytes, deduplication, read-order independence,
  empty/unavailable distinctions, error-text exclusion and invocation/drop isolation.

The existing socket-dependent reauthentication regression reaches MA4 channel creation and
receives `Operation not permitted` before its host-disturbance assertions in this sandbox.
The new producer-backed regressions pass without requiring that channel. The Python archive
reader returns opaque admission bytes, so this schema addition requires Rust/schema/vector edits.
The design, evidence contract and Annex A IN1 sentence describe the new observed authority.

### Host authority repair validation

Cargo commands use `env -u TMPDIR` to select the default writable temporary directory.
The final runs cover the completed source. Test groups ran concurrently, so runtimes include
contention.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass, 19.55 s including build-lock wait |
| `cargo test -p xtask --lib depth_acceptance` | 69 passed, 49.90 s tests |
| `cargo test -p xtask --lib in1_integration` | 26 passed, 4 Unix-socket sandbox failures, 135.94 s tests |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 61 passed, 357.81 s tests |
| `make loc-check` | Pass |
| `git diff --check` | Pass |
| `make hygiene` | Pass |
| Focused new producer-backed regressions | 2 passed, 147.75 s tests |

The four IN1 sandbox failures are `connected::{in1_closeout_candidate_admission_and_post_push_observation,
in1_prepared_closeout_candidate_admission_and_post_push_observation,
in1_refuses_forged_closeout_entries_in_admission_and_observation,
in1_reauthenticates_population_in_admission_and_post_push_observation}`. Each receives
`cannot open the MA4 evidence channel: Operation not permitted (os error 1)` during fixture
preparation. Full socket-capable validation remains a lead-side check.


## Previous round status

Status — round-2 repairs are implemented in `feat/fr1-admission-history` over `8246eb35`.
Changes are uncommitted. Requested validation completed with the default TMPDIR. Four existing
IN1 closeout cases are blocked by the sandbox Unix-socket restriction. All other requested checks pass.

Original scope follows the FR1 sub-slice in `support-depth-freeze-readiness-design.md`.

## Round-2 repairs

- ZIP descriptors are required when flag bit 3 is set. Signed and unsigned 32-bit and ZIP64
  forms must match the central/local declarations and independently decoded length and CRC.
  Missing, truncated, garbage and inconsistent descriptors refuse. The reviewer's zero CRC,
  compressed-size maximum and million-byte expanded-size mutations are covered in all forms.
- The connected public ON2 chain now drives L through real IN1 with protected enrolled T,
  unmerged admitted H, H's runs hidden and L restoring T's complete tree. It requires a retained
  refusal containing `history gap`. The earlier direct-check-only connected assertion was
  replaced. Small lineage tests still cover unchanged-stage ancestors and second parents.
- The late-parent-retry test publishes parent, child and same-run parent attempts two and three through
  `Route::with_admitted_publication`. It authenticates the retained child in both listing orders
  and compares its original payload bytes and predecessor identities. Publication rechecks now
  permit that precise retry inventory/artifact addition while checking earlier facts unchanged.
- Fixed canonical UTF-8 bytes and independently computed SHA-256 vectors pin the admission,
  generation and evaluated-input domains, including their terminating NUL. Fixtures cover ordered
  maps, escaped text and Unicode. They are independent of runtime-generated fixture identities.
- Git object reads check the exact object type, then read at most the object bound plus one byte,
  killing and reaping the reader on overflow. A malformed regular-file tree entry pointing at a
  small tag to a blob larger than 16 MiB refuses at the type check.
- Producer discovery applies the existing Event/P/O filter before passing candidate roots to
  admission history. Independent admission discovery still enumerates every admission attempt.
  The connected positive includes a same-repository dispatch on a pre-enrollment branch without
  an applicable depth record. Supplying that branch directly to history reproduces the gap.
- The 64 KiB bound applies to versioned admission control records. Existing depth and lifecycle
  input reads retain their prior object bound. The connected chain publishes and authenticates
  depth/lifecycle files exceeding 64 KiB through valid formatting, with a small admission record.
  `lineage::legacy_inputs_over_64k_remain_readable_in_ancestor_history` also covers these files
  on an admitted ancestor of a pending candidate.

New test support lives in `admission/tests/vectors.rs`, `admission/tests/vectors/*.json` and
`isolated_maintenance/tests/bootstrap/public_on2/admissions.rs`. Paths starting with `admission/`
are relative to `crates/xtask/src/depth_acceptance/`. Other round-2 edits affect the owning object,
emitter, history, transport, fake-host and population modules and their existing tests, the Python
archive reader/tests, this report, the design status and Annex A.

### Round-2 validation

| Command | Result and runtime |
| --- | --- |
| `cargo fmt --all -- --check` | Pass, 2.71 s wall |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass, 13.30 s wall |
| `cargo test -p xtask --lib depth_acceptance` | 66 passed, 42.73 s tests, 59.41 s wall including compilation |
| `cargo test -p xtask --lib isolated_maintenance::tests::in1_integration` | 24 passed, 4 sandbox channel failures, 94.04 s tests, 97.00 s wall |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on2` | 5 passed, 252.95 s tests, 258.23 s wall |
| `cargo test -p xtask --lib depth_evidence::verdict::github` | 33 passed, 147.93 s tests, 149.74 s wall |
| Python archive tests | 13 passed, 0.023 s tests, 0.110 s wall |
| `make loc-check` | Pass, 0.146 s wall |
| `make hygiene` | Pass, 0.198 s wall |
| Focused admission tests | 23 passed, 27.85 s tests, 38.50 s wall including compile |
| Focused IN1 publication/retry tests | 4 passed, 10.82 s tests, 46.96 s wall including 36.11 s compilation |

The four IN1 failures are the existing connected closeout tests. Each reaches the MA4 evidence
channel and receives `Operation not permitted` when opening its Unix socket. The focused FR1
publication tests pass, including the genuine same-run retries. The full groups ran concurrently, so timings include contention. The connected
`public_on2_connected_freeze_write_admission_and_observation` proof passed within the five-test
public ON2 group. Its containing group runtime is recorded above. The final runs cover the
timestamped unrelated branch, attempt-three/artifact-page recheck and padded ancestor inputs.

The original reader at `8246eb35` accepts all 24 malformed descriptor cases in the new regression
(the six mutations across signed/unsigned and 32-bit/ZIP64 forms). The fixed reader rejects them.
An additional in-memory matrix passed 48 combinations of four compression methods, both descriptor
widths, both signature forms and empty/two-byte/exact-boundary payloads. `git diff --check` passes.

## Earlier implementation and first repair

The remaining sections record the original implementation and first repair, including their
historical runtimes and sandbox restrictions. The round-2 table above is the current gate.

## Adversarial repair at a11f3712

P1 now checks the union of protected lineage and all candidate ancestors for missing admission
authority at same-generation lifecycle transitions. An unmerged transition without retained
authority refuses even when its descendant restores the enrolled protected tree. Ordinary
unchanged-stage ancestors and the pending candidate's own transition remain admissible. This
implements the independent-history and noncircular-evaluation requirements. A stage change alone
cannot distinguish an unadmitted edit from a genuine admission whose host run disappeared.

P2 keeps the Python transport reader and independently streams native ZIP decompressors with a
65,536-byte output bound, completion, excess-input, actual-length and CRC checks. Stored, DEFLATE,
BZIP2 and LZMA entries retain support. Local/central declarations are checked before decoding.
The exact 243-byte archive with 100,002 expanded bytes and a two-byte declared prefix now refuses.

Focused regressions reproduced both defects before repair. Three admission-lineage tests pass
in 2.34 s, including second-parent coverage and the ordinary-ancestor positive control. All twelve
Python archive tests pass, including exact-boundary positives for every supported compression
method and descriptor-based ZIP streams. The real ON2 chain now additionally checks the genuine
unmerged higher-stage admission, restored complete tree, hidden run, ordinary-ancestor positive
and the existing producer-discovery refusal after hiding the protected admission population.
The first-repair unmerged case directly exercised the shared history checker. Round 2 replaces
that connected assertion with actual IN1 publication and retained-refusal checks. An added discovery call at
the enrolled protected source instead reached the independent producer-inventory guard, since
the current checkout included implementation/collector files absent at that source. That
misplaced assertion was removed. The first connected run took 295.75 s and reached all new
history assertions successfully before that fixture mismatch.

Repair files are `admission/{history.rs,tests.rs,tests/lineage.rs}`,
`depth_evidence/verdict/github/tests/onboarding/admissions.rs`, both Python archive files, this
report, the freeze/readiness design and the evidence contract/Annex A.

### Repair validation

Compilation uses the default TMPDIR. Changes remain uncommitted in this worktree.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass |
| `cargo test -p xtask --lib depth_acceptance::admission` | 23 passed, 7.58 s tests |
| `cargo test -p xtask --lib isolated_maintenance::tests::in1_integration::admission` | 3 passed, 1.40 s tests |
| `python3 -m unittest discover -s scripts -p test_depth_ci_archive.py` | 12 passed, 0.025 s |
| Connected ON2 test | Pass, 236.53 s tests, 7.49 s compilation |
| `make loc-check` | Pass, all Rust files at or below 700 code lines |
| `make hygiene` | Pass |
| `git diff --check` | Pass |

All requested repair validation passed. A separate in-memory matrix also passed 36 random-content
size/compression combinations across empty, chunk-boundary, exact-limit and overflow inputs.

## Files changed

- New `crates/xtask/src/depth_acceptance/admission/`: `record.rs`, `policy.rs`, `objects.rs`,
  `transport.rs`, `history.rs`, `emitter.rs`, `mod.rs`, fake host `fake.rs`, `tests.rs` and
  `tests/policy.rs`.
- IN1 integration: `depth_acceptance/{mod.rs,integration.rs,integration/generation.rs}`.
- Evidence integration: `depth_evidence.rs`, `depth_evidence/verdict.rs`,
  `depth_evidence/verdict/github/{mod.rs,population.rs,tests.rs,tests/onboarding.rs}` and new
  `tests/onboarding/admissions.rs`.
- Isolated route: new `isolated_maintenance/admission.rs`, `host.rs`, `mod.rs`; test wiring in
  `tests.rs`, `tests/fixture.rs`, `tests/in1_integration.rs`,
  `tests/in1_integration/regressions.rs`, new `tests/in1_integration/admission.rs`, and
  `tests/bootstrap/{public_on1.rs,public_on2.rs}`.
- Transport: `scripts/depth_ci_archive.py`, `scripts/test_depth_ci_archive.py`.
- Docs: this report, `support-depth-freeze-readiness-design.md`,
  `support-depth-onboarding-producer-freeze-question.md`, and
  `docs/specs/{support-depth-contract.md,support-depth-evidence-v1.md}` (including Annex A).

Paths in the first four bullets are relative to `crates/xtask/src/`; design docs are under
`docs/agents/lifecycle/`. Removed the inherited untracked `crates/xtask/build.rs` and restored
`crates/xtask/Cargo.toml`: there is no build script, new dependency or manifest/lockfile diff.

## Record schema

Independent schema version 1, kind `in1_admission`. Typed, unknown-field-rejecting canonical JSON
uses deterministic ordered members, exact round-trip byte equality, a maximum of 65,536 bytes,
and SHA-256 over `support-depth-in1-admission-v1` + NUL + canonical bytes.

| Fields | Meaning |
| --- | --- |
| `repository` | Canonical name, repository id and owner id |
| `generation`, `generation_sha256` | Original Event/P/O/enrollment identity; nullable for pre-generation bootstrap |
| `candidate`, `parent`, `checkout`, `head_relationship` | Commit/tree objects and exact-head evaluated context |
| `old_state`, `new_state` | Lifecycle path/blob/stage; complete state remains content addressed |
| `checkpoint`, `decision`, `reasons` | Bootstrap or unclassified; admitted/refused; bounded reason enums |
| `predecessors`, `enrollment_anchor` | Sorted unique successful parent-frontier decisions and independent first-record anchor |
| `policy` | Approved implementation/workflow/closure and fixed workflow/job/app identities |
| `protected_tip`, `canonical_tip`, `accepted_origin`, `registry_blob`, `inputs` | Observed roots, accepted repository and authoritative input blobs |
| `observed_authority` | Required canonical digest of normalized producer and isolated-host facts, authorized adoptions and explicit completeness/absence |
| `producer_context`, `execution_evidence` | Nullable context and consumed evidence identities; producer absence is representable |
| `claims` | Run, attempt, emitting job, check and event |

Generation and evaluated-input identities have their own digest domains. Input identity includes
root snapshots and the independently computed `observed_authority`, and excludes retry association
and outcome, allowing conflict detection across decisions over the same observed facts. Refusals retain bounded reason enums, not arbitrary error text.

## Runtime policy identity and approval

The source-installed isolated profile supplies the authenticated protected installation commit
(`Profile.baseline`), independently of candidate JSON, local working files and producer discovery.
That evaluated policy source is pinned separately from subsequent candidate and protected-tip
observations. Runtime Git reads resolve its commit/tree/blob identities and verify object hashes.

The fixed policy closure includes `crates`, `Cargo.toml`, `Cargo.lock`, `.github`, the exact
`.github/workflows/ci.yml`, `scripts`, the exact `scripts/depth_ci_archive.py`, plus installed
profile scopes. Whole tree ids bind additions/deletions and nested modes, not just a chosen file
list. Root Cargo/toolchain configuration overrides remain unsupported as in producer approval.
`implementation` summarizes canonical closure JSON with domain
`support-depth-in1-git-policy-v1` + NUL; the map itself records Git object ids.

The evaluator reconstructs the approved closure from the independent protected source and
compares evaluated source tree/blob objects with that anchor, using the same object-equality
rule as producer closure approval. It refuses a copied approved map over changed source, a valid
recomputed candidate map/digest, omitted closure members, unsupported configuration or a host
record claiming another policy. Candidate-supplied identities never approve themselves. Source
installation also pins workflow/app/job provenance. The fake policy tree is explicitly modeled;
phase C must establish the deployed executable/source/runtime binding and real protected-host
provenance. FR1 claims no compiled-binary attestation or production qualification.

## Archive reader decision

Kept Python `in1` as the existing proof transport's ZIP reader. It returns bytes from exactly one
regular, unencrypted `admission.json`; input archive bound is 2 MiB, declared and streamed expanded
payload bounds are 64 KiB. No extraction or Git lookup occurs. Rust owns canonical JSON, typed
schema, record digest, host association and policy/history authentication. This preserves the
existing transport boundary without adding a Rust ZIP dependency. Twelve Python tests include
existing witness/registry cases, IN1 boundary, duplicate, traversal, extra-entry, nonregular and
encrypted-entry refusals, independent expanded-length/CRC/completion checks, the exact size-lie
reproduction, descriptor-based streams, ZIP64 sizes and all four supported compression methods.

## Proving-test audit

All listed FR1 tests pass in the completed depth-acceptance group or the connected ON2 proof.
The corrected bootstrap group also exercises producer-discovery refusal after the admission
population becomes unavailable, even with retained payload bytes and advanced tips. All 59
bootstrap tests pass. The final focused connected proof also passes with only the other PR's run
present, so refusal on the original PR cannot mask a missing cross-PR check. Runtime references
below are measured containing suites, not invented individual timings: A = 62 depth-acceptance tests, 41.79 s (49.93 s wall including
compile); C = final connected ON2 test, 398.78 s (407.19 s wall including compile).
The earlier connected run took 132.87 s (142.67 s wall including compile/lock wait).
The focused 17-test admission group before the final three adversarial additions took 7.95 s.

Names without a prefix below are under `depth_acceptance::admission::tests::`.

| Required item | Executable coverage | Measured runtime |
| --- | --- | --- |
| Actual IN1 decision; first-record anchor; no producer context | `authenticates_actual_in1_and_first_record_enrollment_anchor` uses `admission_fixture`, which calls real `Route::with_admitted_publication`; bad anchor tree refuses | A |
| Canonical schema, versions, unknown fields, domains, 64 KiB | `canonical_record_roundtrip_bound_versions_unknown_fields_and_domains` | A |
| Candidate JSON, same-name check, missing publication | `refuses_candidate_json_same_name_wrong_job_attempt_and_missing_publication`; an empty decision stream confers no eligibility | A |
| Same-run wrong-job artifact despite valid digest and copied commitment | `policy::same_run_wrong_job_artifact_with_valid_digest_and_copied_commitment_refuses` | A |
| Wrong attempt and every retry separately authenticated | `every_retry_is_authenticated_and_wrong_attempt_commitment_refuses` | A |
| Missing predecessors; input/commit/tree mismatches; first decision without bootstrap | `missing_predecessors_object_mismatches_and_history_gaps_refuse` | A |
| Missing parent branch and raw object corruption | `verifies_raw_object_hashes_and_traverses_second_parent_to_the_anchor` | A |
| 1,000-run/stream pagination, 100-attempt overflow, job/artifact population overflow | `each_host_stream_and_control_bound_overflow_refuses`, `host_pagination_accepts_the_run_bound_and_refuses_gaps_duplicates_and_page_overflow` | A |
| 1,000 unique commits accepted, 1,001 refused without truncation | `unique_commit_traversal_accepts_1000_and_refuses_1001_without_truncation` | A |
| Archive/log/object bounds | `oversized_archives_logs_and_objects_refuse_without_truncation`; Python IN1 tests additionally exercise 64 KiB exact boundary | A; Python 0.018 s |
| Conflicting decisions; distinct roots and retries | `retries_are_distinct_conflicts_refuse_and_roots_distinguish_inputs` | A |
| Late retry cannot rewrite a child's predecessor | `isolated_maintenance::tests::in1_integration::admission::late_parent_retry_does_not_rewrite_existing_child_predecessors` (real emitter, same-run attempts two and three, both listing orders) | Round-2 IN1 publication group above |
| Revocation preserves historical authority | `revocation_preserves_immutable_history_and_changes_present_eligibility`; connected proof also revokes higher-stage authority and still refuses rollback | A, C |
| Changing roots/populations, including consumption recheck | `roots_and_populations_are_rechecked_after_history_and_at_consumption`; `concurrent_host_attempt_registration_preserves_the_complete_population` | A |
| Complete runtime Git policy identity; forged source/map/digest; overrides | `policy_identity_covers_git_policy_and_dependency_closure`, `policy::runtime_git_policy_authentication_rejects_self_selected_closure_and_source`, `policy::runtime_policy_refuses_root_toolchain_configuration_even_with_matching_claims` | A |
| Publication required for success; incomplete attempts; retained refusal | `isolated_maintenance::tests::in1_integration::admission::{in1_publication_failure_refuses_success_and_retains_an_incomplete_attempt,in1_missing_job_commitment_leaves_an_uploaded_attempt_incomplete,in1_retains_reached_refusals_without_granting_admission}` | IN1: 37.92 s tests |
| Admitted higher-stage ancestor with no producer; both producer API orders; another PR | `isolated_maintenance::tests::bootstrap::public_on2::public_on2_connected_freeze_write_admission_and_observation` calls `depth_evidence::...::onboarding::admissions::prove_unobserved_admission`, which invokes actual producer discovery; the producer-only progression observations alone succeed, independently retained higher-stage IN1 authority causes refusal; a separate inventory containing only the other PR prevents the original PR from masking a missing cross-PR check | C; full bootstrap 240.79 s |
| Fake tip movement cannot manufacture admission | `fake_tip_advance_has_no_admission_issuance_side_effect`; connected proof hides the admission population with advanced tips and retained payloads intact and requires history-gap refusal, including from producer discovery | A, C; full bootstrap 240.79 s |
| Existing IN1/MA/ON behavior | Existing manual/closeout/promotion/integration tests pass in A; unchanged GitHub maintenance/onboarding evidence tests pass; runtime-follow-on 45 pass; IN1's socket-dependent cases are blocked; bootstrap 59 pass | See validation |

## Interfaces left for FR2/FR3

- `Record` reserves `request_freeze` / `execution_required` checkpoint enums. FR1 emits bootstrap
  or unclassified; it does not infer a reporting waiver or readiness transition.
- `producer_context` remains nullable; execution-evidence references are representable separately.
  FR3 must supply authenticated reporting context and joins to retained admission/history.
- `Publication` and `Host` expose attempt-specific begin/upload/commit and independent json/bytes
  reads. `History::{check_candidate,check_progression,candidate_predecessors,recheck,eligible}`
  expose authenticated progression, complete parent frontiers, freshness and present eligibility.
- FR2 must add pure writer-derived complete-tree planning and IN1 freeze equality. FR3 owns
  canonical MA2 request/report collection-discovery binding and population classification.

## Deviations and proof limits

No FR2/FR3 behavior was added. The connected unobserved-ancestor proof extends the existing real
ON2 test instead of duplicating expensive writer setup. Small record/host tests share one immutable
real-IN1 fixture and fork independent host facts; they do not depend on test order or thread-local
host state. Existing fixture-installed public-route tests retain their scoped installation guards.
The Python reader remains transport-only. Source policy identity uses runtime Git objects instead
of the inherited unreviewed compile-time hash. The first new closure-coverage test demonstrably
failed against inherited code (missing `crates`, 0.91 s test / 8.66 s wall).

Production `LIVE_PROFILE` remains `None`. Real transport publication, protected-policy app/source
provenance, emitter isolation, retention and deployed runtime qualification remain phase C. The
actual reporting exception, connected ON3–ON5 writers and source-stable publication remain outside
FR1. Historical admission does not grant current eligibility after revocation.

## Commands and results

Full captured logs and machine-readable wall timings are under ignored `target/fr1-validation/`.
`results.json` contains this continuation's runs. Isolated-group logs use the full group name with
`::` replaced by `_` (for example `isolated_maintenance_tests_in1_integration.txt`); older worker
logs with abbreviated names in that directory are not the results reported here.
Times use default TMPDIR; no sandbox restrictions were bypassed.

| Command | Result | Runtime |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | Final 3.22 s wall |
| `git diff --check` | Pass | Below 1 s |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | Final 5.84 s wall (earlier 9.56 s / 13.91 s) |
| `cargo test -p xtask --lib depth_acceptance` | 62 passed | 41.79 s tests / 49.93 s wall |
| `cargo test -p xtask --lib depth_evidence` | 91 passed, 12 environment failures, 1 existing ignored subprocess entry | 77.16 s tests / 77.24 s wall |
| `cargo test -p xtask --lib runtime_follow_on` | 45 passed | 42.77 s tests / 42.86 s wall |
| `cargo test -p xtask --lib isolated_maintenance::tests::in1_integration` | 23 passed, 4 socket-restricted failures | 37.92 s tests / 37.99 s wall |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | Corrected run: 59 passed | 240.79 s tests / 247.88 s wall |
| Final focused connected ON2 test (`--exact`, only other PR also tested) | 1 passed | 398.78 s tests / 407.19 s wall |
| `python3 -m unittest discover -s scripts -p test_depth_ci_archive.py` | 8 passed | Final 0.013 s tests / 0.119 s wall |
| `make loc-check` | Pass, no Rust file above 700 code lines | Final 0.130 s wall |
| `make hygiene` | Pass | Final 0.112 s wall |

The evidence failures reproduce documented sandbox restrictions: ten staged-channel tests cannot
bind Unix sockets (`Operation not permitted`, including the forged-channel child that never binds);
two native local-guard tests fail `ForeignLoader` because temporary-directory fixtures see
injected `.codex` and `.agents` ancestor loaders. Those tests and guards remain unchanged.
The lead must rerun the full affected groups unrestricted; these results are not a fully green gate.

The four IN1 failures are the unchanged connected closeout cases
`in1_closeout_candidate_admission_and_post_push_observation`,
`in1_prepared_closeout_candidate_admission_and_post_push_observation`,
`in1_reauthenticates_population_in_admission_and_post_push_observation`, and
`in1_refuses_forged_closeout_entries_in_admission_and_observation`. Each fails while opening the
MA4 Unix-socket evidence channel, before its connected admission assertions. All three new
admission publication tests pass, as do the other existing IN1 tests (including the parent race).

Bootstrap's initial run exposed an incorrect new test assertion, not an implementation failure:
hiding the admission population preserves payload bytes in the fake artifact store. Discovery
correctly refused the gap, but the assertion incorrectly expected an empty payload store. The
assertion now requires the payload store to remain unchanged across discovery. Full bootstrap
rerun passed all 59 tests. Initial run took 129.13 s tests / 129.21 s wall; the rerun took
240.79 s tests / 247.88 s wall. The later standalone other-PR assertion passed in the focused
connected-test rerun (398.78 s tests / 407.19 s wall).

Final audit tightened the cross-PR proof: after both listing orders, producer discovery is also
called with only the other PR's run. This ensures a refusal on the original PR cannot mask missing
cross-PR progression. This adds no writer setup; the final connected proof passed after this
assertion was added.

Final outcome: all FR1 proving cases pass; all 59 bootstrap tests pass. Across the requested full
groups, 280 tests pass, 16 reproduce environment restrictions, and one pre-existing subprocess
entry remains intentionally ignored by the original suite. Required fmt, clippy, LOC and hygiene
checks pass. No assertions or tests were removed to hide those restrictions. The lead still needs
to rerun the affected evidence and IN1 groups unrestricted.

Runtime limit: small admission cases share immutable setup (the initial 17-test group took 7.95 s).
The connected proof reuses the existing real ON2/IN1 chain, but its final focused run took 398.78 s,
compared with 132.87 s earlier; the full bootstrap rerun took 240.79 s. This variability was not
profiled, and the final focused timing exceeds the earlier five-minute evidence-suite target.
Do not infer a stable small CI runtime from the earlier faster run. No extra writer fixture was
added for the other-PR assertion.
