# Slice 7 onboarding owning predicates

## Status

Private implementation and required focused validation are complete on `feat/s7-onboarding-predicates`, subject to the ten maintenance writer socket blockers listed below. Changes remain uncommitted for the lead. Public ON4/ON5 accepting writes and production activation remain refused. Phase C remains closed.

## Scope

Extract path-neutral adequacy from maintenance admission while preserving maintenance refusal ordering and behavior. Add private onboarding publication and proving predicates with exact approval Event, lifecycle, policy, acquisition and candidate-artifact binding. Enforce a debt-free boundary and owned-generation effect allowlists. Provide read-only coherent-entry and fresh post-entry evaluation for later slices.

## Progress

- Read the routing decisions, existing maintenance predicate, onboarding freeze derivation and IN1 effect dispatch.
- Confirmed the requested branch and initially clean worktree.
- Extracted shared currentness, due-work and evidence adequacy into `depth_acceptance/adequacy.rs`. Maintenance retains its existing debt planner between currentness/work and evidence checks, with its original refusal strings.
- Added private onboarding binding, artifact entry derivation, aggregate effect mediation and proposed-view evaluation modules. Onboarding currentness calls the shared policy resolver and O acquisition derivation with the frozen baseline. It performs no freeze.
- Private predicates read approval, registry, frozen Event/P/O and retained lifecycle fields from a pinned committed source. Publication binds the exact packet and published assertion. Proving binds the exact closed artifact, closed_baseline and proving evidence id.
- Added socket-free private predicate/refusal fixtures and an extension at the connected slice-6 ON3 checkpoint for fresh post-entry evaluation. Focused validation results are recorded below.

## Wiring boundary

`onboarding::evaluate_proposed` borrows a captured isolated proposal/view, freshly authenticates its complete population after the coherent entry, checks aggregate effects and opens a fresh rendering scope. It returns read-only results. Public ON4/ON5 writers retain their existing refusal guards. IN1's existing blanket onboarding-effect refusal and unsupported-new-entry refusal remain active. No onboarding accepting dispatch is installed in IN1.

## Validation

The requested focused depth-acceptance group passes, including unchanged maintenance closeout/promotion predicate suites. The connected private-predicate ON3 extension passed with actual public ON3 output, real IN1 admission and post-tip observation. Maintenance writer socket blockers are listed below. Commands use `env -u TMPDIR` for Cargo because the configured temporary-directory override is unavailable in this sandbox.

| Command | Result |
| --- | --- |
| `make fmt-check` | Passed |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Passed |
| `cargo test -p xtask --lib depth_acceptance` | 104 passed, including all 89 preexisting tests and 15 new tests |
| `cargo test -p xtask --lib depth_acceptance::onboarding` | 15 passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::ma5_closeout` | 3 passed, 7 socket-blocked |
| `cargo test -p xtask --lib isolated_maintenance::tests::ma6_promotion` | 5 passed, 3 socket-blocked |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on3::private_onboarding_predicates_from_connected_on3 -- --nocapture` | 1 passed, 367.50 seconds |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on3::public_on3_connected_active_reporting_e_and_real_in1 -- --nocapture` | Corrected full active run passed, 968.12 seconds |
| `make loc-check` | Passed, 700-code-line cap |

## Implementation details

- The aggregate onboarding checker invokes raw detectors before selection filtering, so an outside-generation pointer cannot disappear from the accepting batch. It checks every registered root and refuses foreign entries and record changes.
- `coherent_entry` constructs an entry-bearing record in memory. `evaluate_proposed` requires that exact current entry, evaluates the complete population freshly, checks aggregate effects and borrows the same bound rendering scope. Neither helper delivers outputs.
- Public ON4/ON5 and IN1 onboarding acceptance wiring is deliberately deferred to slices 8–9. Existing production writers and refusal ordering are preserved.
- The first depth-acceptance run passed all 89 existing tests. The eight initial new fixtures failed before predicate evaluation due to lifecycle-template and canonical result-order setup, which were corrected before the passing runs.

## Maintenance writer regression attempts

The unchanged focused `ma5_closeout` module ran 10 tests with 3 passing and 7 blocked by the sandbox. The unchanged `ma6_promotion` module ran 8 tests with 5 passing and 3 blocked. Every blocked test stopped while creating the MA4 evidence channel with `Operation not permitted (os error 1)`.

Blocked MA5 tests, relative to `isolated_maintenance::tests::ma5_closeout` are

- `ma5_closeout_plan_and_prepare_fetch_refuse_captured_route_drift`
- `ma5_closeout_writer_completes_connected_chain`
- `ma5_prepare_writer_completes_connected_chain_from_isolated_preflight`
- `ma5_writers_keep_production_depth_refusal_without_a_route`
- `ma5_writers_reauthenticate_unverified_population_despite_verified_e_labels`
- `ma5_writers_refuse_missing_or_wrong_generation_marker`
- `ma5_writers_refuse_wrong_generation_policy_or_obligations_without_writes`

Blocked MA6 tests, relative to `isolated_maintenance::tests::ma6_promotion::connected` are

- `ma6_a5_replay_first_promotion_uses_retained_closeout_after_ordinary_later_generation`
- `ma6_apply_refuses_captured_route_drift_and_production_cannot_admit_depth`
- `ma6_positive_promotion_after_each_ma5_writer_with_in1_admission_and_observation`

## Focused validation progress

The corrected onboarding group passed 14 tests before the final metadata-destination extension. All eleven due obligations are independently removed for publication and proving, with literal id/debug-name expectations. Work/classification cases now pass through the full owning predicate. Debt cases cover grants, retained baseline debt, retirement, inventory drift and nonempty current inventory. Additional cases cover exact closeout changes, prepared state, approval mismatch, lifecycle retargeting, drift clearance, frozen-object byte preservation and continued public IN1 refusal.

Review found two aggregate cases that positive-effect detection alone does not enumerate. The onboarding checker now preserves pointer files including min_supported, and uses an extracted record-neutral metadata detector to inspect versions lacking a depth record. Ordinary maintenance comparison keeps its original record-scoped population. The connected actual ON3 proof passed through the new focused private-predicate test.


## Deliverables and proof boundaries

- `crates/xtask/src/depth_acceptance/adequacy.rs` owns common P/O/E, currentness, classification/carried-work and all-due evidence adequacy. The maintenance caller keeps its debt plan and refusal ordering between those phases.
- `crates/xtask/src/depth_acceptance/onboarding/` owns exact approval/lifecycle/artifact binding, Publication and ProvingRunCloseout derivation, aggregate effect mediation, coherent-entry construction and fresh proposed-view evaluation. The private adapter uses the existing captured registry/view/evidence interfaces and immutable committed HEAD authority.
- `crates/xtask/src/support_depth/onboarding.rs` reuses policy resolution and acquisition derivation against authenticated inputs, retaining frozen P. It rejects nonempty baseline/current inventory, and the owning predicate rejects grants, retirements and full inventory-byte drift.
- `depth_acceptance/manifest.rs::detect_versions` is a focused extraction. Maintenance retains its record-scoped population. Onboarding supplies metadata destinations outside that population to prevent an accepting status or passed-target outcome without a record from escaping the batch.
- The new socket-free fixtures use an explicitly private evidence constructor to test predicate and exact-refusal behavior. The connected extension consumes actual slice-6 ON3 output and its independently authenticated modeled-host population for publication and proving. Its proving predecessor is the private publication predicate's in-memory output, since public ON4/ON5 acceptance remains refused. The focused connected test passed, including fresh complete-population evaluation for both exact entry-bearing records. The publication bundle is rendered, materialized only in scratch and independently re-evaluated with actual effect/current-publication checks.
- The fixed wrapper-only subject proves no runtime-agent capability-positive behavior. Public ON4/ON5 composition and IN1 onboarding owning dispatch remain slices 8–9. Production activation and real host/emitter/queue evidence remain phase C.

All source and report changes remain uncommitted on the requested branch for the lead. No push was performed.


## Connected proof outcome

`isolated_maintenance::tests::bootstrap::public_on3::private_onboarding_predicates_from_connected_on3`
passed in 367.50 seconds. It reuses the slice-6 actual chain through public ON1, acquired inputs,
declaration, actual ON2 freeze/implementation and actual ON3. Real IN1 precedes each modeled
protected-tip advance and post-tip observation. The focused slice-7 leg preserves all positive
ON3 assertions. Existing active/repaired ON3 tests retain their own refusal matrices.

At the actual admitted ON3 checkpoint, the test executes public refresh-publication and
prepare-proving-run-closeout and pins their exact unsupported-depth refusals with a complete
unchanged caller tree. In a bound scratch view it constructs the exact Publication entry, freshly
evaluates every obligation, renders a qualified promise, materializes the bundle in scratch and
re-admits its effects/current bytes. It then uses that private publication output as an in-memory
proving predecessor, constructs the exact closed artifact/closed_baseline/evidence state and
ProvingRunCloseout entry, re-derives current P/O and freshly evaluates the exact post-entry record
against the actual modeled-host population. The complete caller tree remains unchanged.

An earlier full active ON3 run passed in 813.15 seconds. An interim connected build exposed that
committed directory inventories include ancestor directories, which the pointer-preservation loop
attempted to read as blobs. The checker now selects canonical `.txt` pointer files and `.toml`
markers before regular-byte reads. The corrected connected private-predicate test and all 104
focused depth-acceptance tests pass. Selected-file read errors continue to propagate.


The corrected full active ON3 run also passed in 968.12 seconds, covering the original active
reporting/refusal matrices and the private predicate extension present in that test build.
The final source provides the focused private-predicate test separately while preserving the
original active/repaired refusal matrices. All requested validation is finished. The remaining
unverified maintenance writer cases are exactly the ten socket-blocked tests listed above.
All 41 unchanged maintenance closeout/promotion predicate regressions pass in the 104-test group.
