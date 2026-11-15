# Slice 11 — E.8 executed mediation matrix

Work is in progress on `feat/s11-mediation-matrix`, based on `c1424fff`.
Changes remain uncommitted for the lead. Phase C remains closed.
The fixtures use credential-free synthetic authority and local subprocesses.
The connected chain retains the existing test-owned S/T executor while running
the real proposal A checks and real IN1 implementations. The subject is
wrapper-only. Foreign agent/target/version cases exercise adversarial effects.
Runtime-agent capability acceptance and real protected host/queue evidence retain
their separate proving boundaries.

## Execution matrix

Every refusal added here checks a bounded literal reason and a complete caller-tree
snapshot including bytes, modes, directories and symlink targets.

The new binary onboarding matrix executes 35 refusal invocations. The new neutral
matrix executes 135, with all three NE6 commands and each NE3–NE5 alias. The NE2
destination regressions add seven refusal invocations, including directory and file
aliases into the nested workspace. The standalone library
lower matrix executes 38, and the public maintenance matrix executes 36.
The connected extensions repeat lower-boundary cases with actual retained producer
records and acceptance entries.

| Route | Cases | Executed test | Result |
| --- | --- | --- | --- |
| MA1 | Ordinary preparation positive, declared/omitted/disabled onboarding, removed record, current and later-version request, cached apply | `isolated_maintenance::tests::mediation_matrix::maintenance::ma_intake_and_apply_refuse_onboarding_and_later_version_displacement` | Passed |
| MA2 | Same public re-freeze builder/apply, ordinary request displacing retained 0.2.0 with requested 0.3.0 | Same maintenance matrix; `unsupported_routes::an_installed_route_grants_unsupported_writers_no_depth_authority` retains acquired maintenance positive | Passed |
| MA3 | Actual ordinary refresh positive, cached refresh apply after each onboarding carrier | Same maintenance matrix | Passed |
| MA4 | Public relay entry on each carrier, retained policy omission, removed record and changed registry at actual reporting checker, committed onboarding pointer effect | Same maintenance matrix; both connected matrix tests | Public entry and all relay assertions passed; containing connected tests running |
| MA5 | Actual final command, prepared command with fetch-forbidden closure and lower closeout writer for each carrier; wrong record output through captured publication apply | Same maintenance matrix; connected checkpoint | Public entries and captured-publication refusal passed; containing connected tests running |
| MA6 | Actual promotion per carrier, canonical deleted-binding refusal, maintenance-path substitution on actual valid ON3/ON5 records | Same maintenance matrix; connected checkpoint | Public entries and ON3 substitution passed; complete chain running |
| ON1 | Declared/omitted/disabled/orphaned onboarding, removed record, foreign approval, omitted depth scope | `mediation_matrix::onboarding::every_on_entry_refuses_onboarding_carriers_without_scope_authority` in `tests/depth_refusal_routes.rs` | Passed |
| ON2 implementation and repair | Same five carriers and foreign approval | Same binary onboarding matrix; `public_repair_sibling_chain_preserves_frozen_generation_and_written_paths` executes actual successful repair, rollback and IN1 | Passed |
| ON3 | Same five carriers; actual authenticated reporting producer | Same binary onboarding matrix; both connected matrix tests | Binary and actual reporting checkpoint passed; complete chain running |
| ON4 | Same five carriers; actual proposal with foreign agent/version/target and partial delivery restoration | Same binary onboarding matrix; `connected_on4_aggregate_and_restoration` | Binary and all three new aggregate assertions passed; inherited partial-delivery/rollback continuation running |
| ON5 preparation | Same five carriers, actual prepared reporting through IN1 | Same binary onboarding matrix; `connected_onboarding_mediation_and_admitted_writers` | Binary, actual prepared/human reporting and their IN1 checkpoints passed; later test phases running |
| ON5 final close | Same five carriers through actual `close-proving-run` binary, actual proving entry and rollback | Same binary onboarding matrix; `connected_onboarding_mediation_and_admitted_writers` | Binary and local final-writer refusal cases passed; final rollback/positive/committed tampering continuation running |
| NE1 | Five carriers, no optional depth scope; actual lower bundle delivery with genuine ON3/ON5 retained records | `mediation_matrix::neutral::every_ne_command_and_alias_refuses_onboarding_carriers`; connected checkpoint | Binary and ON3 lower boundary passed; complete chain running |
| NE2 | Five carriers, default/nested output, proposed direct/nested record and registry destinations, separately declared nested workspace and output aliases | Same binary neutral matrix; all four `mediation_matrix::destinations` tests | Passed |
| NE3 and `codex-version-metadata` | Five carriers, subject/ordinary/nested roots, omitted alias root, requested later version and accepting target status | Same binary neutral matrix | Passed |
| NE4 and `codex-validate` fix | Five carriers, subject/ordinary/nested roots, omitted alias root; actual validator prerequisites | Same binary neutral matrix | Passed |
| NE5 and `codex-retain` apply | Five carriers, subject/ordinary/nested roots, omitted alias root | Same binary neutral matrix | Passed |
| NE6 Codex / Claude / OpenCode legacy snapshots | Each command over five carriers, own and nonexistent nested output directories, no upstream process reached | Same binary neutral matrix | Passed |
| NE7 historical backfill | Five carriers, `--all` and one unrelated ordinary agent | Same binary neutral matrix | Passed |
| NE8 runtime model | Five carriers, omitted depth scope; hand-edited foreign capability/runtime advertising with absent records | Same binary neutral matrix; `depth_acceptance::onboarding::tests::review_round1::foreign_advertising_without_records_refuses_raw_additions` | Passed |
| LW1 batch/lifecycle/packet | Five carriers, nested caller roots, proposed records/registry at direct/nested destinations, first output of refused batch absent, ordinary positives | `isolated_maintenance::tests::mediation_matrix::onboarding_retained_and_proposed_lower_writer_matrix` | Passed |
| ON3–ON5 admitted delivery and restoration | Actual retained onboarding producer chain, owning admission and rollback, lower boundaries on actual ON3/ON5 output | `isolated_maintenance::tests::mediation_matrix::connected_onboarding_mediation_and_admitted_writers` | Running |
| Ordinary neutral output | Actual capability writer from a normalized ordinary fixture, relative parent traversal, absolute paths and ordinary directory/file aliases | `mediation_matrix::onboarding::ordinary_neutral_command_reaches_its_production_result`; `mediation_matrix::destinations::ne2_ordinary_explicit_output_keeps_caller_path_resolution` | Passed |
| IN1 / aggregate | Removed/malformed record, omitted/disabled declaration, deleted lifecycle binding, unrelated requested Codex root with `--tip`, foreign agent/target/version | Both connected matrix tests; `depth_acceptance::onboarding::tests::outside_generation_effect_and_foreign_entry_refuse_the_entire_batch` | Short predicate, all five committed refusal cases, foreign-root adapter and new ON4 aggregate assertions passed; later ON5 committed tampering continuation running |

## Guard gaps

NE2's explicit output was guarded only through the generating workspace. Both
`ne2_proposed_record_destinations_refuse_before_writing` and
`ne2_explicit_output_cannot_hide_a_nested_onboarding_workspace` failed because
the real command succeeded and wrote its output. The first created a canonical
`reports/0.2.0/depth-record.json` in an ordinary workspace. The second wrote
inside a nested workspace with its own onboarding declaration.

The minimal repair adds `depth_routes::refuse_file_destination` at the NE2
entry, after its existing workspace refusal and rendering. It checks the output's
enclosing scope and reuses proposed-output checking for record/registry names.
The writer continues to receive its original resolved output path. Existing
unsupported-stage refusal ordering is preserved. The red regressions remain.
All three NE2 destination regressions now pass, including direct/nested registry
outputs. The complete binary suite passes 25 tests after the guard repair,
including ordinary relative parent traversal and absolute output controls.

The ordinary directory-alias control exposed an overly broad refusal in the first
guard implementation. The guard now observes the canonical existing output
ancestor while the writer uses its existing path. A subsequent real binary
regression demonstrated a successful write through a dangling file alias into the
nested declared workspace. Output observation now follows bounded file-link
chains as well. The same test keeps the direct, directory-alias and file-alias
depth refusals, and ordinary aliases have positive controls. The final complete
binary run passed all 25 tests in 80.51 seconds. Final Clippy, fmt and LOC checks
also pass after that extension.

The first executions found fixture problems before conformance assertions. The
committed ON2 vector contains provenance placeholders, which the refusal-only
fixture now expands and structurally seals. Those placeholder input digests grant
no evidence authority. The connected leg consumes the actual producer's records
separately. The matrix now supplies real validator prerequisites, and respects
manifest-first retained-record refusal ordering. An empty orphan registry is
replaced with the ordinary registry so it retains valid unrelated agents. These
corrections change test setup only.

The source-checkout declaration is recognized through the slice-10
`declared_build_refusal` pattern before ordinary fixture normalization. Unexpected
source parse/binding failures remain errors. The real commands resolve their
disposable copied workspace, so their expectations follow that workspace's own
registry. A declared source checkout selects the existing committed ordinary
registry fixture, which also avoids duplicating the synthetic subject installed
by the lead's declared-leg script. The lead retains execution of the full
depth-declared build leg.

The committed relay acceptance case now changes the owned target's actual
`latest_validated` pointer. An earlier `min_supported` edit reached publication
freshness instead of the positive acceptance detector. The deleted-lifecycle
IN1 case pins its existing `missing bound input` refusal, which precedes the
source reader's missing-input error. Production refusal order remains intact.

## Private delivery and restoration inventory

The forward and reverse source scan includes imported aliases and
`freeze_transition/delivery.rs`, which a filename-only route scan can omit.
The same inventory is reconciled in Annex A.4 E.8.

| Private delivery caller | Owning admission |
| --- | --- |
| Public `workspace_mutation/delivery.rs::apply_mutations` | Workspace-wide declared/retained guard and each proposed output |
| `agent_lifecycle/storage.rs::write_json_file` | Only lifecycle and publication-packet public writers, after workspace/proposed-output guards and typed validation. Lifecycle storage also preserves the existing depth object |
| `agent_maintenance/prepare/guard.rs::apply` | MA1/MA2 exact plan rebuild, reporting effect check, input/index/output binding and final route recheck |
| `freeze_transition/delivery.rs::apply` | Sole production caller ON2 dry-run, after approval/freeze/proposal/view/route admission |
| `candidate_execution/candidate.rs::deliver_with` | MA4 relay and ON2 implementation, unconditional observation, full candidate/Git/envelope/frozen-input checks, reporting admission and exact index publication. Failure delivery contains host diagnostics |
| `candidate_execution/create_lane.rs::execute,recheck` | ON2 authenticated attempt write, then two expected-before original/attempt substitutions under its index lock around captured-route recheck |
| `agent_maintenance/closeout_proposal.rs::Proposal::apply_with` | Public `Proposal::apply` supplies manual-closeout admission and snapshot/view checks |
| `depth_acceptance/publication.rs::Proposal::apply_closeout_publication` | Both MA5 public writers and `CloseoutPlan::apply`, freshly admitted exact closeout, marker/P/O/E/entry and scoped publication, captured-route recheck |
| `manifest_promotion.rs::PromotionPlan::apply` | MA6 exact validation revision/protected tip, snapshot, maintenance owner/evidence/entry, fresh apply admission |
| `depth_evidence/staged.rs::write` | MA4 invocation offer/channel and bundle identity, Event/P/O, reporting destinations, bytes and digests. Later relay and IN1 authenticate reporting E independently |
| `repair_runtime_evidence/routed.rs::deliver` | ON2 protected runtime-integrated generation, exact successful contract/written paths, frozen inputs and preserved record/entries, aggregate reporting proposal and closing observations |
| `prepare_publication/routed.rs::run` | ON3 authenticated E/files, aggregate reporting and relay checks in bound view, final view/route recheck |
| `publication_refresh/routed.rs::deliver` | ON4 exact Publication entry, complete post-entry evaluation, real proposal A checks, bounded test S/T runner, reevaluation and closing observations |
| `depth_acceptance/onboarding/on5.rs::deliver` | Prepared ON5 exact reporting/relay admission and final ON5 exact proving entry/evidence/aggregate admission, both with view/route rechecks |

ON1's scaffold uses the guarded public mutation writer. Test-only failure/hook
variants remain compiled under `#[cfg(test)]` and use the same owning controllers.

| Restoration writer | Guard |
| --- | --- |
| Ordinary ON3 `write_publication_transition` raw lifecycle rewrite and temporary packet cleanup | Outer unsupported-depth guard before `run_body`. Routed ON3 uses its admitted batch |
| Ordinary ON4 `restore_snapshots`, including deletion of new outputs | Outer unsupported-depth write guard and public LW1 for replacement files |
| Routed ON4 rollback | Admitted batch plus expected-before byte/mode/kind comparisons, conflict-preserving recovery files. Connected matrix runs replacement/removal/chmod/symlink siblings |
| Prepared/final ON5 rollback | Both use `on5::deliver` and shared expected-before rollback. Connected final ON5 runs all four conflict siblings |
| ON2 attempt restoration/reinstatement | Exact controller-owned bytes, index lock and captured-route recheck |
| Ordinary ON2 repair raw backup/lifecycle restoration | Outer unsupported-depth refusal. Routed repair uses its own checked batch and shared rollback. The connected repair sibling passed |
| Shared file replacement/rollback and MA4 index unpublication | Every caller above retains its owner. Recovery compares actual delivered bytes/modes/kinds and preserves concurrent state. Index unpublication removes precisely the entries introduced by that delivery |

## Focused validation

Cargo runs use `env -u TMPDIR`. The lead owns the full gate and the
depth-declared build leg.
Focused logs are available under ignored `target/s11-validation/`. The
`mediation-matrix.log` entry follows the active connected run.

| Focused validation | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::unsupported_routes` | 1 passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::mediation_matrix` | Two short tests passed; both connected tests running in inherited rollback/final integration phases |
| Same matrix filter with both connected tests skipped | 2 passed on final source, 9.57 seconds |
| `cargo test -p xtask --test depth_refusal_routes` | 25 passed, 79.69 seconds after final fixture normalization and alias controls |
| `cargo test -p xtask --lib depth_acceptance::manual` | 41 passed |
| `cargo test -p xtask --lib depth_acceptance::onboarding::tests::outside_generation_effect_and_foreign_entry_refuse_the_entire_batch` | 1 passed |
| `cargo test -p xtask --lib depth_acceptance::onboarding::tests::maintenance_and_onboarding_authorities_cannot_substitute` | 1 passed |
| `cargo test -p xtask --lib depth_acceptance::onboarding::tests::review_round1::foreign_advertising_without_records_refuses_raw_additions` | 1 passed |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_repair::public_repair_sibling_chain_preserves_frozen_generation_and_written_paths` | 1 passed, 165.91 seconds |
| `cargo test -p xtask --lib isolated_maintenance::tests::ma5_closeout` | 3 passed, 7 socket-blocked |
| `cargo test -p xtask --lib isolated_maintenance::tests::ma6_promotion` | 5 passed, 3 socket-blocked |
| `make loc-check` | Passed, 700-code-line cap |

## Socket-blocked controls

All ten failures stop at `cannot open the MA4 evidence channel` with
`Operation not permitted (os error 1)`, matching slices 8–9.

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


## Review round 1 fixes

The review-round changes supersede the original NE2 check-then-reopen delivery
and extend the destination and dry-run coverage recorded above.

| Finding | Repair and regression |
| --- | --- |
| P1 NE2 check-to-write race | `depth_routes/destination.rs::write_file_destination` holds the resolved destination and existing ancestor identity across validation. Unix delivery opens the canonical parent, uses `openat` with `O_NOFOLLOW` and creation, and checks parent device/inode identity after opening and before truncation. It also rechecks the original caller aliases. `depth_routes::tests::checked_delivery_refuses_final_and_directory_alias_retargeting` replaces an existing final component, retargets a directory alias, and replaces a parent directory. `checked_delivery_opens_without_following_final_symlinks` substitutes the final link immediately before open. `checked_delivery_rechecks_parent_identity_after_opening` replaces the parent immediately after open. Every case refuses and preserves protected bytes. Ordinary directory-only spellings such as `file/`, `file/.` and a file link ending in `/` retain refusal and preserve existing bytes in `resolution_refuses_loops_and_io_errors_and_keeps_ordinary_delivery_bytes`. Hooks compile only under `#[cfg(test)]` |
| P2 NE2 effective classification | The resolver follows at most 40 links, resolves existing ancestors, and normalizes missing suffix traversal against the canonical ancestor. I/O errors propagate as refusals. `mediation_matrix::destinations::ne2_registry_traversal_refuses_before_replacing_bindings` and `ne2_registry_directory_alias_refuses_before_replacing_bindings` cover `crates/xtask/data/../data/agent_registry.toml` and `alias/agent_registry.toml` with `alias -> crates/xtask/data`. `ne2_missing_prefix_traversal_cannot_hide_a_nested_onboarding_workspace` covers missing `new/../nested/matrix.md`. Each refusal compares the entire tree. `ne2_ordinary_explicit_output_keeps_caller_path_resolution` compares exact rendered bytes across relative traversal, absolute paths and ordinary file/directory aliases |
| P2 LW1 enclosed scopes | `refuse_output` reuses the existing registry/retained-record scans on resolved ancestors between the output and caller root, after specific proposed-output checks. Batch removals receive a destination-scope check too. `isolated_maintenance::tests::mediation_matrix::lower::outer_callers_refuse_enclosed_depth_destinations` exercises declared and retained nested workspaces through batch, lifecycle and packet writers with complete literal refusal reasons and unchanged trees. The batch cases include an earlier ordinary output and a removal. Ordinary nested delivery remains covered |
| P2 ON2 dry-run persistence | The ordinary `runtime_follow_on::run_in_workspace` guard covers `args.dry_run` as well as `args.write`, after owning-route dispatch and before context construction or persistence. `mediation_matrix::onboarding::every_on_entry_refuses_onboarding_carriers_without_scope_authority` now includes dry-run for all five carriers. `on2_dry_run_guards_persistence_for_foreign_approval_and_keeps_ordinary_delivery` makes the foreign ordinary approval runnable, checks all seven persisted files in an ordinary workspace, then requires unchanged-tree refusal with declared, omitted, disabled, orphaned and removed-record carriers. `maintenance::on2_dry_run_depth_approval_cannot_borrow_a_maintenance_route` pins the unsupported-depth refusal for a valid foreign depth approval under an installed maintenance route, before context loading |
| P2 MA6 specificity | The promotion builder uses the existing canonical `check_root` helper and retains its specific violation text in the returned error. `maintenance::ma_intake_and_apply_refuse_onboarding_and_later_version_displacement` requires `DEPTH_LIFECYCLE_BINDING_MISSING` for the omitted-declaration retained onboarding record. The same test first builds a valid ordinary promotion candidate with the same acquired version and targets |
| P2 lower delivery assertions | `lower::ordinary_lower_writers_deliver_literal_bytes` and the existing lower matrix read lifecycle and publication-packet destinations at direct and ordinary nested paths. They compare the full bytes, formatting and terminal newline against the independently pinned literals in `mediation_matrix/lower_expected.rs` |

NE2 delivery uses invocation-local observations. Declaration and record contents
can change after the guard reads. Another process can change directory placement
or hard-link bindings after the final identity observation and before truncation
or write. Parent-handle binding and final-component no-follow opening close the
demonstrated symlink substitutions. Serialization across processes requires the
design's owning integration boundary. A refusal after file creation can leave an empty
ordinary output. The race regressions specifically verify refusal and preserved
protected bytes. The classification and LW1 regressions additionally require the
complete caller tree to remain unchanged.

The Unix implementation uses the existing `rustix` dependency with its filesystem
feature. Focused execution uses Linux. The portable path uses Windows reparse
point opening and directory handles, with creation-time directory observations.
Windows runtime execution remains unverified in this round.

### Regression sensitivity

Temporary guard reversions and delivery mutations run only against disposable
fixtures. Their logs live under `target/s11-review-r1/s11-red-*.log`. The final source restores all
repairs before focused validation. The regression cases require failure when the
legacy check-to-write delivery, lexical NE2 guard, caller-only LW1 guard, write-only
ON2 guard, or generic MA6 diagnostic is restored. Separate lifecycle and packet
byte mutations exercise the two independent delivery expectations.

### Focused validation after review fixes

All Cargo commands use `env -u TMPDIR`. The lead owns the full gate. Focused logs
are under ignored `target/s11-review-r1/`.

| Focused validation | Review round 1 result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Passed |
| `cargo test -p xtask --test depth_refusal_routes` | 29 passed in the ordinary build workspace, 99.09 seconds. Repeated with the build workspace depth-declared, 29 passed, 122.39 seconds |
| `cargo test -p xtask --lib isolated_maintenance::tests::mediation_matrix::lower` | 2 passed, including enclosed declared/retained destinations and independent literal delivery checks. Also passed with the build workspace depth-declared |
| `cargo test -p xtask --lib isolated_maintenance::tests::mediation_matrix::maintenance` | 2 passed, including the specific MA6 diagnostic, valid ordinary sibling and foreign maintenance-route ON2 refusal. Also passed with the build workspace depth-declared |
| `cargo test -p xtask --lib workspace_mutation` | 10 passed in both build workspace configurations |
| `cargo test -p xtask --lib depth_routes` | 4 passed in both build workspace configurations, including check/open/write races, loop/I/O refusals, ordinary delivery and directory-only syntax |
| `cargo test -p xtask --lib isolated_maintenance::tests::mediation_matrix::onboarding_retained_and_proposed_lower_writer_matrix` | 1 passed in both build workspace configurations |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap::public_on2::public_on2_connected_freeze_write_admission_and_observation` | 1 passed in both build workspace configurations. Final depth-declared run passed in 91.52 seconds |
| `make loc-check` | Passed, 700-code-line cap |
| `git diff --check` | Passed |

The declared-build repetition temporarily appended the committed synthetic
onboarding declaration to the build checkout's registry and restored its original
bytes after every focused command completed. The final ordinary binary suite and
`depth_routes` unit tests ran after that restoration. Every temporary guard/byte
mutation failed its regression as expected. The additional directory-syntax and
foreign-route regressions have their own red logs in the same directory.
The full connected matrix and full gate remain assigned to the lead.

## Review round 2 fix

All four round-2 lanes found that checked NE2 delivery created only the normalized parent. For an
ordinary `--out new/../matrix.md` with `new` absent, the command succeeded but the caller's own
spelling stayed unusable, because `new` was never created; the previous `create_dir_all` on the
caller's parent had created it. `depth_routes/destination.rs::create_caller_directories` now
recreates every missing directory named by the caller's parent spelling after validation. Before
creating each one, it refuses at that directory's own enclosing depth scope, so traversal cannot
create a directory inside an enrolled workspace and then leave it.

Regressions, each failing with its part of the repair reverted:

- `ne2_ordinary_explicit_output_keeps_caller_path_resolution` gains
  `absent-prefix/../absent-output/matrix.md` and reads exact bytes back through that spelling.
- `ne2_missing_prefix_traversal_cannot_hide_a_nested_onboarding_workspace` gains
  `nested/new/../../matrix.md`. The effective destination is ordinary, but `nested/new` would be
  created inside the enrolled workspace. The test asserts the refusal and an unchanged tree.
