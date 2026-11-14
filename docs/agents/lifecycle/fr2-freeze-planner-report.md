# FR2 freeze planner implementation report

## FR2a

Status — FR2a implementation and validation complete, with five existing Unix-socket-blocked tests.
Changes remain uncommitted for the lead.
The report was created before implementation and updated during each increment.

### Delivered boundary

The pure `freeze_transition::plan` takes a complete predecessor leaf inventory, an empty-only
`ValidatedAcquisitionDelta` and typed owning-writer inputs. It renders the actual writer outputs
and returns exact create, replace, delete and preserve operations and the complete expected tree.
Every entry carries Git mode, object kind and bytes. Gitlinks carry their commit identity.

The real MA2 and ON2 writers use this planner. Protected IN1 independently derives the inputs
from authenticated predecessor Git objects and independently supplied writer observations,
then requires equality with the complete authenticated candidate tree. Matching output records
`request_freeze`, `exact_freeze_tree` and transition identities. Other depth candidates record
`execution_required` and a bounded reason while retaining the existing owning admission checks.
FR3 supplies the reporting context and population join before any execution receipt can be omitted.

### Files changed

Paths below are relative to `crates/xtask/src/` unless a different root is written.

- Shared core and delivery in `freeze_transition.rs` and
  `freeze_transition/{acquisition,delivery,tree,tests}.rs`, with module wiring in `lib.rs`.
- MA2 extraction in `agent_maintenance/prepare.rs`, `prepare/{depth,planner}.rs`,
  `agent_maintenance/docs.rs` and `agent_maintenance/docs/preparation.rs`.
- ON2 extraction in `runtime_follow_on/{context,depth}.rs` and
  `runtime_follow_on/depth/{approval,packet,planner,reporting}.rs`.
- Protected admission in `depth_acceptance/admission/{emitter,fake,history,mod,record,transport}.rs`,
  new `depth_acceptance/admission/freeze.rs` and
  `depth_acceptance/admission/freeze/{authority,objects,tests}.rs`.
- Authenticated captured-source support in `runtime_advertising/source.rs`, pure reporting
  inputs in `support_matrix.rs` and `support_matrix/preparation.rs`, and the existing acquisition
  destination list extracted in `manifest_acquisition/plan.rs`.
- Connected proofs in `isolated_maintenance/tests/fr2_maintenance.rs` and
  `isolated_maintenance/tests/bootstrap/fr2.rs`, with module wiring in `tests.rs` and
  `tests/bootstrap.rs`. The existing `tests/in1_integration.rs` candidate helper is visible to
  sibling test modules.
- `crates/xtask/tests/{agent_maintenance_prepare,agent_maintenance_audit_status}.rs` each add the
  shared planner module to their existing path-included harness. Their original assertions remain.
- This report and `docs/specs/{support-depth-contract,support-depth-evidence-v1}.md` record the
  measured boundary and additive admission fields.

### Planner inputs and outputs

| Type | Meaning |
| --- | --- |
| `Tree`, `Entry` | Complete path map with mode, kind and exact blob bytes or gitlink identity |
| `ValidatedAcquisitionDelta` | Private-field acquisition-authority input, with an empty constructor |
| `WriterInputs` | Shared pure rendering trait with owning metadata as an associated type |
| MA2 `prepare::planner::Inputs` | Original request arguments, registry entry, independently derived release/audit/contract/prompt, retained P/O/entries/work and derived reporting inputs |
| ON2 `depth::planner::Inputs` | Strict approval-derived execution context, P/O record, lifecycle object, reporting inputs and named packet timestamp |
| `PreparationInputs` | Independently derived support rows/runtime projection, predecessor support artifact and Markdown, rendered through ordinary support/runtime functions |
| `Rendered`, `Planned`, `Plan`, `Operation` | Writer metadata/files, exact operations and complete expected tree |
| `Observation`, `Writer` | Independent protected-host repository, predecessor, Event/P/O/enrollment, approved policy and named invocation inputs |

The policy boundary compares the observation with the independently installed publication authority
and the exact evaluated generation before deriving any output. The candidate cannot select a
planner version or policy closure. The transition's planner digest binds the fixed v1 planner
domain to the approved policy implementation identity. The writer-input digest binds the complete
observation in its own domain.

MA2 generation variables come from the predecessor's standing request through the existing
`--from-request` parser. ON2's `run_id`, `contract_generated_at`, `packet_generated_at`, requested
tier, justification path and rich-surface arguments come from the independently supplied host
observation. The two historical clock reads are separately named. The ordinary writer obtains its
clock values explicitly, and the connected proof passes fixed values to the real checked writer.
Run IDs, clocks and justification path syntax are validated. A justification consumed by protected
planning must be a regular predecessor blob, so its bytes have predecessor provenance.

The host method defaults to absent authority. Candidate files cannot install that observation.
Missing or contradictory observations prevent request-freeze eligibility. Observations are
re-read around planning and publication. Production host issuance remains a phase-C obligation.

### Writer effects

| Writer | Exact effects and source |
| --- | --- |
| MA2 | `prepare/planner.rs` renders standing request and packet through the extracted pure execution/packet renderers. `prepare/depth.rs::append` checks Event/P/O binding and renders the newline-terminated versioned record plus all three reporting files |
| MA2 packet | Existing renderer inventory includes README, scope brief, seam map, threading, review surfaces, remediation log, HANDOFF and PR summary. Packet-PR lanes additionally include OPS_PLAYBOOK, CI_WORKFLOWS_PLAN and the owned execution prompt |
| ON2 | `depth/planner.rs` renders the versioned record, exact lifecycle state with its P/O object and all seven packet files through `depth/packet.rs` |
| ON2 packet | `input-contract.json`, `codex-prompt.md`, `run-status.json`, `validation-report.json`, `run-summary.md`, `written-paths.json`, `handoff.json` |
| Both | `load_preparation` completes support-row/report intake and runtime projection derivation. `PreparationInputs::render` uses the existing JSON, Markdown and actual runtime-support projection renderers |
| Preservation | Every other predecessor leaf remains in `Plan.operations` and `Plan.expected`, including acquired inputs, OF-P4 statements, executable files, symlinks and gitlinks |

Existing reporting-only acceptance checks still precede delivery. P/O derivation preserves
retained entries/work and execution according to the existing rules. The reporting renderer
refuses new positive execution/debt claims. MA2 keeps its original rebuilt-plan and captured
expected-before materialization. ON2 delivers planned operations through admitted mutations.
The external MA2 transport report continues to use its existing outside-workspace destination.

### Protected Git boundary

`admission/freeze/objects.rs` walks and authenticates raw commit/tree/blob objects through FR1's
bounded object reader. It preserves object kinds and Git modes. Bounds include 64 directory
levels, 50,000 leaves, 512 MiB complete-tree material and the existing 16 MiB per-object limit.
Duplicate names, noncanonical tree ordering and hidden empty-directory tree objects refuse.

The predecessor source is captured in memory. Existing approved filesystem readers receive a
private materialization of authenticated predecessor regular blobs. The data view installs a
pinned index without checkout and writes authenticated bytes directly. It runs read-only approved
Rust derivation and Git plumbing. Candidate writers, generated packets, local actions, build
scripts and expected-output directories are never executed or consumed. Nonregular entries are
preserved in equality and refused when used as regular inputs. Local RULES, approval and record
substitutions have no influence on the reconstructed plan.

### Admission record and FR2b seam

The v1 record has optional `transition` fields `planner` and `writer_inputs`. `request_freeze`
requires an admitted exact transition. The optional `bootstrap` flag independently represents
first-decision anchoring when a depth transition has its own checkpoint kind. Earlier v1 records
retain their canonical bytes because absent transition and false bootstrap fields are omitted.
History accepts legacy bootstrap checkpoints and the independently anchored new representation.
The MA2 connected proof exercises its first exact freeze through publication and post-tip history.

The acquisition-authority module alone owns the delta's representation. FR2a can construct only
an empty delta. Its validator compares exact presence, absence, bytes, mode and kind for the
existing acquisition `delivery_paths`, shared with `AcquisitionPlan`. Changed acquisition returns
`acquisition authority unavailable`, retained as `acquisition_authority_unavailable` by IN1.
This includes target/union snapshots, wrapper coverage, coverage reports, version metadata and
lockfile. The MA2 proof adds a lockfile in the freeze commit and observes the bounded refusal
through actual IN1 while the candidate remains execution-required.

FR2b must authenticate non-empty acquisition effects and apply them before writer-input derivation
and complete-tree planning, preserving the existing single-commit transport. The empty-only type
and predecessor/candidate comparison supply that seam. FR3 owns canonical MA2 discovery and the
reporting-context/population join. Live activation remains phase C.

### Proving-test coverage

| Required item | Executable coverage |
| --- | --- |
| Genuine ON2 complete planned tree and canonical outputs | `bootstrap::fr2::fr2_on2_complete_tree_canonical_outputs_authority_and_real_in1` uses actual ON1, protected acquisition, metadata and declaration, then the real ON2 writer. It independently asserts the seven-file inventory, literal full executable prompt and validation JSON, contract clocks/docs, record state, support row and unchanged actual runtime projection |
| Genuine MA2 freeze-only complete planned tree and canonical outputs | `fr2_maintenance::fr2_ma2_freeze_only_complete_tree_canonical_packet_and_real_in1` uses real MA1, protected acquisition and real MA2. It asserts the literal twelve-file packet inventory, request Event/P/O fields, handoff commands, zero support/promotion credit and unchanged runtime projection |
| Extra implementation/dependency/build/workflow/unrelated/profile/packet/report bytes | Shared `complete_tree_equality_includes_bytes_modes_kinds_presence_and_preserved_paths`, plus the ON2 planned-tree mutation matrix and actual extra-edit IN1 publication. The extra edit records execution-required applicability |
| Additions, deletions, modes and file/symlink/gitlink changes | Shared complete-tree tests, `planner_preserves_symlinks_gitlinks_and_binary_blobs`, `deletion_and_output_collisions_are_explicit`, and raw-Git `authenticated_tree_reader_compares_regular_executable_symlink_and_gitlink` |
| Hidden empty directory object | `authenticated_tree_reader_refuses_hidden_empty_tree_object` constructs and authenticates the raw Git tree |
| Wrong predecessor, repository, approval/Event, P/O, enrollment, policy | ON2 independently mutates each authority identity and requires `freeze_authority_mismatch`; candidate approval substitution is also a preserved-tree mismatch |
| Variable-input provenance | Fixed real-writer clock inputs, independently modeled host observations, timestamp substitution causing a complete-tree mismatch, absent observation refusal and `writer_variables_require_valid_named_clocks_run_and_predecessor_relative_inputs` |
| Determinism and expected-output independence | ON2 reconstructs the same complete plan with identical authenticated inputs while local expected-output, RULES, approval and record bytes are substituted. Pure operation determinism is checked separately |
| Empty-only acquisition authority | `acquisition_authority_is_empty_only_including_presence_modes_and_kinds`, MA2 acquisition-destination matrix and actual lockfile-change IN1 refusal |
| ON2 protected acquisition boundary | The connected fixture admits acquisition/OF-P4 before freeze and asserts statement preservation. Existing `public_on2_in1_requires_parent_acquisition_and_intact_binding` and runtime-follow-on acquisition tests retain same-candidate and protected-provenance refusals |
| Connected protected history | Both connected positives call the real IN1 predicate/emitter, inspect retained request-freeze decisions, then advance fake protected history and run observation |
| Existing suites | Requested library groups, existing path-included MA integration suites, isolated prepare and MA2 route groups. Results and environmental restrictions follow |

Each new connected test constructs one mutable repository fixture. Small tree/provenance cases
operate on immutable inputs or one small Git repository and have no thread-local state. Connected
fixtures use the existing scoped installation and explicitly owned fake-host observations.

### Review decisions and corrections

- Pure reporting-input loading/rendering is exposed through the existing `support_matrix` module
  so the two path-included integration harnesses can reuse it. The input fields are private.
- Protected justification input bytes come from the predecessor snapshot. The variable validator
  rejects absolute, traversing and Git-metadata paths before any derivation.
- First-decision anchoring is independent of checkpoint kind. The initial MA2 proof exposed the
  interaction with FR1's bootstrap-only first-record check, and the additive flag preserves both
  meanings and earlier canonical records.
- The initial extraction re-observed MA2 output bytes at apply. Review found that this could
  overwrite an intervening edit. The original `prepare/guard.rs` delivery was restored, retaining
  captured expected-before checks and existing stale-plan behavior.
- Initial canonical test expectations were corrected against the reviewed renderer inventory.
  The ON2 contract carries the contract-document list. The MA2 packet-PR lane includes its three
  owned contract-input files. Existing output renderers and existing assertions were preserved.

- Final purity review found that the loaded-root row helper still read coverage reports during
  rendering. Support rows and runtime projection are now fully derived during input loading.
  The frozen reporting input contains typed rows/projection and the predecessor publication.
  Its renderer consumes frozen typed data. Both connected canonical proofs and existing MA
  integration suites passed after this correction. The MA2 proof also corrupts the selected
  live coverage report after input capture and requires identical rendered files.

### Validation environment

Compilation and tests used the default TMPDIR successfully.
Detailed command logs and wall timings are in ignored `target/fr2-validation/`.
The command/result and runtime tables follow.

Early focused measurements were 130.81 s for the ON2 connected proof and 71.10 s for MA2,
including the authority/refusal matrices and real IN1 observation. Raw Git/provenance tests took
0.09 s and pure tree tests took 0.00 s. Full-group and final focused timings take precedence below.

The sandbox blocks Unix sockets. The existing agent-maintenance channel-revocation test and four
existing connected IN1 closeout tests fail while opening the MA4 evidence channel with
`Operation not permitted`. Their assertions were retained. The lead must rerun these unrestricted.


### File-size boundary

`make loc-check` passed during implementation. Tokei measured `prepare.rs` at 679 code lines,
`prepare/planner.rs` at 148, the extracted packet renderer at 134, the shared planner at 110,
and the protected raw-tree reader at 141. All Rust files stay within the 700 code-line cap.
The final whole-repository gate passed as recorded below.


### Commands and measured results

The latest reporting-purity repair was followed by both connected canonical proofs, the existing
MA integration suites, support-matrix tests and the formatting/lint/LOC/hygiene gates. Times below
include concurrent verification load. Wall times include compilation and Cargo lock waits.

| Command | Result | Test time / wall time |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | 1.859 s wall |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | 6.373 s wall |
| `cargo test -p xtask --lib depth_acceptance` | 69 passed on final implementation | 103.92 s / 104.409 s |
| `cargo test -p xtask --lib runtime_follow_on` | 46 passed on final implementation | 104.57 s / 105.169 s |
| `cargo test -p xtask --lib agent_maintenance` | 50 passed, 1 socket-blocked, 1 existing ignored | 0.09 s / 13.787 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::in1_integration` | 24 passed, 4 socket-blocked | 50.24 s / 60.641 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 62 passed | 490.52 s / 491.042 s |
| `cargo test -p xtask --lib support_matrix` | 39 passed after purity repair | 0.17 s / 14.978 s |
| `cargo test -p xtask --lib fr2_on2` | Full literal executable-prompt expectation and connected ON2 passed | 141.93 s / 160.552 s |
| `cargo test -p xtask --lib fr2_` | Both connected positives passed after purity repair | 185.87 s / 190.867 s |
| `cargo test -p xtask --lib fr2_ma2` | Selected live-report corruption regression and connected MA2 passed | 69.03 s / 86.365 s |
| `cargo test -p xtask --lib depth_acceptance::admission::freeze` | 3 raw-Git/provenance tests passed | 0.07 s / 8.269 s |
| `cargo test -p xtask --lib freeze_transition` | 4 passed | 0.00 s / 0.067 s |
| `cargo test -p xtask --lib runtime_advertising::source::captured_tests` | 2 passed | 0.00 s / 0.067 s |
| `cargo test -p xtask --test agent_maintenance_prepare --test agent_maintenance_audit_status` | 51 preparation and 61 audit tests passed after purity repair | 22.51 s and 0.19 s / 46.005 s combined |
| `cargo test -p xtask --lib isolated_maintenance::tests::prepare` | 4 passed | 21.39 s / 21.525 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::ma2_routes` | 2 passed | 18.53 s / 35.060 s |
| `make loc-check` | Pass, all Rust files within 700 code lines | 0.059 s wall |
| `make hygiene` | Pass | 0.070 s wall |
| `git diff --check` | Pass | 0.028 s wall |

The final full depth-acceptance and runtime-follow-on reruns passed after reporting-input freezing.
The depth group includes the additional named-variable validation test. The final frozen-input
regression substitutes a malformed exact-target coverage report after deriving writer inputs,
then requires byte-identical rendering from those captured inputs.

The connected matrices construct one fixture per flow and reuse immutable trees for byte/mode/kind
and authority mutations. Small pure/raw-object cases complete below one second. The measured
combined connected group took 185.87 s. The entire bootstrap group took 490.52 s and exceeds the
older five-minute evidence-suite target. Runtime variability and this full-group cost remain
visible for the lead's unrestricted validation.

### Sandbox limits

Five existing tests are blocked by the Unix-socket restriction.

- `agent_maintenance::execute::runtime::tests::every_executor_failure_path_revokes_the_evidence_channel`
- `isolated_maintenance::tests::in1_integration::connected::in1_closeout_candidate_admission_and_post_push_observation`
- `isolated_maintenance::tests::in1_integration::connected::in1_prepared_closeout_candidate_admission_and_post_push_observation`
- `isolated_maintenance::tests::in1_integration::connected::in1_reauthenticates_population_in_admission_and_post_push_observation`
- `isolated_maintenance::tests::in1_integration::connected::in1_refuses_forged_closeout_entries_in_admission_and_observation`

Each fails while opening the existing MA4 evidence channel with `Operation not permitted`.
The lead must rerun the affected groups unrestricted. All new FR2a proofs and the 62-test bootstrap
group pass. Real host issuance, deployed source/runtime binding, emitter isolation, credentials,
retention and production activation remain phase C. The changes stay uncommitted on
`feat/fr2-freeze-planner` for the lead to review and commit.

## FR2b

Status — FR2b implementation and validation complete, with the same five existing
Unix-socket-blocked tests as FR2a. Changes remain uncommitted for the lead.

### Authority choice and scope

Use independently supplied fake-host acquisition observations, separate from candidate and producer
artifacts. Authenticate the checked transport's complete output inventory (including absence, mode,
object kind and exact bytes), approved acquisition invocation, and candidate/predecessor association
using retained host job/artifact commitments. Semantic validators alone cannot reproduce timestamps,
encoding, metadata notes/validation fields or complete lockfile contents. Independent observations
therefore authenticate those complete producer bytes, followed by the existing acquisition checks.
RULES/SCHEMA and the acquisition descriptor remain predecessor prerequisites. ON2 retains its
separate protected-parent boundary. No workflow, production route, phase-C activation or FR3 changes.

### Progress and validation

- Read FR2a report and the design's acquisition-boundary prerequisites before implementation.
- Confirmed branch `feat/fr2-freeze-planner`, initial head `986022c4`, clean worktree.
- Implementation and measured validation results will be recorded here as they complete.
- Added a connected combined-commit regression before enabling authority. Its first run exposed
  the existing public isolated route's clean-checkout requirement. The fixture uses the existing
  guarded private MA2 core/test apply shim for the staged transport boundary; production route
  policy is unchanged. The next regression run must fail specifically on absent acquisition
  authority before installing the independently retained observation.
- Implemented complete acquisition inventories, planner application before writer derivation,
  protected descriptor/plan resolution, modeled emitting-job/artifact commitments and closing
  observation rechecks. Metadata is additionally replayed as `reported` against predecessor
  notes/validation, and lockfile checks preserve predecessor rows and top-level content.
- The connected test reproduces the intended FR2a absent-authority failure, then reconstructs
  the complete combined expected tree from retained authority. The protocol is located in
  `isolated_maintenance/acquisition.rs`, separate from producer artifacts, and reuses FR1's
  protected policy closure, canonical/domain digests, bounded paginated host reads and emitting-job
  commitments. The admission record gains no fields: its existing writer-input identity binds the
  freeze observation together with the full authenticated acquisition observation.
- The synthetic subject is wrapper-only and claims no agent_api capability. Reported metadata can
  change its existing name-level coverage label. The proof checks zero qualified depth promises,
  no capability claims, no execution or acceptance entries, no pointer promotion and identical
  actual runtime-support bytes. It does not change existing support classification semantics.

### Delivered implementation

- `freeze_transition/acquisition.rs` alone constructs non-empty deltas after exact inventory
  comparison. Deltas retain expected-before entries, preserve optional absence, reject stale
  predecessors and cannot overlap freeze-owned output paths. The pure planner applies acquisition
  create/replace/delete effects before freeze effects and preserves every other predecessor leaf.
- `isolated_maintenance/acquisition.rs` defines the separate fake-host reference/observation
  protocol. Approved policy comes independently from the installed host, not payload/candidate
  fields. FR1 checks its complete runtime Git policy closure against the protected source.
  Attempt-specific job and artifact inventories, repository/inputs/parent/candidate association,
  exact canonical observation bytes, artifact digest/size, one domain-separated emitting-job
  commitment within the step window, expiration and independent revocations are authenticated.
  Complete observations are re-read during planning and around publication. Production Host
  methods default to absent authority.
- Protected IN1 resolves the full `AcquisitionPlan` against authenticated predecessor registry,
  RULES, required/expected targets and descriptor. Only that plan's exact `output_paths` may
  change. RULES/SCHEMA remain predecessor inputs. A private data view stages authenticated regular
  acquisition blobs with no-filter Git plumbing, preserving the predecessor HEAD. No candidate
  executable, checkout hook or build script runs.
- `freeze/acquisition_checks.rs` reuses manifest validation's schema/pointer/report checks before
  freeze outputs exist, replays the `reported` metadata producer against predecessor metadata and
  the authenticated clock, and checks lockfile top-level/retained-row preservation and complete
  target provenance. The owning MA2 derivation runs union/report intake validators and regenerates
  O; IN1 additionally checks O digests against the combined authenticated view. Complete-tree
  equality then authenticates the canonical request/record/packet/reporting outputs.
- Existing full `manifest-validate` behavior is unchanged. The connected proof runs it on the
  finished combined candidate and on the semantically unchanged byte-substitution candidate.
  Public acquisition version-validation order is preserved. The guarded MA2 apply shim is only
  test-visible; the production clean-checkout route is unchanged.
- Annex A now records combined acquisition admission and its modeled-host/phase-C limits. No
  admission record fields were added, so `support-depth-evidence-v1.md` is unchanged. `.github/`,
  producer transport, production route, live profile and FR3 remain unchanged.

### Proving coverage

The connected `fr2_ma2_combined_acquisition_and_freeze_real_in1` fixture shares one setup. It uses
real union/report writers, metadata planning, predecessor-preserving lockfile generation, the
real guarded MA2 writer and real transport staging/bundle/verification. Exactly one commit carries
all acquisition and freeze outputs. It admits through real IN1 and is re-observed after protected
history advances. The same fixture checks an unrelated edit and schema-valid whitespace changes
to both metadata and lockfile through real IN1. Byte/inventory and host mutation matrices share
immutable trees and independently forked host facts. Every acquired JSON output receives semantic
value-preserving byte, deletion, mode, symlink-kind, object-kind and substitution checks. Missing
inventory entries, extras, RULES/SCHEMA ownership, optional absence and candidate bundle/fingerprint
files refuse. Host mutations cover run, attempt, job, workflow, app, inputs, repository, artifact,
commitment availability/window, policy, predecessor/candidate, generation, resolved plan and
revocation. Pure delta tests cover create/replace/delete/preserve, stale parent and output collision.
A small lockfile test separately refuses top-level/row loss and incorrect target provenance.

### Review and current validation

Review preserved the public acquisition version-error ordering and the existing jq lock writer's
missing/null-artifact-list behavior. The latter now has direct small-test coverage alongside
preserved top-level/row content, duplicate targets and source/digest/size refusals. Acquisition logs
have an explicit bound, and retained acquisition identities honor the independently observed
revocation set. No new code edits are planned while regression validation completes.

Final Clippy passed after these corrections. Formatting, hygiene, complete diff whitespace and
LOC checks pass; `manifest_validate.rs` has 688 code lines and `prepare.rs` remains at 679.
Both MA2 connected proofs passed together in 78.41 s (78.48 s wall), including combined exact-tree
admission, full acquisition validation, actual IN1 semantic-byte-substitution/unrelated-edit
refusals and post-tip observation. The first full depth group passed all 70 tests. The maintenance
library group passed 50, with its one pre-existing Unix-socket-blocked failure and one existing
ignored test. Bootstrap, runtime-follow-on and existing integration regressions passed. Full IN1 has the same
four existing socket-blocked closeout failures as FR2a.
Detailed logs and measured command records are in ignored `target/fr2b-validation/`.

### Final commands and measured results

No network, credentials or live profile were used. The default TMPDIR worked. Wall times include
compilation, Cargo lock waits and concurrent validation load; the test times come from Cargo.
The final depth rerun follows the small lockfile missing/null-list correction. Existing assertions
remain unchanged; the two path-included MA harnesses also run the additional pure planner test.

| Command | Result | Test time / wall time |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | 1.909 s wall |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass on final code | 6.977 s wall |
| `cargo test -p xtask --lib freeze_transition` | 5 passed | 0.00 s / 14.815 s |
| `cargo test -p xtask --lib depth_acceptance` | 70 passed on final code | 60.90 s / 68.944 s |
| `cargo test -p xtask --lib agent_maintenance` | 50 passed, 1 socket-blocked, 1 existing ignored | 0.08 s / 14.090 s |
| `cargo test -p xtask --lib manifest_acquisition` | Pass; no matching library tests (integration coverage below) | 0.00 s / 0.074 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr2_maintenance` | Both freeze-only and combined MA2 proofs passed | 78.41 s / 78.480 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 62 passed, including genuine ON2 and same-candidate acquisition/statement refusals | 256.93 s / 257.053 s |
| `cargo test -p xtask --lib runtime_follow_on` | 46 passed | 80.08 s / 80.204 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::in1_integration` | 24 passed, 4 socket-blocked | 110.21 s / 110.298 s |
| `cargo test -p xtask --test manifest_acquisition_plan --test agent_maintenance_prepare --test agent_maintenance_audit_status` | 15 acquisition, 52 preparation, 62 audit tests passed | 0.03 s, 11.73 s, 0.10 s / 16.829 s combined |
| `cargo test -p xtask --test c0_spec_validate --test c3_spec_reports_metadata_retain` | 83 validator and 5 report/metadata/retention tests passed | 41.85 s and 0.02 s / 59.343 s combined |
| `python3 scripts/test_maintenance_transport.py` | 12 passed | 0.368 s / 0.416 s |
| `make loc-check` | Pass, every Rust file within 700 code lines | 0.059 s wall |
| `make hygiene` | Pass | 0.094 s wall |
| `git diff --check` | Pass | 0.004 s wall |

### Sandbox limits and handoff

The sandbox prevents opening the existing MA4 Unix-socket evidence channel. These unchanged tests
fail with `Operation not permitted (os error 1)` at channel creation and require unrestricted reruns:

- `agent_maintenance::execute::runtime::tests::every_executor_failure_path_revokes_the_evidence_channel`
- `isolated_maintenance::tests::in1_integration::connected::in1_closeout_candidate_admission_and_post_push_observation`
- `isolated_maintenance::tests::in1_integration::connected::in1_prepared_closeout_candidate_admission_and_post_push_observation`
- `isolated_maintenance::tests::in1_integration::connected::in1_reauthenticates_population_in_admission_and_post_push_observation`
- `isolated_maintenance::tests::in1_integration::connected::in1_refuses_forged_closeout_entries_in_admission_and_observation`

All new FR2b proofs pass. New connected cases reuse one fixture and immutable observations; the
combined MA2 group completes in 78.41 s and the full bootstrap group in 256.93 s in this environment.
Production acquisition-observation issuance, deployed source/runtime binding, emitter isolation,
credentials and retention remain phase C. FR3 canonical MA2 discovery/reporting-context/population
joins remain unimplemented. There is no new production reporting exception or support credit.

The worktree remains on `feat/fr2-freeze-planner` at committed head `986022c4`. All FR2b source,
tests, Annex A and report changes are uncommitted for the lead; no push was performed.

## Review round 1 fixes

Status — all six findings repaired and validation complete on `feat/fr2-freeze-planner`.
Changes remain uncommitted for the lead; HEAD remains `107a57b1`.
Scope: complete independent canonical output expectations, connected acquisition-authority
refusals, ON2 committed inventory under the repository ignore policy, and checked delivery of
exact modes and object kinds. Validation results and sandbox restrictions are recorded below.

### Repairs and regression evidence

- Complete literal expectations live in `crates/xtask/tests/fixtures/fr2-canonical/`, with an
  explicit inventory independent of the planner. Both connected fixtures compare every packet,
  depth record, support JSON/Markdown and runtime projection output; ON2 also compares its entire
  lifecycle object. Unchanged publication text is shared between lanes. The ON2 contract compares
  every field structurally, including its complete baseline file list and SHA-256 values derived
  independently from raw predecessor Git leaves. ON2 provenance slots (actual approval commit,
  approval hash, predecessor classification basis and acquired-byte hashes) likewise come from
  predecessor approval/registry/blobs. Expected P/O identities are independently hashed from the
  complete literal expected parts. No expected field comes from candidate output or a planner.
  MA2's union clock is fixed through the ordinary engine's test-only explicit clock. There is no
  fixture update mode in the tests.
- Coherent wrong parent, candidate, generation, invocation, descriptor plan and policy observations
  retain their direct binding assertions and now pass through `classify_authenticated`, requiring
  `execution_required` and `acquisition_authority_unavailable`.
- Real IN1 receives a committed candidate containing canonical-looking acquisition reference,
  observation and checked transport fingerprint files, with independent host authority withheld. It requires
  `acquisition_authority_unavailable`. Separate metadata-only and lockfile-only whitespace
  substitutions remain manifest-valid, retain matching host associations, and each require that
  same bounded refusal at real IN1.
- ON2's protected committed plan always includes reporting outputs and accounts for rendered
  packet files already tracked or present in the candidate. Absent local packet creations are
  omitted. Present packet bytes, modes and kinds remain subject to full equality. The connected
  fixture installs the repository's actual `.gitignore`, exercises normal staging without the
  seven local packet files, and separately force-adds packet files and substitutes bytes/modes.
- Checked delivery rejects nonregular plan entries, retains Git mode preconditions through
  mutation classification, applies planned modes to new files, and compares the delivered output
  inventory with `Plan.expected`. The chmod regression failed before the repair and passed
  afterward; the ignored-packet ON2 fixture likewise reproduced the original tree mismatch.

### Round 1 validation

Cargo commands used `env -u TMPDIR`; logs are ignored artifacts under `target/fr2-*.log`.
The intended red regressions failed before their repairs. Initial canonical-fixture clock and
provenance differences were resolved with explicit inputs and predecessor-only assertions.

| Command | Result | Test time |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | — |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | — |
| `cargo test -p xtask --lib freeze_transition` | 7 passed | 0.00 s |
| `cargo test -p xtask --lib depth_acceptance` | 70 passed | 46.33 s |
| `cargo test -p xtask --lib agent_maintenance` | 50 passed, 1 socket-blocked, 1 existing ignored | 0.21 s |
| `cargo test -p xtask --lib runtime_follow_on` | 46 passed | 46.76 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr2_maintenance` | 2 passed, including combined acquisition | 115.26 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | 62 passed | 507.32 s |
| `cargo test -p xtask --test agent_maintenance_prepare` | 54 passed | 20.28 s |
| `make loc-check` | Pass; all Rust files within 700 code lines | — |
| `cargo test -p xtask --lib workspace_mutation` | 10 passed | 0.00 s |
| `cargo test -p xtask --test c1_spec_union` | 7 passed | 0.01 s |
| `make hygiene` and `git diff --check` | Pass | — |

The sandbox blocks the existing
`agent_maintenance::execute::runtime::tests::every_executor_failure_path_revokes_the_evidence_channel`
while opening the MA4 evidence channel: `Operation not permitted (os error 1)`. That test and its
assertions remain intact and require an unrestricted lead rerun. No other sandbox limitation was
encountered by the requested validation commands. No existing test assertion was weakened or disabled. No changes have been committed or pushed.

## Review round 2 fixes

Status — all round 2 findings repaired and validation complete on `feat/fr2-freeze-planner`.
HEAD remains `80ff35fb`; changes remain uncommitted and no push was performed.

Scope: recompute refusal classifications from committed mutated candidates, connect acquired-input
byte substitutions to real IN1 with original authenticated output bytes, and retain MA2's checked
transition through delivery, including a final-recheck HANDOFF mode-change regression.

### Repairs and regression evidence

- Every ON2 complete-tree refusal mutation is now committed and sent through `freeze::classify`,
  which recomputes the planner from the protected predecessor and independent writer inputs.
  Each requires `ExecutionRequired` and `FreezeTreeMismatch`, including Cargo.lock, Cargo.toml,
  build/workflow files, authority/lifecycle/record files, packet/publication outputs and OF-P4
  statement bytes. Existing direct complete-tree assertions remain. Present-packet byte/mode
  refusals also assert the checkpoint. Separate ON2 acquired-output substitutions require
  `AcquisitionAuthorityUnavailable`. Each substituted ON2 candidate is one commit directly over
  the intact protected predecessor. A coherent substituted union plus a valid matching OF-P4
  statement in that same commit receives `AcquisitionAuthorityUnavailable`; the original
  protected statement and protected ref are explicitly checked.
- MA2's freeze-only acquired-output matrix and request-byte substitution now commit candidates
  and re-run classification, retaining the direct low-level assertions and requiring the specific
  execution-required refusal reason.
- Combined MA2 acquisition reuses its existing fixture for each input path (shard, union, wrapper
  coverage, and all coverage reports). Each independent whitespace-only substitution preserves
  JSON values, is staged and re-frozen through the real guarded writer, produces a changed O
  binding the new raw digest, and passes manifest validation. The independent host is updated
  to the new candidate and generation; its original authenticated output inventory is kept
  byte-for-byte. Host observation reading and binding must succeed before real IN1 requires
  `ExecutionRequired` with `AcquisitionAuthorityUnavailable`. Every such candidate is a single
  commit over the original protected predecessor.
- `PreparePlan` retains `planned.transition`; MA2 guard delivery converts it with the canonical
  `delivery::from_plan` checked mode/kind-aware mutations. The regression injects a HANDOFF.md
  executable-bit change after the final route recheck. It reproduced successful delivery before
  the fix (five writes) and now requires refusal before any output-byte or index change. The
  test restores the mode and continues the genuine canonical freeze and real-IN1 success proof.

### Round 2 validation

Cargo commands use `env -u TMPDIR`; logs are ignored artifacts under `target/fr2-r2-*.log`.
The first combined-input test attempt correctly hit the writer's working-tree/index agreement
check; staging the substituted input before re-freezing corrected the fixture setup.

| Command | Result | Test time |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | — |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | — |
| `cargo test -p xtask --lib freeze_transition` | 7 passed | 0.00 s |
| `cargo test -p xtask --lib agent_maintenance` | 50 passed, 1 socket-blocked, 1 existing ignored | 0.09 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::fr2_maintenance -- --nocapture` | 2 passed, including combined acquisition | 246.45 s |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap -- --nocapture` | 62 passed | 277.71 s |
| `cargo test -p xtask --test agent_maintenance_prepare` | 54 passed | 18.23 s |
| `make loc-check` | Pass; all Rust files within 700 code lines | — |
| `make hygiene` and `git diff --check` | Pass | — |

The sandbox again blocks
`agent_maintenance::execute::runtime::tests::every_executor_failure_path_revokes_the_evidence_channel`
while opening the MA4 evidence channel: `Operation not permitted (os error 1)`. The existing test
and its assertions remain intact and need an unrestricted lead rerun. All other requested checks
passed. No existing test assertion was weakened or disabled. No changes have been committed or pushed.
