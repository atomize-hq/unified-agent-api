# FR3a reporting context implementation report

Status — implemented and validated on `feat/fr3-reporting-context`, subject to the environment restrictions below. Changes are uncommitted for the lead.

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
- The maintenance fixture restores a freeze-labelled tree after an execution-required admission. Real IN1 classifies it execution-required. The evaluator refuses its reporting claim on another PR in both population orders.
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
