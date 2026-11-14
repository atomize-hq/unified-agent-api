# Support depth: onboarding routing and complete mediation

Status: maintainer decisions recorded 2026-11-07; implementation follows the slices below.
This design covers roadmap E.6, E.7 and E.8; the decisions authorize those slices, not production
enablement or the independent phase C host run.
Source baseline: a merge of PR #344 at `c33360ed` (onboarding slices 0–6) and PR #346 at `c6d75418`
(B.7a/B.7b), treated as staging after both land. All source citations refer to that tree.

The [support-depth contract](../../specs/support-depth-contract.md),
[onboarding charter](../../specs/cli-agent-onboarding-charter.md#support-depth),
[registry contract](../../specs/agent-registry-contract.md#support-depth) and
[evidence contract](../../specs/support-depth-evidence-v1.md) remain normative.
The approved [OF-P1–P9 decisions](support-depth-onboarding-freeze-decisions.md) and
[CP-P1–P6 decisions](support-depth-closeout-promotion-design.md) are the starting point.
Where an approved decision needs a normative amendment, that amendment must land separately before
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

### Inspection A — producer population and CI terminality

Read-only finding at the source baseline: **no self-terminal-result cycle is required by OR-P5's
synthetic binding and CP-P5's agreed ordering**. Onboarding authentication and CP-P5 wiring
remain unimplemented; this is not an executed onboarding/native-host positive. Here `github/`
means `xtask/depth_evidence/verdict/github/`, and workflow names are under `.github/workflows/`.

Today only a maintenance Event is supported. Its `request_commit` identifies the unique
`staging`-targeting PR in the authenticated repository, used for independent adequacy adoption
(`github/mod.rs:133-156`, `:215-218`; `github/adoption.rs:23-35`). That selects the **adoption
PR, not a PR-only run population**. All `ci.yml` runs since the Event date are inspected at their
API-derived heads: matching profile-record Event/P/O makes every attempt `1..=run_attempt`
applicable before artifacts/outcomes are considered; PR, merge-group, dispatch and push events
count and population stability is rechecked (`github/mod.rs:157-205`, `:265-291`). Later
publication/proving CI on the same Event/P/O therefore counts, even on another PR or without a
witness. Slice 5 must implement the actual approval-linked onboarding generation-PR binding
and committed ON2 producer inputs; there is no onboarding selector/positive today. It cannot
assume only the first ON2 run supplies evidence or conceal source/adoption mismatch in a fake host.

IR-P3 prevents the hypothesized cycle without excluding current runs. Channel/CORE require a
completed workflow; authenticated isolated synthetic evidence needs the terminal producer job
while its workflow may still run (`github/mod.rs:39-70`, `:201-205`). Every applicable attempt
still requires terminal `Rust tests (workspace)` with success/failure conclusion, successful
collection/upload/binding, authenticated logs and immutable artifact (`github/artifact.rs:16-89`).
Existing test `terminal_producer_job_verifies_every_due_obligation_while_the_workflow_runs`
(`github/tests/synthetic.rs:28`) pins that distinction, with unfinished/aborted/failed-step
refusals at `:97` and unchanged CORE whole-workflow refusal at `:131`.

The witness is collected/bound in `ci.yml:166-280`; `preflight` and per-root validators currently
lack producer `needs` edges (`ci.yml:153-164`, `:334-390`). CP-P5 requires failure-preserving
consumer scheduling after terminal `rust-tests`, never producer dependence on packet admission
([G9](support-depth-closeout-promotion-design.md#g9--ci-and-host-wiring-of-the-isolated-repository-phase-c), lines 354–379;
[IR-P3 scheduling](support-depth-isolated-route-design.md), lines 142–161). Slice 5's required
`onboarding_same_generation_later_runs_wait_for_producer_not_workflow` test must include initial
and later publication/proving runs and all retries: same-run IN1 can pass after terminal producer
binding while the workflow runs; unfinished/missing/aborted producers and changing populations
refuse, and failed attempts remain observable. Phase C must test the actual invocation/`needs`
graph, including the recursive test leg: the producer neither invokes nor waits for its own
packet admission. No applicable attempt is discarded to obtain this ordering.

Whole-workflow completion remains **before MERGE**, outside evidence consumers (IR-P3 `:151-161`;
CP-P5). The final `staging-integration-gate` waits for the exact 17 job results and calls
`scripts/check_staging_integration.py gate` (`ci.yml:623-659`); the evaluator checks results and
candidate-justified skips (`scripts/check_staging_integration.py:117-154`), not its own workflow's
`completed` status. The protected host must wait for workflow completion, direct required checks
and that terminal gate before the conditional update (`contract:735-779`); post-push
`depth-tip.yml:8-10`, `:27-38` only detects transitions. Maintenance closeout separately resolves
completed matching workflow runs (`xtask/agent_maintenance/closeout/evidence.rs:109-209`), but
runs outside CI/preflight (`xtask/agent_maintenance/closeout.rs:99-104`); it must not move inside
the workflow it resolves. These boundaries require no gate to await its own workflow's terminal
result, so no OR-P15 decision is opened.

## Maintainer decisions

The maintainer adopted the advisory recommendation on 2026-11-07: ten rows as written and
OR-P1, OR-P2, OR-P3 and OR-P13 with amendments incorporated below; none rejected. Each row
authorizes its later design/implementation slices only, not production enablement. Alternatives
remain explicitly not recommended.

| Id | Recommendation | Grounds / boundary; alternatives | Status |
| --- | --- | --- | --- |
| OR-P1 — bootstrap and rule-4 ordering | Run ON1 before enrollment, preserving its rule that it writes no `support_depth` table. Extend the existing installed synthetic route with a finite pre-enrollment bootstrap state for ON1 creation: the subject entry/root may be absent, or the entry may be the exact unenrolled scaffold; no retained record or acceptance is admitted. Fix the expected descriptor from compiled synthetic templates and bind the exact approval; neither supplies declaration authority. Admit only the exact absent-to-scaffold registry delta at bootstrap integration, with no depth table. After scaffold, land reviewed Annex-B governing files, direct acquisition and truthful `snapshotted` metadata through existing NE3 **before** the maintainer adds the onboarding declaration; then freeze through ON2. Between integrated ON1 and OR-P14, admit record-free preparation checkpoints for reviewed governing adoption, actual acquisition/union/coverage/report generation and ordinary NE3 snapshotted metadata. Each preserves the exact scaffold registry entry and approval identity, introduces no depth table, record, entry or acceptance effect, and undergoes IN1 admission followed by post-tip observation. Only ON1 and OR-P14 receive their separately bounded registry-delta exceptions. | Charter rules 2–4 (`charter:285-325`) distinguish scaffold, declaration and freeze. ON1 cannot invent acquired metadata (`xtask/onboard_agent/preview/governing.rs:32-52`), and NE3 refuses after declaration. Current route scope requires an already protected subject entry/table (`xtask/isolated_maintenance/scope.rs:75-110`); it cannot represent a genuinely new ON1. A bounded additional state in that same source-installed route is necessary; no new command, file kind or enrollment authority is needed. This needs an evidence-contract bootstrap amendment before code, including the narrow exception to protected-registry equality for that exact ordinary bootstrap delta. **Not recommended:** preseed the entry, lifecycle and record and call that an ON1 proof; it skips the route. **Not recommended:** let ON1 install the table or claim `validated`; registry rule 9 and charter rule 11 forbid it. | **Approved by the maintainer 2026-11-07 with amendment** |
| OR-P2 — non-runtime synthetic orchestration | Extend the existing finite synthetic approval projection into a private adapter shared by routed ON1–ON5 and IN1. Keep this subject non-runtime, capability-free and wrapper-only. Adapt runtime-follow-on evidence/handoff checks to the wrapper work actually due, while preserving the existing lifecycle stages and record/packet kinds. Fixed subject/version/target, expected descriptor, producer case mapping and offline-executor selection belong in the compiled synthetic profile and its private adapter. Shared ON/IN1 orchestration, ownership, adequacy, currentness, effect admission, integration and delivery operate on validated path/Event/subject inputs; neither `depth_fixture` nor Codex identity may waive those checks. This grants no additional production approval acceptance and requires no second synthetic subject. | The template deliberately omits `backend_module`; the private reader compares the whole descriptor (`xtask/runtime_follow_on/depth/approval.rs:1-82`), whereas the full loader requires it. The private packet and execution loader are currently incompatible. The amendment must name the bounded wrapper-only interpretation of checklist implementation/evidence steps (`charter:194-207`); no claim of adapter qualification follows. **Not recommended:** add a fake backend or fake `agent_api` test to satisfy the loader; the synthetic profile maps no M obligation (`evidence:454-458`). **Not recommended:** make production approval loading generally permissive; that exceeds this proof. | **Approved by the maintainer 2026-11-07 with amendment** |
| OR-P3 — public routing and failure behavior | Route existing public ON forms through captured isolated authority and their owning cores; an installed route failure is an error, and absence retains production refusal. Keep private LW1 mechanics behind checked proposals. Validate each ON3–ON5 proposal and its applicable proposal-phase checks before checked local delivery; only ON4 gates identified in OR-P13 use the agreed two-phase split. Use expected-before, conflict-preserving delivery/restoration. Local delivery prepares a branch candidate; independent committed-candidate checks, current IN1 admission and the existing terminal gate must pass before protected acceptance. A subsequent failure leaves the candidate unintegrated and grants no permission to restore over changed caller or protected state. | Reuses `Route::capture`, candidate execution, `Proposal`/`IsolatedView` and admitted mutation mechanics (`xtask/isolated_maintenance/mod.rs:64-101`, `:140-170`; `view.rs:41-80`). The public forms include ON2 dry-run/write/repair and both ON5 commands, not just the five primary names. Existing ON3–ON5 bodies have raw or multi-stage delivery. **Not recommended:** remove outer guards or make route presence a general lower-writer bypass. **Not recommended:** treat local preparation as protected acceptance or restore arbitrary current bytes after a later gate failure; ownership may have changed. Final serialization remains IN1; no filesystem lock or crash-atomic transaction is claimed (`contract:746-779`). | **Approved by the maintainer 2026-11-07 with amendment** |
| OR-P4 — what onboarding accepts | ON4 owns only its generation's `Publication` entry, exact packet-bound `published` assertion and supported qualification/advertising effects. ON5 final close owns `ProvingRunCloseout`, exact `closed` closeout, `closed_baseline` and its proving evidence id. ON3 and prepared ON5 remain reporting-only and refuse any detected acceptance/retargeting. Neither path gains pointers, passed target outcomes, `validated`/`supported` status, marker retirement or maintenance drift clearance. | These effects already exist in `charter:454-472` and `contract:636-660`; Annex B already has both entry kinds (`contract:1436-1442`). Even an unverified support row does not make `published` reporting-only. Use CP-P3's coherent entry → fresh post-entry evaluation → render → re-admit pattern without its maintenance authority. Aggregate reporting may update other rows truthfully; any acceptance destination outside the owned generation refuses the entire ON batch. **Not recommended:** create an onboarding promotion or borrow MA5's entry; no authority exists. **Not recommended:** allow `published`/`closed` with insufficient depth; predicate items 5–6 still apply. | **Approved by the maintainer 2026-11-07** |
| OR-P5 — E.7's evidence claim | Aim for a qualified **synthetic wrapper promise** on the onboarding Event. First amend the evidence contract to add a finite onboarding variant of the existing synthetic producer binding; reuse its fake CLI, cases, target, obligations, receipt schema and terminality policy. Keep maintenance and onboarding Event/P/O populations separate, with fresh evidence/adoption for onboarding. | A truthful-insufficiency test is valuable but cannot complete approval → publication → closed closeout under `contract:824-835`. Today `evidence:550-555` explicitly prevents the positive. Minimal producer work must bind the actual approval Event and frozen onboarding P/O in its fixture/witness, authenticate its approval-linked PR and complete applicable attempt population, then evaluate only matching receipts. Update the compiled fixture selection, collector/archive/verifier and fixture identities together; use the existing `rust-tests` job, no new command or live profile. **Not recommended:** relabel maintenance evidence or permit cross-Event reuse; maintenance evidence does not substitute. **Not recommended:** call an unverified/refused chain the complete E.7 proof. Without approval of this amendment, only the insufficiency/refusal proof can ship and E.7 stays incomplete. | **Approved by the maintainer 2026-11-07** |
| OR-P6 — who delivers onboarding E | Make routed ON3's controller assemble and deliver reporting E plus bound receipt/payload files and coherent reporting publication in the same checked proposal as `publication_ready` and its packet, after committed ON2 implementation has producer evidence. Preserve Event/P/O and every entry. Repair continues to repair runtime evidence only. | The existing private host-taking assembly can serve; ON2's executor cannot write its protected record. ON3 is the existing runtime-evidence-selection handoff before ON4, so no new command/file kind is needed. This does amend the checklist's present ON3 write scope (`charter:208-213`) and Annex A: that change must precede code. The dry-run freeze similarly includes any derived reporting publication needed for coherence, reusing the maintenance preparation pattern; neither writer may add qualification or another acceptance effect. Every consumer still re-authenticates; assembly is a claim, not a receipt granting admission (`xtask/depth_evidence/verdict/github/assembly.rs:1-16`). **Not recommended:** let the executor edit the record, or make repair manufacture verified E. **Not recommended:** manually prefill E in the chain fixture; it omits the owning reporting writer. | **Approved by the maintainer 2026-11-07** |
| OR-P7 — admission, currentness and debt | Add private onboarding publication/proving predicates that share path-neutral adequacy logic with maintenance, but independently bind the exact approval Event, lifecycle object, registry policy, O acquisition and candidate bytes. Keep the synthetic positive debt-free; refuse onboarding debt grants/retirements or baseline/inventory drift until corroboration is designed. | `manual::admitted` already supplies the due-work/evidence model, but its current binding and debt calls are maintenance-specific. Reuse shared policy/O derivation without performing a freeze or rebasing P. Check every due obligation, classification and carried work; empty onboarding `acceptance_work` is not an exemption from depth (`contract:1423`). No packet marker is required on the create lane (`charter:464-466`). **Not recommended:** call the maintenance predicate with a forged request/marker. **Not recommended:** remove its lifecycle checks or skip debt/currentness because the initial fixture is empty. A shared extraction must preserve maintenance behavior. | **Approved by the maintainer 2026-11-07** |
| OR-P8 — D7 attribution | Dispatch IN1 by each retained tuple's owning path and actual candidate artifact/effect. Independently re-establish onboarding freezes and reporting E, ON4 publication and ON5 proving acceptance through the same predicates as local preparation. Require each new entry to match the independently admitted exact packet/closeout and current record digest; re-evaluate unchanged qualifications too. | Existing detectors already attribute through approval bytes or durable exact file references (`xtask/depth_acceptance/artifacts.rs:117-181`); entry locators themselves grant no authority. Replace the blanket onboarding-effect refusal only for admitted cases. Separate `Publication` effects from CP-P3 closeout/promotion classification; its fallback otherwise misclassifies ON effects (`integration.rs:270-331`, `:338-381`). **Not recommended:** trust command names, workflow names, E labels, a copied entry or an earlier local result. **Not recommended:** treat every new qualification as promotion once onboarding is supported. All-root checks and admission/observation binding remain unchanged. | **Approved by the maintainer 2026-11-07** |
| OR-P9 — frozen acquisition through publication | Reuse frozen acquisition bytes on ON3–ON5; run generators in check/planning form there, and refuse changed bytes, including regenerated wrapper coverage. Keep complete-union admission and the post-`runtime_integrated` re-freeze refusal. | OF-P4–P7 already decide these boundaries. ON4 renders publication; its fixed required command list uses check modes (`charter:214-223`). Ensure candidate gates cannot replace O inputs, and re-observe after gates. **Not recommended:** exclude wrapper coverage from O or silently re-freeze during publication. **Not recommended:** admit a partial union for this proof; OF's partial-target question stays open and the subject needs one complete target only. | **Approved by the maintainer 2026-11-07** |
| OR-P10 — onboarding to maintenance handoff | Finish and integrate ON5 `closed` before any maintenance handoff. Keep same-version retained path transfer unsupported in E.6–E.8, with an explicit refusal test and a separately required transition design. Release-watch readiness alone transfers no depth authority. | Charter rule 11 describes a future handoff; current resolver and isolated scope cannot execute it. Deleting a record or changing its Event to make it fit loses retained authority. A later ordinary version may use its own lifecycle rules and displace reporting state while preserving the earlier record/work (`contract:723-733`); the fixed isolated subject still refuses an extra depth version. **Not recommended:** overlap both paths on one version, auto-transfer at closeout, or claim this design implements the charter's same-version handoff. Resolve its normative/code gap explicitly before promising that operation. | **Approved by the maintainer 2026-11-07** |
| OR-P11 — E.8's unsupported routes | Retain broad refusal for NE1–NE8, public LW1 and unsupported maintenance writers in onboarding scope. Add executed onboarding declaration/retained/proposed-destination cases at every actual public boundary and alias; keep reporting-only maintenance behavior only where its existing predicate admits the actual touched set. | Complete mediation permits refusal, and the current guard mechanisms cover omitted and enclosing scope without a new inventory (`xtask/depth_routes.rs:9-105`). E.8 needs maintenance-versus-onboarding tests as well as ON positives. **Not recommended:** add a neutral “reporting writer” bypass, narrow aggregate scope to the named agent or equate source inspection with conformance. Lower callers remain individually inventoried. | **Approved by the maintainer 2026-11-07** |
| OR-P12 — proof boundary | Require both a connected in-process installed-route chain and a later independent disposable-repository run under phase C's protected integration/CI conditions. The former is the credential-free E.7 implementation gate; the latter remains required evidence before claiming the contract's independent run/path enablement. | `Fixture::install()` does not exist in a CLI subprocess, and ON4 runs shell gates (`xtask/isolated_maintenance/mod.rs:115-121`; `xtask/publication_refresh.rs:577-603`). In-process tests may use scoped fake host/gate seams but must label those limits. Phase C separately authorizes repository/credential references (`support-depth-machinery-proof-rescope.md:86-88`); CP-P5 owns canonical CI prerequisites. **Not recommended:** call private-core tests or stubbed gates a native queue proof. **Not recommended:** perform the host run or install a live profile as part of this packet. | **Approved by the maintainer 2026-11-07** |
| OR-P13 — authenticated gates before and after delivery | Split ON4's fixed green gate for depth-enrolled generations into two bound phases, with no change to the command inventory itself. **Before delivery**, the controller runs the depth-reading checks behind each fixed command (`support-matrix --check`, `capability-matrix --check`, `capability-matrix-audit` and the validator, support-matrix and `runtime-advertising-model --check` parts of `make preflight`, `Makefile:234`) as in-process calls against the proposal's scratch view, inside the borrowed fresh evaluation scope bound to the exact proposed record. This is the pattern B.7b already uses for MA6 (`xtask/isolated_maintenance/tests/publication.rs`), and failure refuses the whole batch without unauthorized effects, preserving concurrent state as specified below. **After delivery**, the unchanged subprocess commands run where they can authenticate: a standalone check resolves its own installed route over a clean committed candidate whose HEAD the host already holds (`xtask/isolated_maintenance/host.rs:107-109`; `mod.rs:498-499`), that is, the pushed candidate commit in CI and IN1 for the actual integration candidate, before the protected reference advances. Format, lint and build keep running as subprocesses before delivery. The workspace test leg is not depth-independent: existing tests read the repository registry through the production parser (for example `crates/xtask/tests/c8_spec_capability_matrix_staleness.rs:48`, `xtask/release_doc.rs:97`) or assert that it declares no depth (`crates/xtask/tests/agent_registry/support_depth.rs:65`), so they fail on any declared candidate registry, whatever is routed. For every leaf of the fixed inventory, including the recursive `make preflight` test leg, slice 10 records proposal-phase execution and required independent committed-candidate execution. Local checks use hermetic subprocesses or authenticated in-process shared checking functions as appropriate. Those consumers are inventoried and made either hermetic (an undeclared fixture registry for production-parser controls) or routed (authenticated reading for candidate-registry checks). Every registry reader these checks use must read the routed registry read-only, including `runtime-advertising-model --check` (`xtask/runtime_advertising/mod.rs:54`, `revision.rs:36`), whose production parser otherwise rejects the onboarding declaration in both phases. Preserve each converted test's assertion purpose, including authenticated validation of the actual candidate registry where that was its purpose. ON4 local success means local preparation succeeded and a candidate was prepared, using the existing lifecycle and file kinds. All required independent committed-candidate checks, current IN1 admission and the existing terminal integration gate must succeed for the actual integration candidate before the protected reference advances; post-tip observation cannot substitute. Each evaluation binds its actual inputs; an earlier local result supplies no authority for changed candidate bytes. A later failure leaves the candidate unintegrated and authorizes no restoration over subsequently changed state. No new command, file kind, lifecycle stage or reusable gate token is introduced. Keep default generation, neutral write mode, NE8 write refusal and production verdicts unchanged. | A thread-local scope cannot cross `sh`/`cargo` processes, `IsolatedView` forbids Git metadata, and a locally committed proposal is not on the host, so no subprocess can authenticate unpublished proposal bytes without a live host write; candidate hosting would need credentials and a live call, which this design excludes. The in-process phase checks the exact bytes to be delivered with the same functions the commands call; the subprocess phase is the independent check of committed bytes. Charter rule 7's "own ... the required green gate" needs a bounded amendment naming this split for depth-enrolled generations only. A credential-free composition test must show both phases run, and that an in-process pass followed by a change to an admission-relevant input fails independent committed-candidate evaluation. A harmless change may pass after fresh evaluation; the earlier pass supplies no authority. **Not recommended:** make all derivation under a route implicitly trust isolated evidence; B.7 deliberately prevents that. **Not recommended:** skip the fixed gates, serialize a controller verdict into an environment token or reusable receipt, or push candidates to the host to authenticate them. **Fallback, not recommended:** defer this decision and keep the accepting ON4 route unavailable, so E.7 stops at ON3's reporting checkpoint. | **Approved by the maintainer 2026-11-07 with amendment** |
| OR-P14 — first declaration transition | Admit the maintainer's scaffold-to-declared commit as its own bounded, maintainer-owned reporting transition at IN1: exactly the onboarding `support_depth` table for the scaffolded entry and approval version, its reviewed CORE classification rows, the integration-branch acquisition statement (OF-P4 B) and any coherent reporting publication the declaration changes, with no record, entry or acceptance effect. The installed route captures the declared registry only after that commit is on the protected tip; the scope's protected-registry equality then holds again. ON1 still cannot write the table. | OR-P1's bootstrap exception covers only the depth-free ON1 delta, so without this row the declaration candidate is refused by the existing scope check before IN1 can admit it, and advancing the fake tip first would skip admission at a checkpoint. A declaration is a policy change the maintainer owns (charter rule 2, `charter:276-280`; registry contract `docs/specs/agent-registry-contract.md:455-457`). It needs the same evidence-contract bootstrap amendment as OR-P1, and admission plus readback tests. **Not recommended:** fold the declaration into ON1 or ON2; neither owns registry policy. **Not recommended:** treat the declaration as a hand edit judged only after the fact; IN1 must still admit it, and a named transition makes its bounds testable. | **Approved by the maintainer 2026-11-07** |

## Route composition

This section composes the approved decisions; none is installed by this document.

1. **Bootstrap, with no declaration or record.** The compiled route's bounded ON1 bootstrap
   captures the protected registry/source and exact synthetic approval. ON1 writes its normal
   scaffold and lifecycle identity, without the depth table, metadata invented from a version
   string, or acceptance. The proof does not preseed those outputs. Reviewed governing adoption
   and real single-target acquisition follow. NE3 records `snapshotted` without passed outcomes
   before declaration. Bootstrap registry reads may serve these ordinary input/planning commands;
   they grant no depth acceptance or ON2 freeze authority. Every record-free preparation checkpoint
   preserves the exact scaffold entry/approval and receives IN1 admission then post-tip observation;
   only ON1 and OR-P14 have registry-delta exceptions. A maintainer then commits the onboarding
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
   validates the proposed `published` packet reference and runs OR-P13's proposal-phase checks
   before checked local delivery. A successful local return promises that preparation succeeded
   and the branch candidate was delivered. Protected acceptance still requires the independent
   committed-candidate command inventory, current IN1 admission of the actual integration candidate
   and the existing terminal gate before the protected reference advances; post-tip observation is
   no substitute. A later failure leaves the candidate unintegrated and authorizes no restoration
   over changed caller/protected state. No new command, file kind, lifecycle stage or token is needed. A
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

The following negative groups are required. With an unchanged caller tree, refusal leaves the
relevant outputs unchanged, subject to ON2's separately asserted trusted attempt/diagnostic
exception. With a concurrent writer, the controller delivers no unauthorized acceptance effect
and restores only state it still owns; concurrent bytes, modes, links and removals are preserved.
Checked local delivery is not a cross-process filesystem transaction.

| Group | Required cases |
| --- | --- |
| Bootstrap and identity | No installed route; foreign subject/version/target; unknown/misplaced approval fields; a fake backend; undeclared approval version; preexisting record during bootstrap; ON1 replay over bindings; proposed registry table from ON1; missing Annex B adoption; metadata claiming an unacquired version; full manifest validation of the actual ON1 output, not the minimal metadata object in today's freeze fixture. |
| Freeze and acquisition | P-only truth; absent, branch-only, extra, missing or changed statement entries; every changed/deleted O input; incomplete union; changed P; post-integration re-freeze; earlier approval; record/object deletion; hardened Git replacement/graft/alternate/environment regressions retained from slice 5. |
| Execution and repair | Forged serialized baseline; edits to protected inputs, snapshots or controller outputs; failure/interruption; caller changes during validation/delivery; fake nonempty written paths; repair cannot alter P/O, backfill verified E, lose the written-path digest or heal an orphan binding. |
| Evidence | Existing maintenance receipts against onboarding; another approval Event; wrong P/O; unadopted mapping; missing obligation; partial/failed/nonterminal/retried attempt; revoked adoption; changed applicable population; changed producer source; dependency-only invalidation; stale pre-entry verdict after entry append. All remain unverified or refuse, never qualify by asserted labels. |
| Owning acceptance | Missing/inconsistent P/O/E; classification-required or carried work; debt additions/renewals/retirements without proof; wrong path; forged publication/proving entry; packet/closeout digest drift; semantic/timestamp changes to `closed`; lifecycle assertion retarget at the same path; maintenance drift clearance; forbidden pointer/status/passed-target/marker effects. |
| Complete mediation | Aggregate effects on another agent/target/version; ordinary requested scope displacing a retained tuple; declaration omitted/disabled; record removed; nested root; proposed declaration/record; every NE command and alias; both ON5 entries; LW1 batch/lifecycle/packet and every newly added admitted caller. MA positives continue on maintenance scope and refuse onboarding acceptance. |
| IN1 and delivery | Wrong/stale parent; moved protected tip; invalid admission/observation binding; multi-root change with a different requested root; unchanged stale qualification; entry without an admitted effect; effect without the required entry; an entry for one exact artifact cannot authorize unrelated effects in the same aggregate batch; hand edits; gate failure; scratch Git/registry/identity changes; concurrent output replacement/removal/mode/link change during failure handling. |

The fixed wrapper-only subject cannot positively prove shared capability advertising or a second
agent's onboarding acceptance. Keep adversarial detector/wrong-owner cases for those effects and
record their limits in A.4; a later runtime-agent positive is separate support work. Likewise,
the gate composition test must execute the real shared checking implementations, not a gate stub
that returns success. It must distinguish a parent-installed route absent from a child, a separately
initialized test-owned child running the real checks with fake authority, and an ordinary child
without authority refusing. Run the full test leg over a depth-declared fixture checkout and retain
ordinary generation/neutral-write controls. The changed-candidate negative changes an admission-relevant
input (approval binding, frozen acquisition, evidence population, record identity, accepting artifact
or another gate invariant); independent evaluation must fail. Harmless changes may pass after fresh
evaluation, but never on the authority of an earlier local result.

Phase C must use real full-schema artifacts, canonical workflow source, real subprocess checks,
producer terminality and native queue admission/readback. An in-process gate stub is labeled as
such in results and cannot close those requirements.

## Implementation slices

The original fourteen slices retain their ids. The 2026-11-13
[shared freeze/readiness amendment and implementation](support-depth-freeze-readiness-design.md)
is immediately next after slices 0–3 and 5, alongside slice 4 and before slice 6. The landing
order is **0 → 1 → 2 → 3 → 5 → shared freeze/readiness work and 4 → 6 → 7 → 10 → 8 → 9 → 11 → 12 → 13**.
Its six inspections found activation blockers in retained IN1 authority, complete freeze-delta
validation, receipt-free checkout/context authentication and genuine MA2 discovery. Its open
questions, write scope, complete test matrix and phase C requirements are recorded in that
design. The conditional evidence amendment grants no implemented checkpoint exception.
The connected fixture chain grows from slices 2–3: each later slice consumes actual preceding
outputs and adds its checkpoint; slice 12 finishes and documents it rather than first connecting
independently seeded positives. These are future write scopes; this packet writes only
this design document. All Rust files are capped at 700 code lines. Extract focused submodules in
the owning slice before adding behavior where a file would exceed the cap; use `make loc-check`.
Do not grow the existing monoliths or move unrelated policy. Every route-changing slice updates
its Annex A forward/backward rows and test references in that same PR; slice 13 reconciles them
against the final enforcement revision. No slice installs a live profile or production declaration.

- **Slice 0 — Normative decisions.**
  - Implemented in this PR: normative amendments and pending validation cases only; no behavior,
    profile, registry declaration or path enablement is installed.
  - Content: implement the recorded decisions; amend the evidence contract for bootstrap and the finite
    onboarding producer variant; amend the charter for wrapper-only proof interpretation and ON3
    E/reporting-publication delivery and freeze reporting coherence. Define explicit read-only gate
    evaluation and the local-preparation/independent-before-protected-acceptance split, without a
    new lifecycle stage or token. State ON4/ON5 entry ownership and handoff refusal boundary. Do not rewrite approved
    OF or CP choices.
  - Write scope: `docs/specs/{support-depth-contract,support-depth-evidence-v1,cli-agent-onboarding-charter,agent-registry-contract}.md`
    only where the approved rows require it, plus the validation test list and this decision status.
  - Dependencies: decisions OR-P1–P14. Proving checks: cross-contract review of acceptance categories,
    no pointer/status authority, no maintenance-evidence substitution and retained handoff semantics.

- **Slice 1 — Finite bootstrap scope and approval adapter.**
  - Status: implemented as credential-free scope/IN1 machinery in
    `xtask/isolated_maintenance/onboarding/` and `depth_acceptance/integration/bootstrap.rs`.
    Tests in `isolated_maintenance/tests/bootstrap.rs` and `bootstrap/declaration.rs` admit before
    protected-tip advance and observe afterward. Public ON1 routing remains slice 2; no live
    profile, production parser enablement or independent host run is installed.
  - Round-4 validation sweep: creation replays ordinary ON1 against **every regular blob of
    the pinned parent**, with unsupported modes refused; no selected-input reconstruction.
    Preparation shares the strict production pointer format core and requires `none\n` bytes.
    Reviewed governing adoption and empty `current.json` now require exact expected bytes,
    including inherited empty-current bytes. The acquisition schema compiler/validator and
    supported-field projections are shared with the production freeze planner; selecting a
    specific artifact definition further restricts the compiled profile schema. Production
    behavior is unchanged by these extractions. New regressions in `bootstrap/sweep.rs` cover
    retained crate/backend regular-file conflicts, all four pointer paths (missing newline,
    whitespace and multiple lines), governing JSON byte drift, and inherited current byte drift.

  - Round-5 validation sweep: all four publication outputs compare complete committed bytes.
    Support Markdown uses freshly derived candidate rows and the **pinned parent's** surrounding
    text; JSON, runtime projection and capability publication use their complete renderers.
    Raw help is admitted only at the compiled version/target's two producer filenames, with
    exact bytes from the snapshot producer's own writer and the help constants compiled into
    the fixture CLI. Foreign/extra/missing files and changed bytes refuse. All bootstrap
    acquisition positives now run the actual snapshot producer against a built fixture binary,
    plus the real wrapper, union and report writers, rather than handwritten shards.
    The follow-up sweep also closes metadata's candidate-retained notes/validation, duplicate
    JSON members and non-writer encodings in acquired JSON, and declaration edits outside its
    table. Metadata reuses NE3 with pinned-parent retained fields and a validated candidate
    timestamp; union/report checks compare complete reconstructed producer bytes (including
    all provenance), and shard/wrapper checks combine complete schema/semantic validation with
    exact typed writer encoding. These stronger byte checks are confined to bootstrap admission;
    ordinary production intake keeps its existing semantics. Regressions cover every publication
    file in preparation and declaration, every acquired JSON output's byte drift, metadata notes,
    duplicate members and byte drift, and unrelated declaration registry text.

    Sweep paths below are relative to `crates/xtask/src/`. Creation checks exact ordinary ON1
    output bytes (only the compiled non-runtime backend field is projected away); subsequent
    checkpoints preserve non-admitted paths byte-for-byte and reject deletions/mode changes.

    | Path class | Production writer / validator reused (`file:fn`), or stronger exact check |
    | --- | --- |
    | Governing approval | `depth_record/onboarding.rs:OnboardingApproval::parse,attribution_mismatch`; `isolated_maintenance/onboarding/approval.rs:read` exact compiled descriptor and preserved protected bytes |
    | Registry entry | `agent_registry.rs:AgentRegistry::parse_unenforced`; `onboard_agent.rs:run_in_workspace` replay; `isolated_maintenance/onboarding/mod.rs:Adapter::registry_delta` exact compiled entry |
    | Governing files | `onboard_agent/preview/governing.rs:governing_files` via ON1 replay; `isolated_maintenance/onboarding/inputs.rs:governing` exact reviewed template bytes |
    | `current.json` | `onboard_agent/preview.rs:render_current_json` exact ordinary ON1 bootstrap bytes through creation, preparation and declaration; `isolated_maintenance/onboarding/inputs.rs:record_free` and `checkpoint.rs:check` refuse any change, including replacement by the acquired versioned union (root-pointer replacement belongs to promotion) |
    | Versioned snapshots / shards | `support_depth/acquisition_schema.rs:compile,validate`; `support_depth/acquisition_basis.rs:supported_shard`; `manifest_union/intake.rs:validate_shards`; `isolated_maintenance/onboarding/encoding.rs:check` complete typed producer bytes |
    | Raw help | `depth_fixture_snapshot.rs:write_raw_help` with the fixture CLI's compiled `bin/depth_fixture/help.rs` constants; `isolated_maintenance/onboarding/raw_help.rs:validate` exact complete files and bounded inventory, plus checkpoint mode checks |
    | Union | `support_depth/acquisition_schema.rs:validate`; `support_depth/acquisition_basis.rs:supported_union`; `manifest_union/intake.rs:validate,validate_bytes` complete reconstructed producer bytes; the finite single-target profile has no cross-target conflicts |
    | Wrapper coverage | `support_depth/acquisition_schema.rs:validate`; `support_depth/acquisition_basis.rs:supported_wrapper`; `manifest_report.rs:run` and `manifest_report/intake.rs:validate`, including wrapper-only checkpoints; `isolated_maintenance/onboarding/encoding.rs:check` complete typed writer bytes |
    | Reports | `support_depth/acquisition_schema.rs:validate`; `manifest_report/intake.rs:validate_bytes` reconstructs the complete expected report set and compares complete writer bytes |
    | Version metadata | `manifest_version_metadata.rs:plan_at` complete NE3 bytes with retained fields from the pinned parent and the candidate's validated RFC3339 writer timestamp |
    | Pointer files | `manifest_validate/pointers.rs:parse_pointer_content` (extracted from `read_pointer_file`), then exact `none\n` bytes |
    | Lifecycle docs / state | `onboard_agent/preview/render.rs:build_docs_preview` and `onboard_agent/lifecycle.rs:build_lifecycle_state_preview` exact replay; `agent_lifecycle.rs:LifecycleState::validate`; later checkpoints preserve parent bytes |
    | Written-paths inventory | `onboard_agent.rs:written_paths_inventory` via ON1 replay, exact host-derived inventory bytes |
    | Release changes / workspace manifest | `onboard_agent/preview/release.rs:build_release_preview` via ON1 replay against all pinned member manifests, exact bytes |
    | Declaration table | `agent_registry.rs:AgentRegistry::parse_unenforced`; `isolated_maintenance/onboarding/mod.rs:Adapter::registry_delta` exact compiled selection; `checkpoint.rs:check` complete parent registry bytes plus the canonical declared table |
    | Classification rows | `support_depth/freeze/onboarding.rs:FreezePlan::onboarding` → `support_depth/acquisition.rs:derive_with_pilot` checks current basis, surfaces and resolved CORE rows |
    | Acquisition statement | `depth_record/acquisition.rs:AcquisitionStatement::parse,mismatch` against the planner's acquired input references; complete TOML parsed with unknown fields refused (maintainer-authored command strings remain attestation) |
    | Reporting publication | `support_matrix.rs:generate_publication_artifacts_from_markdown` with pinned-parent surrounding text; `capability_matrix.rs:generate_markdown_for_workspace` complete output bytes in `depth_acceptance/integration/bootstrap.rs:publication` |
    | `.gitkeep` files | `onboard_agent/preview.rs:build_manifest_preview` → `onboard_agent.rs:preview_writes` exact empty bytes via ON1 replay; later edits refused |
  - Content: extend the existing compiled synthetic route with the bounded ON1 creation state,
    record-free preparation checkpoints preserving the scaffold/approval with IN1 admission and
    post-tip observation, and the OR-P14 scaffold-to-declared transition (IN1 admission of exactly the declaration,
    classification rows, acquisition statement and coherent reporting publication);
    extract/share the strict finite approval reader. Keep fixture identity/descriptor/case/executor
    configuration in the compiled profile/private adapter; shared ON/IN1 checks use validated
    path/Event/subject inputs without identity-based waivers. Keep production parsing unchanged.
  - Write scope: `xtask/isolated_maintenance/{mod,scope}.rs`, focused new onboarding submodule,
    `xtask/runtime_follow_on/depth/approval.rs`, synthetic onboarding templates and new scope tests.
  - Dependencies: 0. Proving tests: absent/unselected subject only in bootstrap, no records there,
    exact descriptor, unknown/foreign scope, no route fallback, maintenance/Codex controls; the
    declaration transition admitted and read back, and refused with any extra table, record, entry
    or acceptance effect.

- **Slice 2 — ON1 public scaffold and truthful rule-4 inputs.**
  - Content: route ON1's existing plan through the bounded adapter; emit no table or fictitious
    metadata. Reuse governing templates, ordinary acquisition and NE3 ordering from OR-P1.
  - Write scope: `xtask/onboard_agent.rs`, `xtask/onboard_agent/{approval,descriptor,lifecycle,validation,mutation}.rs`,
    `xtask/onboard_agent/preview/` and new installed-route bootstrap tests/fixture helpers.
  - Dependencies: 1. Proving tests: actual empty-root scaffold/replay, full canonical validation after
    acquisition/adoption, metadata before declaration, no enrollment writes, unchanged ordinary cases.
    Start the connected fixture chain here with actual ON1 outputs; admit and observe every
    record-free preparation checkpoint and the later OR-P14 declaration before advancing the tip.

- **Slice 3 — ON2 executable freeze and candidate write.**
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
    extend slice 2's actual output chain through IN1 admission/observation of freeze and implementation
    checkpoints; refuse a hand-assembled record.

- **Slice 5 — Onboarding producer Event binding.**
  - Freeze/readiness follow-on status, 2026-11-13 — the genuine historical-freeze positive and
    progression against authenticated IN1 history, including a higher-stage admitted ancestor
    without a producer run, require the shared slice above. Observed-source progression and the
    existing closure-refusal characterization remain the implemented boundary.
  - Content: finite onboarding binding of the existing synthetic producer, exact approval-linked
    population authentication and fixture/adoption identities. Keep cases/receipt schema/obligations.
  - Write scope: `xtask/depth_ci/profile.rs`, relevant collector/archive code,
    `xtask/depth_evidence/verdict/github/` profile/population tests, existing synthetic fixture tree;
    canonical `ci.yml` collection steps only if compiled selection cannot use existing steps.
  - Dependencies: 0–3; land immediately after 3, before repair (4). Proving tests: onboarding eleven-obligation positive through fake authenticated
    host, maintenance/onboarding cross-Event refusal, all-attempt/adoption/source/currentness cases,
    unchanged maintenance producer; Inspection A's named same-generation later-run test and the
    CP-P5 producer/consumer invocation-graph check before phase C. No new job or real host call.

- **Slice 4 — ON2 repair delivery.**
  - Content: scoped repair loading/proposal, preserve P/O and lifecycle written-path integrity;
    check frozen acquired inputs and actual reporting effects before delivery.
  - Write scope: `xtask/repair_runtime_evidence.rs`, focused repair submodule if needed,
    `xtask/runtime_evidence_bundle.rs` only for shared identity/preservation, new routed repair tests.
  - Dependencies: 3 and 5 (landing order). Proving tests: repair sibling chain, stale run/object/digest, delivery rollback,
    no verified depth E and no acceptance, ordinary repair regressions.

- **Slice 6 — ON3 reporting E and runtime selection.**
  - Content: captured approval/registry, authenticated controller assembly, one reporting E/evidence/
    lifecycle/packet/reporting-publication proposal, acquired-input and runtime evidence checks,
    public route wiring. IN1 admits the ON3 reporting checkpoint: E and its files re-authenticated
    against the onboarding producer population, evaluation of onboarding records that carry no
    maintenance entry (today `depth_acceptance/publication.rs:316-336` evaluates only
    maintenance-entry-backed records), and no entry or qualification.
  - Write scope: `xtask/prepare_publication.rs`, `xtask/prepare_publication/`, shared private assembly
    call site, `xtask/depth_acceptance/{publication,integration}.rs` for the reporting dispatch and
    new installed-route reporting tests. Keep the executor record immutable.
  - Dependencies: 3, 5, 4 and completed shared freeze/readiness work. Proving tests: real assembly files bound by delivered E, active-run selection,
    repaired-run alternative, stale/missing evidence and no-entry/no-qualification assertions.

- **Slice 7 — Onboarding owning predicates, without public accepting writes.**
  - Content: factor common adequacy, add exact onboarding binding/currentness and publication/proving
    entry derivation, strict debt-free boundary, effect allowlists and read-only proposed evaluation.
  - Write scope: `xtask/depth_acceptance/` focused onboarding/common modules, narrow changes to
    `manual.rs`/`isolated.rs`; shared `support_depth/` derivation call sites and focused tests.
  - Dependencies: 5–6. Proving tests: positive private predicates, wrong path/Event, all-due failures,
    debt/work/classification refusals, full maintenance closeout/promotion regression suites.

- **Slice 10 — Authenticated read-only gate consumers.**
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
  - Dependencies: 0, 7. Slice 8 uses the proposal phase, so this slice lands before it.
    Track this slice as a maintenance phase C prerequisite alongside CP-P5; the declared-registry
    test-leg repair is required on both paths.
    Proving tests: a credential-free composition test executing the real shared checking code in both phases, with every fixed
    command including `runtime-advertising-model --check` passing scoped and standalone, the
    full `make preflight` test leg passing over a declared candidate registry, and production
    and NE8 write-refusal controls; an in-process pass
    followed by an admission-relevant changed committed candidate failing independent evaluation
    (a harmless change may pass fresh checks), exact-record checks,
    source/host/parent drift, no implicit
    evaluation during ordinary generation, no profile in a child process, and independent test-owned
    process evaluation with fake authority. Actual CLI/green-gate success remains phase C evidence.

  **Inspection B — fixed commands and preflight leaves (read-only baseline inventory).**

  The ON4 inventory is exactly four commands (`xtask/agent_lifecycle.rs:32-37`):
  `cargo run -p xtask -- support-matrix --check`, `cargo run -p xtask -- capability-matrix --check`,
  `cargo run -p xtask -- capability-matrix-audit`, and `make preflight`. The first three occur
  again inside preflight; their independent executions remain required. There is no separate
  top-level `manifest-validate` in this inventory: validators also run through the test leg,
  and per-root `--tip`/all-root IN1 remain separate required CI/integration consumers.

  The actual dependency tree is `preflight → hygiene + flightcheck`; flightcheck sequentially
  calls `fmt-check`, Python tests, `clippy`, `cargo clean`, `check`, `test`,
  `support-matrix-check`, `capability-matrix-guard`, `publish-guards`, `loc-check`, `actionlint`,
  `security`, `unsafe-report` (`Makefile:210-247`). `loc-check → ensure-tokei`,
  `actionlint → ensure-actionlint`, and both `security` and `unsafe-report → ensure-security-tools`
  (`Makefile:98-208`). Below are **23 distinct recipe/check leaves**, counting the two default
  deny checks separately and each conditional setup recipe once; echoes and Make dispatch are
  not checks. The repeated outer commands and repeated security-tool setup are still executed.
  `DENY_CHECKS` defaults to `advisories licenses` (`Makefile:53`); overrides must expand the
  recorded inventory rather than silently dropping a check.

  In this table **S** means a hermetic subprocess over the exact proposal source/files before
  delivery, then the unchanged standalone subprocess over the independently authenticated
  committed candidate before protected acceptance. **A** means the real shared checker over
  the proposal's read-only bound view/borrowed fresh evaluation scope, then the unchanged
  standalone command independently authenticating committed inputs. **T** means the full
  test subprocess, with production-parser controls on explicit undeclared fixtures, candidate
  checks retaining authenticated reading of the actual proposal/candidate, and test-owned
  children initializing their own fake authority; the unchanged full test command must also
  pass independently over a depth-declared committed fixture checkout. A parent route is never
  inherited authority. Git-dependent checks use hermetic repositories or borrowed captured Git
  inputs; scratch `.git` is forbidden. Setup/build outputs live in disposable build/tool areas,
  not frozen acquisition paths. Re-observe frozen and expected-before inputs after all checks.
  External package/advisory/tool services below are not the depth evidence host; hermetic local
  execution needs prepared caches/tools, without changing required checks or result policy.

  | Leaf and source | Registry / depth records / Git / host reads | Proposal phase | Independent committed-candidate phase |
  | --- | --- | --- | --- |
  | `./scripts/check_repo_hygiene.sh` (`Makefile:245`; script `:4-108`) | No registry/depth parsing; Git root/index/tracked content, including tracked docs/data; no host. | S, disposable candidate index/Git view; never run scratch as if it were its enclosing caller repo. | Unchanged script over actual committed checkout/index. |
  | `cargo fmt --all -- --check` (`Makefile:80`) | Source/manifests; no registry/depth/Git/depth-host checking. | S. | Same standalone command. |
  | `python3 -m unittest discover -s scripts -p 'test_*.py'` (`Makefile:215`) | Fixture registry/depth data, temporary Git repositories, mocked host APIs; some product/source assertions. Includes staging gate/transport/archive tests (`scripts/test_staging_integration.py:61`, `scripts/test_maintenance_transport.py:20`, `scripts/test_depth_ci_archive.py:43`, `scripts/test_recommend_next_agent.py:28`). | T, keep fixture Git and fake services isolated. | Full unchanged discovery; no live-host proof claimed. |
  | `cargo clippy --workspace --all-targets --all-features -- -D warnings` (`Makefile:87`) | Compiles source and `include_str!` registry fixtures; does not execute registry/depth/Git/host checks. Cargo may resolve package dependencies. | S. | Same standalone command. |
  | `cargo clean` (`Makefile:217`) | Cargo build state; no registry/depth/Git/depth-host validation. | S, isolated build directory. | Same standalone command. |
  | `cargo check --workspace --all-targets --all-features` (`Makefile:91`) | Same compile-time source/fixture and Cargo inputs as clippy; no executed depth check. | S. | Same standalone command. |
  | `cargo test --workspace --all-targets --all-features` (`Makefile:95`) | Yes: actual registry/depth artifacts, Git/host seams and child commands in tests; reader inventory follows. | T, execute real checking code and preserve each assertion's purpose. | Full unchanged command over declared candidate; candidate-registry checks remain authenticated, not replaced with fixture-only controls. |
  | `cargo run -p xtask -- support-matrix --check` (`Makefile:230`) | Registry, retained depth records/publication, lifecycle/evidence; Git and evidence host under owning routed evaluation (`xtask/support_matrix/derive.rs:264`, `:402`). | A, exact post-entry record and aggregate proposal; includes validation helpers. | Unchanged standalone with its own route/host/candidate binding. |
  | `cargo run -p xtask -- runtime-advertising-model --check` (`Makefile:234`) | Registry twice (`xtask/runtime_advertising/mod.rs:54`, `revision.rs:36`), source/feature closure and generated data; no direct depth-record evaluation; Git/host for authenticated routing. | A; convert both read-only registry readers, retain conformance/revision checks. | Unchanged standalone, both readers routed; ordinary generation and NE8 writes retain refusal. |
  | `cargo run -p xtask -- capability-matrix --check` (`Makefile:235`) | Registry, lifecycle and current manifest projections; no direct depth-record verdict; Git/host through route (`xtask/capability_publication.rs:64-88`). | A, same inventory/render/check functions. | Unchanged independently routed standalone. |
  | `cargo run -p xtask -- capability-matrix-audit` (`Makefile:236`) | Registry, lifecycle/current manifests and capability projections; no direct depth-record verdict; Git/host through route (`xtask/capability_publication.rs:119-140`, `:366-369`). | A, same audit and collection code. | Unchanged independently routed standalone. |
  | `python3 scripts/validate_publish_versions.py` (`Makefile:240`; script `:22-46`) | `VERSION`, Cargo metadata/versions/pins; no agent registry/depth/Git/depth-host parser. | S, real `cargo metadata --no-deps`. | Unchanged script/metadata command. |
  | `python3 scripts/check_publish_readiness.py` (`Makefile:241`; script `:45-82`, `:127-193`) | Cargo metadata/package files; Cargo packaging may consult Git and package service; no agent-registry/depth authority. | S, execute metadata, package listing, dependent `cargo package --no-verify` and dependency-free `cargo publish --dry-run` with existing bootstrap handling; no real publish. | Unchanged script and nested Cargo checks on committed files. |
  | `ensure-tokei` conditional setup (`Makefile:135-146`) | PATH/cache/tool package service; no registry/depth/Git authority. | S, prepare locked installation/cache if needed. | Unchanged setup/cache resolution. |
  | tokei scan → Python LOC policy (`Makefile:98-132`) | Files/source LOC (excluded artifact roots); no registry/depth/Git/host semantics. | S, retain 700-code-line cap and actual scan. | Same scan/policy over committed source. |
  | `ensure-actionlint` conditional setup (`Makefile:149-166`) | PATH/cache/platform and tool release download; no depth host/registry/record parsing. | S, prepared versioned tool/cache. | Unchanged setup/cache resolution. |
  | `actionlint -no-color`, optional shellcheck (`Makefile:169-172`) | Workflow/shell source and installed tools; no registry/depth/Git/depth-host state. | S, retain explicit optional-shellcheck behavior. | Same lint command and tool availability reporting. |
  | `ensure-security-tools` conditional setup (`Makefile:175-188`) | PATH/cache, cargo-audit/deny/geiger package installation; no agent-registry/depth authority. | S, prepared tools/cache. | Unchanged setup; also reached by unsafe-report. |
  | `cargo audit` (`Makefile:194`) | Cargo lock/advisory DB and service/cache; no agent-registry/depth/Git authority beyond tool/advisory retrieval. | S, same audit with prepared DB/cache. | Same audit and security Cargo environment. |
  | `cargo deny check advisories` (`Makefile:195-199`, security recipe) | Cargo graph/lock/advisory configuration and DB/service; no depth evidence host. | S, default first deny check. | Same standalone check. |
  | `cargo deny check licenses` (`Makefile:195-199`) | Cargo graph/license/source metadata/cache; no agent-registry/depth authority; package service as needed. | S, default second deny check. | Same standalone check. |
  | `cargo geiger -p codex` (`Makefile:206`) | Compiled dependency/source unsafe scan and Cargo cache/service; no executed registry/depth evidence. | S, informational `\|\| true` policy and logged output retained. | Same informational command/policy. |
  | `cargo geiger -p xtask` (`Makefile:207`) | Same, including compiled xtask source/embedded registry bytes, not runtime registry/depth acceptance. | S, informational policy retained. | Same informational command/policy. |

  **Registry reader inventory.** Repository-wide search for `AgentRegistry::load` (including
  the callback) and `AgentRegistry::parse(` finds **103 consumer sites: 23 non-test production
  reader sites and 80 test/helper reader sites**. These are syntactic source sites, not runtime
  invocation or test-case counts. The parser implementation (`xtask/agent_registry.rs:50-64`)
  delegates load → load_from_path → parse; it remains production-refusing. Unenforced/isolated
  parses are distinct and are not counted as production-parser readers. This inventory does
  not authorize wholesale conversion of unrelated writers: their existing refusal/default
  behavior remains, and ON behavior changes belong in their owning slices.

  | Non-test production reader sites (all `xtask/`) | Role and execution boundary |
  | --- | --- |
  | `agent_maintenance/docs.rs:498` | Ordinary maintenance doc-context reader; no onboarding acceptance authority. |
  | `agent_maintenance/prepare.rs:202`, `agent_maintenance/prepare.rs:352` | Ordinary planning/apply fallback; existing routed authority remains separate. |
  | `agent_maintenance/request.rs:284` | Production loader callback; keep ordinary authority distinct from routed request validation. |
  | `agent_maintenance/watch.rs:211` | Ordinary release-watch registry; same-version depth transfer remains unsupported. |
  | `close_proving_run.rs:123` | Final ON5 context; owning slice 9. |
  | `depth_acceptance/input.rs:96` | Unscoped source View loader; keep default production refusal; scoped owners use validated registry/views. |
  | `historical_lifecycle_backfill.rs:119` | NE7 ordinary reader; retain write refusal. |
  | `isolated_maintenance/mod.rs:137` | No-route registry fallback; must remain production-refusing. |
  | `manifest_promotion.rs:170` | MA6 non-routed fallback over scratch; preserve fallback refusal and existing captured-registry branch. |
  | `manifest_validate/depth/merge/git.rs:47` | Ordinary baseline Tip reader; preserve routed IN1 dispatch and pinned baseline, no parser bypass. |
  | `onboard_agent.rs:317` | ON1 ordinary planning parse; bounded bootstrap in slices 1–2. |
  | `onboard_agent/validation.rs:128` | ON1 candidate registry validation; keep ordinary scaffold production validation, bounded exceptions in slices 1–2. |
  | `prepare_proving_run_closeout.rs:164` | Prepared ON5 context; owning slice 9. |
  | `prepare_publication.rs:249` | ON3 ordinary context; routed conversion belongs in slice 6, not an ambient parser relaxation. |
  | `publication_refresh.rs:307` | ON4 ordinary context; owning slice 8 adds checked candidate routing after slice 10. |
  | `recommend_next_agent_research/contract.rs:35` | Ordinary recommendation input; preserve production refusal outside the finite route. |
  | `release_doc.rs:22` | ON1 release rendering; scoped candidate reading needed by scaffold and its tests; retain ordinary parser controls. |
  | `repair_runtime_evidence.rs:432` | ON2 repair ordinary context; owning slice 4. |
  | `runtime_advertising/mod.rs:54` | NE8/check preparation; slice 10 converts check reading only. |
  | `runtime_advertising/revision.rs:36` | Check/revision source reader; slice 10 must convert this reader as well as preparation. |
  | `runtime_follow_on.rs:320` | ON2 fallback when no captured registry; slice 3 supplies finite routing, no permissive fallback. |
  | `wrapper_scaffold.rs:114` | Ordinary scaffold consumer; no new depth acceptance or neutral writer authority. |

  For tests the purpose codes are **P — production-parser control**, **C — candidate-registry
  check**, **U — product assumption**. P keeps ordinary acceptance/depth refusal and other
  ordinary behavior controls on explicit hermetic undeclared inputs, rather than the changing
  repository registry. C retains validation of the actual candidate registry and the artifacts
  it names through authenticated reading in each phase; adding a fixture control cannot replace
  it. U narrows only the production-versus-isolated assumption the contracts actually distinguish,
  retaining ordinary controls and adding candidate coverage where a repository invariant is tested.
  A helper site is classified by its assertion consumers; these are not 80 separate tests.
  Prefixes below are `xtask/` for source tests and `crates/xtask/tests/` otherwise.

  | Test/helper reader sites | Purpose and required treatment in both phases |
  | --- | --- |
  | `xtask/agent_maintenance/support_audit/tests.rs:258` | P: constructs synthetic audit fixtures for derivation behavior; use an explicit ordinary registry and keep the derivation assertions. |
  | `xtask/agent_maintenance/support_audit/tests.rs:424` | C: the actual debt-inventory check; keep actual registry/agent-root and inventory reading authenticated. |
  | `xtask/isolated_maintenance/tests.rs:149` | U: canonical-ordinary versus isolated-declared separation; explicit production fixture plus authenticated candidate separation, no blanket candidate-depth absence. |
  | `xtask/isolated_maintenance/tests/registry_tables.rs:36`, `xtask/isolated_maintenance/tests/registry_tables.rs:88`, `xtask/isolated_maintenance/tests/registry_tables.rs:109`, `xtask/isolated_maintenance/tests/registry_tables.rs:138`, `xtask/isolated_maintenance/tests/registry_tables.rs:150` | P at 36/88/109: accepted ordinary versus refused declared/malformed tables and no fallback. U + C at 138/150: committed no-table/no-synthetic assumption plus workspace-load equality; preserve authenticated actual candidate validation while narrowing the production-only assertion. |
  | `xtask/release_doc.rs:97` | P: registry-order/tail-package rendering control; freeze the ordinary fixture and retain release rendering against the actual candidate separately. |
  | `crates/xtask/tests/agent_lifecycle_state.rs:143`, `crates/xtask/tests/agent_lifecycle_state.rs:357`, `crates/xtask/tests/agent_lifecycle_state.rs:524` | C at 143: actual registry-target lifecycle validation. P at 357/524: fixed ordinary sample-state helpers; use fixtures without losing the actual-candidate check. |
  | `crates/xtask/tests/agent_maintenance_audit_status.rs:459`, `crates/xtask/tests/agent_maintenance_audit_status.rs:516` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_audit_status_debt_baseline.rs:237`, `crates/xtask/tests/agent_maintenance_audit_status_debt_baseline.rs:338` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_closeout.rs:107`, `crates/xtask/tests/agent_maintenance_closeout.rs:121` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_closeout/write_outputs.rs:241` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_execute.rs:34` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_prepare.rs:496`, `crates/xtask/tests/agent_maintenance_prepare.rs:556` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_prepare/depth_binding.rs:11` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:248`, `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:415`, `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:440`, `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:460`, `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:540`, `crates/xtask/tests/agent_maintenance_prepare/from_request.rs:565` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_refresh.rs:70`, `crates/xtask/tests/agent_maintenance_refresh.rs:84` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs:14`, `crates/xtask/tests/agent_maintenance_refresh/automated_requests.rs:61` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_maintenance_watch.rs:264`, `crates/xtask/tests/agent_maintenance_watch.rs:545`, `crates/xtask/tests/agent_maintenance_watch.rs:575`, `crates/xtask/tests/agent_maintenance_watch.rs:602`, `crates/xtask/tests/agent_maintenance_watch.rs:629`, `crates/xtask/tests/agent_maintenance_watch.rs:655`, `crates/xtask/tests/agent_maintenance_watch.rs:691` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/agent_registry.rs:41`, `crates/xtask/tests/agent_registry.rs:234`, `crates/xtask/tests/agent_registry.rs:251`, `crates/xtask/tests/agent_registry.rs:264`, `crates/xtask/tests/agent_registry.rs:323`, `crates/xtask/tests/agent_registry.rs:353`, `crates/xtask/tests/agent_registry.rs:398`, `crates/xtask/tests/agent_registry.rs:452`, `crates/xtask/tests/agent_registry.rs:482`, `crates/xtask/tests/agent_registry.rs:521`, `crates/xtask/tests/agent_registry.rs:629`, `crates/xtask/tests/agent_registry.rs:637`, `crates/xtask/tests/agent_registry.rs:678`, `crates/xtask/tests/agent_registry.rs:714`, `crates/xtask/tests/agent_registry.rs:741` | P except 234: ordinary parsing, identity/path/array/gate/capability/governance/release-watch schema controls. C + U at 234: actual workspace loader with fixed five-agent expectation; keep real candidate validation, move only fixed ordinary inventory expectations to fixtures. |
  | `crates/xtask/tests/agent_registry/non_runtime.rs:79` | P: ordinary non-runtime entry control on explicit inputs, without weakening capability/backend rules. |
  | `crates/xtask/tests/agent_registry/support_depth.rs:65`, `crates/xtask/tests/agent_registry/support_depth.rs:85` | U at 65/66: seeded canonical no-depth assumption. P at 85: production depth refusal; keep it with explicit ordinary/declared fixtures and add actual candidate coverage. |
  | `crates/xtask/tests/c0_spec_validate/depth_records/acceptance/publication.rs:15`, `crates/xtask/tests/c0_spec_validate/depth_records/acceptance/publication.rs:176` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/c0_spec_validate/depth_records/acceptance/relay_boundary.rs:37` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/c8_capability_matrix_unit.rs:49` | P: capability projection/gating unit fixtures; no ambient repository parser dependence. |
  | `crates/xtask/tests/c8_spec_capability_matrix_parity.rs:34` | P: fixture-generated manifest/runtime projection parity; use explicit ordinary registry, retain production/runtime inventory assertion. |
  | `crates/xtask/tests/c8_spec_capability_matrix_registry_driven.rs:38`, `crates/xtask/tests/c8_spec_capability_matrix_registry_driven.rs:90`, `crates/xtask/tests/c8_spec_capability_matrix_registry_driven.rs:112` | C at 90 and its 38 seed helper for actual-registry reproduction; preserve authenticated actual candidate reproduction. P at 112 and 38 for the explicit extra-agent fixture: retain registry-driven extension control. |
  | `crates/xtask/tests/c8_spec_capability_matrix_staleness.rs:48` | P: tests fresh/stale detection and refusal without rewriting, not checked-in candidate consistency; use an explicit ordinary registry fixture and preserve those assertions. |
  | `crates/xtask/tests/depth_synthetic_governing.rs:83`, `crates/xtask/tests/depth_synthetic_governing.rs:84`, `crates/xtask/tests/depth_synthetic_governing.rs:116` | P at 83/84: synthetic template production refusal and undeclared non-runtime acceptance. U at 116: canonical excludes synthetic subject/root; narrow to production scope, allow only the installed isolated proof candidate and validate its actual entry/root. |
  | `crates/xtask/tests/prepare_publication_entrypoint.rs:90` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/runtime_advertising/fixtures.rs:41` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/support/agent_maintenance_closeout_harness.rs:95` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/support/agent_maintenance_harness.rs:354`, `crates/xtask/tests/support/agent_maintenance_harness.rs:404` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/support/agent_maintenance_refresh_harness.rs:230` | P: ordinary behavior/fixture construction; make the undeclared registry explicit, preserve the existing request, evidence, target or projection assertions and ordinary refusal controls. |
  | `crates/xtask/tests/support_depth.rs:60` | P: declared policy resolution does not enable production command parsing; retain depth refusal. |
  | `crates/xtask/tests/validator_spec_roots.rs:89`, `crates/xtask/tests/validator_spec_roots.rs:128`, `crates/xtask/tests/validator_spec_roots.rs:158` | C: every actual registry root, Annex B adoption and non-adopting committed records/pointers; retain actual candidate roots, not a fixed ordinary substitute. |

  **Explicit depth-absence assertions.** The ten whole-depth absence assertion sites below
  include registry, lifecycle, request, approval and runtime-contract assumptions, not just
  direct registry parser calls; overlapping reader sites above are not additional readers.
  Keep the refusal setup assertions and ordinary controls; do not require an isolated declared
  candidate to be undeclared.

  | Assertion site | Purpose and slice-10 treatment |
  | --- | --- |
  | `xtask/isolated_maintenance/tests.rs:149-154` | U: production canonical/isolated separation; scope absence to the ordinary control, keep installed/candidate declaration validation. |
  | `xtask/isolated_maintenance/tests/registry_tables.rs:139` | U + C: committed no-depth assertion, with actual workspace load at `:150`; retain authenticated actual candidate coverage. |
  | `crates/xtask/tests/agent_registry/support_depth.rs:66` | U: seeded production registry has no table; scope the assumption, retain P refusal at `:85`. |
  | `crates/xtask/tests/agent_lifecycle_support_depth.rs:56` | U + C: every committed lifecycle round-trips, currently also assumed depth-free; validate actual candidate objects/round-trip and restrict no-depth to ordinary fixtures. |
  | `xtask/prepare_publication/acquired_inputs_tests.rs:260` | P: deliberate deletion creates an orphan-binding refusal case; keep absence and unchanged-state assertion, not a candidate product invariant. |
  | `crates/xtask/tests/agent_maintenance_prepare/depth_binding.rs:10` | P: ordinary request has no depth table before adding a binding; preserve transport/control semantics. |
  | `crates/xtask/tests/depth_record/onboarding_binding.rs:230` | P: committed ordinary approval attribution approves no depth enrollment; retain exact approval/Event attribution. |
  | `crates/xtask/tests/c0_spec_validate/depth_records/acceptance/attribution.rs:7-11` | P: legacy approval attribution remains depth-free; preserve exact attribution/refusal coverage. |
  | `crates/xtask/tests/onboard_agent_entrypoint/governing_validation.rs:85-87` | P: ON1 scaffold intentionally has no lifecycle depth binding; preserve before-declaration rule. |
  | `crates/xtask/tests/runtime_follow_on_entrypoint.rs:79` | P: ordinary runtime input contract omits depth; retain ordinary control beside the new routed positive. |

  Related absence checks have narrower meanings: `xtask/isolated_maintenance/tests/runtime_follow_on.rs:135-137`
  and `crates/xtask/tests/depth_record/onboarding_binding.rs:200` permit an absent O identity in
  a P-only depth object (P: keep partial-freeze semantics); `crates/xtask/tests/depth_synthetic_governing.rs:79-81`
  checks an undeclared version has no resolved policy, and `:117` checks no synthetic subject
  in production (P/U: preserve production scope, not a ban on the installed candidate). These
  do not assert absence of the whole depth object. Source/registry assertions must be adapted
  only at their actual boundary; neither compiled fixture knowledge nor a test's fake authority
  can waive real candidate validation.

- **Slice 8 — ON4 and its IN1 authority together.**
  - Content: packet finalization → exact publication entry → fresh post-entry evaluation/rendering →
    aggregate admission → candidate gates → checked delivery; D7 publication/unchanged-qualification
    dispatch. Production and public neutral writers still refuse.
  - Write scope: `xtask/publication_refresh.rs` and new submodules,
    `xtask/depth_acceptance/{integration,publication}.rs` and focused integration submodules,
    `xtask/isolated_maintenance/tests/` onboarding publication/IN1 tests.
  - Dependencies: 6–7 and 10. Proving tests: qualified synthetic promise without pointers/status,
    exact entry/packet, aggregate wrong-owner effects, pre-entry verdict refusal, gate/concurrent
    restoration failures, admission plus observation, B.7 maintenance chains unchanged.

- **Slice 9 — Both ON5 commands and their IN1 authority together.**
  - Content: prepared-only reporting proposal with acquired-input/retarget checks; final closed
    bytes/entry/lifecycle/docs batch, post-entry evaluation, proving-effect D7 dispatch. No drift clearance.
  - Write scope: `xtask/prepare_proving_run_closeout.rs`, `xtask/close_proving_run.rs` and extracted
    submodules, onboarding predicates and `xtask/depth_acceptance/integration/`, new ON5 tests.
  - Dependencies: 8. Proving tests: prepared → closed, deferred/release-watch settlement, timestamps
    and replacement bytes, lifecycle evidence, forged/retained entries, no same-version path transfer,
    local/final agreement and conflict-preserving refusal.

- **Slice 11 — E.8 executed mediation matrix.**
  - Content: run every MA/ON/NE/LW1 boundary and alias with onboarding enrollments, retained records,
    omitted scope and proposed outputs; repair only demonstrated guard gaps. Reconcile all callers
    of private delivery and all restoration writers with Annex A.
  - Write scope: focused route tests under `xtask/isolated_maintenance/tests/` and
    `crates/xtask/tests/`, binary entrypoint harnesses; `xtask/depth_routes.rs` and named affected
    entry guards only if a failing conformance test requires a fix; Annex A/A.4.
  - Dependencies: 8–10. Proving tests: rules 1–7 matrix above, production depth refusal controls,
    retained/deleted declarations, nested/proposed destinations, later-version displacement,
    wrong-path MA effects and ordinary non-depth regressions. No generic bypass is introduced.

- **Slice 12 — E.7 connected new-agent implementation proof.**
  - Content: finish and document the chain grown from slices 2–3, consuming actual preceding
    outputs through public in-process entries; fake host and offline
    executor, independent onboarding producer/adoption, IN1 at each committed checkpoint. Record
    any chain-test gate seams and the missing phase C host run explicitly; slice 10's real
    checking-code composition remains required and cannot be replaced by those seams.
  - Write scope: new connected onboarding chain/fixture helper tests and a bounded proof result in
    `docs/agents/lifecycle/`; reuse B.1 fixtures instead of another subject/profile family.
  - Dependencies: 2–11. Proving tests: qualified chain and truthful-insufficiency sibling chain,
    repair alternative, every negative group; ordinary and maintenance connected controls.

- **Slice 13 — E.8 final inventory and phase C handoff.**
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
  forbids the positive; OR-P5 is an approved amendment to land in slice 0, not an inference that the existing
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
  it. Slice 10 resolves it and is a recorded maintenance phase C prerequisite alongside CP-P5.
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
