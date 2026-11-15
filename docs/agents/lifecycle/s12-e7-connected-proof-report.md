# Slice 12 — E.7 connected new-agent implementation proof

Status: the credential-free in-process E.7 implementation gate passed. The qualified new-entry
chain, repaired qualified sibling, bounded release-watch variant and truthful-insufficiency
sibling all passed, as did the two new execution negatives and five ON2 controls. Required
phase C host evidence remains missing; the release-watch setup seam is bounded below.
Review round 1 changes remain uncommitted in the detached fix worktree at baseline `95bc9706`
for `feat/s12-e7-connected-proof`.

This bounded proof follows the [routing design](support-depth-onboarding-routing-design.md#proving-evidence-and-refusal-coverage),
OR-P5, OR-P12 and OR-P13, and [contract Annex A/A.4](../../specs/support-depth-contract.md).
It composes the existing real-writer bootstrap chain for the B.1 `depth_fixture` wrapper-only
subject at exact version `0.2.0`, retaining its onboarding approval Event. No new subject,
profile family, live profile, credentials, network operation or production behavior is added.

## Connected composition

The named E.7 chain consumes actual public ON1 outputs, governing adoption, the built B.1
`depth-fixture-snapshot` outputs, snapshotted metadata and protected classification; public ON2
freeze and offline candidate; modeled independent onboarding producer/artifact/adoption population with eleven adopted obligations;
public ON3; qualified ON4; prepared ON5, bounded human metrics and final closed ON5. Repair
and release-watch siblings retain their own committed checkpoints. The complete new-entry
chain runs real IN1 before every modeled protected advance and observes the post-tip transition.
The insufficiency sibling remains unverified and refuses owning acceptance with specific
diagnostics and unchanged caller trees. The release-watch authority setup is separately bounded below.

## Proof seams and limits

The host/API and admission transport are modeled in process. ON2 uses an offline subprocess
executor. ON4's test S/T runner validates the committed literal command inventory and supplies successful
format/lint/build/test/hygiene/security results; it does not execute those subprocess gates. ON4 nevertheless runs
slice 10's actual shared depth-reading checking implementations against the exact proposal and
fresh committed candidates. These seams cannot replace the separately executed slice 10 gate
composition tests or the independent committed-candidate CI gate.

**Missing required evidence:** the phase C independent disposable-repository host run under
CP-P5 / OR-P12 remains unperformed and unauthorized here. Phase C stays closed. This in-process
implementation proof cannot establish native queue serialization, deployed host transport,
live-agent support, runtime backend/capability advertising, or another agent's onboarding acceptance.
The lead owns the full gate and `scripts/s10_declared_test_leg.py` declared-registry leg.

## Negative coverage map

The following is a group → case → exact test-name inventory. Leaf test names are exact;
module prefixes below disambiguate them. **Prior** means executed evidence recorded in the
linked slice report, not a new run in slice 12. A prior containing test still marked running
is not counted as a completed positive. No existing tests are removed.

Prefixes: `B = isolated_maintenance::tests::bootstrap`,
`P = B::public_on3`, `D = depth_acceptance::onboarding::tests`,
`H = depth_evidence::verdict::github::tests`,
`I = isolated_maintenance::tests::in1_integration`,
`M = isolated_maintenance::tests::mediation_matrix`.

| Group | Required cases | Exact test names and executed source |
| --- | --- | --- |
| Bootstrap and identity | No route; foreign subject/version/target; fake backend; undeclared version; unknown/misplaced approval fields | `B::public_on1::public_on1_refuses_foreign_descriptors_and_mixed_approval_without_writes`, `B::public_on1::public_on1_absent_route_keeps_ordinary_success_and_depth_refusal`, `B::finite_approval_projection_rejects_unknown_misplaced_backend_and_foreign_fields`, `B::bootstrap_scope_refuses_records_foreign_descriptor_and_unknown_scope_without_fallback`, `B::bootstrap_absent_and_unselected_subject_require_exact_approval_and_scaffold` — prior [S10 bootstrap run](s10-readonly-gates-report.md#declared-registry-test-leg-fixes), 68 passed |
| Bootstrap and identity | Preexisting records; ON1 replay over bindings; proposed declaration/record/entry | `B::public_on1::public_on1_refuses_divergent_scaffolds_and_retained_acceptance`, `B::public_on1::public_on1_creation_admission_refuses_injected_table_record_and_entry`, `B::declaration::first_declaration_refuses_extra_tables_records_entries_acceptance_and_selection_changes` — prior S10 bootstrap run |
| Bootstrap and identity | Missing Annex B adoption; unacquired metadata; full validation of actual ON1 output | `B::preparation_refuses_every_binding_and_acceptance_effect` (including the unacquired 0.3.0 metadata case), `B::sweep::bootstrap_preparation_requires_exact_reviewed_governing_bytes`, `B::public_on1::public_on1_connected_preparation_metadata_and_first_declaration_chain`, `B::declaration::bootstrap_publication_checks_complete_files_for_preparation_and_declaration` — prior S10 bootstrap run. The new chains also call the actual full `manifest-validate` entry in ON1 preparation |
| Freeze and acquisition | P-only; absent/malformed/extra/missing/changed statement entries; branch-only statement; incomplete union | `isolated_maintenance::tests::fixture::runtime_follow_on::runtime_follow_on_freezes_p_record_object_and_contract_then_o`, `runtime_follow_on_refuses_missing_malformed_mismatched_and_extra_statement_inputs`, `runtime_follow_on_branch_local_statement_is_ineffective_until_integration`, `runtime_follow_on_requires_inputs_to_equal_both_statement_and_protected_tip`, `runtime_follow_on_incomplete_union_names_missing_targets_without_a_freeze` (same module) — prior [FR2 retained coverage](fr2-freeze-planner-report.md), [S7 depth-acceptance/connected coverage](s7-onboarding-predicates-report.md) |
| Freeze and acquisition | Every changed/deleted frozen O input at the ON3–ON5 acquired-input precheck | `isolated_maintenance::tests::fixture::runtime_follow_on::acquired_inputs_tests::publication_entrypoints_refuse_each_changed_frozen_acquired_input_without_writes` and `publication_entrypoints_refuse_each_deleted_frozen_acquired_input_without_writes` (same module) — round 1 focused validation below. Both compare the committed statement and frozen vector to eight reviewed literal paths. Each isolated deletion requires the complete literal refusal naming that path and an unchanged caller snapshot. Public dispatch deletion-specific coverage remains a gap because its declared-depth guard precedes this precheck |
| Freeze and acquisition | Changed P; post-integration re-freeze; earlier approval; record/object deletion; authentic parent acquisition | `B::public_on2::public_on2_in1_refuses_lifecycle_rollback_and_closed_generation_refreeze`, `public_on2_in1_requires_parent_acquisition_and_intact_binding`, `public_on2_connected_freeze_write_admission_and_observation` (same module) — prior [S11 ordinary/declared runs](s11-mediation-matrix-report.md), repeated here in the five-test ON2 filter; `B::fr2::fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1` — prior [FR2](fr2-freeze-planner-report.md) |
| Freeze and acquisition | Git replacements/grafts/alternates/environment; hidden bytes/modes/kinds and complete-tree acquisition authority | `isolated_maintenance::tests::fixture::runtime_follow_on::authority::runtime_follow_on_replace_ref_cannot_admit_a_branch_only_statement`, `runtime_follow_on_replaced_statement_blob_cannot_supply_provenance`, `runtime_follow_on_replaced_bound_blob_cannot_fake_integration_equality`, `runtime_follow_on_grafts_and_alternates_cannot_supply_integration_authority`, `runtime_follow_on_inherited_git_redirects_cannot_change_statement_authority` (same authority module) — prior FR2 46-test runtime-follow-on run; `B::regressions::bootstrap_checkpoint_refuses_masked_acquisition_bytes_and_modes`, `B::regressions::bootstrap_checkpoint_rejects_uncommitted_additions_in_its_bound_view`, `B::fr2::fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1` — prior S10 bootstrap/FR2 runs. These protections are unchanged |
| Execution and repair | Forged serialized baseline; interruption | `isolated_maintenance::tests::fixture::runtime_follow_on::e7_chain::e7_forged_serialized_baseline_cannot_hide_a_snapshot_edit` and `e7_interrupted_offline_executor_retains_only_trusted_failure_diagnostics` (same new module) — original execution run passed in 3.90 seconds, with strengthened round 1 validation below. SIGTERM follows substantive source and successful handoff writes in the retained candidate. The caller retains its original contract, prompt, handoff and raw lifecycle `support_depth` bytes. Exact trusted attempt fields and eight diagnostic files are checked while every other caller node is compared |
| Execution and repair | Protected input/snapshot/controller edits; execution failure; caller validation/delivery changes; nonempty written-path claim | `isolated_maintenance::tests::fixture::runtime_follow_on::public::runtime_follow_on_public_failed_candidates_preserve_record_inputs_and_truthful_diagnostics`, `runtime_follow_on_public_delivery_preserves_concurrent_caller_and_reports_refusal` (same public module) — prior FR2 runtime-follow-on run; `B::public_repair::public_repair_refuses_forged_success_when_protected_generation_is_enrolled`, `B::public_repair::public_repair_sibling_chain_preserves_frozen_generation_and_written_paths` — prior [S11 repair run](s11-mediation-matrix-report.md); `P::public_on3_connected_active_reporting_e_and_real_in1` — prior [S6](s6-on3-reporting-report.md), including changed written-paths and frozen-input refusals |
| Execution and repair | Repair preserves P/O; cannot backfill E, lose digest or heal orphan binding; runtime selection after repair | `B::public_repair::public_repair_sibling_chain_preserves_frozen_generation_and_written_paths`, `P::public_on3_connected_repaired_runtime_selection_and_real_in1` — prior S11/S6; new `P::e7_chain::e7_connected_onboarding_repaired_qualified_chain` continues the repair sibling through public acceptance |
| Evidence | Maintenance receipts vs onboarding; another Event; wrong P/O; unadopted mapping | `H::onboarding::onboarding_refuses_cross_event_population_and_independent_adoption_mismatch`, `P::public_on3_connected_active_reporting_e_and_real_in1` — prior [FR3 complete population](fr3-reporting-context-report.md), S6/S10 onboarding runs |
| Evidence | Missing obligation; partial/failed/nonterminal/retried attempts; changed population | `D::each_due_obligation_missing_has_its_own_bounded_refusal`, `H::onboarding::onboarding_same_generation_later_runs_wait_for_producer_not_workflow`, `H::synthetic::nonterminal_aborted_or_failed_producer_steps_never_verify`, `H::missing_attempt_or_wrong_job_or_artifact_digest_refuses` — prior S7/FR3; new `P::e7_chain::insufficient::e7_connected_truthful_insufficiency_refuses_on4_and_both_on5_entries` covers a genuinely reporting-only sibling |
| Evidence | Revoked adoption; changed source; dependency-only invalidation; stale pre-entry verdict after append | `H::onboarding::onboarding_source_adoption_and_dependency_currentness_are_independent`, `D::stale_evidence_and_pre_entry_verdicts_cannot_satisfy_onboarding`, `P::public_on3_connected_active_reporting_e_and_real_in1`, `P::public_on4::public_on4_connected_publication_real_in1_and_observation` — prior FR3/S7/S6/[S8 final round-1b result](s8-on4-publication-report.md#refusal-ordering) |
| Owning acceptance | Missing/inconsistent P/O/E; classification-required/carried work; debt additions/renewals/retirements | `D::each_due_obligation_missing_has_its_own_bounded_refusal`, `D::classification_carried_work_and_debt_refuse`, `D::adversarial::missing_execution_and_inconsistent_execution_bindings_refuse`, `D::adversarial::grants_renewals_baseline_debt_and_current_inventory_are_all_refused` — prior [S7](s7-onboarding-predicates-report.md), S9 20-test onboarding run |
| Owning acceptance | Wrong path; forged publication/proving entry; digest drift; same-path lifecycle retarget; maintenance drift clearance | `D::wrong_path_event_packet_and_lifecycle_refuse`, `D::adversarial::published_and_closed_assertions_cannot_be_retargeted_to_foreign_paths`, `D::adversarial::drift_clearance_and_frozen_object_byte_changes_refuse`, `P::public_on5::public_on5_refusals_local_final_and_conflict_preservation` — prior S7/[S9 final five-test ON5 run](s9-on5-closeout-report.md#round-1-focused-validation) |
| Owning acceptance | Closed semantic/timestamp edits; forbidden pointers/status/passed-target/marker; exact-entry preservation | `P::public_on5::public_on5_refusals_local_final_and_conflict_preservation`, `D::aggregate_effect_allowlists_refuse_pointers_status_passed_targets_and_marker_retirement`, `D::adversarial::metadata_without_a_record_and_min_supported_cannot_escape_the_batch` — prior S9/S7; new qualified E.7 tests retain exact final timestamp and entries through the same writer helper |
| Complete mediation | Foreign agent/target/version; ordinary requested scope displacing retained tuple; omitted/disabled declaration; removed record; nested root; proposed declaration/record | `M::onboarding_retained_and_proposed_lower_writer_matrix`, `M::lower::outer_callers_refuse_enclosed_depth_destinations`, `depth_refusal_routes::mediation_matrix::onboarding::every_on_entry_refuses_onboarding_carriers_without_scope_authority` (integration target), `D::outside_generation_effect_and_foreign_entry_refuse_the_entire_batch` — prior [S11 ordinary/declared matrix](s11-mediation-matrix-report.md) |
| Complete mediation | Every NE command/alias; both ON5 entries; LW1 batch/lifecycle/packet; newly admitted callers | `mediation_matrix::neutral::every_ne_command_and_alias_refuses_onboarding_carriers`, `mediation_matrix::onboarding::every_on_entry_refuses_onboarding_carriers_without_scope_authority` (both `depth_refusal_routes` integration target); `M::onboarding_retained_and_proposed_lower_writer_matrix`, `M::lower::ordinary_lower_writers_deliver_literal_bytes` — prior S11 29-test binary and short matrix runs |
| Complete mediation | Maintenance positives and refusal of onboarding ownership | `D::maintenance_and_onboarding_authorities_cannot_substitute`, `M::maintenance::on2_dry_run_depth_approval_cannot_borrow_a_maintenance_route`, `M::maintenance::ma_intake_and_apply_refuse_onboarding_and_later_version_displacement` — prior S7/S11. Connected maintenance control is listed separately below |
| IN1 and delivery | Wrong/stale parent; moved tip; invalid admission/observation binding; multi-root differing requested root | `I::in1_refuses_old_or_stacked_parent_and_non_tip_observation`, `I::in1_refuses_forged_parent_and_non_descendant_despite_host_ancestry_claim`, `I::in1_refuses_local_tip_mismatch_and_failing_installed_route`, `I::in1_refuses_tip_moved_after_authentication`, `I::in1_per_root_tip_validation_uses_the_routed_registry_and_comparison` — prior FR1; S11 committed foreign-root adapter cases |
| IN1 and delivery | Stale unchanged qualification; entry without effect; effect without entry; exact artifact cannot own unrelated aggregate effects; hand edits | `P::public_on4::public_on4_connected_publication_real_in1_and_observation`, `P::public_on5::public_on5_refusals_local_final_and_conflict_preservation`, `D::outside_generation_effect_and_foreign_entry_refuse_the_entire_batch`, `D::private_publication_and_proving_bind_exact_artifact_and_record` — prior S8 round 1b/S9/S7 |
| IN1 and delivery | Gate failure; changed candidate after local pass; scratch Git/registry/identity changes | `isolated_maintenance::tests::readonly_gates::readonly_gates_two_phases_real_checkers_and_independent_process`, `readonly_gates_closing_observations_refuse_host_and_parent_drift` (same module), `D::readonly_gates::readonly_gates_proposal_runs_every_a_checker` — prior [S10 final gate composition](s10-readonly-gates-report.md), including real independently initialized child and ordinary child refusal |
| IN1 and delivery | Concurrent replacement/removal/mode/link change; preserved recovery bytes and unrelated outputs | `P::public_on5::public_on5_refusals_local_final_and_conflict_preservation` — prior S9 completed five-test run; `workspace_mutation` and `depth_routes` exact race controls listed in S11. ON4 preparation's historical containing test remained running in S8/S11 and is not counted as a completed test here |

The inventory records all seven required groups. Historical S8/S11 unfinished containing
chains retain their stated limits; the completed S9 conflict-preservation suite supplies the
four concurrent restoration cases. The wrapper-only subject cannot prove a shared capability
positive or another agent's onboarding acceptance. Foreign effects are refusal tests only.

## Controls and slice 10 composition

Ordinary onboarding control: `B::public_on1::public_on1_absent_route_keeps_ordinary_success_and_depth_refusal`
actually creates the ordinary non-depth agent, checks its generated registry/backend descriptor,
then adds enrollment and proves refusal. It passed in S10's ten-test public ON1 and 68-test
bootstrap runs. `I::in1_admission_and_observation_of_ordinary_transition` supplies the ordinary
IN1/readback control (FR1).

Connected maintenance control:
`isolated_maintenance::tests::fr2_maintenance::fr2_ma2_freeze_only_complete_tree_canonical_packet_and_real_in1`
and `isolated_maintenance::tests::fr2_maintenance_acquisition::fr2_ma2_combined_acquisition_and_freeze_real_in1`
use actual MA1/MA2 writers, authenticated acquisition, real IN1 and post-tip observation. Their
completed FR2/FR3 runs are prior evidence; they are not rerun here. Socket-dependent MA4→MA5/MA6
chains remain blocked as listed in those reports and are not substituted for completed controls.

Slice 10 independently executed `readonly_gates_two_phases_real_checkers_and_independent_process`
and `D::readonly_gates::readonly_gates_proposal_runs_every_a_checker` (nine passed plus its
explicitly invoked ignored child, and the connected proposal test passed). They execute shared
checking code, distinguish parent-only authority from independent child authority and ordinary
child refusal, and reject admission-relevant committed changes after local success. The new
chain preserves ON4's actual `check_proposed` call and fresh committed `check_shared` calls;
its S/T seam supplies no replacement evidence for that gate-composition test. The full fixed
S/T inventory is asserted against the committed `bootstrap/vectors/on4-publication.json` literal.
ON5 also retains the scoped test clock that pins final `recorded_at` to
`2026-11-14T12:34:56Z`; the prepared human timestamp is `2020-01-01T00:00:00Z` and must differ.
These are fixture observations, not production clock behavior or reusable authority.

## Focused validation

All commands use `env -u TMPDIR`. Logs are ignored artifacts under
`target/s12-validation/`; they are not part of the deliverable.

| Focused command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass after final Rust edits |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass after final Rust edits, 8.28 seconds |
| `cargo test -p xtask --lib e7_chain -- --nocapture` | 4 connected tests passed, 807.95 seconds; see discovery note below |
| `cargo test -p xtask --lib e7_connected_truthful_insufficiency -- --nocapture` | 1 passed, 128.43 seconds |
| `cargo test -p xtask --lib fixture::runtime_follow_on::e7_chain -- --nocapture` | 2 passed, 3.90 seconds |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on2` | 5 passed, 116.97 seconds |
| `make loc-check` | Pass after final Rust edits; every Rust file within 700 code lines |
| `git diff --check` | Pass |

No focused test was blocked by Unix sockets. The historical socket-dependent maintenance
failures stay listed in the S7/S8/S11 and FR reports; they were not rerun or counted as passes.
The lead owns the full gate and declared-registry leg. No `.github/`, Makefile or production
source is modified; all Rust changes are below the existing `#[cfg(test)]` module.

Superseded runs exposed test expectation/fixture issues, not a production change: ON4 names its
missing-obligation owner `onboarding`; ON5 preserves its unsupported-stage outer guard. A
release-watch-from-empty experiment confirmed that the finite ON1 reader requires an already
matching registry declaration. That experiment was reverted. The first execution-only filter
omitted the `fixture` module prefix and matched zero tests; the corrected filter ran both tests
and passed. Zero-test output is not coverage.

## Final invariants and additional boundary

The four connected tests live under `P::e7_chain`. The two missing execution negatives
are mounted as `isolated_maintenance::tests::fixture::runtime_follow_on::e7_chain` from the
same new test directory and reuse the existing smaller declared refusal fixture.
The final-state checker requires eleven due/result
rows, two acceptance entries, exact approval Event/version, verified independent results,
`closed_baseline`/`closed`, every expected successful lifecycle checkpoint, pointers `none`,
`snapshotted` metadata with absent validation outcomes, empty capability declarations and no
runtime backend. Final ON5 also explicitly preserves its original Event/P/O. The repaired sibling
uses public repair and its own real IN1 checkpoints before reselecting actual runtime evidence.
The main freeze now enters through public `runtime_follow_on::run_in_workspace`, rather than
calling the lower fixed-clock helper; the fake authority observes the emitted invocation clocks.

The release-watch sibling retains S9's explicit **pre-freeze authority-setup seam**: after the
ordinary explicit-deferral bootstrap, the fixture commits matching approval/registry/lifecycle
release-watch authority before constructing its separate frozen generation. That setup commit
uses the fixture's protected host installer rather than real IN1. It is not a new-entry
release-watch bootstrap proof. Its ON2→ON5 settlement continuity remains a bounded variant;
the complete new-entry E.7 claim belongs to the explicit-deferral primary chain. Approval
Event fields and exact version remain the same; approval artifact bytes differ between the
separately frozen siblings. The finite ON1 adapter's matching-registry restriction is unchanged.

The independent substantive producer is `SyntheticHost::onboarding`: a test-owned source closure,
full-schema witness/artifact population and approval-linked adoption observations served through
the fake authenticated host. Its case outcomes and host terminality are modeled; it is not a
live/native producer job. The actual ON3 assembler/evaluator consumes those independent
observations, and every writer supplies its own derived record/files. ON2's offline executor
copies the B.1 wrapper source and tests; its controller validates substantive source/test presence
and boundaries, but this chain does not itself execute the full wrapper test suite. Those are
additional explicit fixture seams, consistent with the separately required phase C run.

Final-run result: `e7_connected_onboarding_qualified_chain`,
`e7_connected_onboarding_repaired_qualified_chain`,
`e7_connected_onboarding_release_watch_chain` and
`e7_connected_truthful_insufficiency_refuses_on4_and_both_on5_entries` all passed.
The connected run used the final connected sources; its invocation began before the two
execution-only tests were mounted. Those two tests passed separately on final sources. Current
`e7_chain` discovery lists six tests, all covered by the successful connected and execution runs.
The round 1 section below records subsequent execution-test changes and focused reruns.

The insufficiency sibling models one genuine admitted reporting checkpoint and zero substantive
producer executions. Public ON3 writes its actual empty reporting E; all eleven independently
evaluated due obligations remain `unverified`, the promise stays unqualified and entries stay
empty. ON4 refuses with `onboarding is missing a due depth obligation` naming `R1`, operation
`run`, target `x86_64-unknown-linux-gnu`. Prepared and final ON5 each retain their existing
`unsupported depth write (declared enrollment)` guard because no published predecessor exists.
Each refusal separately preserves the complete caller tree. The sibling does not invent a
published predecessor to reach ON5's deeper predicate; S7's all-due publication/proving matrix
separately proves the evidence requirement in both owning predicates.

## Review round 1 fixes

The deletion regression iterates the eight committed OF-P4 statement literals individually,
including all three coverage reports. The statement and frozen O vector must each equal the
nonempty literal inventory, including its cardinality. Each deletion exercises the isolated
prepare-publication, publication-refresh and close-proving-run entry bodies and requires
`changed or missing frozen acquired inputs; publication refused: <literal path>` while the
complete caller snapshot stays equal. The existing per-input mutation regression now uses the
same independently reviewed inventory. These tests use the small declared FR2 refusal fixture.
The public entrypoints' declared-depth guard precedes this precheck, so public dispatch has a
deletion-specific diagnostic coverage gap. The coverage map records that boundary.

The execution refusal helper captures the full caller tree and raw lifecycle binding before
execution. Only the following lifecycle attempt keys may differ, with their expected values
asserted individually and the timestamp bounded to the invocation.

- `current_owner_command` becomes `runtime-follow-on --write`.
- `last_transition_by` becomes `xtask runtime-follow-on --write`.
- `last_transition_at` records the invocation time.
- `active_runtime_evidence_run_id` stays null.
- `side_states` becomes `["failed_retryable"]`.
- `blocking_issues` stays empty.
- `retryable_failures` becomes `["write attempt freeze started; consult run diagnostics"]`.

All remaining lifecycle keys must equal their captured values, and the parsed raw JSON bytes
of `support_depth` must match exactly. The immutable run files are `input-contract.json`,
`codex-prompt.md` and `handoff.json`. The eight permitted diagnostic outputs are
`validation-report.json`, `written-paths.json`, `run-status.json`, `run-summary.md`,
`recovery-candidate.txt`, `codex-execution.json`, `codex-stdout.log` and `codex-stderr.log`.
Assertions require failure status/errors, empty written paths, false handoff readiness, the
expected exit code and captured streams, a failure summary and an existing retained candidate.
That candidate must contain substantive source and its successful executor-authored handoff.
The caller's original pending handoff must remain byte-identical. Every remaining caller node,
including run-directory entries and modes, participates in the full-tree comparison.
Trusted output files must retain their captured modes, with new diagnostics at mode `0644`.

The execution module mount now has `#[cfg(unix)]`. The other new E.7 mount is enclosed by
`public_on3.rs`'s existing Linux module gate. The platform-gating regression is
`isolated_maintenance::tests::fixture::runtime_follow_on::e7_executor_module_mounts_are_platform_gated`.

### Round 1 focused validation

All commands used `env -u TMPDIR`. Scratch logs are under ignored
`target/s12-review-round1/`.

| Focused command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass after all temporary mutations were restored |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass on final sources, 7.07 seconds |
| `cargo test -p xtask --lib fixture::runtime_follow_on::e7_chain -- --nocapture` | 2 passed on final sources, 2.73 seconds |
| `cargo test -p xtask --lib publication_entrypoints_refuse_each_ -- --nocapture` | 2 passed on final sources, 6.22 seconds, including all eight deletions across three entry bodies |
| `cargo test -p xtask --lib e7_executor_module_mounts_are_platform_gated -- --nocapture` | 1 passed |
| `make loc-check` | Pass |
| `git diff --check` | Pass |

A supplementary `cargo check -p xtask --lib --tests --target x86_64-pc-windows-gnu`
failed in existing Unix-only tests outside this packet, including
`src/support_matrix/depth/tests/generator.rs` and integration test files under
`tests/c0_spec_validate`, `tests/depth_record` and `tests/agent_maintenance_closeout`.
The new E.7 execution module produced no Windows diagnostic. Clean non-Unix test compilation
remains unverified because of those existing failures. The source-mount regression independently
checks both new E.7 include sites' platform gates.

### Regression sensitivity

Temporary test-harness edits changed disposable fixture state or an in-memory source copy.
Each targeted test failed with its intended assertion, and each temporary edit was restored.

| Disposable mutation | Observed failing regression |
| --- | --- |
| Restore and commit the isolated deleted `coverage.all.json` before the entry call | `publication_entrypoints_refuse_each_deleted_frozen_acquired_input_without_writes` rejects the missing complete path-specific acquisition refusal |
| Copy the retained candidate's successful `handoff.json` into the caller after interruption | `e7_interrupted_offline_executor_retains_only_trusted_failure_diagnostics` fails at `immutable run file handoff.json` |
| Clear the caller lifecycle `support_depth` after interruption | The same interruption regression fails its captured lifecycle equality |
| Compact the caller's `support_depth` JSON while preserving its parsed value | The same interruption regression fails its raw-byte equality |
| Remove the Unix gate from the in-memory module-mount source | `e7_executor_module_mounts_are_platform_gated` fails its gate assertion |

A separate disposable `rustc --target x86_64-pc-windows-gnu --crate-type lib` fixture copied
the actual execution-module mount and substituted its absolute source path. The gated fixture
compiled successfully. Removing that gate from the disposable copy failed compilation on the
Unix-only `public_on2::executor` import. This isolates the mount behavior from the unrelated
repository-wide Windows test failures.

Final focused checks ran after all temporary mutations were restored. The fix remains
uncommitted in the detached worktree at `95bc9706` for lead review.
