# Support depth: onboarding routing and complete mediation

Status: proposed. Every OR-P row below is **Proposed — awaiting maintainer decision**.
This is a design for roadmap E.6, E.7 and E.8, not approval to implement or enable them.
Source baseline: a merge of PR #344 at `c33360ed` (onboarding slices 0–6) and PR #346 at `c6d75418`
(B.7a/B.7b), treated as staging after both land. All source citations refer to that tree.

The [support-depth contract](../../specs/support-depth-contract.md),
[onboarding charter](../../specs/cli-agent-onboarding-charter.md#support-depth),
[registry contract](../../specs/agent-registry-contract.md#support-depth) and
[evidence contract](../../specs/support-depth-evidence-v1.md) remain normative.
The approved [OF-P1–P9 decisions](support-depth-onboarding-freeze-decisions.md) and
[CP-P1–P6 decisions](support-depth-closeout-promotion-design.md) are the starting point.
Where a recommendation needs a normative amendment, that amendment must land separately before
the behavior. This document changes no contract, code, registry, profile or host setting.

Paths are repository-relative. In citations, `xtask/` means `crates/xtask/src/`;
`contract` means `docs/specs/support-depth-contract.md`; `charter` means
`docs/specs/cli-agent-onboarding-charter.md`; `evidence` means
`docs/specs/support-depth-evidence-v1.md`. The charter is under `docs/specs/`, not
`docs/agents/lifecycle/`.

## Inventory before decisions

### ON1–ON5: what exists

These are source findings, not claims of successful connected execution. In particular, a
refusal test that reaches the next loader does not prove the rest of that route.

| Route | What the public entry already does | Existing core and its limit |
| --- | --- | --- |
| ON1 — `onboard-agent` | Write mode refuses a declared or retained depth workspace, then ordinary planning uses the production registry parser (`xtask/onboard_agent.rs:303-325`). A depth approval is independently refused by the scaffold (`xtask/onboard_agent/preview/governing.rs:26-54`). | The ordinary plan writes registry/workspace/release docs, an `enrolled` lifecycle state, governing scaffold, `current.json` and its written-path inventory (`xtask/onboard_agent.rs:488-550`; `xtask/onboard_agent/lifecycle.rs:27-60`). Pointers begin at `none`; RULES deliberately names no depth record and adopts no Annex B (`xtask/onboard_agent/preview/governing.rs:1-10`, `:57-98`). It writes no exact version metadata. There is no private depth-positive ON1 scaffold comparable to ON2's freeze core. |
| ON2 — `runtime-follow-on --dry-run` | The public entry still builds the ordinary full approval context, then writes ordinary run artifacts (`xtask/runtime_follow_on.rs:190-202`, `:302-321`). It does not dispatch to the private freeze. | `isolated_depth_dry_run` is test-only (`xtask/runtime_follow_on.rs:13-23`). Its core captures the installed route, validates the finite non-runtime approval, requires `enrolled`, checks rule 4, complete union and protected-tip acquisition statement, then plans through the shared freeze (`xtask/runtime_follow_on/depth.rs:28-127`). It writes record, lifecycle P/O object and packet together after input/route rechecks (`:220-283`). `support_depth/freeze/onboarding.rs:34-129` resolves onboarding policy, checks the object/Event, derives O, preserves entries/carried work and retains E only for unchanged same-generation O. |
| ON2 — `runtime-follow-on --write` | There is already a conditional isolated branch in the public entry: a full-loader depth approval plus an installed route avoids the outer refusal, and execution selects `depth_candidate::execute` (`xtask/runtime_follow_on.rs:176-188`, `:205-221`). This is partial routing, not a supported synthetic positive: the full loader still requires `backend_module` (`xtask/approval_artifact.rs:253`). | Shared candidate execution captures a controller-held tree, protects approval/lifecycle/record/statement, observes after execution, forbids snapshots and controller-run-file edits, re-observes after validation, and delivers diagnostics or successful source plus trusted lifecycle changes (`xtask/candidate_execution/create_lane.rs:45-180`, `:202-284`). The private synthetic dry-run packet explicitly cannot feed this execution loader: it has no backend and uses a different reporting prompt (`xtask/runtime_follow_on/depth/packet.rs:45-50`, `:69-96`; `xtask/candidate_execution/create_lane.rs:320-380`). A connected synthetic write remains missing. |
| ON2 — `repair-runtime-evidence` | Write mode unconditionally refuses depth; context uses the full approval and production registry loaders (`xtask/repair_runtime_evidence.rs:102-110`, `:428-457`). | Repair already copies the lifecycle's version/P/O identity into the staged contract, without deriving it (`:282-305`, `:460-466`). It promotes a temporary run bundle before lifecycle persistence and restores on failure (`:126-173` and following error handling). This is runtime-evidence repair, not a producer of verified depth E or acceptance. |
| ON3 — `prepare-publication` | Write mode refuses depth (`xtask/prepare_publication.rs:105-114`). The body checks frozen acquired inputs before the full approval and production registry loaders (`:244-260`). | The installed-route-only test adapter enters the same body; it authenticates route presence but supplies no owning admission (`xtask/prepare_publication/isolated_entrypoint.rs:1-14`). Ordinary ON3 selects runtime evidence, writes a packet and advances to `publication_ready` (`xtask/prepare_publication.rs:152-192`, `:271-288`). Packet staging, lifecycle write and raw rename/restoration are separate (`:424-458`). There is no depth E delivery or acceptance core here. |
| ON4 — `refresh-publication` | Write mode refuses depth (`xtask/publication_refresh.rs:192-201`); its body also checks acquired inputs, then uses full approval/production registry loading (`:301-307`). | It renders aggregate support JSON/Markdown/runtime projection and optional capability publication (`:131-174`), refreshes packet continuity, advances lifecycle to `published`, delivers through neutral mutations, then runs gates and restores snapshots on failure (`:229-280`, `:448-486`, `:523-603`). The same private test adapter exists. No onboarding `Publication` entry or admission predicate exists. Presence of an isolated route alone does not admit this batch. |
| ON5 — `prepare-proving-run-closeout` | Write mode refuses depth; check mode continues to context loading (`xtask/prepare_proving_run_closeout.rs:101-117`). | It writes `state = prepared`, docs and next-command lifecycle changes, keeping the lifecycle stage (`:120-151`, `:225-273`). It has no acquired-input check equivalent to final closeout's check. Replacing a previously closed artifact can also retarget an existing lifecycle assertion; `prepared` alone is not a universal reporting exemption. |
| ON5 — `close-proving-run` | Public entry unconditionally refuses depth (`xtask/close_proving_run.rs:96-103`). The private adapter reaches the body, whose acquired-input check precedes the full approval/registry loaders (`:106-134`). | It finalizes `closed` with approval digest, maintenance settlement and timestamp, then writes closeout, docs and lifecycle in separate calls (`:135-141`, `:499-537`, `:605-618`). Lifecycle becomes `closed_baseline`, records closeout evidence, references packet/closeout and clears `drifted` (`:461-490`). There is no onboarding `ProvingRunCloseout` admission. The settlement supports both release-watch and explicit deferral (`:546-601`); it is not a depth-path transfer. |

OF-P6 is already shared by ON3, ON4 and final ON5: `xtask/prepare_publication/acquired_inputs.rs:17-84`
checks onboarding record/object binding and every frozen acquired file's digest. Discovery fails
closed while preserving ordinary approval filesystem behavior (`:87-149`). It grants no authority.
The tests deliberately stop at the missing `backend_module` after unchanged inputs pass
(`xtask/prepare_publication/acquired_inputs_tests.rs:136-149`). They do not prove publication or
closeout success. Runtime evidence separately checks passing status/handoff/nonempty written paths
(`xtask/prepare_publication/runtime_evidence.rs:200-250`) and the lifecycle-bound written-path
digest (`:296-332`). That bundle is not authenticated depth evidence.

### Complete mediation rules 1–7: what exists

The admission predicate requires truthful reporting and retained-record preservation for every
touched tuple; acceptance additionally requires its owning path, current P/O/E and satisfaction
of all due work (`contract:664-686`). Broad refusal is conforming mediation. Merely loading the
isolated registry is not admission.

| Rule (`contract:694-721`) | Existing mechanism and evidence | Remaining onboarding boundary |
| --- | --- | --- |
| 1 — actual changed values determine touched tuples | `xtask/depth_acceptance/mod.rs:81-133` combines manifest, artifact and aggregate publication detectors. `publication.rs:149-192` detects qualification, advertising and runtime effects; `artifacts.rs:275-344` charges semantic closeouts and retargeted lifecycle assertions. | ON4 cannot narrow admission to its approval's agent. Its complete aggregate proposal must be compared, including unchanged qualifications and other agents' rows. |
| 2 — an unenrolled request cannot rewrite an enrolled record | Guards scan declarations, retained records and enclosing roots (`xtask/depth_routes.rs:9-49`), and proposed record/registry destinations (`:55-83`). IN1 preserves old records, prefix entries and selection (`xtask/depth_acceptance/integration.rs:217-228`). Relay preserves Event/P/O/entries (`relay.rs:74-99`). | Execute these guards with onboarding retained records and an omitted declaration, not only maintenance fixtures. ON1 replay must not reset an existing binding. |
| 3 — authority per tuple under owning path | Maintenance closeout and promotion explicitly require maintenance ownership (`xtask/depth_acceptance/manual.rs:177-206`; `promotion.rs:129-146`). Current-binding derivation reads maintenance requests for closeout (`xtask/support_depth/closeout.rs:58-92`). | Reuse common adequacy, not maintenance Event/request/marker authority. No onboarding owning predicate exists. |
| 4 — no acceptance on another path's tuple | IN1 rejects onboarding accepting lifecycle effects and requires maintenance authority even for unchanged qualification (`xtask/depth_acceptance/integration.rs:234-250`, `:281-331`). The isolated scope requires one path and a matching Event on every retained record (`xtask/isolated_maintenance/scope.rs:95-148`). | Keep MA1–MA6 unable to accept onboarding tuples after ON routes gain positives. ON routes must similarly refuse maintenance-owned acceptance destinations. Reporting displacement remains governed by Later versions. |
| 5 — every caller of a lower writer is mediated | Public mutation delivery refuses depth before delegating to private mechanics (`xtask/workspace_mutation/delivery.rs:4-21`). Public lifecycle/packet storage independently refuses (`xtask/agent_lifecycle/storage.rs:54-107`). Private ON2 freeze/candidate paths already use admitted mechanics after their own checks. | Add private checked ON owners; do not relax public LW1 or neutral writers because an isolated route exists. Inventory temporary writes, gate execution, restoration and every admitted-delivery caller too. |
| 6 — omitted optional scope never narrows touched set | Workspace guards do not depend on an optional depth argument; they inspect ancestors and retained records (`xtask/depth_routes.rs:9-39`). IN1 enumerates the union of old/new registered agents before requested-root narrowing (`xtask/depth_acceptance/integration.rs:124-180`; `xtask/manifest_validate/depth/merge.rs:44`). | Add onboarding tests for other roots, an ordinary requested version/agent, proposed records and nested output destinations. Malformed/deleted binding is an error, not absence. |
| 7 — repository validation backs every producer, including hand edits | D7 rechecks changed statements, root adoption, entries, accepting values and aggregate publication (`xtask/depth_acceptance/integration.rs:157-184`, `:206-335`). Generation checking has an onboarding branch, but it still invokes the full approval loader (`integration/generation.rs:40-78`). Artifact attribution already follows exact approval bytes or durable publication/proving entry references (`artifacts.rs:117-181`). | Add onboarding owner dispatch, finite approval validation and freeze re-establishment for reporting changes. New `Publication` and `ProvingRunCloseout` entries currently fail the final unmatched-entry check. CP-P3's maintenance attribution cannot authorize them. |

### Other Annex A routes and delivery

| Routes | Existing behavior and citation | E.8 treatment to decide |
| --- | --- | --- |
| MA1–MA2 | Shared maintenance freeze requires a maintenance Event (`xtask/support_depth/freeze.rs:81`). Isolated registry scope does not itself establish maintenance authority. | Keep refusal of onboarding acceptance; test both intake and apply, including an ordinary later-version request displacing onboarding working files. |
| MA3 | `refresh-agent` apply refuses depth before planning/delivery through the neutral writer (`xtask/agent_maintenance/refresh.rs:212-230`). | Keep refusal, including aggregate onboarding rows. |
| MA4 | Relay checks every registered agent, forbids registry changes, record addition/removal, acceptance effects, frozen-input edits and unsupported debt transitions (`xtask/depth_acceptance/relay.rs:8-99`). | Reporting-only work cannot borrow onboarding acceptance authority. Exercise mixed/retained-path cases without weakening the relay. |
| MA5–MA6 | Current maintenance predicates require their path. CP-P3 is implemented: closeout admits scoped publication (`xtask/depth_acceptance/manual.rs:71-77`, `:154-155`); IN1 attributes a new qualification to an exact new maintenance closeout only without promotion effects (`integration.rs:270-331`). | Preserve B.7 positives. Never use that attribution for ON4/ON5. |
| NE1 | Support bundle delivery refuses depth (`xtask/support_matrix/publication.rs:134`). Rendering can still serve an owning proposal. | Keep neutral delivery refusal. |
| NE2 | Capability-matrix write refuses depth (`xtask/capability_matrix.rs:31`). | Keep refusal; test the real command as well as renderer callers. |
| NE3 | Version-metadata writer refuses depth (`xtask/manifest_version_metadata.rs:194-204`); private planning requires a real union (`:249-265`). | Use ordinary `snapshotted` metadata before declaration; no new metadata authority after declaration. |
| NE4 | Validator fix mode refuses depth (`xtask/manifest_validate/fix_mode.rs:6`). | Keep refusal; retain read-only validation and routed `--tip`. |
| NE5 | Retention write refuses before pruning (`xtask/manifest_retain.rs:76`). | Keep refusal for retained onboarding records, even without declaration. |
| NE6 | Legacy snapshot destinations are guarded for Codex, Claude and OpenCode (`xtask/codex_snapshot.rs:85`; `claude_snapshot.rs:89`; `opencode_snapshot.rs:95`). | Test all three and aliases. Versioned acquisition remains input-only under OF-P3. |
| NE7 | Historical backfill refuses depth (`xtask/historical_lifecycle_backfill.rs:218`). | Keep refusal; historical status cannot authorize lifecycle retargeting. |
| NE8 | Runtime-advertising write refuses depth (`xtask/runtime_advertising/mod.rs:64`). IN1 detects actual advertising through `depth_acceptance/publication.rs:191-192`. | Keep refusal and test source edits that bypass generation. |
| LW1 | Public batch/lifecycle/packet writers refuse declared, retained and proposed depth; lifecycle preservation is separately checked (`xtask/workspace_mutation/delivery.rs:4-21`; `xtask/agent_lifecycle/storage.rs:65-107`, `:113-130`). | Keep these public boundaries. Only an owning proposal may call private delivery. |
| IN1 / HE1 | Authenticated isolated admission/observation binds the parent, borrows the captured host, admits all roots and rechecks binding (`xtask/depth_acceptance/integration.rs:13-51`). Hand edits get no command authority. | Extend owning dispatch, keeping exact candidate/parent binding and the native conditional ref update. |

The installed-route refusal suite already executes ON1–ON4, ON2 repair, prepared ON5, NE1,
NE3/NE4 and public LW1 against a maintenance record and checks the tree is unchanged
(`xtask/isolated_maintenance/tests/unsupported_routes.rs:65-208`). It does not by itself execute
every NE route or binary-only final ON5. Annex A.4 explicitly requires execution evidence at
every actual unsupported boundary (`contract:1116`); source guards alone are insufficient.

The final gated sequence for branch candidates is integration, not local plan delivery
(`contract:735-779`). Existing compare-and-refuse delivery is not a cross-process lock.
The scratch binding already borrows captured registry/Git inputs, forbids scratch Git and grants
no authority (`xtask/isolated_maintenance/view.rs:19-80`, `:84-103`). ON4's current restoration,
however, plans replacement from then-current bytes and separately deletes newly created paths
(`xtask/publication_refresh.rs:549-573`). ON3 also has raw restoration (`prepare_publication.rs:452-454`).
Neither pattern should become a new depth path by removing its outer guard.

### Evidence and ownership limits already established

1. The synthetic selection is wrapper-only `depth_fixture/0.2.0` on one GNU Linux target, with
   eleven due obligations, no capability and no not-applicable permission (`evidence:448-458`).
   The same subject admits onboarding scope but only one path per workspace; its onboarding
   evidence explicitly stays unverified (`evidence:528-555`). `LIVE_PROFILE` is absent
   (`xtask/isolated_maintenance/mod.rs:39`), and fixture installation is thread-local test
   behavior (`:113-121`). No subprocess inherits that installation.
2. `Profile::Synthetic` fixes a maintenance fixture record, test target, receipt namespace and
   executor (`xtask/depth_ci/profile.rs:24-42`, `:68-113`). The verifier authenticates the reviewed
   producer closure and then accepts only a maintenance Event to locate the unique generation PR
   and applicable attempts (`xtask/depth_evidence/verdict/github/mod.rs:90-184`). Matching case
   names or changing a receipt's Event cannot bridge this boundary. Exact Event/P/O also separates
   applicable producer runs (`:170-178`).
3. Shared assembly already produces reporting E and its receipt/payload files from an authenticated
   population (`xtask/depth_evidence/verdict/github/assembly.rs:1-82`). The existing host-taking
   assembly/evaluation entries are private (`github/mod.rs:675-697`). ON2 protects the record
   from its executor, so an executor cannot simply write this E into it.
4. Verdicts are sealed to the complete record, acceptance entries included; appending an entry
   requires fresh evaluation (`xtask/depth_evidence/results.rs:6-38`). Production verdicts stay
   empty (`:42-48`). B.7's view evaluator borrows the caller's host/objects and reads proposed bytes
   (`xtask/depth_evidence/verdict/github/mod.rs:700-720`), so it can serve another owning path
   only after that path's Event/profile/currentness checks exist. Outside an owning evaluation
   scope, even an installed route/scratch view uses production verdicts; the existing test checks
   that qualified publication then disagrees with ordinary no-`--tip` validation
   (`xtask/isolated_maintenance/tests/publication.rs:60-101`). A route alone does not fix ON4's
   subprocess green gates.
5. Common adequacy code exists in `xtask/depth_acceptance/manual.rs:242-337`: current P/O/E,
   classifications, required work, debt and independently evaluated due obligations. It cannot be
   reused unchanged for onboarding: its host binding reads a maintenance request, and the debt
   planner refuses every non-maintenance Event (`xtask/support_depth/closeout.rs:58-92`;
   `xtask/support_depth/debt.rs:59-73`).
6. Two declarations for one version are forbidden (`docs/specs/agent-registry-contract.md:455-457`).
   The charter describes moving a version to maintenance only after `closed`
   (`charter:467-472`), but current policy resolution refuses a retained path move
   (`xtask/support_depth.rs:250-252`), as does isolated retained-scope checking. That is an
   unresolved handoff, not permission for E.6 to change the Event or discard the record.
7. Acquisition has an existing real synthetic snapshot adapter: it invokes the fixture binary's
   `--surface-json` (`xtask/depth_fixture_snapshot.rs:1-4`, `:68-102`). Today's chain helper instead
   constructs the shard directly, then invokes real union/report generation
   (`xtask/isolated_maintenance/tests/prepare.rs:143-186`). Reusing the helper is not proof that
   snapshot acquisition ran. The positive also needs maintainer classification of the acquired
   surface: the existing unclassified helper correctly leaves `classification_required` rows
   (`:207-219`); the new chain must not silently clear them.
8. A record/declaration change can change support facts before qualification: versions/targets
   include declarations and records (`xtask/support_matrix/derive.rs:306-325`), and published
   obligation results are independent of the current-entry qualification test
   (`xtask/support_matrix/depth.rs:330-378`). IN1 requires aggregate consistency even on reporting
   transitions. Maintenance preparation already renders a coherent reporting bundle and checks
   that it has no acceptance effects (`xtask/agent_maintenance/prepare/depth.rs:202-228`);
   onboarding freeze currently does not. A future connected proof must address this, not bypass
   publication validation at its reporting checkpoints.

## Decisions for the maintainer

Each row has one recommendation. Alternatives are explicitly not recommended. Approval of any
row would authorize its later design/implementation slices only, not production enablement.

| Id | Recommendation | Grounds / boundary; alternatives | Status |
| --- | --- | --- | --- |
| OR-P1 — bootstrap and rule-4 ordering | Run ON1 before enrollment, preserving its rule that it writes no `support_depth` table. Extend the existing installed synthetic route with a finite pre-enrollment bootstrap state for ON1 only: the subject entry/root may be absent, or the entry may be the exact unenrolled scaffold; no retained record or acceptance is admitted. Fix the expected descriptor from compiled synthetic templates and bind the exact approval; neither supplies declaration authority. Admit only the exact absent-to-scaffold registry delta at bootstrap integration, with no depth table. After scaffold, land reviewed Annex-B governing files, direct acquisition and truthful `snapshotted` metadata through existing NE3 **before** the maintainer adds the onboarding declaration; then freeze through ON2. | Charter rules 2–4 (`charter:285-325`) distinguish scaffold, declaration and freeze. ON1 cannot invent acquired metadata (`xtask/onboard_agent/preview/governing.rs:32-52`), and NE3 refuses after declaration. Current route scope requires an already protected subject entry/table (`xtask/isolated_maintenance/scope.rs:75-110`); it cannot represent a genuinely new ON1. A bounded additional state in that same source-installed route is necessary; no new command, file kind or enrollment authority is needed. This needs an evidence-contract bootstrap amendment before code, including the narrow exception to protected-registry equality for that exact ordinary bootstrap delta. **Not recommended:** preseed the entry, lifecycle and record and call that an ON1 proof; it skips the route. **Not recommended:** let ON1 install the table or claim `validated`; registry rule 9 and charter rule 11 forbid it. | Proposed — awaiting maintainer decision |
| OR-P2 — non-runtime synthetic orchestration | Extend the existing finite synthetic approval projection into a private adapter shared by routed ON1–ON5 and IN1. Keep this subject non-runtime, capability-free and wrapper-only. Adapt runtime-follow-on evidence/handoff checks to the wrapper work actually due, while preserving the existing lifecycle stages and record/packet kinds. | The template deliberately omits `backend_module`; the private reader compares the whole descriptor (`xtask/runtime_follow_on/depth/approval.rs:1-82`), whereas the full loader requires it. The private packet and execution loader are currently incompatible. The amendment must name the bounded wrapper-only interpretation of checklist implementation/evidence steps (`charter:194-207`); no claim of adapter qualification follows. **Not recommended:** add a fake backend or fake `agent_api` test to satisfy the loader; the synthetic profile maps no M obligation (`evidence:454-458`). **Not recommended:** make production approval loading generally permissive; that exceeds this proof. | Proposed — awaiting maintainer decision |
| OR-P3 — public routing and failure behavior | Route existing public ON forms through captured isolated authority and their owning cores; an installed route failure is an error, and absence retains production refusal. Keep private LW1 mechanics behind checked proposals. Validate proposed ON3–ON5 batches and their gates in an independent candidate before caller delivery; use expected-before, conflict-preserving delivery/restoration. | Reuses `Route::capture`, candidate execution, `Proposal`/`IsolatedView` and admitted mutation mechanics (`xtask/isolated_maintenance/mod.rs:64-101`, `:140-170`; `view.rs:41-80`). The public forms include ON2 dry-run/write/repair and both ON5 commands, not just the five primary names. Existing ON3–ON5 bodies have raw or multi-stage delivery. **Not recommended:** remove outer guards or make route presence a general lower-writer bypass. **Not recommended:** run gates after accepting caller writes and restore arbitrary current bytes; ownership may have changed. Final serialization remains IN1; no filesystem lock or crash-atomic transaction is claimed (`contract:746-779`). | Proposed — awaiting maintainer decision |
| OR-P4 — what onboarding accepts | ON4 owns only its generation's `Publication` entry, exact packet-bound `published` assertion and supported qualification/advertising effects. ON5 final close owns `ProvingRunCloseout`, exact `closed` closeout, `closed_baseline` and its proving evidence id. ON3 and prepared ON5 remain reporting-only and refuse any detected acceptance/retargeting. Neither path gains pointers, passed target outcomes, `validated`/`supported` status, marker retirement or maintenance drift clearance. | These effects already exist in `charter:454-472` and `contract:636-660`; Annex B already has both entry kinds (`contract:1436-1442`). Even an unverified support row does not make `published` reporting-only. Use CP-P3's coherent entry → fresh post-entry evaluation → render → re-admit pattern without its maintenance authority. Aggregate reporting may update other rows truthfully; any acceptance destination outside the owned generation refuses the entire ON batch. **Not recommended:** create an onboarding promotion or borrow MA5's entry; no authority exists. **Not recommended:** allow `published`/`closed` with insufficient depth; predicate items 5–6 still apply. | Proposed — awaiting maintainer decision |
| OR-P5 — E.7's evidence claim | Aim for a qualified **synthetic wrapper promise** on the onboarding Event. First amend the evidence contract to add a finite onboarding variant of the existing synthetic producer binding; reuse its fake CLI, cases, target, obligations, receipt schema and terminality policy. Keep maintenance and onboarding Event/P/O populations separate, with fresh evidence/adoption for onboarding. | A truthful-insufficiency test is valuable but cannot complete approval → publication → closed closeout under `contract:824-835`. Today `evidence:550-555` explicitly prevents the positive. Minimal producer work must bind the actual approval Event and frozen onboarding P/O in its fixture/witness, authenticate its approval-linked PR and complete applicable attempt population, then evaluate only matching receipts. Update the compiled fixture selection, collector/archive/verifier and fixture identities together; use the existing `rust-tests` job, no new command or live profile. **Not recommended:** relabel maintenance evidence or permit cross-Event reuse; maintenance evidence does not substitute. **Not recommended:** call an unverified/refused chain the complete E.7 proof. Without approval of this amendment, only the insufficiency/refusal proof can ship and E.7 stays incomplete. | Proposed — awaiting maintainer decision |
| OR-P6 — who delivers onboarding E | Make routed ON3's controller assemble and deliver reporting E plus bound receipt/payload files and coherent reporting publication in the same checked proposal as `publication_ready` and its packet, after committed ON2 implementation has producer evidence. Preserve Event/P/O and every entry. Repair continues to repair runtime evidence only. | The existing private host-taking assembly can serve; ON2's executor cannot write its protected record. ON3 is the existing runtime-evidence-selection handoff before ON4, so no new command/file kind is needed. This does amend the checklist's present ON3 write scope (`charter:208-213`) and Annex A: that change must precede code. The dry-run freeze similarly includes any derived reporting publication needed for coherence, reusing the maintenance preparation pattern; neither writer may add qualification or another acceptance effect. Every consumer still re-authenticates; assembly is a claim, not a receipt granting admission (`xtask/depth_evidence/verdict/github/assembly.rs:1-16`). **Not recommended:** let the executor edit the record, or make repair manufacture verified E. **Not recommended:** manually prefill E in the chain fixture; it omits the owning reporting writer. | Proposed — awaiting maintainer decision |
| OR-P7 — admission, currentness and debt | Add private onboarding publication/proving predicates that share path-neutral adequacy logic with maintenance, but independently bind the exact approval Event, lifecycle object, registry policy, O acquisition and candidate bytes. Keep the synthetic positive debt-free; refuse onboarding debt grants/retirements or baseline/inventory drift until corroboration is designed. | `manual::admitted` already supplies the due-work/evidence model, but its current binding and debt calls are maintenance-specific. Reuse shared policy/O derivation without performing a freeze or rebasing P. Check every due obligation, classification and carried work; empty onboarding `acceptance_work` is not an exemption from depth (`contract:1423`). No packet marker is required on the create lane (`charter:464-466`). **Not recommended:** call the maintenance predicate with a forged request/marker. **Not recommended:** remove its lifecycle checks or skip debt/currentness because the initial fixture is empty. A shared extraction must preserve maintenance behavior. | Proposed — awaiting maintainer decision |
| OR-P8 — D7 attribution | Dispatch IN1 by each retained tuple's owning path and actual candidate artifact/effect. Independently re-establish onboarding freezes and reporting E, ON4 publication and ON5 proving acceptance through the same predicates as local preparation. Require each new entry to match the independently admitted exact packet/closeout and current record digest; re-evaluate unchanged qualifications too. | Existing detectors already attribute through approval bytes or durable exact file references (`xtask/depth_acceptance/artifacts.rs:117-181`); entry locators themselves grant no authority. Replace the blanket onboarding-effect refusal only for admitted cases. Separate `Publication` effects from CP-P3 closeout/promotion classification; its fallback otherwise misclassifies ON effects (`integration.rs:270-331`, `:338-381`). **Not recommended:** trust command names, workflow names, E labels, a copied entry or an earlier local result. **Not recommended:** treat every new qualification as promotion once onboarding is supported. All-root checks and admission/observation binding remain unchanged. | Proposed — awaiting maintainer decision |
| OR-P9 — frozen acquisition through publication | Reuse frozen acquisition bytes on ON3–ON5; run generators in check/planning form there, and refuse changed bytes, including regenerated wrapper coverage. Keep complete-union admission and the post-`runtime_integrated` re-freeze refusal. | OF-P4–P7 already decide these boundaries. ON4 renders publication; its fixed required command list uses check modes (`charter:214-223`). Ensure candidate gates cannot replace O inputs, and re-observe after gates. **Not recommended:** exclude wrapper coverage from O or silently re-freeze during publication. **Not recommended:** admit a partial union for this proof; OF's partial-target question stays open and the subject needs one complete target only. | Proposed — awaiting maintainer decision |
| OR-P10 — onboarding to maintenance handoff | Finish and integrate ON5 `closed` before any maintenance handoff. Keep same-version retained path transfer unsupported in E.6–E.8, with an explicit refusal test and a separately required transition design. Release-watch readiness alone transfers no depth authority. | Charter rule 11 describes a future handoff; current resolver and isolated scope cannot execute it. Deleting a record or changing its Event to make it fit loses retained authority. A later ordinary version may use its own lifecycle rules and displace reporting state while preserving the earlier record/work (`contract:723-733`); the fixed isolated subject still refuses an extra depth version. **Not recommended:** overlap both paths on one version, auto-transfer at closeout, or claim this design implements the charter's same-version handoff. Resolve its normative/code gap explicitly before promising that operation. | Proposed — awaiting maintainer decision |
| OR-P11 — E.8's unsupported routes | Retain broad refusal for NE1–NE8, public LW1 and unsupported maintenance writers in onboarding scope. Add executed onboarding declaration/retained/proposed-destination cases at every actual public boundary and alias; keep reporting-only maintenance behavior only where its existing predicate admits the actual touched set. | Complete mediation permits refusal, and the current guard mechanisms cover omitted and enclosing scope without a new inventory (`xtask/depth_routes.rs:9-105`). E.8 needs maintenance-versus-onboarding tests as well as ON positives. **Not recommended:** add a neutral “reporting writer” bypass, narrow aggregate scope to the named agent or equate source inspection with conformance. Lower callers remain individually inventoried. | Proposed — awaiting maintainer decision |
| OR-P12 — proof boundary | Require both a connected in-process installed-route chain and a later independent disposable-repository run under phase C's protected integration/CI conditions. The former is the credential-free E.7 implementation gate; the latter remains required evidence before claiming the contract's independent run/path enablement. | `Fixture::install()` does not exist in a CLI subprocess, and ON4 runs shell gates (`xtask/isolated_maintenance/mod.rs:115-121`; `xtask/publication_refresh.rs:577-603`). In-process tests may use scoped fake host/gate seams but must label those limits. Phase C separately authorizes repository/credential references (`support-depth-machinery-proof-rescope.md:86-88`); CP-P5 owns canonical CI prerequisites. **Not recommended:** call private-core tests or stubbed gates a native queue proof. **Not recommended:** perform the host run or install a live profile as part of this packet. | Proposed — awaiting maintainer decision |
| OR-P13 — authenticated gates before and after delivery | Split ON4's fixed green gate for depth-enrolled generations into two bound phases, with no change to the command inventory itself. **Before delivery**, the controller runs the depth-reading checks behind each fixed command (`support-matrix --check`, `capability-matrix --check`, `capability-matrix-audit` and the validator, support-matrix and `runtime-advertising-model --check` parts of `make preflight`, `Makefile:234`) as in-process calls against the proposal's scratch view, inside the borrowed fresh evaluation scope bound to the exact proposed record. This is the pattern B.7b already uses for MA6 (`xtask/isolated_maintenance/tests/publication.rs`), and failure refuses the whole batch with the caller tree unchanged. **After delivery**, the unchanged subprocess commands run where they can authenticate: a standalone check resolves its own installed route over a clean committed candidate whose HEAD the host already holds (`xtask/isolated_maintenance/host.rs:107-109`; `mod.rs:498-499`), that is, the pushed candidate commit in CI and IN1 at the integration tip. Format, lint and build keep running as subprocesses before delivery. The workspace test leg is not depth-independent: existing tests read the repository registry through the production parser (for example `crates/xtask/tests/c8_spec_capability_matrix_staleness.rs:48`, `xtask/release_doc.rs:97`) or assert that it declares no depth (`crates/xtask/tests/agent_registry/support_depth.rs:65`), so they fail on any declared candidate registry, whatever is routed. Those consumers are inventoried and made either hermetic (an undeclared fixture registry for production-parser controls) or routed (authenticated reading for candidate-registry checks). Every registry reader these checks use must read the routed registry read-only, including `runtime-advertising-model --check` (`xtask/runtime_advertising/mod.rs:54`, `revision.rs:35`), whose production parser otherwise rejects the onboarding declaration in both phases. Keep default generation, neutral write mode, NE8 write refusal and production verdicts unchanged. | A thread-local scope cannot cross `sh`/`cargo` processes, `IsolatedView` forbids Git metadata, and a locally committed proposal is not on the host, so no subprocess can authenticate unpublished proposal bytes without a live host write; candidate hosting would need credentials and a live call, which this design excludes. The in-process phase checks the exact bytes to be delivered with the same functions the commands call; the subprocess phase is the independent check of committed bytes. Charter rule 7's "own ... the required green gate" needs a bounded amendment naming this split for depth-enrolled generations only. A credential-free composition test must show both phases run, and that an in-process pass with a changed committed candidate still fails the subprocess phase. **Not recommended:** make all derivation under a route implicitly trust isolated evidence; B.7 deliberately prevents that. **Not recommended:** skip the fixed gates, serialize a controller verdict into an environment token or reusable receipt, or push candidates to the host to authenticate them. **Fallback, not recommended:** defer this decision and keep the accepting ON4 route unavailable, so E.7 stops at ON3's reporting checkpoint. | Proposed — awaiting maintainer decision |
| OR-P14 — first declaration transition | Admit the maintainer's scaffold-to-declared commit as its own bounded, maintainer-owned reporting transition at IN1: exactly the onboarding `support_depth` table for the scaffolded entry and approval version, its reviewed CORE classification rows, the integration-branch acquisition statement (OF-P4 B) and any coherent reporting publication the declaration changes, with no record, entry or acceptance effect. The installed route captures the declared registry only after that commit is on the protected tip; the scope's protected-registry equality then holds again. ON1 still cannot write the table. | OR-P1's bootstrap exception covers only the depth-free ON1 delta, so without this row the declaration candidate is refused by the existing scope check before IN1 can admit it, and advancing the fake tip first would skip admission at a checkpoint. A declaration is a policy change the maintainer owns (charter rule 2, `charter:276-280`; registry contract `docs/specs/agent-registry-contract.md:455-457`). It needs the same evidence-contract bootstrap amendment as OR-P1, and admission plus readback tests. **Not recommended:** fold the declaration into ON1 or ON2; neither owns registry policy. **Not recommended:** treat the declaration as a hand edit judged only after the fact; IN1 must still admit it, and a named transition makes its bounds testable. | Proposed — awaiting maintainer decision |

## Proposed route composition

This section describes the recommended choices together; none is installed by this document.

1. **Bootstrap, with no declaration or record.** The compiled route's bounded ON1 bootstrap
   captures the protected registry/source and exact synthetic approval. ON1 writes its normal
   scaffold and lifecycle identity, without the depth table, metadata invented from a version
   string, or acceptance. The proof does not preseed those outputs. Reviewed governing adoption
   and real single-target acquisition follow. NE3 records `snapshotted` without passed outcomes
   before declaration. Bootstrap registry reads may serve these ordinary input/planning commands;
   they grant no depth acceptance or ON2 freeze authority. A maintainer then commits the onboarding
   declaration, with reviewed CORE classification rows and the actual acquired classification
   basis, and the acquisition statement at the protected integration tip. ON2 cannot run while the
   approval's enrollment
   remains unresolved. This is an explicit extension to today's one-selection scope, not a
   generic registry parse bypass. Current scope compares registry bytes with the protected tip
   (`xtask/isolated_maintenance/scope.rs:75-80`), so bootstrap IN1 must independently compare its
   exact ordinary ON1 delta instead. After declaration that exception ends. ON1 validates its
   proposed depth-free registry with the ordinary parser; it does not relax the scratch binding
   that requires captured registry bytes to stay identical (`xtask/isolated_maintenance/view.rs:46-60`).
2. **Freeze and implementation.** Public ON2 dry-run calls the existing freeze core and writes its
   actual executable wrapper-only input contract/prompt, not today's reporting-only placeholder.
   It alone freezes P/O and includes any coherent reporting publication needed by the proposed
   record, with acceptance effects forbidden. Public write uses the existing independent candidate boundary; a successful
   implementation records lifecycle-bound runtime evidence. Failed attempts may leave the trusted
   attempt state and controller diagnostics; they deliver no executor changes or acceptance.
   Repair copies identities and preserves P/O/entries; it cannot reopen the freeze window or make
   depth evidence verified. The wrapper implementation and frozen generation are committed before
   producer evidence is collected. Source must satisfy the protected reviewed producer closure;
   a fake host cannot be used to conceal a source/adoption mismatch in the phase C proof.
3. **Reporting E and handoff.** ON3 authenticates the onboarding producer population, assembles E
   and evidence files, checks the selected runtime evidence and acquired-input binding, and delivers
   one reporting proposal, including derived support facts so IN1 sees a coherent reporting tree.
   ON3 lists no entry and produces no qualified publication. Its new
   write scope requires OR-P6's charter amendment. Missing E can be reported truthfully, but cannot
   pass the following acceptance predicate.
4. **Publication.** ON4 finalizes the exact packet bytes first, obtains the current-digest
   `Publication { packet, record_sha256 }` entry from owning admission, and evaluates the post-entry
   record freshly in the bound view. It renders all requested outputs, checks every touched tuple,
   validates the proposed `published` packet reference and runs gates in the candidate. A
   qualified wrapper promise is possible without advancing any pointer or version status.
   The synthetic subject has no runtime family/capability, so projection changes that would accept
   another tuple refuse; production adapter advertising remains unproved by this positive.
5. **Proving closeout.** Prepared ON5 keeps `published` and writes only reporting changes. It must
   not replace the packet that assertion reports, a retained closed closeout or any other accepting
   assertion without owning admission; conservative refusal is sufficient. Final ON5 creates the
   actual timestamped `closed` bytes, preserves human metrics/maintenance settlement, admits the
   `ProvingRunCloseout { closeout, record_sha256 }` entry and lifecycle evidence in one proposal,
   then freshly evaluates and validates the post-entry tree. It preserves acquisition, P/O and all
   prior entries. A depth ON5 with `drifted` refuses rather than clearing maintenance drift; the
   contract attributes that clearance through the actual maintenance closeout (`contract:655-660`),
   which ON5 does not own.
6. **Integration at every checkpoint.** IN1 judges exact old/candidate values across every registered
   root. A reporting-only new freeze must re-establish its approval/P/O object, acquisition
   statement provenance at the parent, and selection/entry preservation; a record assembled by a
   hand edit is not excused by lacking accepting effects. Acceptance reuses the ON predicates
   without trusting proposed entries. Existing qualified claims re-establish current evidence even
   without a record diff. After the fake protected tip advances, observation checks the same
   transition. The real conditional ref update and whole-workflow terminal gate remain phase C.

The integration serialization domain includes ON, MA, neutral generators, public/private lower
writers, hand edits and dependency changes that can invalidate admission. Branch-local checked
delivery prepares a candidate; it certifies no exclusive cross-process ownership. Candidate
validation and restoration must preserve concurrent bytes, modes, links and removals. The existing
mutation rollback has conflict-preserving recovery (`xtask/workspace_mutation.rs:614-665`), which
the routed adapters should reuse instead of ON4's raw snapshot restoration. Any direct writer
outside the branch-to-IN1 model that cannot demonstrate the contract's serialization must refuse.

## Proving evidence and refusal coverage

The E.7 in-process proof starts with the compiled fixture CLI/source/templates available but no
synthetic registry entry, manifest root or lifecycle output installed by setup. Those are the new
ON1's outputs. It must be one connected chain, retaining the same approval Event and exact version:

1. ON1 → reviewed governing adoption → actual direct snapshot/union/coverage/report generation and
   ordinary `snapshotted` metadata → protected maintainer classification/declaration/acquisition
   statement. Use `depth-fixture-snapshot` with the built B.1 binary, not a handwritten shard.
2. Public ON2 complete P/O freeze; public candidate write with a
   subprocess fake executor and substantive wrapper changes/evidence; commit the implementation.
   Exercise the repair alternative on a sibling checkpoint and show both select valid runtime
   evidence without changing P/O.
3. An onboarding-bound synthetic producer run with independent adoption of all eleven due
   obligations; ON3 materializes its actual reporting E/files and pinned runtime handoff packet.
4. ON4 publishes the qualified wrapper promise, a new exact `Publication` entry and `published`
   lifecycle assertion. Pointers stay `none`; metadata stays `snapshotted`, without passed outcomes;
   no runtime backend/capability is added. Local and IN1 results agree.
5. Prepared ON5, bounded human metrics, then final ON5 `closed`, its exact proving entry and
   `closed_baseline` assertion, with deferred maintenance still deferred. IN1 admission and
   post-tip observation pass at each committed transition. A release-watch variant checks
   settlement continuity without conferring maintenance depth ownership.

The following negative groups are required; each asserts no changed depth-gated output on refusal,
plus preservation of unrelated/concurrent state. ON2's trusted attempt/diagnostic exception is
asserted separately from executor delivery.

| Group | Required cases |
| --- | --- |
| Bootstrap and identity | No installed route; foreign subject/version/target; unknown/misplaced approval fields; a fake backend; undeclared approval version; preexisting record during bootstrap; ON1 replay over bindings; proposed registry table from ON1; missing Annex B adoption; metadata claiming an unacquired version; full manifest validation of the actual ON1 output, not the minimal metadata object in today's freeze fixture. |
| Freeze and acquisition | P-only truth; absent, branch-only, extra, missing or changed statement entries; every changed/deleted O input; incomplete union; changed P; post-integration re-freeze; earlier approval; record/object deletion; hardened Git replacement/graft/alternate/environment regressions retained from slice 5. |
| Execution and repair | Forged serialized baseline; edits to protected inputs, snapshots or controller outputs; failure/interruption; caller changes during validation/delivery; fake nonempty written paths; repair cannot alter P/O, backfill verified E, lose the written-path digest or heal an orphan binding. |
| Evidence | Existing maintenance receipts against onboarding; another approval Event; wrong P/O; unadopted mapping; missing obligation; partial/failed/nonterminal/retried attempt; revoked adoption; changed applicable population; changed producer source; dependency-only invalidation; stale pre-entry verdict after entry append. All remain unverified or refuse, never qualify by asserted labels. |
| Owning acceptance | Missing/inconsistent P/O/E; classification-required or carried work; debt additions/renewals/retirements without proof; wrong path; forged publication/proving entry; packet/closeout digest drift; semantic/timestamp changes to `closed`; lifecycle assertion retarget at the same path; maintenance drift clearance; forbidden pointer/status/passed-target/marker effects. |
| Complete mediation | Aggregate effects on another agent/target/version; ordinary requested scope displacing a retained tuple; declaration omitted/disabled; record removed; nested root; proposed declaration/record; every NE command and alias; both ON5 entries; LW1 batch/lifecycle/packet and every newly added admitted caller. MA positives continue on maintenance scope and refuse onboarding acceptance. |
| IN1 and delivery | Wrong/stale parent; moved protected tip; invalid admission/observation binding; multi-root change with a different requested root; unchanged stale qualification; new entry without effect; new effect without entry; hand edits; gate failure; scratch Git/registry/identity changes; concurrent output replacement/removal/mode/link change during failure handling. |

The fixed wrapper-only subject cannot positively prove shared capability advertising or a second
agent's onboarding acceptance. Keep adversarial detector/wrong-owner cases for those effects and
record their limits in A.4; a later runtime-agent positive is separate support work. Likewise,
phase C must use real full-schema artifacts, canonical workflow source, real subprocess checks,
producer terminality and native queue admission/readback. An in-process gate stub is labeled as
such in results and cannot close those requirements.

## Implementation slices

Every numbered slice is one reviewed PR. These are future write scopes; this packet writes only
this design document. All Rust files are capped at 700 code lines. Extract focused submodules in
the owning slice before adding behavior where a file would exceed the cap; use `make loc-check`.
Do not grow the existing monoliths or move unrelated policy. Every route-changing slice updates
its Annex A forward/backward rows and test references in that same PR; slice 13 reconciles them
against the final enforcement revision. No slice installs a live profile or production declaration.

0. **Normative decisions.**
   - Content: after maintainer decisions, amend the evidence contract for bootstrap and the finite
     onboarding producer variant; amend the charter for wrapper-only proof interpretation and ON3
     E/reporting-publication delivery and freeze reporting coherence. Define explicit read-only gate
     evaluation. State ON4/ON5 entry ownership and handoff refusal boundary. Do not rewrite approved
     OF or CP choices.
   - Write scope: `docs/specs/{support-depth-contract,support-depth-evidence-v1,cli-agent-onboarding-charter,agent-registry-contract}.md`
     only where the approved rows require it, plus the validation test list and this decision status.
   - Dependencies: decisions OR-P1–P14. Proving checks: cross-contract review of acceptance categories,
     no pointer/status authority, no maintenance-evidence substitution and retained handoff semantics.
1. **Finite bootstrap scope and approval adapter.**
   - Content: extend the existing compiled synthetic route with the ON1-only pre-enrollment state
     and the OR-P14 scaffold-to-declared transition (IN1 admission of exactly the declaration,
     classification rows, acquisition statement and coherent reporting publication);
     extract/share the strict finite approval reader. Keep normal scope/production parsing unchanged.
   - Write scope: `xtask/isolated_maintenance/{mod,scope}.rs`, focused new onboarding submodule,
     `xtask/runtime_follow_on/depth/approval.rs`, synthetic onboarding templates and new scope tests.
   - Dependencies: 0. Proving tests: absent/unselected subject only in bootstrap, no records there,
     exact descriptor, unknown/foreign scope, no route fallback, maintenance/Codex controls; the
     declaration transition admitted and read back, and refused with any extra table, record, entry
     or acceptance effect.
2. **ON1 public scaffold and truthful rule-4 inputs.**
   - Content: route ON1's existing plan through the bounded adapter; emit no table or fictitious
     metadata. Reuse governing templates, ordinary acquisition and NE3 ordering from OR-P1.
   - Write scope: `xtask/onboard_agent.rs`, `xtask/onboard_agent/{approval,descriptor,lifecycle,validation,mutation}.rs`,
     `xtask/onboard_agent/preview/` and new installed-route bootstrap tests/fixture helpers.
   - Dependencies: 1. Proving tests: actual empty-root scaffold/replay, full canonical validation after
     acquisition/adoption, metadata before declaration, no enrollment writes, unchanged ordinary cases.
3. **ON2 executable freeze and candidate write.**
   - Content: public dry-run dispatch to existing freeze, align its synthetic contract/prompt with
     write preparation, use wrapper-only implementation checks with shared candidate finalization,
     and deliver coherent reporting publication for freeze candidates without accepting effects.
     IN1 re-establishes an onboarding reporting freeze (approval/P/O object, acquisition statement
     at the parent, selection/entry preservation) through the finite approval adapter instead of
     the full loader in `integration/generation.rs:67-68`; from this slice an ON2 freeze candidate
     is integration-admissible.
   - Write scope: `xtask/depth_acceptance/integration/generation.rs` (onboarding reporting dispatch),
     `xtask/runtime_follow_on.rs`, `xtask/runtime_follow_on/{depth,models,lifecycle,codex_exec}.rs`,
     `xtask/runtime_follow_on/depth/`, create-lane adapter/tests in `xtask/candidate_execution/`.
     Shared candidate mechanics change only if required by a demonstrated adapter boundary.
   - Dependencies: 1–2. Proving tests: connected public P/O freeze → write, all OF regressions,
     protected record, truthful failure/diagnostics, production refusal and unchanged MA4 suite;
     IN1 admission of the freeze checkpoint and refusal of a hand-assembled record.
4. **ON2 repair delivery.**
   - Content: scoped repair loading/proposal, preserve P/O and lifecycle written-path integrity;
     check frozen acquired inputs and actual reporting effects before delivery.
   - Write scope: `xtask/repair_runtime_evidence.rs`, focused repair submodule if needed,
     `xtask/runtime_evidence_bundle.rs` only for shared identity/preservation, new routed repair tests.
   - Dependencies: 3. Proving tests: repair sibling chain, stale run/object/digest, delivery rollback,
     no verified depth E and no acceptance, ordinary repair regressions.
5. **Onboarding producer Event binding.**
   - Content: finite onboarding binding of the existing synthetic producer, exact approval-linked
     population authentication and fixture/adoption identities. Keep cases/receipt schema/obligations.
   - Write scope: `xtask/depth_ci/profile.rs`, relevant collector/archive code,
     `xtask/depth_evidence/verdict/github/` profile/population tests, existing synthetic fixture tree;
     canonical `ci.yml` collection steps only if compiled selection cannot use existing steps.
   - Dependencies: 0–3. Proving tests: onboarding eleven-obligation positive through fake authenticated
     host, maintenance/onboarding cross-Event refusal, all-attempt/adoption/source/currentness cases,
     unchanged maintenance producer. No new job or real host call.
6. **ON3 reporting E and runtime selection.**
   - Content: captured approval/registry, authenticated controller assembly, one reporting E/evidence/
     lifecycle/packet/reporting-publication proposal, acquired-input and runtime evidence checks,
     public route wiring. IN1 admits the ON3 reporting checkpoint: E and its files re-authenticated
     against the onboarding producer population, evaluation of onboarding records that carry no
     maintenance entry (today `depth_acceptance/publication.rs:316-336` evaluates only
     maintenance-entry-backed records), and no entry or qualification.
   - Write scope: `xtask/prepare_publication.rs`, `xtask/prepare_publication/`, shared private assembly
     call site, `xtask/depth_acceptance/{publication,integration}.rs` for the reporting dispatch and
     new installed-route reporting tests. Keep the executor record immutable.
   - Dependencies: 3–5. Proving tests: real assembly files bound by delivered E, active-run selection,
     repaired-run alternative, stale/missing evidence and no-entry/no-qualification assertions.
7. **Onboarding owning predicates, without public accepting writes.**
   - Content: factor common adequacy, add exact onboarding binding/currentness and publication/proving
     entry derivation, strict debt-free boundary, effect allowlists and read-only proposed evaluation.
   - Write scope: `xtask/depth_acceptance/` focused onboarding/common modules, narrow changes to
     `manual.rs`/`isolated.rs`; shared `support_depth/` derivation call sites and focused tests.
   - Dependencies: 5–6. Proving tests: positive private predicates, wrong path/Event, all-due failures,
     debt/work/classification refusals, full maintenance closeout/promotion regression suites.
8. **ON4 and its IN1 authority together.**
   - Content: packet finalization → exact publication entry → fresh post-entry evaluation/rendering →
     aggregate admission → candidate gates → checked delivery; D7 publication/unchanged-qualification
     dispatch. Production and public neutral writers still refuse.
   - Write scope: `xtask/publication_refresh.rs` and new submodules,
     `xtask/depth_acceptance/{integration,publication}.rs` and focused integration submodules,
     `xtask/isolated_maintenance/tests/` onboarding publication/IN1 tests.
   - Dependencies: 6–7 and 10. Proving tests: qualified synthetic promise without pointers/status,
     exact entry/packet, aggregate wrong-owner effects, pre-entry verdict refusal, gate/concurrent
     restoration failures, admission plus observation, B.7 maintenance chains unchanged.
9. **Both ON5 commands and their IN1 authority together.**
   - Content: prepared-only reporting proposal with acquired-input/retarget checks; final closed
     bytes/entry/lifecycle/docs batch, post-entry evaluation, proving-effect D7 dispatch. No drift clearance.
   - Write scope: `xtask/prepare_proving_run_closeout.rs`, `xtask/close_proving_run.rs` and extracted
     submodules, onboarding predicates and `xtask/depth_acceptance/integration/`, new ON5 tests.
   - Dependencies: 8. Proving tests: prepared → closed, deferred/release-watch settlement, timestamps
     and replacement bytes, lifecycle evidence, forged/retained entries, no same-version path transfer,
     local/final agreement and conflict-preserving refusal.
10. **Authenticated read-only gate consumers.**
    - Content: OR-P13's two phases. In-process: callable forms of the fixed depth-reading checks run
      against the proposal's scratch view inside the borrowed evaluation scope, before ON4 delivery.
      Subprocess: the unchanged commands resolve their own installed route over a clean committed
      candidate the host already holds. Every registry reader in those checks reads the routed
      registry read-only, including `runtime-advertising-model --check`'s preparation and revision
      loader. Keep default rendering and neutral write refusals. No cross-process verdict token and
      no candidate hosting. Inventory every existing test and command that reads the repository
      registry with the production parser or asserts it has no depth, and convert each to a
      hermetic undeclared fixture (production-parser controls) or authenticated routed reading
      (candidate-registry checks); production parsing itself stays unchanged.
    - Write scope: `xtask/support_matrix.rs`, relevant `xtask/manifest_validate/` and capability-audit
      read consumers, `xtask/runtime_advertising/{mod,revision}.rs` check-mode registry reading,
      `xtask/release_doc.rs` and the inventoried registry-reading tests under `crates/xtask/tests/`, focused private evaluation adapters/tests, evidence-contract behavior notes.
    - Dependencies: 0, 7. Slice 8 uses the in-process phase, so this slice lands before it.
      Proving tests: a credential-free composition test running both phases, with every fixed
      command including `runtime-advertising-model --check` passing scoped and standalone, the
      full `make preflight` test leg passing over a declared candidate registry, and production
      and NE8 write-refusal controls; an in-process pass
      followed by a changed committed candidate failing the subprocess phase, exact-record checks,
      source/host/parent drift, no implicit
      evaluation during ordinary generation, no profile in a child process, and independent test-owned
      process evaluation with fake authority. Actual CLI/green-gate success remains phase C evidence.
11. **E.8 executed mediation matrix.**
    - Content: run every MA/ON/NE/LW1 boundary and alias with onboarding enrollments, retained records,
      omitted scope and proposed outputs; repair only demonstrated guard gaps. Reconcile all callers
      of private delivery and all restoration writers with Annex A.
    - Write scope: focused route tests under `xtask/isolated_maintenance/tests/` and
      `crates/xtask/tests/`, binary entrypoint harnesses; `xtask/depth_routes.rs` and named affected
      entry guards only if a failing conformance test requires a fix; Annex A/A.4.
    - Dependencies: 8–10. Proving tests: rules 1–7 matrix above, production depth refusal controls,
      retained/deleted declarations, nested/proposed destinations, later-version displacement,
      wrong-path MA effects and ordinary non-depth regressions. No generic bypass is introduced.
12. **E.7 connected new-agent implementation proof.**
    - Content: run the connected chain above through public in-process entries; fake host and offline
      executor, independent onboarding producer/adoption, IN1 at each committed checkpoint. Record
      stubbed gates and the missing phase C host run explicitly.
    - Write scope: new connected onboarding chain/fixture helper tests and a bounded proof result in
      `docs/agents/lifecycle/`; reuse B.1 fixtures instead of another subject/profile family.
    - Dependencies: 2–11. Proving tests: qualified chain and truthful-insufficiency sibling chain,
      repair alternative, every negative group; ordinary and maintenance connected controls.
13. **E.8 final inventory and phase C handoff.**
    - Content: re-establish forward entrypoints and backward outputs at the actual enforcement
      revision, update ON/NE/LW1/IN1 and A.4 status with executed test names, list each serialization
      domain/restoration and every refusal. Document the independent host run still due under CP-P5.
    - Write scope: support-depth contract Annex A/A.4 and bounded present-behavior notes, charter and
      evidence present-behavior notes, validation test list, isolated proof/runbook documentation.
    - Dependencies: 11–12. Proving checks: every route/effect has owning admission or an executed
      refusal; every claimed positive cites results; per-root validator adoption is distinguished
      from all-root D7. Run relevant doc/spec checks and repository hygiene. No host mutation.

The implementation PRs run focused tests plus required format/lint/check/test/LOC/hygiene gates.
Any future manifest changes also run manifest validation for their affected root. CI wiring
changes, if needed in slice 5, require the CP-P5 scheduling/latency review; gate policy and the
terminal workflow result policy do not change. The real phase C run is a separate authorized
operation, not an unlisted implementation slice here.

## Contradictions, gaps and unchanged boundaries

- **Roadmap shorthand exceeds installed behavior.** ON1 has an ordinary scaffold, not a private
  depth-positive scaffold. ON2 write has partial conditional routing but its synthetic freeze
  packet cannot be executed by the full loader. ON3–ON5 slice-6 tests prove acquired-input
  refusal, not accepting cores. The inventory above deliberately distinguishes these facts.
- **Historical Annex A/A.4 statements are stale at this baseline.** The inventory names PR #296
  and still describes onboarding freezes as uninstalled and CP-P3 as unimplemented
  (`contract:922-930`, `:973-979`, `:1114-1118`). The merged code has private onboarding freezes
  and B.7 qualified maintenance publication. Its normative requirements still govern; future
  inventory updates must cite the final tree rather than retroactively treating the historical
  source revision as current evidence. The charter's “no create-lane command implements them yet”
  note (`charter:272-277`) likewise needs a bounded present-behavior refresh.
- **E.7 cannot accept an unverified generation.** Honest insufficiency is necessary reporting
  evidence, but does not waive the publication/closeout predicate. The evidence contract currently
  forbids the positive; OR-P5 is an explicit amendment request, not an inference that the existing
  maintenance producer already supports it.
- **Reporting checkpoints need coherent publication.** Freeze/ON3 can alter derived support facts
  before acceptance. Reuse maintenance's reporting bundle mechanism, with no acceptance effects,
  and validate each checkpoint rather than stitching together individually stale trees.
- **Subprocess gates cannot authenticate unpublished proposal bytes.** Installed registry scope
  alone still leaves default publication verdicts unverified, a thread-local scope cannot cross
  processes and a local commit is not on the host. OR-P13 splits ON4's gate into an in-process
  phase over the exact proposal and a subprocess phase over committed candidates the host holds,
  preserving default generation and neutral refusal. CP-P5 covers CI prerequisites; slice 10 must
  land before ON4's accepting route (slice 8) and before the phase C run can pass.
- **No onboarding E writer is connected.** ON2 protects the record, and ON3 currently writes only
  packet/lifecycle. OR-P6 names the missing controller writer and its charter amendment. Runtime
  run evidence, including repair, does not substitute for authenticated E.
- **Debt-free does not make the maintenance predicate reusable unchanged.** Its debt planner rejects
  an onboarding Event before considering whether debt is empty. OR-P7 must preserve that existing
  maintenance contract while introducing the bounded onboarding checks.
- **A declared registry fails today's workspace test leg, on both paths.** Existing tests and
  `release_doc` parse the repository registry with the production parser or assert it declares no
  depth, so `make preflight` cannot pass in any repository whose registry carries a depth table.
  This affects the maintenance path's phase C run as much as onboarding, and CP-P5 does not list
  it. Slice 10 resolves it; the maintainer should treat it as a phase C prerequisite in addition
  to CP-P5.
- **The same-version handoff is unresolved.** Charter rule 11 describes it, but retained policy and
  isolated scope refuse it. E.8 can conform by refusing; it cannot claim that maintenance promotion
  of that onboarding-owned version is available. Transition semantics, retained entries/work,
  existing qualification and currentness through that move need a separate design.
- **Production remains refusing.** No live isolated profile, canonical registry table, production
  depth enrollment or path enablement is installed. Path enablement items remain unmet by this
  design: isolated code/tests are not landed enforcement evidence, the independent host run is
  outstanding, full route/final-admission conformance is unproved and no bounded production
  authorization is given. No qualification of a real agent follows.
- **CI stays credential-free; no live calls.** Tests use fake hosts, the compiled fixture CLI and
  offline executors. This packet neither accesses a live service nor authorizes a repository,
  credential reference, host setting, native queue run, commit or profile installation. The create
  lane still gains no packet freeze, pointer, `validated` or `supported` route. OF's partial-union
  and non-depth candidate-expansion questions remain open and outside these slices.
