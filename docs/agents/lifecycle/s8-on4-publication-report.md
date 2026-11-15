# Slice 8 ON4 publication and IN1 authority

## Status

Implementation in progress on `feat/s8-on4-publication`. Changes are uncommitted for the lead.
The task has a 60-minute budget. Focused validation follows implementation.

## Intended delivery

- Route public ON4 through captured onboarding authority, exact packet finalization, coherent
  Publication entry, fresh post-entry evaluation and aggregate admission.
- Run the slice-10 proposal gates before expected-before local delivery.
- Dispatch IN1 onboarding Publication and unchanged qualifications through the slice-7 predicate.
- Extend the genuine ON3 chain with ON4, real IN1 and protected-tip observation.
- Update Annex A and record focused validation, including sandbox socket failures.

## Progress

Read the routing decisions and slice-6/7/10 reports. Confirmed the requested branch and clean
initial worktree. The existing private predicate requires exact committed packet bytes, and the
ON3 packet already binds its publication-ready predecessor. The controller will preserve that
authority while checking packet freshness before constructing its entry-bearing proposal.

The public ON4 route now captures a proposal before reading context, validates the committed
approval and packet freshness, constructs its published assertion and exact Publication entry in
scratch, freshly evaluates and renders, and invokes the real proposal A gates. S/T gate execution
is extracted into a focused module. The complete proposal is evaluated again after gates and
delivered through the shared expected-before mechanical writer. IN1 has a separate onboarding
dispatch module and retains the maintenance closeout/promotion branch.

The first all-target Clippy run exposed a Rust visibility error in the effect-checker re-export.
The visibility was corrected and the second run passed. The first connected public ON4 proof is
running from the actual ON1/acquisition/ON2/ON3 chain. Its subprocess gate seam checks a committed
literal command inventory while the proposal A gates execute their real checking code.

Added full-tree refusal assertions around injected gate failure, removed current entries,
another agent's capability/pointer destination, partial mechanical-delivery failure and a gate
failure concurrent with a caller lifecycle edit. IN1 refusal siblings independently commit
packet/entry tampering and own/foreign pointer edits. The connected positive also checks that a
pre-entry verdict becomes unverified for the entry-bearing record, measures protected source
closure, and runs fresh committed checks before modeled protected movement and after observation.

The ordinary context loader was extracted into `publication_refresh/context.rs`. The routed
controller retains the normal production loader and root-resolution behavior through that shared
validation seam. A private Python S/T runner reuses slice 10's environment/cache setup and builds
the independent candidate-registry test reader. The focused proof replaces the expensive S/T
executor with its test seam. Its actual full subprocess execution belongs to the lead's gate.

Annex A's ON4, IN1, output and proving-reference rows now describe the Slice 8 composition.

The initial connected writer/admission/observation proof passed in 639.87 seconds. The expanded
refusal chain is running. Dedicated production/maintenance/neutral-writer refusal controls and
the S/T subprocess-environment/independent-reader test passed. The tooling checkout now stages
the captured caller's tracked inventory plus ON4 outputs explicitly, preserving ignored tracked
artifacts while leaving unrelated scratch files outside the index. A focused real-Git test
covers that inventory boundary and passed. Capability rendering in the depth-only caller uses the bound
scratch root. The shared production planner retains its existing root resolution.

## Validation

Focused validation is running. The lead owns the full gate and declared-registry test leg.

| Command | Result |
| --- | --- |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Passed, including final controller/tooling changes |
| `cargo test -p xtask --lib depth_acceptance::publication` | 1 passed |
| `cargo test -p xtask --lib depth_acceptance::onboarding -- --test-threads=1` | 17 passed in 849.23 seconds |
| `cargo test -p xtask --lib public_on4 -- --nocapture --test-threads=2` | Running |
| `cargo test -p xtask --lib isolated_maintenance::tests::ma5_closeout -- --test-threads=1` | 3 passed, 7 socket-blocked |
| `cargo test -p xtask --lib isolated_maintenance::tests::ma6_promotion -- --test-threads=1` | 5 passed, 3 socket-blocked |
| `cargo fmt --all -- --check` | Passed |
| `make loc-check` | Passed, every Rust file within 700 code lines |

## Sandbox-blocked maintenance tests

All ten failures stop when opening the MA4 evidence channel with
`Operation not permitted (os error 1)`.

Under `isolated_maintenance::tests::ma5_closeout`

- `ma5_closeout_plan_and_prepare_fetch_refuse_captured_route_drift`
- `ma5_closeout_writer_completes_connected_chain`
- `ma5_prepare_writer_completes_connected_chain_from_isolated_preflight`
- `ma5_writers_keep_production_depth_refusal_without_a_route`
- `ma5_writers_reauthenticate_unverified_population_despite_verified_e_labels`
- `ma5_writers_refuse_missing_or_wrong_generation_marker`
- `ma5_writers_refuse_wrong_generation_policy_or_obligations_without_writes`

Under `isolated_maintenance::tests::ma6_promotion::connected`

- `ma6_a5_replay_first_promotion_uses_retained_closeout_after_ordinary_later_generation`
- `ma6_apply_refuses_captured_route_drift_and_production_cannot_admit_depth`
- `ma6_positive_promotion_after_each_ma5_writer_with_in1_admission_and_observation`


## Review progress

Reviewed the ON4 proposal and IN1 path dispatch against OR-P3/4/8/9/13 and the charter split.
The unchanged-qualification regression now creates a distinct committed descendant before
requesting admission against its protected parent, then revokes adoption and repeats that real
admission. This preserves IN1's authenticated parent/candidate distinction. A final connected
run includes that correction and all four new test cases. The initial shorter connected positive
already passed. Full subprocess inventory and depth-declared workspace validation stay assigned
to the lead.


The first expanded refusal run reached the concurrent caller edit and returned the existing
specific clean-tree guard, `isolated caller has uncommitted or untracked local changes`, before
the proposal snapshot check. Its test expectation now pins that authoritative refusal. The
controller preserves the caller edit. The complete rerun includes the corrected expectation.


The full requested onboarding predicate filter passed all 17 tests, including the actual ON3
checkpoint's private post-entry evaluation, all proposal A checks and private proving controls.
The neutral/maintenance controls, S/T environment test and real tooling-index test each passed.


The corrected expanded run passed every preparation assertion, including partial-delivery
restoration and preservation of the concurrent caller edit. Its first IN1 packet-tampering case
then hit the lifecycle object's byte-preservation guard because the fixture serialized that
object through a generic JSON value. The fixture now edits a typed `LifecycleState`, preserving
its depth object exactly. The controller and predicate refusal ordering were retained.
Preparation refusals and publication/IN1 assertions now have separate connected tests sharing the
same genuine ON2 template. The final `public_on4` filter runs both phases with two test threads
so all required cases can finish inside the packet's 60-minute budget.

Validation logs are ignored artifacts under `target/s8-validation/`. Cargo uses the default
temporary root through `env -u TMPDIR`. The full subprocess inventory and declared-registry leg
remain assigned to the lead. 
