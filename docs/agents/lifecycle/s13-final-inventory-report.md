# Slice 13 — E.8 final inventory and phase C handoff

## Scope and enforcement revision

The inventory is reconciled on `feat/s13-final-inventory` at
`e0d6ffbcd8cce99ccdcdfb448c662622b1e65ddc`, the merged slices 1–12 tree.
PR #369 also repaired the slice-11 ordinary ON2 dry-run persistence guard regression.
The actual guard is `crates/xtask/src/runtime_follow_on.rs:173`, with its ordinary
persistence check at line 190 and workspace refusal at line 207. Source citations
in this report refer to that enforcement revision. Paths are repository-relative.
Changes remain uncommitted for the lead.

The report was created before documentation edits and updated during reconciliation.
Inputs were the routing design including E.8/OR-P12, canonical contract/charter/evidence
requirements, CP-P5, validation list, isolated proof/rescope plans and FR1–FR3/S6–S12
reports, supplemented by the lead-gate results below. Those unrestricted-host results
supersede the worker reports' execution limits for completed implementation checks.

## Deliverables

| Document | Reconciliation |
| --- | --- |
| `docs/specs/support-depth-contract.md` Annex A.1 | Forward inventory covers MA1–MA6, ON1, each ON2 form including repair, ON3, ON4, both ON5 entries, NE1–NE8 and aliases, LW1, IN1 and HE1. Every row names current file/line ownership or refusal and an executed result/report |
| Annex A.2 | Backward inventory covers records/entries, runtime and CI evidence, source/diagnostics, every aggregate, metadata/pointers/current, requests/markers, lifecycle/packets/closeouts/docs and acquisition inputs. Writers and indirect unsupported destinations are mapped back to their guards |
| Annex A.3/A.4 | Serialization and restoration inventories distinguish expected-before local preparation, index locking, transport and final protected-ref admission. Current completed S6–S12 results replace stale pending claims. S11 private delivery inventory is reconciled below |
| `docs/specs/cli-agent-onboarding-charter.md` | ON present behavior describes the implemented finite synthetic chain. Pre-routing observations are labeled historical. Required behavior and phase C/path-enablement requirements remain in force |
| `docs/specs/support-depth-evidence-v1.md` | Present behavior records completed gate/ON/FR mechanisms and the fake-host/S/T proof boundary. Event/population separation and independent artifact/adoption requirements continue to govern |
| `docs/agents/lifecycle/support-depth-validation-test-list.md` | Executed S6–S12 index adds exact current names, lead-gate result provenance and independent phase C boundaries. Original required scenarios remain listed |
| `docs/agents/lifecycle/support-depth-isolated-proof-plan.md` | Phase C handoff specifies prerequisites, canonical workflow layout review, real subprocesses/producers/artifacts and native queue/readback proof |
| `docs/agents/lifecycle/support-depth-machinery-proof-rescope.md` | Current implementation status and link to the independently authorized phase C handoff |

## Results supporting the inventory

Names below are exact leaf function names. The validation list gives module scopes and
per-case coverage. Result provenance combines the linked slice reports with the supplied
lead-gate reconciliation below.

| Route/effect | Executed result |
| --- | --- |
| MA1/MA2 complete-tree and acquisition | `fr2_ma2_freeze_only_complete_tree_canonical_packet_and_real_in1` and `fr2_ma2_combined_acquisition_and_freeze_real_in1` passed through actual writers, checked transport, IN1 and modeled observation. [FR2](fr2-freeze-planner-report.md) |
| FR1 history and FR3 population/context | `public_on2_connected_freeze_write_admission_and_observation` passed including higher-stage admission without a producer. `fr3_reporting_context_real_ma2_and_later_execution_population` passed including genuine zero-credit freezes, exact context/history join and retained execution after identical source admission. [FR1](fr1-admission-history-report.md), [FR3](fr3-reporting-context-report.md) |
| ON1 bootstrap/preparation/declaration | `public_on1_connected_preparation_metadata_and_first_declaration_chain` passed in the S10 bootstrap run, then actual new-entry bootstrap participates in S12's qualified primary chain. [S10](s10-readonly-gates-report.md), [S12](s12-e7-connected-proof-report.md) |
| ON2 freeze/write | `public_on2_connected_freeze_write_admission_and_observation` passed in ordinary and declared S11 builds. `e7_connected_onboarding_qualified_chain` completed the full primary implementation chain. [S11](s11-mediation-matrix-report.md), [S12](s12-e7-connected-proof-report.md) |
| ON2 repair | `public_repair_sibling_chain_preserves_frozen_generation_and_written_paths` passed in S11, including actual repair, rollback and IN1. `e7_connected_onboarding_repaired_qualified_chain` passed through publication and proving. [S11](s11-mediation-matrix-report.md), [S12](s12-e7-connected-proof-report.md) |
| ON3 reporting E/files/runtime selection | `public_on3_connected_active_reporting_e_and_real_in1` and `public_on3_connected_repaired_runtime_selection_and_real_in1` passed in the final S6 run, preserving Event/P/O and empty entries/unqualified reporting. [S6](s6-on3-reporting-report.md#review-round-2-fixes) |
| ON4 Publication/unchanged qualification | `public_on4_connected_publication_real_in1_and_observation` passed in S8 round 1b. `e7_connected_onboarding_qualified_chain` passed with qualified wrapper publication. [S8](s8-on4-publication-report.md#refusal-ordering), [S12](s12-e7-connected-proof-report.md) |
| Both ON5 entries and proving | `public_on5_prepared_closed_real_in1_and_observation`, `public_on5_refusals_local_final_and_conflict_preservation` and `public_on5_release_watch_prepared_closed_real_in1_and_observation` passed in S9's final five-test filter. [S9](s9-on5-closeout-report.md#round-1-focused-validation) |
| Real shared checks | `readonly_gates_two_phases_real_checkers_and_independent_process` and `readonly_gates_proposal_runs_every_a_checker` passed with all substantive A-check refusals and independently initialized child authority. [S10](s10-readonly-gates-report.md#focused-validation) |
| Truthful insufficiency | `e7_connected_truthful_insufficiency_refuses_on4_and_both_on5_entries` passed, retaining actual empty E, eleven unverified obligations and unqualified publication, then unchanged-tree acceptance refusals. [S12](s12-e7-connected-proof-report.md) |

### Lead-gate reconciliation

The lead supplied these local/offline gate results from an unrestricted host. They supersede
worker-report execution limits; they are not new runs performed for this documentation repair.

| Slice / revision | Lead-gate result |
| --- | --- |
| S11 full gate, `c9920d58` | Ordinary `cargo test -p xtask --tests`: 1767 passed; declared xtask library leg: 702 passed. Failures were limited to the two `depth_evidence::native::context` ForeignLoader tests (host flakes), EAGAIN (`os error 11`) under load, and the stale-packet checkpoint repaired in `8a34a55b` |
| S11 focused after fixes, including `8a34a55b` | `isolated_maintenance::tests::mediation_matrix` 7/7 in 2670s, including both long connected tests; `depth_routes` 4/4; `workspace_mutation` 10/10; `depth_refusal_routes` 29/29 |
| S12 full gate, `95bc9706` | Ordinary gate: 1780 passed. Failures were limited to the same ForeignLoader pair and EAGAIN host flakes, plus five `create_candidate` tests affected by the slice-11 ON2 dry-run persistence-guard regression, repaired in PR #369 (`994b28ab` / `b272e8ab`) |
| S12 focused, `b272e8ab` | `create_candidate` 17/17; `runtime_follow_on` 50/50; `on2_dry_run` borrow refusal 1/1; `e7_chain` 6/6; `acquired_inputs_tests` 10/10; `depth_refusal_routes` 29/29 |

The S11 7/7 result includes `connected_onboarding_mediation_and_admitted_writers` and
`connected_on4_aggregate_and_restoration`, including the ON4 preparation/gate/rollback body.

Earlier FR1–FR3/S6–S10 PRs likewise merged after lead full gates. Worker sandboxes could
not open sockets: their socket-blocked/unfinished/unverified execution labels are historical
provenance only, superseded by these lead results for the implementation checks, including
MA5/MA6, FR1–FR3 IN1, mediation, aggregate and restoration coverage. The independent
protected-host phase C run remains required and unauthorized. These local/offline gates do
not substitute for real subprocess/producer/artifact issuance, protected integration or native
queue admission/readback proof.

## Refusals and their guards

The public binary tests run actual commands and aliases. The library matrices cover
lower/cached apply boundaries and compare complete caller trees. S11's final results
include 29 binary passes in both ordinary and depth-declared build checkouts.
[S11 completed focused validation](s11-mediation-matrix-report.md#focused-validation-after-review-fixes).

| Refusal | Current guard | Executed evidence |
| --- | --- | --- |
| Unsupported ON forms across declared/omitted/disabled/orphaned/removed-record carriers and foreign approval | `crates/xtask/src/depth_routes.rs:15`, individual public dispatch, production parser and captured route failures | `every_on_entry_refuses_onboarding_carriers_without_scope_authority` |
| ON2 dry-run persistence, including unrelated depth and foreign approval under maintenance authority | `crates/xtask/src/runtime_follow_on.rs:190` guards both persistence modes before context building, installed own-subject dry-run exemption is checked explicitly | `on2_dry_run_guards_persistence_for_foreign_approval_and_keeps_ordinary_delivery`, `on2_dry_run_depth_approval_cannot_borrow_a_maintenance_route`. These record PR #369's guard repair |
| MA1/MA2 intake/cached apply and later ordinary displacement, MA3 cached refresh, MA4 public relay, both MA5 entries/lower writer and MA6 promotion against onboarding | `agent_maintenance/prepare/guard.rs:194`, `refresh.rs:212`, `depth_acceptance/relay.rs:8`, `manual.rs:163`, `promotion.rs:121`, all under `crates/xtask/src/` | `ma_intake_and_apply_refuse_onboarding_and_later_version_displacement`, with real ordinary siblings |
| NE1 complete-bundle write | `crates/xtask/src/support_matrix/publication.rs:130` workspace refusal, production loader may refuse earlier | `every_ne_command_and_alias_refuses_onboarding_carriers` |
| NE2 generating scope and explicit destination, traversal, directory/file alias, proposed record/registry and intermediate directory creation | `crates/xtask/src/capability_matrix.rs:29`, `depth_routes/destination.rs:80`, `:124`, `:136` | Same neutral matrix, `ne2_proposed_record_destinations_refuse_before_writing`, `ne2_explicit_output_cannot_hide_a_nested_onboarding_workspace`, `ne2_registry_output_refuses_before_replacing_bindings`, `ne2_registry_traversal_refuses_before_replacing_bindings`, `ne2_registry_directory_alias_refuses_before_replacing_bindings`, `ne2_missing_prefix_traversal_cannot_hide_a_nested_onboarding_workspace` |
| NE3 metadata/status and historical alias | `crates/xtask/src/manifest_version_metadata.rs:194` manifest guard | Same neutral matrix, including subject/ordinary/nested roots and alias root omission |
| NE4 fix/current/pointers and historical alias | `crates/xtask/src/manifest_validate/fix_mode.rs:5` manifest guard | Same neutral matrix |
| NE5 prune/retained record removal and historical alias | `crates/xtask/src/manifest_retain.rs:68` guard before pruning | Same neutral matrix |
| NE6 all three legacy snapshot writers | `crates/xtask/src/codex_snapshot.rs:82`, `claude_snapshot.rs:86`, `opencode_snapshot.rs:92` enclosing destination guards | Same neutral matrix, existing and nonexistent nested destinations |
| NE7 backfill all/ordinary agent | `crates/xtask/src/historical_lifecycle_backfill.rs:218` and guarded lower storage | Same neutral matrix |
| NE8 runtime advertising write | `crates/xtask/src/runtime_advertising/mod.rs:86` workspace guard | Same neutral matrix and `readonly_gates_generation_and_neutral_writers_keep_their_controls` in S10 |
| LW1 batch, removal, lifecycle/packet, nested enclosing scope and proposed records/registry | `crates/xtask/src/workspace_mutation/delivery.rs:4`, `agent_lifecycle/storage.rs:54`, `depth_routes.rs:57` | `onboarding_retained_and_proposed_lower_writer_matrix`, `outer_callers_refuse_enclosed_depth_destinations`, `ordinary_lower_writers_deliver_literal_bytes` |
| ON acceptance outside owned generation, pointers/status/passed target/marker/drift, debt/work/classification, wrong path/Event | `crates/xtask/src/depth_acceptance/onboarding/` owning binding, adequacy and aggregate checks | `aggregate_effect_allowlists_refuse_pointers_status_passed_targets_and_marker_retirement`, `outside_generation_effect_and_foreign_entry_refuse_the_entire_batch`, `classification_carried_work_and_debt_refuse`, `maintenance_and_onboarding_authorities_cannot_substitute`, completed S7 predicates |
| Raw foreign advertising without a retained record | `crates/xtask/src/depth_acceptance/onboarding/advertising.rs:6` checks additions before record filtering | `foreign_advertising_without_records_refuses_raw_additions`, completed S7 |
| Frozen acquisition drift/deletion | `crates/xtask/src/prepare_publication/acquired_inputs.rs:17`, captured O/currentness | `publication_entrypoints_refuse_each_changed_frozen_acquired_input_without_writes`, `publication_entrypoints_refuse_each_deleted_frozen_acquired_input_without_writes`, completed S12 body-level matrix over eight literal files |
| Missing/forged/stale E or entry, exact artifact/runtime/settlement retargeting, closed semantic/timestamp edits and same-version maintenance transfer | `crates/xtask/src/depth_acceptance/integration/onboarding.rs:20`, shared ON binding/runtime and semantic loaders | `each_due_obligation_missing_has_its_own_bounded_refusal` in S7, `public_on5_refusals_local_final_and_conflict_preservation` in S9 |
| Gate host/parent/source drift or changed committed candidate after local pass | `crates/xtask/src/readonly_gates.rs:17`, `:41`, `:57` and closing route/source observations | `readonly_gates_two_phases_real_checkers_and_independent_process`, `readonly_gates_closing_observations_refuse_host_and_parent_drift` in S10 |
| Forged executor baseline and interruption after substantive output | `crates/xtask/src/candidate_execution/create_lane.rs:50`, `:262` unconditional observation/finalization | `e7_forged_serialized_baseline_cannot_hide_a_snapshot_edit`, `e7_interrupted_offline_executor_retains_only_trusted_failure_diagnostics` in S12 |

## Private delivery and restoration

Paths in the following tables are relative to `crates/xtask/src/`. The source scan
followed every `apply_admitted_mutations` call, including imported `apply_mutations`
aliases, and both public lifecycle/packet callers of raw `write_json_file`.
This reconciles the S11 inventory against the actual enforcement tree.

| Delivery caller | Owning admission before mechanics | Result anchor |
| --- | --- | --- |
| `workspace_mutation/delivery.rs:4`, call at `:28` | Workspace/enclosing/proposed-output guards, including removals | Completed S11 lower matrix |
| `agent_lifecycle/storage.rs:36`, callers `:54`, `:83` | Typed lifecycle/packet checks, public workspace/output guards, preserved depth object | Completed S11 literal lower-writer tests |
| `agent_maintenance/prepare/guard.rs:194`, call at `:232` | Exact plan rebuild, reporting effect check, input/index/output binding and final route recheck | Completed FR2 MA1/MA2 writers and S11 wrong-owner matrix |
| `freeze_transition/delivery.rs:84` | ON2 dry-run approval/derived freeze/reporting proposal/view/route checks | Completed S11 ON2 and S12 primary chain |
| `candidate_execution/candidate.rs:164`, call at `:241` | MA4/ON2 unconditional observation, candidate/Git/envelope/frozen-input and reporting admission, exact index publication | Completed S12 ON2 chain and failure negatives; MA4 implementation checks are covered by the lead full gates above |
| `candidate_execution/create_lane.rs:88`, `:316`, `:326` | Authenticated ON2 attempt, exact owned attempt/original substitutions under index lock and captured-route recheck | Completed S11 public ON2 and S12 chains |
| `agent_maintenance/closeout_proposal.rs:274`, call at `:289` | Public apply at `:258` supplies exact manual-closeout admission, captured snapshots/view | S7 maintenance predicates and lead full gates completed the local MA5 controls |
| `depth_acceptance/publication.rs:521`, call at `:532` | Both MA5 writers/lower CloseoutPlan apply, exact closeout/marker/P/O/E/entry/scoped publication and route recheck | S7 maintenance predicates, S11 wrong-owner matrix and lead full gates completed the local controls |
| `manifest_promotion.rs:49`, call at `:63` | Exact clean validated HEAD/protected tip, snapshot, maintenance ownership/evidence/entry and fresh apply admission | S7 maintenance predicates, S11 ordinary promotion sibling and wrong-owner refusal |
| `depth_evidence/staged.rs:374` | Invocation-bound MA4 offer/channel, bundle identity/Event/P/O, allowed reporting paths and exact bytes/digests. Relay/IN1 independently authenticate E | MA4 channel and MA5/MA6 local controls are covered by the lead full gates above. Independent protected-host channel/artifact issuance remains phase C |
| `repair_runtime_evidence/routed.rs:296`, call at `:309` | Protected runtime-integrated generation, exact contract/inventory/digest, frozen acquisition, record/entry preservation and reporting batch, closing view/route observations | Completed S11 repair sibling and S12 repaired qualified chain |
| `prepare_publication/routed.rs:195` | Authenticated E/files, complete aggregate reporting/relay proposal and closing view/route recheck | Completed S6 ON3 siblings |
| `publication_refresh/routed.rs:216`, call at `:235` | Exact current Publication entry, full fresh post-entry/aggregate admission, real proposal A checks, S/T runner and closing observations | Completed S8 IN1 positive and S12 qualified chain with the documented S/T seam |
| `depth_acceptance/onboarding/on5.rs:126`, call at `:145` | Prepared exact reporting/relay checks or final proving/aggregate admission, each with captured view/route rechecks | Completed S9 five-test ON5 filter |

ON1 bootstrap uses guarded public mutation delivery. Test-only hook/failure variants
at `workspace_mutation/delivery.rs:32` and `:50` retain the same owning controllers.
Scratch materialization in `depth_acceptance/onboarding/on5.rs:115`, ON3/ON4 views,
freeze predecessor data views and candidate tree copies is scoped to disposable views
and checked path jails. Resulting caller writes cross the inventoried owner again.

| Restoration writer | Guard and scope | Executed evidence / limit |
| --- | --- | --- |
| `prepare_publication.rs:385`, raw lifecycle restore `:414` and temporary packet cleanup | Ordinary ON3 outer workspace refusal at `:121` precedes multi-step writer. Routed ON3 uses one admitted batch | Completed S11 public ON3 refusal and S6 routed ON3 |
| `publication_refresh.rs:499`, replacement and newly created output removal | Ordinary ON4 outer guard at `:206`, replacements also cross LW1. Routed ON4 uses shared expected-before rollback | Completed S11 public ON4 refusal and connected ON4 aggregate/preparation/restoration in the lead mediation matrix (7/7, 2670s) |
| `repair_runtime_evidence.rs:183`, `:198`, `:354` backup/lifecycle restoration | Ordinary repair outer guard at `:115`, routed repair uses its admitted batch | Completed S11 repair sibling includes delivery failure/rollback |
| `candidate_execution/create_lane.rs:308`, writes at `:316`, `:326` | Recheck substitutes exactly its owned attempt/original bytes under caller index lock, reinstating attempt after checked outcome | Completed S11 ON2 and S12 primary chain |
| `workspace_mutation.rs:509`, immediate backup restoration `:544` | Just-observed expected-before source, retained backup, refusal if destination appeared before immediate restoration | Ten S11 mechanical writer tests passed. Retained backup remains on conflict |
| `workspace_mutation.rs:581`, rollback/removal/directory cleanup through `:683` | Every admitted caller retains its owner. Compare delivered bytes/modes/kinds, preserve replacements/removals/mode/link changes, continue other owned restoration and retain recovery bytes | `public_on5_refusals_local_final_and_conflict_preservation` completed all four actual partial-delivery siblings in S9 |
| `candidate_execution/delivery_index.rs:246` | Unpublish precisely the introduced evidence entries under index lock, preserve intervening caller index changes | Shared MA4/ON2 delivery, S11 mechanical/index controls retain original assertions |
| `candidate_execution/delivery_index.rs:364`, restoration at `:384` | Private independent candidate index temporarily receives captured caller index plus admitted evidence for checking, then restores captured executor index. Candidate path jail bounds both writes | Candidate validation exercised by completed ON2 chain. This restoration operates on private candidate metadata |
| `agent_maintenance/execute/index_lock.rs:78`, drop cleanup `:82` | Exclusive caller index lock, private work index and atomic install. Drop removes its own work/lock files | Shared MA4/ON2 index protocol. Local index serialization scope is explicit |

## Serialization domains

| Domain | Writers and guard | Final authority |
| --- | --- | --- |
| Protected integration branch | All routes, HE1 and aggregate effects. `depth_acceptance/integration.rs:172` compares old/new roots and each owner on exact candidate/base | Native tip-conditional MERGE plus required checks/terminal gate. `.github/workflows/ci.yml:623` invokes the existing gate. Independent live queue/readback remains phase C |
| Maintenance packet branch | MA1/MA2 transport, MA3, MA4, MA5. Rebuilt plan, stand-down and bound source/output/index checks. `agent_maintenance/prepare/guard.rs:194` | IN1 still evaluates exact integration result |
| Onboarding branch candidate | ON1–ON5 including dry-run, repair and prepared closeout. Captured approval/registry/route/view, immutable acquisition and expected-before output checks | Local delivery prepares branch state. Independent committed checks and IN1 precede protected acceptance |
| Promotion candidate | MA6 pins validated HEAD/protected tip, exact post-entry rendering, marker/evidence/debt and fresh apply admission at `manifest_promotion.rs:49` | IN1 plus native queue |
| Aggregate publication | Every support/runtime/capability renderer and hand edit. Fresh exact-record derivation and all touched destinations at `depth_acceptance/integration.rs:231` | All-root integration admission |
| Local file batch | All public/admitted mutation callers. `workspace_mutation.rs:248` reclassifies each expected-before mutation, stages jailed temporary files and conditionally restores owned writes | Best-effort conflict-preserving mechanics, with the owner's semantic admission |
| Caller/candidate index | MA4/ON2 evidence publication, attempt recheck, private candidate validation. `candidate_execution/delivery_index.rs:209`, `:246`, `:364`, existing index lock | Exclusive local index protocol. Protected ref serialization remains native queue work |
| Producer/admission/reporting transport | Existing collector, reporting dispatcher, IN1 emitter. `depth_ci/reporting/dispatcher.rs:5`, `depth_acceptance/admission/emitter.rs:272` pin exact job/attempt/artifact/context and closing observations | Independently authenticated retained host facts. Deployed emitter isolation/retention/transport remains phase C |

The existing final-admission requirement still applies to any final gated sequence outside
IN1. Local file/index guards provide their documented bounded conflict handling. Crash-atomic
multi-file delivery and a cross-route filesystem lock remain outside the implementation claim.

## Per-root adoption and all-root D7

`RULES.json` names the canonical record path and the root's validator spec adopts Annex B.
Each retained root also requires full manifest validation. Codex, Claude Code and OpenCode
currently adopt it and have the fixed CI loops. Aider and Gemini CLI retain schema-free,
unadopted specs. `readonly_gates.rs:86` selects required validation from the registry inventory,
with exact exemptions for those two unadopted root pairs and full checking if depth/partial
prerequisites appear.

`manifest_validate/depth/merge.rs:44` additionally invokes all-root IN1 before requested-root
narrowing. `depth_acceptance/integration.rs:172` compares the old/new registry union. An
ordinary requested root therefore still reaches retained foreign-root admission. Per-root
adoption/complete checking and D7 are independently required. CP-P5 must replace the fixed
root loops with registry-derived coverage including the disposable subject.

## Proof limits and phase C handoff

The completed E.7 primary chain is explicit-deferral, wrapper-only and debt-free. Its final
state has eleven obligations, two exact owning entries, pointers at `none`, snapshotted
metadata and zero runtime/capability mappings. The release-watch sibling has a pre-freeze
authority-setup seam. Its bounded settled ON2→ON5 claim is documented in S9/S12.
The fake authenticated host models producer outcomes and terminality using full-schema
artifacts. ON2 copies substantive source/tests. ON4's test S/T runner supplies successful
subprocess outcomes, while real shared A checks and real IN1 execute separately.

The lead-gate reconciliation above records completed S11 mediation (7/7, 2670s), including
both long connected tests and ON4 preparation/aggregate/restoration, alongside completed S9
conflict preservation and S12 writer chains. The unrestricted-host full gates supersede the
worker-only limits on MA5/MA6, FR1–FR3 IN1 and evidence assertions. The enumerated host
flakes and repaired checkpoint/create-candidate regressions are recorded there separately
from the independent phase C proof.

S12's eight-input deletion matrix exercises isolated entry bodies. Public dispatch's earlier
declared-depth refusal leaves deletion-specific public diagnostics unproved. Its supplementary
Windows test build encounters existing failures outside the new gated modules. The isolated
module-mount regression passed. These limitations remain explicit in Annex A.4 and the test list.

Phase C remains closed. Explicit maintainer authorization is required for the disposable public
repository, protected baseline and read-only credential REFERENCE. The canonical `ci.yml`
producer/consumer layout amendment, recursive workspace-test leg, trusted reporting branch and
ordinary scheduling/latency need review. The independent run must prove protected integration,
canonical workflow source, real full subprocess gates, producer terminality, independently
issued full-schema artifacts/adoption and native queue admission/readback. Its detailed
checkpoints and retained evidence are in the [phase C handoff](support-depth-isolated-proof-plan.md#phase-c-handoff-after-slice-13).

## Validation

### Original slice 13 checks (historical)

The following checks were recorded before this documentation repair. Every original command
used `env -u TMPDIR`.

| Check | Result |
| --- | --- |
| Starting branch/revision/status | Clean `feat/s13-final-inventory`, exact expected integration revision |
| Makefile doc/spec target scan | `rg -n -i 'doc\|spec\|adr' Makefile` found ADR hash check/fix targets. The named contract/list docs have no dedicated text consistency target. Selected applicable validator/spec targets below |
| `make hygiene` | Passed |
| `cargo test --offline -p xtask --test validator_spec_roots --test c0_spec_validate` | Passed, three validator-spec/root-adoption tests and 83 contract/manifest validation tests |
| Test-name existence | Passed. All 70 exact S6–S12 status rows have current function definitions. Each of 142 individually cited function names across the changed docs passed `rg --glob '*.rs'` with a `fn` definition pattern. Grouped inventories were included in extraction |
| Current source citations | Passed. Literal file/line anchors resolve in the enforcement tree, with abbreviated module paths checked against their stated source roots |
| Relative links and normative requirement preservation | Passed. All 230 local file/anchor links resolve. Required MUST/SHALL/SHOULD/MAY clauses in the three canonical docs retain their required behavior, including CP-P3 runtime-byte preservation and explicit labeling of host/gate seams |
| `cargo test --offline -p xtask --test c4_spec_ci_wiring --test depth_record` | Passed, 45 CI/transport/gate tests and 37 canonical record/binding/digest tests. The four focused targets total 168 passes |
| `git diff --check` and final scope/status | Passed. Exactly seven named Markdown files changed, all uncommitted on the expected branch/revision |

The original focused checks required no fixture update and covered the seven named Markdown
deliverables above. The lead retains commit ownership.


### Round 1 documentation repair

This repair started from clean `feat/s13-final-inventory` at `c192a506` and changes only
this report, contract Annex A/A.4 and the validation list. The lead-gate reconciliation
supersedes worker-only execution limits, the cached-closeout claim is removed, and the
persistence/aggregate anchors now point to `runtime_follow_on.rs:190` and
`depth_acceptance/integration.rs:231`.

The case-insensitive `socket|unfinished|blocked|unverified` audit of all three changed docs
retains only historical sandbox provenance, contract result values/qualification rules,
required behavior/design questions, negative scenarios (including eleven empty-E obligations)
and exact test names. No worker execution limit remains an outstanding implementation claim.
The independent protected-host phase C run remains required and unauthorized.

`git diff --check` and `make -s hygiene` passed. No Cargo command ran for this repair.
All three Markdown changes remain uncommitted; nothing was pushed.
