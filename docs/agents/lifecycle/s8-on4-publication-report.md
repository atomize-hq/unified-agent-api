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

## Review round 1 fixes

IN1 now calls the same packet-pinned runtime-evidence validator as local ON4 against the
committed candidate's clean workspace. The shared validator checks the frozen depth input
contract, lifecycle `written_paths_sha256` and complete runtime bundle, including the green
validation report. Two committed-tampering siblings traverse real IN1 and require the specific
written-paths digest mismatch and non-green validation-report refusals. Previously IN1 admitted
both candidates because its Publication dispatch bypassed ON3's runtime validation.

Publication dispatch now distinguishes retained historical entries from newly added entries,
new acceptance effects and remaining qualifications. Remaining qualifications still receive
fresh ON4 acceptance, including unchanged qualifications. A published reporting-only candidate
preserves Event/P/O, entries, lifecycle assertion, packet, debt claims and inventory, acquired
inputs and aggregate output ownership. A connected downgrade changes E to `unverified`, removes
its evidence selections, regenerates truthful publication and passes real IN1 while retaining
its historical Publication entry. The previous dispatcher refused this candidate during full
ON4 evidence acceptance. Siblings retaining a qualification or appending a current Publication
entry require the specific independently-unsatisfied-obligation refusal.

The connected `refresh-publication --write` preparation test now corrupts support JSON only
inside the existing proposal A-check fault seam. It requires the support checker's stale-output
failure and an unchanged complete caller tree. Removing the controller's `check_proposed` call
leaves this corruption unapplied, allowing refresh to succeed and failing the refusal assertion.
The downstream subprocess failure test remains covered separately.

Four connected rollback siblings inject replacement, removal, executable-mode and symlink
changes after two actual ON4 output writes, then fail before the next write. Each verifies the
concurrent state, full restoration of every other caller path and retained original bytes in the
writer's recovery directory. The test restores its own concurrent edit and recovery files between
siblings. These controls fail if rollback overwrites the changed output, loses its original
recovery bytes, skips restoration of another applied output or never reaches partial delivery.

Validation is limited to the packet's focused commands. The fixtures use their captured
registry and genuine ON1/ON2/ON3 chain, independent of whether the build workspace has a declared
enrollment. The first combined public ON4 run exposed a fixture loader that tried to compare the
retained packet's publication-ready predecessor hash against the current published lifecycle.
The tampering fixture now parses and validates the retained packet directly. The downgrade
fixture also commits its E change before invoking standalone readers that require a clean
candidate. Corrected IN1 and continuing preparation results are tracked below.

| Focused command | Review round 1 result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed after final Rust edits |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Passed after final Rust edits |
| `cargo test -p xtask --lib depth_acceptance::integration` | Passed command, zero matching tests |
| `cargo test -p xtask --lib depth_acceptance::onboarding -- --test-threads=2` | Passed, 17 tests |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on3::public_on4 -- --nocapture --test-threads=2` | Production refusal passed; preparation running; integration fixture failure corrected |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on3::public_on4::public_on4_connected_publication_real_in1_and_observation -- --nocapture --test-threads=1` | Corrected IN1 rerun running |
| `make loc-check` | Passed, every Rust file within 700 code lines |

At the 60-minute work limit, the two public ON4 processes remain active. The corrected IN1
rerun has passed committed written-paths tampering and entered validation-report tampering.
The preparation run has passed the proposal A-check refusal, existing preparation controls,
and replacement/removal rollback checks, and entered the mode-change check. The downgrade,
remaining rollback cases and final aggregate outcomes remain unverified. The continuing logs
are `target/s8-validation/s8fix-public-on4-rerun.log` and
`target/s8-validation/s8fix-public-on4-integration-final.log`. The former includes the superseded
packet-loader fixture failure from its older integration binary. An initial open log was moved
across filesystems and lost later output, requiring a rerun. All changes remain uncommitted.

### Refusal ordering

Round 1b mirrors slice 6's write-only unsupported-stage guard in the routed ON4 context loader.
Stages outside `publication_ready` reach the existing `unsupported depth write` refusal before
the shared context's stage-specific checks. Supported `publication_ready` candidates continue
through every existing slice 8 check. The authoritative acquired-inputs regression was reproduced
before the fix and passes unchanged afterward. All nine acquired-inputs tests, workspace Clippy,
formatting and `make loc-check` pass. At the round 1b handoff, the requested public ON4 filter has
passed its production refusal and connected IN1/observation tests. Its connected preparation test
remains running, with its final outcome unverified. The round-1 fixes are preserved and changes
are uncommitted.
