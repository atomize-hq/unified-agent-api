# Slice 10a authenticated read-only gates

## Status

Implementation and requested focused validation are complete on `feat/s10-readonly-gates`. Changes remain uncommitted for the lead. Phase C remains closed.

## Scope

Implement OR-P13 proposal-phase shared checks and independently routed committed-candidate readers. Expose the proposal entry point for slice 8. Preserve ordinary parsing, generation and neutral write refusals. Phase C remains closed.

## Progress

- Confirmed the requested branch and clean starting worktree.
- Read the fixed-command inventory, production reader inventory and routing decisions OR-P3, OR-P12 and OR-P13.
- Reused slice 7's exact-record borrowed evaluation scope and the commands' real shared check entry points.

## Slice 10b handoff

The inventoried 80 test/helper registry-reader sites and ten depth-absence assertions remain assigned to slice 10b. The full preflight test leg over a declared registry remains required there. Slice 8 owns ON4 wiring and execution of the S/T inventory leaves. This slice supplies the A checks.

## Validation

All requested focused commands passed, including the final composition and connected proposal runs. Results are recorded below. Socket-blocked tests are none.

## Implementation checkpoint

- Added `readonly_gates::check_proposed` for slice 8. It calls slice 7's real fresh owning evaluation, compares the exact expected record including entries, then runs the shared A checks inside that borrowed scope.
- Added an explicit committed-candidate evaluation boundary for standalone support checking. It captures its own installed route and performs fresh read-only population evaluation with closing observations. It emits no IN1 decision.
- Extracted shared support and capability check-mode entry points and the validator's read-only adapters. Capability inventory/audit already use routed readers.
- Routed both runtime-advertising check readers while preserving Source regular-file checks, registry equality, revision/data conformance and the production parser fallback. Generation keeps its guarded production path.
- Added credential-free composition coverage using real MA5-generated proposed publication bytes, a non-Git scratch view, committed fixture checks and a test-owned subprocess. The child initially verifies that no profile or verdict scope arrived, then installs independently supplied fake host observations and executes real command entry points.
- Added exact-record, changed receipt, harmless candidate change, production parsing, ordinary-generation, neutral write, host/transport/source/parent drift and unchanged-tree assertions. Validation is pending.


## Boundaries for integration

The unchanged command inventory remains in `agent_lifecycle.rs` and the Makefile. Standalone
support checking uses fresh read-only committed evaluation without emitting an IN1 admission.
Capability check/audit and runtime-advertising check independently resolve and close their own
registry route. Ordinary manifest validation without `--tip` retains the existing production-verdict
control. The proposal adapter runs its real check-mode validator inside the borrowed evaluation
for each validator-equipped or depth-bearing registered root. Existing `--tip` and all-root IN1 own committed admission.
Slice 8 must run `check_proposed` while the captured scratch registry guard is held, supply the
exact coherent post-entry record, account for every S/T leaf and re-observe frozen and
expected-before inputs before checked delivery. An error returns no delivery authority.

The process test transports only test-owned fake host input observations on stdin. The child
first proves that the parent installed no profile or verdict in it, then installs its own fake
source authority. No production transport, environment loader or reusable verdict is added.

## Focused validation checkpoint

`make fmt-check` and `make loc-check` passed. The 700-code-line cap is satisfied after extracting
the validator adapters into a focused module. Clippy and the new composition test group are
running with `TMPDIR` unset because the configured override is unavailable in this sandbox.

The initial all-target Clippy run found that the existing capability parity harness embeds the
source module and therefore cannot resolve a new crate-private helper. The capability check now
uses the library's existing public route/recheck methods directly. The harness and its registry
fixtures retain their current assertions and remain assigned to slice 10b where inventoried.

The requested library filters passed with 16 runtime-advertising tests, 39 support-matrix tests
and five capability tests. The corrected all-target/all-feature Clippy run passed. The first new
composition run exposed an empty fixture commit after canonical capability output was already
current. Setup now commits only an actual fixture change, and the composition group is rerunning.
The connected exact-record ON3 proposal extension is also running.


The first composed validator run identified a scope error in the new adapter. Gemini CLI and
aider intentionally commit no RULES/schema prerequisites, as their validator specs state. The
adapter now runs full manifest validation for registered roots with validator inputs or declared/
retained depth. Any partial prerequisite, malformed/nonregular read or depth-bearing root still
runs the full checker and retains its specific refusal. Schema-free roots' existing spec/pointer
assertions remain T leaves owned by slice 10b. No missing input is synthesized for those roots.

The corrected two-phase composition passed all three active tests in 220.09 seconds. Its
independent process entry is marked ignored for ordinary discovery and is explicitly launched
and asserted by the passing composition test. No socket was needed. Additional substantive
capability staleness, audit orthogonality, manifest-schema and runtime-data refusal cases have
been added inside the borrowed scope and are running in the final composition rerun.

The initial connected ON3 run used the earlier all-root adapter binary and reached that known
schema-free-root error after 409.69 seconds. It is rerunning with the corrected applicable-root
selection. The ON4 proposal entry point's connected positive remains pending that rerun.

## Reader and API handoff

- `support_matrix::check_in_workspace` is the standalone committed read-only entry. Its private
  `check_publication` is the shared checker used inside a borrowed proposal scope.
- Capability check and audit have standalone `check_in_workspace` entries. Their underlying
  inventory/render/audit functions remain available to owning callers without implicit evidence
  evaluation. The capability module uses the public route directly because existing parity
  integration tests embed that source module.
- Runtime check mode routes preparation and `Revision::workspace` through the same authenticated
  source/registry equality helper. `Revision::load`, immutable `tip` loading and ordinary generation
  retain their production parsing boundary. Slice 10b still owns its inventoried runtime fixture
  production-parser conversion.
- `readonly_gates::check_proposed` is crate-private for the future ON4 controller. It accepts the
  caller, captured proposal/registry, scratch view and exact expected record. It returns only
  success or a refusal. Every fixed S/T leaf still needs the controller's separate execution.
- The fixed top-level commands remain support check, capability check, capability audit and
  preflight. All-root IN1 and explicit per-root `--tip` remain separate admission consumers.


## Fixed inventory accounting

| Inventory boundary | Proposal phase in 10a | Committed-candidate phase in 10a |
| --- | --- | --- |
| Outer support check and repeated preflight support leaf | Real shared support checker in the borrowed fresh scope | Actual support command entry in the independently initialized child, with fresh read-only route/evidence evaluation |
| Outer capability check and repeated preflight capability leaf | Real inventory/render/staleness checker in the borrowed scope | Actual capability command entry with its own routed registry boundary |
| Outer capability audit and repeated preflight audit leaf | Real inventory/orthogonality checker in the borrowed scope | The standalone audit's shared entry with its own routed registry boundary |
| Preflight validator consumers | Real check-mode validator on each validator-equipped/depth-bearing root in the borrowed scope | Same validator code in a newly authenticated committed evaluation. Existing per-root `--tip` and all-root IN1 admission remain separate |
| Preflight runtime-advertising check | Real preparation, revision/data, feature and source-conformance checks, with both registry readers routed | Actual runtime-advertising command entry, resolving and closing its own route |
| S leaves in Inspection B | Assigned to the future ON4 controller. This packet's focused format/Clippy/LOC checks validate the implementation checkout | Every unchanged standalone S leaf remains required on the actual integration candidate |
| T leaves in Inspection B | The full test subprocess and reader conversions remain slice 10b. This packet executes the focused test filters and composition | The full declared-candidate preflight test leg remains slice 10b. Fake authority is installed independently in test-owned children |

Inspection B's 23 recipe/check leaves, repeated outer commands and repeated security setup remain
the authoritative complete inventory. This packet claims A-code composition and focused code
validation. It supplies no complete preflight, native queue or protected-host success evidence.

`check_shared` explicitly executes the outer support/capability/audit checks and repeats those
A leaves in the preflight portion, retaining the fixed inventory's repetition. The preflight
portion also runs the applicable validators and runtime-advertising check. The controller still
owns the actual S/T subprocess execution, including the complete test leg.

The expanded shared-check refusal matrix passed in the second complete composition run
(249.88 seconds). Final literal production refusals, host-read-only assertions and protected-parent
drift coverage are included in the final rerun. Final format, Clippy, LOC and all three requested
library filters passed again.

Final review pinned validator prerequisite selection to the authenticated committed source as
well as the proposed/current source. Removing all three schema prerequisites from scratch now
still invokes the required validator. The new regression removes all three from the codex root,
asserts the root-specific missing-input refusal and restores the test-owned scratch inputs.
The adapter borrows the existing GitInputs provenance and verification path and propagates read
failures. Schema-free roots that are schema-free in both sources retain their existing T-leaf
spec coverage.

The corrected connected proposal positive and exact-record/capability refusals reached the
runtime mutation case. That mutation is an authenticated source input, so the existing GitInputs
working-versus-index guard refuses it before runtime-data conformance. The test now pins the
specific `working bytes disagree with index: crates/agent_api/src/runtime_advertising/data.rs`
reason. The shared-check composition separately exercises the runtime-data staleness refusal
inside an already evaluated scope. The GitInputs guard is preserved.

A final overlapping Cargo rebuild replaced the running test executable's pathname before the
composition launched its child. The Linux-only process harness now uses the procfs handle for its running executable,
which pins the running image's inode through replacement. The child still starts without a
profile and independently installs fake input authority. This is a test-harness repair, and the
composition is rerunning with the new prerequisite-removal regression.

## Final focused results

All Cargo commands below use `TMPDIR` unset.

| Command | Result |
| --- | --- |
| `make fmt-check` | Passed |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Passed |
| `cargo test -p xtask --lib runtime_advertising` | 16 passed |
| `cargo test -p xtask --lib support_matrix` | 39 passed |
| `cargo test -p xtask --lib capability` | 5 passed |
| `cargo test -p xtask --lib readonly_gates -- --nocapture` | 3 passed in 151.40 seconds. The discovery-ignored process entry was launched and asserted by the composition test |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on3::private_onboarding_predicates_from_connected_on3 -- --nocapture` | 1 passed in 717.63 seconds |
| `make loc-check` | Passed, 700-code-line cap |

Socket-blocked tests are none. The composition uses the existing socket-free assembly
companion and still executes real publication, checking, routing and evaluation code.


## Final proof coverage and handoff

| Test | Executed boundary |
| --- | --- |
| `readonly_gates_two_phases_real_checkers_and_independent_process` | Real MA5-generated post-entry bytes checked in a borrowed non-Git proposal view, then through independently routed committed check entries and a test-owned child. Exact entry-bearing record sealing, validator prerequisite deletion, substantive A-check refusals, a harmless committed change and a changed receipt requiring fresh refusal. Caller bytes/index/HEAD and fake admission-host facts remain unchanged by checking |
| `readonly_gates_generation_and_neutral_writers_keep_their_controls` | Ordinary generation keeps unverified production verdicts with no implicit evaluation. Production parsing and both runtime readers refuse declarations without a route. Actual support and NE8 writers retain their specific refusal and caller snapshot |
| `readonly_gates_closing_observations_refuse_host_and_parent_drift` | Real shared checking followed by changed host identity, unavailable transport, changed protected parent or moved local HEAD. Source closure drift independently refuses. Concurrent caller state survives each refusal |
| `process::independent_process` | Explicitly launched by the composition. Starts with no profile/evaluation, refuses production depth, installs its own fake input authority, executes standalone command entry points and rejects the actual capability writer. Its discovery-ignore annotation prevents stdin-dependent standalone discovery |
| `private_onboarding_predicates_from_connected_on3` | Actual ON1/ON2/ON3 checkpoint, real IN1 admission/observation and slice 7's private coherent Publication proposal. Calls the slice-8 proposal adapter, verifies exact expected-record and capability refusals, preserves the working/index source guard and completes the existing aggregate/currentness/proving controls |

Test names in the first four rows are under `isolated_maintenance::tests::readonly_gates`.
The last is under `isolated_maintenance::tests::bootstrap::public_on3`.

The primary two-phase composition uses the fixed synthetic maintenance subject and real MA5
outputs. The connected extension separately exercises the onboarding-specific proposal adapter
over actual ON3 output. These are credential-free fake-host machinery proofs. The full public
ON4/ON5 accepting chain, native queue and real protected host evidence retain their owning slices
and phase-C boundary.

Slice 10b must convert the inventoried 80 test/helper production-registry readers and ten
whole-depth absence assertions while preserving P/C/U assertion purposes. It must execute the
full declared-candidate preflight test leg on both paths, including independent test-child fake
authority. The schema-free aider/Gemini spec and pointer tests remain actual-candidate T coverage.
The capability parity source-embedding harness and runtime production-parser fixture remain in
that inventory. Slice 8 consumes the private exact-record proposal adapter and owns all fixed
S/T execution, re-observation and checked delivery. No live profile is installed.

The final worktree contains the requested source, focused tests, Annex A/evidence notes and this
report. Changes are uncommitted for the lead, and no push was performed.

## Slice 10b

Work follows the design's registry-reader inventory in packet order. Site numbers below identify
the design baseline. Changes remain uncommitted for the lead.

### Per-site progress

| Site | Treatment | Status |
| --- | --- | --- |
| `xtask/agent_maintenance/support_audit/tests.rs:258` | P | done |
| `xtask/agent_maintenance/support_audit/tests.rs:424` | C | done |
| `xtask/isolated_maintenance/tests.rs:149` | U | done |
| `xtask/isolated_maintenance/tests/registry_tables.rs:36` | P | done |
| `xtask/isolated_maintenance/tests/registry_tables.rs:88` | P | done |
| `xtask/isolated_maintenance/tests/registry_tables.rs:109` | P | done |
| `xtask/isolated_maintenance/tests/registry_tables.rs:138` | C/U | done |
| `xtask/isolated_maintenance/tests/registry_tables.rs:150` | C/U | done |
| `xtask/release_doc.rs:97` | P | done |
| `crates/xtask/tests/agent_lifecycle_state.rs:143` | C | done |
| `crates/xtask/tests/agent_lifecycle_state.rs:357` | P | done |
| `crates/xtask/tests/agent_lifecycle_state.rs:524` | P | done |
| `crates/xtask/tests/agent_maintenance_audit_status.rs:459` | P | done |
| `crates/xtask/tests/agent_maintenance_audit_status.rs:516` | P | done |
| `crates/xtask/tests/agent_maintenance_audit_status_debt_baseline.rs:237` | P | done |
| `crates/xtask/tests/agent_maintenance_audit_status_debt_baseline.rs:338` | P | done |
| `crates/xtask/tests/agent_maintenance_closeout.rs:107` | P | done |
| `crates/xtask/tests/agent_maintenance_closeout.rs:121` | P | done |
| `crates/xtask/tests/agent_maintenance_closeout/write_outputs.rs:241` | P | done |
| `crates/xtask/tests/agent_maintenance_execute.rs:34` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare.rs:496` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare.rs:556` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare/depth_binding.rs:11` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:248` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:415` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:440` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:460` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:540` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:565` | P | done |
| `crates/xtask/tests/agent_maintenance_refresh.rs:70` | P | done |
| `crates/xtask/tests/agent_maintenance_refresh.rs:84` | P | done |
| `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs:14` | P | done |
| `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs:61` | P | done |
| `crates/xtask/tests/agent_maintenance_watch.rs:264` | P | done |
| `crates/xtask/tests/agent_maintenance_watch.rs:545` | P | done |
| `crates/xtask/tests/agent_maintenance_watch.rs:575` | P | done |
| `crates/xtask/tests/agent_maintenance_watch.rs:602` | P | done |
| `crates/xtask/tests/agent_maintenance_watch.rs:629` | P | done |
| `crates/xtask/tests/agent_maintenance_watch.rs:655` | P | done |
| `crates/xtask/tests/agent_maintenance_watch.rs:691` | P | done |
| `crates/xtask/tests/agent_registry.rs:41` | P | done |
| `crates/xtask/tests/agent_registry.rs:234` | C/U | done |
| `crates/xtask/tests/agent_registry.rs:251` | P | done |
| `crates/xtask/tests/agent_registry.rs:264` | P | done |
| `crates/xtask/tests/agent_registry.rs:323` | P | done |
| `crates/xtask/tests/agent_registry.rs:353` | P | done |
| `crates/xtask/tests/agent_registry.rs:398` | P | done |
| `crates/xtask/tests/agent_registry.rs:452` | P | done |
| `crates/xtask/tests/agent_registry.rs:482` | P | done |
| `crates/xtask/tests/agent_registry.rs:521` | P | done |
| `crates/xtask/tests/agent_registry.rs:629` | P | done |
| `crates/xtask/tests/agent_registry.rs:637` | P | done |
| `crates/xtask/tests/agent_registry.rs:678` | P | done |
| `crates/xtask/tests/agent_registry.rs:714` | P | done |
| `crates/xtask/tests/agent_registry.rs:741` | P | done |
| `crates/xtask/tests/agent_registry/non_runtime.rs:79` | P | done |
| `crates/xtask/tests/agent_registry/support_depth.rs:65` | U | done |
| `crates/xtask/tests/agent_registry/support_depth.rs:85` | P | done |
| `crates/xtask/tests/c0_spec_validate/depth_records/acceptance/publication.rs:15` | P | done |
| `crates/xtask/tests/c0_spec_validate/depth_records/acceptance/publication.rs:176` | P | done |
| `crates/xtask/tests/c0_spec_validate/depth_records/acceptance/relay_boundary.rs:37` | P | done |
| `crates/xtask/tests/c8_capability_matrix_unit.rs:49` | P | done |
| `crates/xtask/tests/c8_spec_capability_matrix_parity.rs:34` | P | done |
| `crates/xtask/tests/c8_spec_capability_matrix_registry_driven.rs:38` | C/P | done |
| `crates/xtask/tests/c8_spec_capability_matrix_registry_driven.rs:90` | C | done |
| `crates/xtask/tests/c8_spec_capability_matrix_registry_driven.rs:112` | P | done |
| `crates/xtask/tests/c8_spec_capability_matrix_staleness.rs:48` | P | done |
| `crates/xtask/tests/depth_synthetic_governing.rs:83` | P | done |
| `crates/xtask/tests/depth_synthetic_governing.rs:84` | P | done |
| `crates/xtask/tests/depth_synthetic_governing.rs:116` | U | done |
| `crates/xtask/tests/prepare_publication_entrypoint.rs:90` | P | done |
| `crates/xtask/tests/runtime_advertising/fixtures.rs:41` | P | done |
| `crates/xtask/tests/support/agent_maintenance_closeout_harness.rs:95` | P | done |
| `crates/xtask/tests/support/agent_maintenance_harness.rs:354` | P | done |
| `crates/xtask/tests/support/agent_maintenance_harness.rs:404` | P | done |
| `crates/xtask/tests/support/agent_maintenance_refresh_harness.rs:230` | P | done |
| `crates/xtask/tests/support_depth.rs:60` | P | done |
| `crates/xtask/tests/validator_spec_roots.rs:89` | C | done |
| `crates/xtask/tests/validator_spec_roots.rs:128` | C | done |
| `crates/xtask/tests/validator_spec_roots.rs:158` | C | done |
| `xtask/isolated_maintenance/tests.rs:149-154` | U | done |
| `xtask/isolated_maintenance/tests/registry_tables.rs:139` | U | done |
| `crates/xtask/tests/agent_registry/support_depth.rs:66` | U | done |
| `crates/xtask/tests/agent_lifecycle_support_depth.rs:56` | U | done |
| `xtask/prepare_publication/acquired_inputs_tests.rs:260` | P | done |
| `crates/xtask/tests/agent_maintenance_prepare/depth_binding.rs:10` | P | done |
| `crates/xtask/tests/depth_record/onboarding_binding.rs:230` | P | done |
| `crates/xtask/tests/c0_spec_validate/depth_records/acceptance/attribution.rs:7-11` | P | done |
| `crates/xtask/tests/onboard_agent_entrypoint/governing_validation.rs:85-87` | P | done |
| `crates/xtask/tests/runtime_follow_on_entrypoint.rs:79` | P | done |

### Treatment details

The shared immutable ordinary registry is `crates/xtask/tests/fixtures/undeclared_registry.toml`.
Every ordinary registry builder uses `tests/support/registry_fixture.rs`. Existing scratch
workspaces still materialize their command inputs from those shared bytes. The fixed ordinary lifecycle sample helpers also read the frozen lifecycle corpus. All production
`AgentRegistry::load` and `parse` behavior is preserved. Repeated helper module imports are
intentional because the harnesses embed source tests under separate module roots.

Candidate checks use `tests/support/candidate_registry.rs`. Ordinary discovery uses the existing
routed registry reader. The declared-checkout leg launches the private library-test child
`isolated_maintenance::tests::declared_test_leg::authenticated_registry_child` independently.
That child first verifies no installed route arrived, installs its own fixed synthetic test
profile and fake host observations, authenticates the actual committed checkout through the
real route, verifies production parsing still refuses its declaration, and closes the route
before returning exact checked bytes or checker results. A failed child fails its caller.
Actual spec, lifecycle, debt and release-artifact readers repeat authentication after reading.
Capability reproduction runs the actual candidate generator inside the authenticated child.
The declared runtime check and Claude adoption validation also execute inside that child.

The lifecycle round-trip test continues to parse, validate and serialize every actual committed
state byte for byte, including depth-bearing states. Its ordinary absence assertions now apply
to the frozen ordinary lifecycle objects in `tests/fixtures/undeclared_lifecycle_states.json`.
All existing test names are preserved. Annex A has no renamed test references to update.

The deliberate orphan-lifecycle deletion, ordinary request binding, legacy approval attribution,
ON1 before-declaration scaffold and ordinary runtime-input absence assertions retain their
original assertions. Their setup is already deliberately ordinary or is now supplied by the
shared ordinary harness.

### Additional fixture consumers

These setup consumers also embedded or copied the changing registry. Their ordinary fixture
inputs now use the same shared helper so the declared workspace leg reaches their assertions.

| Site | Treatment | Status |
| --- | --- | --- |
| `tests/support/onboard_agent_harness.rs` ordinary scaffold registry seed | P | done |
| `src/isolated_maintenance/tests/fixture.rs` ordinary baseline before isolated enrollment | P | done |
| `src/isolated_maintenance/tests/bootstrap.rs` ordinary and pre-declaration controls | P | done |
| `src/isolated_maintenance/tests/bootstrap/public_on1.rs` ordinary and no-route controls | P | done |
| `src/isolated_maintenance/tests/bootstrap/regressions.rs` ordinary renderer control | P | done |
| `src/isolated_maintenance/tests/in1_integration/regressions.rs` ordinary baseline before onboarding | P | done |
| `src/isolated_maintenance/tests/onboarding_scope.rs` ordinary baseline before onboarding | P | done |
| `src/isolated_maintenance/tests/runtime_follow_on.rs` ordinary baseline before enrollment | P | done |
| `src/runtime_follow_on/depth/policy_tests.rs` ordinary baseline before policy resolution | P | done |
| `src/agent_maintenance/request/depth_tests.rs` ordinary request fixtures | P | done |
| `src/agent_maintenance/stand_down/tests.rs` ordinary watcher fixtures | P | done |
| `src/candidate_execution/create_lane_tests.rs` ordinary context before bound depth | P | done |
| `src/depth_acceptance/manual/tests.rs` ordinary manual-entry fixtures | P | done |
| `src/depth_acceptance/onboarding/tests.rs` ordinary baseline before declaration | P | done |
| `src/depth_evidence/verdict/tests/native_admission.rs` ordinary fallback control | P | done |
| `src/support_matrix/depth/tests/generator.rs` ordinary unpublished-root control | P | done |
| `tests/agent_maintenance_prepare/depth_core/fixture.rs` ordinary baseline before enrollment | P | done |
| `tests/agent_maintenance_audit_status_target_version.rs` ordinary version-mismatch setup | P | done |
| `tests/c6_spec_report_iu_validator.rs` ordinary validator fixtures | P | done |
| `tests/support_depth_debt.rs` ordinary declared-debt fixture seed | P | done |
| `tests/support_depth_freeze/fixture.rs` ordinary baseline before freeze | P | done |
| `tests/support_matrix_consistency.rs` ordinary publication-control fixtures | P | done |
| `tests/support_matrix_derivation.rs` ordinary derivation fixtures | P | done |
| `tests/support_matrix_entrypoint.rs` ordinary command fixtures | P | done |
| `tests/support_matrix_staleness.rs` ordinary freshness fixtures | P | done |
| `tests/runtime_advertising/behavior.rs` ordinary host/feature/config-domain control | P | done |
| `tests/runtime_advertising.rs` actual candidate conformance checker | C | done |
| `tests/agent_lifecycle_state.rs` ordinary Claude adoption mutation fixture | P | done |

| `tests/agent_maintenance_prepare/from_request.rs` explicit-plan/CLI ordinary workspace | P | done |
| `tests/agent_maintenance_refresh/planning_and_apply.rs` four cwd-resolving ordinary controls | P | done |
| `tests/c8_capability_matrix_unit.rs` two cwd-resolving runtime-inventory controls | P | done |
| `tests/onboard_agent_entrypoint/governing_validation.rs` ordinary pre-existing root inventory | P | done |
| `tests/support_matrix_derivation.rs` actual committed runtime-support data equality | C | done |
| `tests/support/registry_fixture.rs` shared immutable ordinary workspace and process isolation | P | done |

Paths in this additional table are relative to `crates/xtask/`.

### Declared workspace leg

`scripts/s10_declared_test_leg.py` snapshots the current tracked and untracked source files into
a new disposable checkout under `target/`, installs the reviewed synthetic registry entry,
manifest-root templates and a synthetic lifecycle object, then commits that fixture checkout.
It uses the onboarding declaration by default. `--lifecycle-path maintenance` selects the
reviewed maintenance declaration. Each checkout carries one lifecycle path as the existing
isolated scope requires. The production profile stays absent.

The runner uses offline Cargo and a dedicated build cache under `target/s10-declared-build`.
It independently builds the private test reader and supplies that reader only to test helpers.
It rebuilds workspace packages before testing because Cargo dep-info can otherwise reuse an
executable naming a previous fixture checkout. Cached external dependencies remain available
between fixture runs in that dedicated cache. Successful completion also checks that the
committed fixture checkout stayed clean.

Run each full-workspace leg with a fresh destination.

```sh
python3 scripts/s10_declared_test_leg.py --destination target/s10-declared-workspace-onboarding
python3 scripts/s10_declared_test_leg.py --destination target/s10-declared-workspace-maintenance --lifecycle-path maintenance
```

Each default invocation executes `cargo test --offline --workspace` in its depth-declared,
committed fixture checkout. Full-workspace execution is reserved for the lead as requested.
Focused validation passed in both declared and ordinary checkouts.


### Validation discoveries

The wider declared maintenance run found five affected integration targets beyond the direct
parser calls. The explicit-plan/CLI control now uses an ordinary seeded workspace. Its positive uses the real
Clap parser, argument conversion and public workspace command entry to compare the exact preview.
The binary command fixes its root at compile time, so changing cwd cannot provide that control.
The mixed-argument and missing-mode refusals continue through the actual CLI executable. Four refresh
controls and two capability runtime-inventory controls resolve production inputs through cwd.
They now execute every original assertion in a test-owned child with a shared immutable ordinary
workspace as cwd, preserving concurrency isolation. The child asserts that exactly its intended
test passed. The onboarding scaffold prerequisite copier now selects only roots named by the
ordinary fixture registry plus the shared support-publication artifact, preserving the empty-root creation precondition. The actual committed
runtime-support-data equality check now runs its real renderer inside the authenticated child.
The declared fixture includes an empty current manifest and version inventory for that renderer.



### Final focused validation

All Cargo validation used offline dependency resolution with `TMPDIR` unset. The parent checkout
remains on `feat/s10-readonly-gates` at `7eb5932c`. The implementation changes are uncommitted.

| Validation | Result |
| --- | --- |
| `make fmt-check` | Passed |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Passed |
| `make loc-check` | Passed, 700-code-line cap |
| Focused ordinary integration targets listed below | All 30 passed |
| Same 30 targets over a committed onboarding-declared fixture checkout | Passed in one final complete focused run, with a clean fixture checkout afterward |
| Maintenance-declared rerun of the five initially affected targets plus `agent_registry` | All six passed, with a clean fixture checkout afterward |
| Library filters listed below over ordinary and committed maintenance-declared checkouts | All 19 nonempty filters passed |
| Final dedicated-cache runner with `-- -p xtask --test agent_registry` | 24 passed over the committed onboarding-declared checkout, including production depth refusals and actual-candidate routed reading |
| `git diff --check` | Passed |

The ordinary return run initially reused the depth fixture crate's acquired-binary test executable
from a declared checkout. Rebuilding that package at the ordinary source path made the affected
`depth_synthetic_governing` target pass. The runner now uses its dedicated cache so subsequent
fixture runs keep their build outputs together. The final dedicated-cache registry run passed.

No focused test remains failed or socket-blocked. The full `cargo test --workspace` execution
remains for the lead, using the two exact runner commands above.

#### Integration targets

Each target was invoked with `cargo test -p xtask --test <target>` through a combined focused
Cargo invocation. The ordinary `depth_synthetic_governing` target also has its successful
package-rebuild rerun.

- `agent_lifecycle_state`
- `agent_lifecycle_support_depth`
- `agent_maintenance_audit_status`
- `agent_maintenance_audit_status_debt_baseline`
- `agent_maintenance_audit_status_target_version`
- `agent_maintenance_closeout`
- `agent_maintenance_drift`
- `agent_maintenance_execute`
- `agent_maintenance_prepare`
- `agent_maintenance_refresh`
- `agent_maintenance_watch`
- `agent_registry`
- `c0_spec_validate`
- `c6_spec_report_iu_validator`
- `c8_capability_matrix_unit`
- `c8_spec_capability_matrix_parity`
- `c8_spec_capability_matrix_registry_driven`
- `c8_spec_capability_matrix_staleness`
- `depth_synthetic_governing`
- `onboard_agent_entrypoint`
- `prepare_publication_entrypoint`
- `runtime_advertising`
- `support_depth`
- `support_depth_debt`
- `support_depth_freeze`
- `support_matrix_consistency`
- `support_matrix_derivation`
- `support_matrix_entrypoint`
- `support_matrix_staleness`
- `validator_spec_roots`

#### Library filters

Each filter was invoked with `cargo test -p xtask --lib <filter>`. Three initially misspelled
filter paths matched zero tests and were corrected to the nonempty module paths recorded here.

- `agent_maintenance::request::depth_tests`
- `agent_maintenance::stand_down::tests`
- `agent_maintenance::support_audit_tests`
- `runtime_follow_on::depth_candidate::create_candidate`
- `depth_acceptance::manual::tests`
- `depth_acceptance::onboarding::tests`
- `depth_evidence::verdict::tests::native_admission`
- `isolated_maintenance::tests::canonical_ordinary_registry_and_isolated_protected_declaration_are_distinct`
- `isolated_maintenance::tests::registry_tables`
- `isolated_maintenance::tests::onboarding_scope`
- `isolated_maintenance::tests::fixture::runtime_follow_on::runtime_follow_on_freezes_p_record_object_and_contract_then_o`
- `isolated_maintenance::tests::bootstrap::bootstrap_absent_and_unselected_subject_require_exact_approval_and_scaffold`
- `isolated_maintenance::tests::bootstrap::bootstrap_route_absence_keeps_production_depth_refusal`
- `isolated_maintenance::tests::bootstrap::public_on1::public_on1_absent_route_keeps_ordinary_success_and_depth_refusal`
- `isolated_maintenance::tests::bootstrap::regressions::bootstrap_current_rejects_populated_and_foreign_input_identities`
- `isolated_maintenance::tests::in1_integration::regressions::in1_proposal_error_precedes_equal_head_and_parent`
- `runtime_follow_on_policy`
- `release_doc::tests`
- `support_matrix::depth::tests::generator`
