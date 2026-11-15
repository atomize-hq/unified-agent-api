# FR3a reporting context implementation report

Status — implemented and validated on `feat/fr3-reporting-context`, subject to the environment restrictions below. Changes are uncommitted for the lead.

## Review round 1 fixes

The review fixes add collector-to-IN1-to-population regressions for authenticated PR test-merge
selection, closing index inventory, structurally admitted freezes after readiness, independent
execution without a witness, and publication-time observation changes.
Identity fields are pinned before evaluation. Transient observation failures retain incomplete
results with captured identities. Tests assert specific bounded refusals through the real paths
and retain the committed canonical literals as independent expectations.

The original PR merge case called `dispatch` directly. The reopened-freeze case reached structural
refusal before the readiness guard. Failed outcome checks mutated a literal without exercising
dispatcher publication. Independent execution
was covered by substantive witness artifacts, leaving the observation-only path unproved.

Both real-writer proofs now collect at an identical-tree PR merge whose second parent is the
freeze candidate. The host supplies the current candidate independently, and the context retains
the separate checkout object and both parents. Dispatch and closing publication recheck the pinned
current candidate association. Real IN1 admits the candidate and population resolution joins that
same merge context.

Closing publication compares the complete captured stage-0 index listing, including modes and
blob identities, after the existing raw-worktree verification.
`reporting_dispatcher_retains_captured_inputs_on_upload_changes` and the connected
`publication_failures` matrix inject staged additions, deletions, modes and bytes, a same-tree
checkout change, changed invocation and current candidate association, independent execution and
an unavailable execution observation during actual upload. The matrix compares the entire
retained context with the captured context after changing only outcome/completeness, authenticates
the retained failure, and asserts exact IN1-capture and population refusals.

`freeze_after_readiness` preserves the implementation tree and proposes a new identical-tree
freeze transition. Real IN1 must admit `RequestFreeze` with `ExactFreezeTree` and the exact context
reference. Authenticated history and population resolution must then return
`reporting checkpoint cannot reopen irreversible readiness`. The proof reverses admission and
producer listings and includes the original packet PR and another PR.

`observed_execution_without_witness` runs the collector for successful and failed collection,
sets independent execution to true, and proves that no substantive witness exists. Real IN1
capture requires `reporting context has substantive execution observation`. Population resolution
continues to the substantive path and requires `missing/ambiguous attempt witness artifact`.

All requested review-round checks pass with `TMPDIR` unset. Logs live under the ignored
`target/fr3-round1-validation/` directory.

| Command or observation | Review round 1 result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass |
| `cargo test -p xtask --lib depth_ci` | 8 passed |
| `cargo test -p xtask --lib depth_evidence::verdict -- --test-threads=2` | 81 passed |
| `cargo test -p xtask --lib depth_acceptance -- --test-threads=2` | 89 passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr3_reporting` | 1 connected proof passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr2_maintenance -- --test-threads=1` | 3 passed |
| `cargo test -p xtask --lib fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1` | 1 connected proof passed |
| `make loc-check`, `make hygiene`, `git diff --check` | Pass |
| Sandbox-prevented requested checks | 0 |

The old HEAD-based collector selection failed the PR-merge proof. Removing the closing inventory
check failed the staged-addition regression. Removing the readiness loop failed after exact
structural IN1 admission. Removing population's independent execution check changed the expected
missing-witness refusal to a failed-reporting refusal. Removing the closing checkout check allowed
the same-tree checkout mutation to succeed and failed the publication regression. Each temporary
guard removal was restored before final validation.

An intermediate native interruption test returned `NotFound` while spawning its subprocess during
concurrent rebuilds. The stable-source verdict-group rerun passed all
81 tests. A temporary-root placement inside the workspace also caused Cargo workspace-discovery
errors during an initial run. Validation with `TMPDIR` unset completed successfully.

## Delivered mechanism

The existing collector selects its trusted reporting dispatcher before witness construction or candidate build/execution. Context v1 uses deterministic canonical JSON, the 65,536-byte limit and the independent NUL-terminated `support-depth-reporting-context-v1` digest domain. The committed literal `depth_ci/reporting/context-v1.json` has independently computed SHA-256 `63e791e20e3d0828b64e1e258fe565868afa6d39fe673379da12207c5e4e4c6a`.

FR1's exact-job/attempt envelope reader authenticates both control kinds. The context uses `DEPTH_REPORTING_CONTEXT_V1 kind=reporting_only sha256=<digest> artifact=<id>` in its emitting job's retained log. The existing bounded `admission.json` ZIP envelope carries the payload. Separate kind, parser, prefix and digest domain distinguish the records. The Python archive implementation is reused directly.

The dispatcher reads candidate trees as bounded data through FR2's complete-tree reader. It compares the entire stage-0 inventory, raw file bytes and executable modes against the actual checkout, then closes those observations through the existing GitInputs batch reader. This catches assume-unchanged files and dirty indexed/worktree content. Checkout, protected history and normalized invocation facts receive closing checks around publication. A failed closing observation retains pinned inputs with failed/incomplete reporting outcome.

IN1 resolves the authenticated context in `Emitter::prepare`, before owning predicates, and pins its identity in `producer_context`. Closing failures append `ObservationIncomplete` while retaining that identity and the reached decision. Absent context is an authenticated absent association. Reporting eligibility requires the later successful eligible exact-transition IN1 record, its context reference, complete protected history and irreversible readiness checks.

Population discovery retains every reporting attempt separately from execution receipts. Authenticated structural freeze heads receive zero-credit treatment even when their context cannot be joined. Context-only populations produce unverified results and an empty support selection. Historical reporting attempts skip execution-source approval and contribute zero credit. Substantive receipts and independently observed execution invalidate reporting treatment and continue through receipt authentication and counterexample/conflict evaluation. Failed reporting, missing admissions and incomplete producers prevent verification. Normalized context identity, producer terminal success and independent execution observations participate in IN1 observed-authority capture. Availability failures mark incomplete capture, and closing execution changes preserve the prepared reference.

## Identity capture audit

| Record fields | Prepared authoritative source |
| --- | --- |
| `schema_version`, `kind`, `selected_mode` | Compiled independent v1 schema and selected trusted reporting dispatcher. |
| `repository`, `enrollment_anchor`, `policy` | Source-installed isolated authority, bounded rehashed policy objects and full approved build/workflow closure. |
| `generation`, `candidate_record` | Typed canonical candidate depth-record blob, original Event/P/O/selection and its Git identity. |
| `candidate`, `predecessor`, `predecessor_admissions` | Bounded rehashed commit/tree objects, actual first parent and independent authenticated predecessor frontier. |
| `invocation` | Independently supplied exact run/attempt/job/check/event, checkout and parents. The consumed `depth_mode` and `profile` configuration keys are normalized before capture. Ignored prose cannot salt identity. |
| `head_relationship`, `source_observations` | Rehashed candidate, checkout and predecessor objects. Exact head or a two-parent PR merge with the candidate second and the identical freeze tree. |
| `indexed_tree`, `worktree_tree` | Complete stage-0 inventory and raw clean checkout comparisons before reporting dispatch. |
| `enrolled_substantive_path_not_invoked` | Trusted in-process branch observation of its enrolled execution path. |
| `reporting_outcome`, `collection_complete` | Terminal reporting and closing publication observations over the pinned inputs. |
| IN1 `producer_context` | Authenticated context identity captured before owning evaluation. Retention and closing failures preserve that reference. |

Required read/parse failures abort complete capture. Incomplete admission records retain FR1's quarantine and conflict rules. Contexts reference admitted predecessors and complete without requiring their own future admission.

## FR3b interface and proof boundary

`depth_ci::profile::GenerationResolver` is the single interface for current and historical resolved generation bindings. The production implementation retains the existing profile resolver. FR3b owns canonical MA2 resolution and evaluation-time source-approval/currentness proofs.

The connected MA2 test supplies a canonical binding through a fixture-only resolver. It emits context at genuine MA2 before auxiliary fixture insertion and real IN1 admission. ON2 exercises the ordinary `collect_synthetic` entry point. The existing checked MA2/ON2 writers and real IN1 predicates run before fake protected-tip advancement.

The credential-free fixture assumes a trusted test process and independent modeled host observations. It establishes local predicate, binding, classification, publication and history behavior. Real GitHub issuance, deployed VM isolation, actual job/log/artifact association, retention, merge queues and credentials require phase C.

## Proving coverage

- Genuine MA2 and ON2 context publication precedes real IN1. Only-checkpoint populations remain unverified with zero selected receipts. Later qualifying execution/adoption verifies the generation while preserving historical checkpoints.
- Every retry has separate context and admission association. The maintenance fixture issues a historical retry after an execution-required admission. Populations include pull requests, merge groups, workflow dispatches and pushes across distinct PR identities in both listing orders.
- Shared transport refusals cover wrong job, cross-attempt replay, checkout/tree/parent substitutions, changed dispatch/workflow provenance, failed/cancelled/nonterminal producers, missing independent context and substituted artifact bytes.
- Real IN1 preparation rejects changed or unavailable context. A reached admission with a late context change retains the prepared reference with incompleteness. Ignored invocation prose preserves canonical identity.
- Complete actual-checkout checks cover hidden worktree edits, genuine identical-tree PR merges, changed merge trees and reversed parents.
- After an execution-required admission, the maintenance fixture preserves the implementation tree and creates a new exact freeze transition. Real IN1 admits `RequestFreeze`. Authenticated history and population resolution then refuse reopening irreversible readiness, in both listing orders and on the original packet PR and another PR.
- Passing and failing substantive checkpoint execution is covered at the evaluator and connected population boundaries, preserving zero-credit passes and qualifying counterexamples. Existing execution-failure, progression, completeness and cross-flow suites also run.

## Validation

Final logs are ignored artifacts under `target/fr3-validation/`. Cargo uses `env -u TMPDIR`.

| Command | Final result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass |
| `cargo test -p xtask --lib depth_ci` | 7 passed |
| `cargo test -p xtask --lib depth_acceptance` | 88 passed, 25.13 s |
| `cargo test -p xtask --lib depth_evidence` | 92 passed, 12 environment failures, 1 existing ignored |
| `cargo test -p xtask --lib isolated_maintenance::tests::in1_integration` | 35 passed, 4 socket-blocked, 88.21 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 62 passed, 418.13 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr2_maintenance` | 4 passed, 136.83 s |
| `cargo test -p xtask --lib git_inputs` | 33 passed, 1 socket-blocked |
| `python3 scripts/test_depth_ci_archive.py` | 13 passed, 0.047 s |
| `make loc-check` | Pass |
| `make hygiene` | Pass |
| `git diff --check` | Pass |

The depth-evidence environment failures comprise ten MA4 Unix-socket cases and two native guard cases returning `ForeignLoader`. The native guard observes managed temporary-root `.codex` and `.agents` loader directories. Four IN1 connected closeout cases and one additional GitInputs-filtered scratch-view case also fail at the sandbox's Unix-socket restriction, with `Operation not permitted`. Their dependent assertions require an unrestricted rerun.

Initial validation caught a preserved refusal-order regression and test-only untracked receipt/context files. The refusal ordering and fixture cleanup were repaired. The MA2 population fixture now uses a complete original request commit while the existing FR2 literal fixture retains its original parameters. The context canonical bound test now enlarges a schema-authorized field, preserving the distinction between normalization and size validation.

The requested test groups complete with 16 environment-dependent failures across depth evidence and IN1. The additional GitInputs run exposes one further socket-blocked case, for 17 distinct environment-dependent cases. Their dependent assertions remain unverified. The final ON2 and MA2 proving cases pass, including context publication, real IN1, protected-history joins, separately authenticated retries and both substantive outcomes. Full-group durations reflect concurrent validation and repeated source/history authentication. Shared immutable fixtures and independent host forks serve the mutation matrices.

The lead can commit the source, schema text, Annex A and this report together. FR3b's canonical production resolver and approval/currentness proofs, and phase C's deployed host qualification, are pending within their assigned passes.

## FR3b

Status — implemented and validated on `feat/fr3-reporting-context` over `c8ee17b9`, subject to
the sixteen environment-dependent cases below. Changes remain uncommitted for the lead.

### Binding and discovery

`depth_ci::maintenance` is the sole canonical MA2 resolver behind `GenerationResolver`. Collector
dispatch, substantive collection, build-graph acquisition inputs, witness authentication and
population discovery consume it. The fixture-only resolver has been removed. Exact immutable host
heads supply Git-authenticated standing-request and versioned-report bytes. Current indexed views
use the same resolver and retain GitInputs freshness checks.

The resolver captures both required reads before parsing, checks supported structures/versions,
original Event, agent/version/enrollment and request P/O agreement, validates acquired digests, and
reuses approved policy/acquisition derivation over the same inputs. The original finite machinery
profile retains its independently compiled P/O expectation. Later E/entries are projected out after
validation. No execution outcome, session, retry, ordering or failed lookup selects a binding.
Ignored request metadata and auxiliary fixture bytes contribute no generation identity. Fixture
bytes continue to belong to the ordinary execution source closure when present.

Either record's target Event associates a head even when the other record is invalid or absent.
Authenticated original packet-PR membership and complete retained IN1 generation association also
prevent record removal/replacement from hiding a head. Retained association follows rehashed bounded ancestry through unadmitted descendants, stopping
at a complete admission for another generation on each branch. Incomplete retained decisions
confer no association authority and do not stop an older complete association. Coherent other generations and heads without target association retain
ordinary unrelated treatment. The real MA2 matrix also re-derives a coherent different O under the
same original Event and checks that independent target association changes its treatment. Ambiguous versioned-report claims refuse. Read unavailability
propagates as incompleteness. Matching attempts are retained before substantive artifact reads.
Existing execution-source refusal ordering is preserved. Closing population checks bind normalized
packet-PR membership. Unrelated PR numbers and ignored PR prose do not salt captured identity,
while a newly appearing target association refuses the population.

The mutation matrix runs through real discovery with bounded reason assertions. It covers both
record directions with and without packet-PR association, missing/malformed/contradictory records,
unsupported versions, wrong agent, ambiguous reports, stale acquired input, removed/replaced
associated records, coherent unrelated heads and unavailable report transport. Producer and
historical discovery bindings agree under fixture absence, mismatch and mutation. Those changed
fixture heads remain discoverable and reach the ordinary execution-closure refusal.

### Source approval and publication

The genuine MA2 test uses `collect_synthetic` before auxiliary insertion. The real MA2 and ON2
writers precede real IN1, retained context/admission authentication and protected-tip advancement.
The shared connected proof constructs an execution at S while protected source approval still
refuses it, then runs real IN1 over that exact implementation source before advancing modeled
protected history. Evaluation retains the original run/attempt and verifies all eleven obligations.
A separately changed current collector S2 refuses the retained S1 run. Scoped fake-host installation
restores the enclosing route after the connected real-IN1 approval callback.

Both real freeze writers preserve the entire execution closure by Git object equality for `crates`,
Cargo manifest/lock, workflow and archive reader. Existing publication generation reproduces the
committed support JSON, Markdown and runtime projection byte for byte. FR2's reviewed literal
packet/report/runtime expectations remain independent of the planners. The genuine MA2 binding
also compares its complete P/O with the committed FR2 literal. The MA2 reporting test now lives
under `isolated_maintenance::tests::fr3_reporting::maintenance` so the requested FR3 filter executes
its connected proof directly.

Real ON3–ON5 publication-writer closure proofs are deferred to slices 6–9, as assigned by the
design. The credential-free host protocol proves local predicates, binding, classification,
retention/history joins and the measured real-writer source closure. Its test process remains
trusted. Real GitHub issuance, exact deployed job/log/artifact association, emitter isolation,
merge queues, retries/retention and credentials remain phase C.

### Row-by-row design coverage

| Positive or boundary | Executing tests and measured boundary |
| --- | --- |
| Reproduced defect | `fr3_reporting_context_real_ma2_and_later_execution_population` and `fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1` call `prove_reporting`, retaining genuine older freezes with zero credit before later execution/adoption verifies the generation. |
| Real MA2 discovery | `fr3_reporting_context_real_ma2_and_later_execution_population` runs actual collector dispatch at genuine MA2 with no auxiliary fixture. It also executes `binding_matrix`. `canonical_maintenance_generation_association_refusals_and_fixture_independence` separately exercises fixture absence/mismatch/mutation through producer binding and real discovery. |
| Real MA2 combined commit | Existing FR2 `fr2_ma2_combined_acquisition_and_freeze_real_in1` covers the real transport's single acquisition/freeze commit, complete byte/mode/kind authority, canonical freeze output, unqualified support, absent E/entries and real IN1 before observation. |
| ON2 split | `fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1` authenticates the real reporting freeze, its context and IN1. Its extra-edit candidate is execution-required. `public_on2_connected_freeze_write_admission_and_observation` executes the real implementation writer and IN1. |
| ON2 acquisition boundary | Existing FR2 `fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1` preserves the protected parent's OF-P4 statement/acquisition. `public_on2_in1_requires_parent_acquisition_and_intact_binding` refuses same-candidate acquisition/statement substitution. |
| Freeze retries | Both connected freeze tests execute `fr3_reporting::after`, including attempt 1, independently emitted/admitted retry and the maintenance retry after execution-required admission. `every_retry_is_authenticated_and_wrong_attempt_commitment_refuses` covers retained retry transport. |
| Other PRs/events | Both connected freeze tests emit contexts on pull_request, merge_group, workflow_dispatch and push, including distinct PR numbers. `prove_reporting` consumes the full population in both listing orders. `onboarding_same_generation_later_runs_wait_for_producer_not_workflow` retains execution attempts across events. |
| Only checkpoints | Both connected freeze tests use `prove_reporting` to assert no execution receipts, empty support selection, the complete reporting-attempt count and an unverified verdict. |
| Source-approval timing | Both connected freeze tests execute `approve_source` through real IN1 after execution S exists, then verify the retained original run against identical protected S. `publication_closure` checks complete freeze closure and byte-stable publication. Real ON3–ON5 writer closure is deferred to slices 6–9 because those writers connect there. |
| Noncircular ordering | Both connected freeze tests run collection before their own admission and assert that no earlier IN1 references the new context. History refuses the context until its later IN1. `onboarding_reuses_canonical_collection_and_has_no_packet_admission_invocation` retains the ordinary collector boundary. |

| Refusal | Executing tests and bounded refusal |
| --- | --- |
| Forged authority | `refuses_candidate_json_same_name_wrong_job_attempt_and_missing_publication`, `same_run_wrong_job_artifact_with_valid_digest_and_copied_commitment_refuses` and both connected freeze tests' `refusals` cover candidate copies, wrong job/attempt, substituted context and source/workflow claims. Canonical binding matrix rejects wrong agent/version/Event/P/O. |
| Implementation hidden in freeze | `fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1`, `fr2_ma2_freeze_only_complete_tree_canonical_packet_and_real_in1` and `complete_tree_equality_includes_bytes_modes_kinds_presence_and_preserved_paths` cover source/dependency/build/workflow and other extra bytes through complete-tree structural classification. |
| Extra change in combined MA2 | Existing FR2 `fr2_ma2_combined_acquisition_and_freeze_real_in1` checks unrelated edits, schema-valid changed authenticated metadata/lockfile bytes, missing authority, inventory, modes and kinds through real acquisition/classification/IN1. |
| Failure before readiness | `onboarding_same_generation_later_runs_wait_for_producer_not_workflow`, `nonterminal_aborted_or_failed_producer_steps_never_verify`, `missing_attempt_or_wrong_job_or_artifact_digest_refuses` and `native_complete_population_preserves_counterexamples_and_later_missing_attempts` retain adverse/missing earlier executions despite later success. |
| Backward progression | `public_on2_connected_freeze_write_admission_and_observation` executes `prove_unobserved_admission`, covering a higher-stage admitted ancestor with no producer, both API orders and another PR. `onboarding_producer_refuses_descendant_lifecycle_rollback_in_either_api_order` retains observed-source progression. The review-round `freeze_after_readiness` requires real structural admission before asserting the irreversible-readiness refusal through history and population, both listing orders and both PR associations. |
| Changed actual checkout | Both real-writer proofs collect at a genuine identical-tree PR merge through `collect_synthetic`, then run real IN1 and population resolution. `checkout_relationships` retains direct-dispatch negative relationship coverage. `dirty_checkout_and_pinned_context` covers hidden worktree edits and late IN1 context changes. `publication_failures` covers closing index additions, deletions, modes, bytes and checkout changes. |
| Changed dispatch/workflow context | Both connected freeze tests' `refusals` and checkout matrix authenticate dispatch/configuration, workflow policy and invocation independently. Changed substantive mode cannot inherit reporting classification. |
| Unexpected substantive execution | Both connected freeze tests execute `unexpected_execution` with witness artifacts through population discovery and the evaluator for passing and failing execution. `observed_execution_without_witness` exercises independent execution with no substantive artifact through collection, IN1 capture and population resolution for successful and failed producers. `reporting_checkpoint_substantive_pass_and_failure_remain_conflicts_and_counterexamples` preserves zero-credit passes, failed counterexamples and conflicts. |
| Failed reporting producer | Both connected freeze tests mutate authenticated producer job conclusion and require `failed reporting producer or incomplete collection`. Their transport matrix covers cancelled/nonterminal jobs and missing publication. `publication_failures` exercises actual upload-time changes, retained captured identities, failed/incomplete outcomes and exact downstream IN1/population refusals. The committed literal separately pins canonical bytes and digest. |
| Historical execution closure | Both connected freeze tests retain S1 while independently current collector S2 causes `producer closure is not approved on protected staging`. `onboarding_genuine_freeze_attempt_remains_applicable_and_blocks_later_proof` retains its original execution-source mismatch assertion. |
| Missing/conflicting authority | Both connected freeze tests remove/revoke later exact-context IN1 and require `reporting context lacks later exact-transition IN1 admission`. `history_deficiency_quarantines_while_object_and_input_mismatches_refuse`, `retries_are_distinct_conflicts_refuse_and_roots_distinguish_inputs`, `roots_and_populations_are_rechecked_after_history_and_at_consumption` and the FR1 history suite cover missing/conflicting/incomplete/object/root authority. |
| Population incompleteness | `missing_attempt_or_wrong_job_or_artifact_digest_refuses`, `pagination_detects_truncation_duplicates_and_saturation`, `onboarding_same_generation_later_runs_wait_for_producer_not_workflow`, `each_host_stream_and_control_bound_overflow_refuses`, `host_pagination_accepts_the_run_bound_and_refuses_gaps_duplicates_and_page_overflow`, and `unique_commit_traversal_accepts_1000_and_refuses_1001_without_truncation` cover missing attempts/artifacts, unstable totals, duplicates, gaps, unsupported events and bounds. |
| Contradictory execution | `all_attempts_drive_verdict_even_when_e_selected_receipt_passes`, `complete_population_prevents_selecting_only_the_green_attempt`, `reporting_checkpoint_substantive_pass_and_failure_remain_conflicts_and_counterexamples` and both connected tests' substantive matrix retain pass/fail conflicts and counterexamples across retries/PRs. |
| Flow separation and scope | `onboarding_refuses_cross_event_population_and_independent_adoption_mismatch` resolves both real populations and rejects cross-flow use in both directions. `core_authenticated_bridge_evaluates_all_due_and_preserves_production_refusal`, `bootstrap_route_absence_keeps_production_depth_refusal` and `synthetic_requires_the_authenticated_isolated_binding` preserve scope and public refusal. |

Additional FR1–FR3 rows remain covered by the retained FR1/FR2 report inventories. FR3b adds the
explicit request/report two-direction matrix, malformed/unsupported/stale/ambiguous binding,
associated removal/replacement, unrelated heads, fixture independence, identity agreement and
`unavailable_canonical_report_never_becomes_unrelated_absence`. `generation_association_follows_unadmitted_descendants_and_stops_at_a_complete_new_generation`
checks inherited association and the newer-generation boundary through real history resolution.
The FR1 `history_deficient_admissions_cannot_grant_frontier_progression_or_transition_history`
regression also checks that quarantined transitions leave complete predecessor association intact.
The connected MA2 population's `associated_descendants` removes or coherently replaces both records
on an unadmitted descendant on another PR and requires the bounded association refusal.

### Validation

Logs are ignored artifacts under `target/fr3b-validation/`. Cargo uses `env -u TMPDIR`.
Final test groups use two or four test threads to bound temporary storage pressure. The FR3
filter executes the genuine MA2 chain. The ON2 extension runs in the bootstrap group.

| Command | Final result | Test duration |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | — |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | — |
| `cargo test -p xtask --lib depth_ci` | 7 passed | 0.00 s |
| `cargo test -p xtask --lib depth_acceptance` | 89 passed | 54.54 s |
| `cargo test -p xtask --lib depth_evidence` | 94 passed, 12 environment failures, 1 existing ignored | 108.77 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::in1_integration` | 35 passed, 4 socket-blocked | 67.15 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr3_reporting` | 1 connected proof passed | 226.76 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr2_maintenance` | 3 passed, including combined acquisition/freeze | 169.86 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 62 passed, including extended ON2 proof | 270.57 s |
| `cargo test -p xtask --lib freeze_transition` | 7 passed | 0.00 s |
| `make loc-check` | Pass, every Rust file within 700 code lines | — |
| `make hygiene` | Pass | — |
| `git diff --check` | Pass | — |

The sixteen environment-dependent cases match FR3a's categories. Ten depth-evidence MA4 channel
cases require Unix sockets prohibited by this sandbox, including one dependent forged-channel
assertion that reports `fake never bound`. Two native local-guard cases report `ForeignLoader`
from managed ancestor loader directories. Four connected IN1 closeout/population cases also fail
at channel creation with `Operation not permitted`. Their dependent assertions remain unverified
and need an unrestricted environment. The existing ignored subprocess entry remains ignored.

Initial parallel runs exhausted the temporary filesystem and caused unrelated file-write/Git
failures. A temporary worktree-local TMPDIR exposed Cargo ancestor-workspace discovery in existing
standalone producer fixtures. Returning to the normal temporary root with bounded concurrency
recovered those failures. A test-module relocation exposed repository-copy fixtures that require
every tracked path to remain readable before the lead commits. Its source path is preserved and
the test is mounted under the FR3 module. The existing commit-bound test moved into the lineage
suite to retain the 700-code-line cap, and now checks the shared 1,000-commit bound across retained
history and newly associated ancestry. Known authenticated parents are reused during association.

All FR3b binding/refusal, real-writer, source-timing and publication-closure proofs pass. The final
MA2 proof includes retained-admission ancestry association for removed and coherently replaced
records on another PR. Annex A and the binding rule are updated. Source changes remain confined
to the worktree and uncommitted for the lead. Real ON3–ON5 writer closure proofs remain in slices
6–9, and deployed host qualification remains phase C.

## Round 2 fix

FR3b made producer generation resolution read the canonical versioned report through a mutable
`GitInputs` view. MA5/MA6 writers legitimately rewrite that report's execution and acceptance
entries before staging it, so generation resolution rejected the proposal with
`working bytes disagree with index: cli_manifests/depth_fixture/reports/0.2.0/depth-record.json`.
The existing socket-free `publication_closeout_writer_socket_free_qualified_batch` reproduced this
failure before the fix. Inspection also found population discovery's current-generation lookup
on the same writer-time path, in addition to the graph and witness callers identified by the lead.

The authority choice follows design §4, “Canonical MA2 generation binding”: generation identity
comes from the immutable checkout commit. Witness collection captures the commit first and uses
its `Source` for the canonical resolver and both pre-build and post-build graph resolutions.
Maintenance population discovery resolves its target generation from `GitInputs::provenance`
at the caller's HEAD; a bound scratch view uses the original caller's object store. Historical
host heads continue through the same canonical resolver. Proposed build/acquired-input bytes and
inventories still pass their existing `GitInputs` checks and evidence evaluation. No freshness
check is weakened and no path exemption is introduced. Other profiles retain their existing
population target lookup.

`GenerationResolver::committed_generation` only requires a result from the existing resolver;
it adds no alternative derivation or fixture fallback. The standing request and canonical report
remain the generation inputs, the auxiliary fixture retains no generation authority, and read
failures still propagate. The resolver validates the full committed report before clearing
execution/acceptance entries, so an MA5-extended immutable report yields precisely the original
Event/P/O without accepting malformed execution data.

The new socket-free regression,
`writer_edits_do_not_change_the_committed_maintenance_generation`, failed with the exact reported
index mismatch before the fix and passes after it. It writes valid execution and closeout entries
without staging, confirms that an ordinary `GitInputs` read still rejects those bytes, and reruns
production population resolution. It checks unchanged Event/P/O and population size, preserves
the writer bytes and index, commits the extended report and checks the frozen projection again,
then commits an invalid execution identity and requires canonical-report refusal.

Validation logs are ignored artifacts in `target/fr3-round2-validation/` and
`target/fr3-round2-*.log`. Tests use `env -u TMPDIR` and `--test-threads=1` to bound temporary
storage use. Changes remain uncommitted on `feat/fr3-reporting-context`; nothing is pushed.

| Command | Round 2 result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass |
| `cargo test -p xtask --lib depth_ci` | 8 passed |
| `cargo test -p xtask --lib depth_evidence::verdict` | 82 passed, including the new regression |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr3_reporting` | 1 connected proof passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr2_maintenance` | 3 passed |
| `make loc-check`, `make hygiene`, `git diff --check` | Pass |

The affected publication, MA5, MA6, view-evaluation and connected IN1 results will be recorded
when their validation run completes.

## Round 3 fix

The ordinary 0.3.0 request coexists with the retained 0.2.0 depth report. That report's Event
makes the head relevant during discovery, and the evaluator's initial target lookup treats every
checkout as relevant. Both paths previously required `support_depth` before distinguishing an
ordinary later packet. Retained-report relevance triggers the parse failure before ancestry checks.

The resolver now parses an optional support-depth binding while retaining required typed request
fields and propagating request/report read and parse failures. A supported same-agent ordinary
request to another version, with a different Event, resolves to no active depth generation.
Evaluation then selects the fully parsed retained versioned statement from the caller's immutable
HEAD and projects its Event/P/O. Historical producer heads still use canonical request/report
resolution. Explicit packet-PR association refuses replacement, and discovery still checks retained
admission ancestry after an absent binding.

Removing `support_depth` from the target-version request yields an error and therefore
unverified/refusal under design §4. Changing only its version also refuses while its Event matches
the target. The new socket-free
`ordinary_later_request_preserves_retained_evaluation_and_target_refusals` reproduced the original
missing-field panic before the fix. It covers ordinary resolution, retained evaluation/population,
associated replacement, malformed TOML/typed fields/bindings and stripped target requests. The
transport regression now covers unreadable requests as well as reports.

Every Cargo/test command uses `TMPDIR=$PWD/target/tmp`. Changes remain uncommitted on
`feat/fr3-reporting-context` for the lead.

| Check | Round 3 result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass |
| `cargo test -p xtask --lib depth_ci` | 8 passed |
| `cargo test -p xtask --lib depth_evidence::verdict::github::tests::generation` | 4 passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr3_reporting -- --test-threads=1` | 1 passed |
| Additional `cargo test -p xtask --lib depth_evidence::verdict -- --test-threads=1` | 76 passed, 7 onboarding fixture failures |
| `make loc-check`, `make hygiene`, `git diff --check` | Pass |

The requested MA6 replay test was attempted before and after the fix. Both runs stop before the
reported path because the sandbox denies the MA4 evidence-channel socket with
`Operation not permitted (os error 1)`.

The additional verdict run's seven onboarding fixtures fail in their existing producer build
before resolver execution. Cargo discovers the enclosing repository workspace from the required
`target/tmp` location and reports that the standalone fixture package believes it belongs to that
workspace. Details are in the ignored `target/fr3-round3-verdict-tests.log`.

## Round 4 fix

The shared maintenance resolver now validates the retained report before returning ordinary-request
treatment. Report structure, duplicate Event claims across versioned reports, acquired-input freshness
and policy/obligation re-derivation all run before that return. An explicit retained binding supplies
population evaluation with the validated Event/P/O projection. Producer discovery continues to receive
no active generation for a clean ordinary later request. Retained validation uses the report's Event
and carried acceptance work, so the later request's trigger and audit cannot redefine its generation.

The socket-free generation regression reproduced a duplicate retained-Event claim yielding `verified`
before the fix. Its clean, duplicate-claim and stale-input cases now exercise population authentication
and the real evaluator. The clean case remains `verified`; both refusals yield `unverified` and retain
the exact ambiguity or stale-input resolver diagnostic. Direct resolution checks both target discovery
and initial evaluation. The existing ordinary-request regression also uses a different later trigger.

The requested temporary directory is read-only in this sandbox. Every Cargo/test command uses the
default temporary directory through `env -u TMPDIR`. The lead runs the socket-dependent MA6 replay.

| Check | Round 4 result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass |
| `cargo test -p xtask --lib depth_ci` | 8 passed |
| `cargo test -p xtask --lib depth_evidence::verdict::github::tests::generation -- --test-threads=1` | 5 passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr3_reporting -- --test-threads=1` | 1 passed |
| `make loc-check`, `git diff --check` | Pass |

Changes remain uncommitted on `feat/fr3-reporting-context` for the lead.
