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
