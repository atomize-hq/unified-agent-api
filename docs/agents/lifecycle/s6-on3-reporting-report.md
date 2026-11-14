# Slice 6 ON3 reporting E and runtime selection

## Status

All ten review-round-1 findings are addressed on `feat/s6-on3-reporting`, based on checkpoint
`c86f5c79`. The requested focused commands have completed. The integration filter matched zero
tests, as recorded below. Changes remain uncommitted for the lead and the unrestricted gate
rerun remains with the lead.

## Scope and approach

Consume the existing FR1 admission/history, FR2 freeze planner and FR3 canonical generation,
reporting context and population interfaces. Extend the connected public ON2 chain with the real
ON3 reporting writer, runtime selection and real IN1 admission before modeled protected-tip
advancement and post-tip observation. Deliver reporting E, bound evidence files, lifecycle,
packet and coherent reporting publication through one checked proposal. Preserve Event/P/O and
entries, with no qualification.

The isolated fake host establishes predicate, binding, population/history and real-writer closure
behavior against independently supplied modeled observations. Real GitHub issuance, deployed
emitter isolation, job/artifact association and protected-ref serialization remain phase C.

## Implementation and proving evidence

The routing design's slice-6 proving items map to the executed connected tests below. Test names
are relative to `isolated_maintenance::tests::bootstrap::public_on3`.
[Routing design](support-depth-onboarding-routing-design.md#implementation-slices).

| Slice-6 proving item | Test and asserted evidence |
| --- | --- |
| Real assembly files bound by delivered E | `public_on3_connected_active_reporting_e_and_real_in1` and `public_on3_connected_repaired_runtime_selection_and_real_in1` authenticate the real assembly, all eleven verified obligations, every receipt hash and every bound payload hash against the admitted implementation commit. Both execute real IN1 before modeled protected-tip advancement and post-tip observation. |
| Active-run selection | `public_on3_connected_active_reporting_e_and_real_in1` pins `fr2-freeze`. Its `review::runtime_selection` and `review::reporting_candidate` cases independently validate a coherent alternate run, then refuse substituted lifecycle/packet selectors through public ON3 and committed-candidate IN1 with the exact selection reason. |
| Repaired-run alternative | `public_on3_connected_repaired_runtime_selection_and_real_in1` admits the missing-handoff checkpoint, executes public repair, admits the repair and selects its run through actual ON3. `review::publication_checkpoint` restores the older run's valid handoff and refuses reselecting it after publication-ready through public ON3 and IN1. |
| Stale evidence | The active connected test runs `review::runtime_selection` and `review::reporting_candidate`, substituting contract version, P, O, run id, workflow version and missing depth. It additionally changes the written-paths inventory and frozen wrapper coverage. Each asserts the owning reason. |
| Missing evidence | `public_on3_connected_active_reporting_e_and_real_in1` deletes runtime status and deletes/edits a delivered receipt. Both connected tests delete/edit the publication packet with E and the complete depth record unchanged, requiring IN1 refusal. |
| No entry | Both connected tests pin literal empty acceptance entries and preserve Event/P/O. The active test forges a Publication entry and requires IN1's unsupported-owning-authority refusal. |
| No qualification | Both connected tests compare all eleven published obligation identities/results with committed literals, require an unqualified promise, keep all supported/validated pointers at `none` and metadata at `snapshotted`. |

Additional connected assertions cover revoked-adoption refusal with an immediately unchanged
caller tree, acquisition currentness, onboarding/maintenance receipt separation, unchanged
protected source/workflow/archive closure, strict unstaged-record reads and fresh route capture
after tip advancement. `public_on3_no_route_and_maintenance_scope_refuse_without_writes` covers
production/no-route and installed maintenance refusal with a complete unchanged caller tree.

## Validation

The lead's unrestricted gate at `c86f5c79` reported **1712 passed, 5 failed**. Two failures were
identified as EAGAIN load flakes in
`isolated_maintenance::tests::prepare::the_public_prepare_route_freezes_p_then_o_under_the_isolated_context`
and `bootstrap::fr2::fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1`.
The three actionable failures were the connected ON3 fixture's absolute runtime path and the
unsupported-writer refusal-ordering regressions listed in review items 8–10 below.

| Focused command | Round-1 result |
| --- | --- |
| `make fmt-check` | Passed |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on3 -- --nocapture` | 3 passed, 0 failed, including both connected siblings. Runtime 1204.99 seconds. |
| `cargo test -p xtask --lib depth_acceptance::integration` | 0 matching tests, exit 0. IN1 regressions execute in the connected ON3 group. |
| `cargo test -p xtask --lib prepare_publication` | 1 passed |
| `make loc-check` | Passed, 700-code-line cap |

The lead retains the unrestricted full gate and reruns the original three actionable failures.
The local connected tests can use the required sockets in this sandbox.

Regression sensitivity checks temporarily restored individual old behaviors, ran only a focused
ON3 test, then restored the fixed source. Restoring the original undeclared-route refusal broke
`public_on3_no_route_and_maintenance_scope_refuse_without_writes` on the exact reason assertion.
Removing the complete depth input-contract validation made
`public_on3_connected_active_reporting_e_and_real_in1` fail when ON3 accepted a substituted
contract version. Removing the protected runtime-selection comparison also made the active
connected test fail when ON3 accepted a coherent alternate run. These expected negative runs are
separate from the final fixed-source results.

The first round-1 connected run reached the final rendered-obligation assertion and found that
the new literal used execution-result ordering. The committed publication literal now follows
the renderer's canonical obligation ordering. The final focused run passed with that correction.

## Implementation progress

- Extracted the ordinary transition construction before extending ON3. The routed controller
  captures approval/registry and proposal inputs, uses the shared authenticated assembler,
  materializes a bound read-only view and derives reporting publication with fresh verdicts.
  Aggregate reporting/effect checks precede expected-before delivery.
- Extended IN1 publication dispatch to evaluate onboarding E without a maintenance entry and
  authenticate the delivered E/receipt/payload bytes. The generation check now distinguishes
  E-only reporting from a new freeze while retaining lifecycle/history/P/O checks.
- Corrected the shared proposed-record reader to parse the full record directly. The synthetic
  maintenance profile rejects onboarding Events, while the onboarding profile projects away E
  for producer generation resolution. Proposed evaluation must retain the exact E and entries.
- A focused connected ON3 test consumes actual public ON1/acquisition/declaration outputs and
  the shared FR2 ON2 freeze writer. It calls the extracted FR3 collector-context helper before
  the freeze's real IN1 admission. Actual public ON2 implementation follows. A retained modeled
  substantive producer population over those exact implementation bytes feeds the real resolver
  and assembler. Real source admission precedes ON3 active and public-repair siblings. The
  existing broader FR2/FR3 refusal matrix stays on its original fixture.
- The ON3 siblings keep runtime handoff files in the committed candidate. The repair sibling
  first admits the missing-handoff reporting checkpoint, then executes and admits the real
  repair delivery before ON3 selection. Every protected-tip advance follows real IN1 and gets
  post-tip observation.
- IN1's E-only dispatch also checks the exact publication-ready packet and pinned predecessor
  runtime selection. Earlier lifecycle checkpoints cannot obtain ON3 reporting treatment merely
  by appending E.

Initial Clippy passed before adding the connected proof extension. Round-1 results are recorded above.


## Earlier checkpoint validation

The first complete connected run reached ON3 refusals and exposed that an uncommitted acquired
input edit exercised the clean-worktree guard. Refusal fixtures now commit each intentional
substitution in their disposable repositories, assert its owning bounded reason and preserve
the caller tree. A subsequent focused run exposed a stale modeled route after protected-tip
movement. Each advance now obtains a fresh route before observation.

Before checkpoint `c86f5c79`, the initial test groups passed with 1 prepare-publication test,
89 depth-acceptance tests and 38 GitHub-verdict tests. The round-1 table above records the requested
focused validation after the fixes and connected proof extension.

## Proposed evidence and generation reads

The real writer exposed two shared-reader dependencies. Onboarding population discovery now uses
FR3's immutable committed-head generation lookup, as maintenance already does. The proposed
lifecycle comes from the attested `EvaluationSource`, while approval, acquisition, build closure
and currentness continue through strict `GitInputs` reads. The shared evaluator reads proposed
receipt/payload artifacts from that same frozen source, authenticates them against the complete
host population and verifies the full proposal fingerprint before returning verdicts. Ordinary
and native artifact reads retain their existing Git-bound reader. No `GitInputs` check changes.
The connected test explicitly requires the ordinary unstaged-record read to keep rejecting E.

This requires narrow changes in `depth_evidence/verdict.rs` and the GitHub population/view call
site alongside the shared assembly interface. Generation resolution and population discovery use
the existing FR3 resolver. The execution witness in this credential-free fixture is an independent
modeled host observation over the actual committed ON2 source. Reporting context collection and
IN1 issuance execute their real implementations. Real substantive host issuance remains phase C.

The MA5 proposed-view regression caught an initial artifact-read regression. Existing committed
receipt files now always retain their Git-bound checks. Only newly assembled artifacts use the
attested proposal source. The socket-free stale-index/receipt/dependency test passes again.
Reporting derivation also receives receipt hashes in the exact fresh publication scope, since
new ON3 artifacts have no caller index entries yet. It rechecks those bytes at each rendering
read. Standalone derivation uses the existing `bound_evidence` reader. This adds a narrow call
site in `support_matrix/derive.rs`; the public GitInputs and depth-record readers retain their
checks.


## Review round 1 fixes

| Finding | Implementation and regression |
| --- | --- |
| 1 — selected-run frozen contract | Extracted `SupportDepthContract::check_input_contract` from routed repair's full comparison and reused it in ON3's depth runtime validator. Repair and independent reporting-transition validation share approval, agent, manifest, workflow, run id and version/P/O checks. The active connected test substitutes each contract identity through public ON3 and committed-candidate IN1. |
| 2 — selection after publication-ready | Public ON3 and IN1 resolve the predecessor selector from its active lifecycle run or committed publication packet. Both compare it with the candidate's validated selection. Both connected tests retain valid alternate bundles and refuse packet reselection after protected publication-ready. |
| 3 — adversarial alternate selection | Coherent alternate runs carry matching contract run ids, relative status paths, exact inventory digests and valid neutral handoffs. Tests first validate each alternate independently, then require the specific protected-generation/publication-predecessor selection refusal through both owning entrypoints. |
| 4 — exact published obligations | Added eleven literal published identities/results to `bootstrap/vectors/on3-reporting.json`. Both connected tests compare the entire rendered obligation array and require exactly one promise. |
| 5 — revoked-adoption mutation check | The active connected test captures and compares the complete caller tree immediately around revoked-adoption ON3 refusal. |
| 6 — unchanged-E checkpoint integrity | IN1 checks reporting lifecycle/packet transitions before the unchanged-statement return whenever the retained onboarding generation carries E. Both connected tests commit packet deletion and packet edits, assert the depth-record bytes are unchanged and require the specific packet failure. |
| 7 — Annex A references | Corrected the contract to name `public_on3_connected_active_reporting_e_and_real_in1` and `public_on3_connected_repaired_runtime_selection_and_real_in1`. |
| 8 — portable runtime status | The shared immutable ON3 template is reused through Git clones. Runtime follow-on and generated runtime bundles now record repo-relative `run_dir`. The active connected test asserts the exact relative status path in the copied repo before ON3. The repaired sibling validates the restored older run from that copy. Existing absolute-path compatibility in the historical reader is preserved. |
| 9 — installed-route unsupported writer | Restored `depth_routes::refuse_workspace` before the undeclared-onboarding fallback. The public no-route/maintenance regression pins the exact unsupported-depth reason and unchanged caller tree. The lead's `an_installed_route_grants_unsupported_writers_no_depth_authority` keeps its existing assertion. |
| 10 — publication entrypoint ordering | The same restored guard reaches the existing unsupported-depth refusal for acquired-input fixtures under maintenance authority. The lead's `publication_entrypoints_unchanged_freeze_reaches_the_existing_approval_refusal` keeps its existing assertion. |
