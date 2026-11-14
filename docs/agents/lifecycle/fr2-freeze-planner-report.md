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
