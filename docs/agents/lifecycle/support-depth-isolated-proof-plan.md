# Phase E1: isolated maintenance-chain proof plan

> Approved [MP-P1](support-depth-machinery-proof-rescope.md) replaces E1-P1's proof subject with a
> synthetic isolated agent (machinery proof only); E1-P2's threshold and the chain are unchanged.

Status: E1-P1 and E1-P2 approved by the maintainer on 2026-10-25.
The historical prerequisite sequence is retained below. The current implementation and remaining
independent host obligations are recorded in the phase C handoff at the end of this plan.
The original E1-P1 one-CORE mapped-proof premise is retired by approved
[PI-P1–PI-P3](support-depth-packet-intake-design.md): initial mutation effects and complete
initial/resume/fork declarations need separate review before positive mapped proof. The GNU target
premise was superseded by IR-P2's acquired musl target; existing CORE fixtures remain diagnostic.
Historical plan baseline is staging `dd10558157583f1969b4ac0a412107ef617648e1` (PR #296).
The phase C handoff is reconciled at `e0d6ffbcd8cce99ccdcdfb448c662622b1e65ddc`.
This plan accompanies the refreshed [Annex A inventory](../../specs/support-depth-contract.md#annex-a-route-inventory).
The [support-depth contract](../../specs/support-depth-contract.md) remains normative. E1 changes
no code, enrollment, host settings, credentials or automation. It creates no background jobs.

## Outcome and present limits

The desired result is one connected maintenance run whose final acceptance is made through a
protected isolated integration branch: acquisition → frozen P/O → execution/evidence → audit →
manual closeout → separate promotion → exact-result integration. Every rejected attempt must leave
no unadmitted final acceptance effect. Existing unchanged historical records and truthful
`unverified` reporting must continue to work.

At the E1 source baseline, the following gaps prevented claiming that positive public chain.
The evidence-bridge increment adds the isolated CORE consumer described in the
[evidence contract](../../specs/support-depth-evidence-v1.md#isolated-codex-core-evidence-bridge);
public routes and production evidence remain separate work:

1. `AgentRegistry::parse` rejects declared depth enrollment; public preparation also refuses
   retained depth. Unsupported refresh/neutral/onboarding writers have their own refusals.
   Existing positive preparation/closeout/promotion tests enter private cores. Removing one parser
   check would neither connect every public caller nor establish authority.
2. `depth_evidence/results.rs::production_results` returns empty verdicts. The live D2 adapter is
   fixed to the canonical repository and isolated `codex/0.0.0/controlled_channel/V1` on Linux.
   `depth-ci-prove` produces a read-only preview, not a result set consumable by the public manual
   and promotion routes. Copying that witness or changing its repository/version is not an adapter.
3. The fixed-producer positives under `depth_acceptance/manual/tests*` establish owning-predicate
   composition. They supply no authenticated adequacy for the other due obligations and no positive
   public-entrypoint evidence. Likewise shell fixtures that stub freeze cannot prove acquisition
   through real P/O freezing. We will reuse these tests without relabeling their claims.

Consequently an isolated rehearsal using those seams may diagnose wiring, but is not the contract's
complete-chain/path-enablement proof. There is no reason to introduce a new rehearsal framework as
an additional acceptance milestone. The missing bounded wiring/evidence work comes first.

## Approved decisions

| Id | Recommendation | Reason and boundary |
| --- | --- | --- |
| E1-P1 — original proof scope (superseded by IR-P2 and PI-P1–PI-P3) | Originally: use one Codex `run` / `CORE` promise through the actual adapter, one headless selector and `x86_64-unknown-linux-gnu`. Pin the exact existing upstream version/artifact evidence at implementation start; baseline `latest_validated` is `0.156.1`. Include its mapped capability where the resolved P claims it | Exercises the actual streaming maintenance path while containing producer/fixture work. Cover every resolved BASE, output/lifecycle and mapping/capability obligation; isolated channel V1 is not a substitute. Other agents/targets get preservation/refusal cases, not new qualification claims. No new debt or blanket `not_applicable` permissions to make the positive pass |
| E1-P2 — proof threshold and next work | Require authenticated all-due evidence plus the real supported public entrypoints before declaring the full-chain proof complete. Authorize the two bounded prerequisite increments below, then the isolated run; keep production enrollment unchanged | Avoids counting more fabricated-verdict successes as readiness. The isolated profile must remain distinguishable from production; there is no generic `--allow-depth`, environment bypass, receipt-supplied host/repository or unconditional parser removal. Any public routing change must be independently reviewed and exercised in isolation before landing |

Existing requirements need no new vote: separate closeout/promotion, exact generation marker on the
integration parent, current bindings/evidence/debt, unsupported-route refusal, native merge queue,
and later separate per-agent production authorization. This plan does not amend those policies.

## Small implementation sequence after approval

1. **Evidence bridge for the one selected profile.** Reuse collector payloads, typed verdicts and
   GitHub authentication. Map existing substantive wrapper/adapter tests to the profile's complete
   resolved obligation set, with exact case population, environment/build/input closure and
   independent adequacy review/maintainer adoption. Expose the evaluated result to owning consumers
   without a caller-constructible success callback. The D2 isolated origin remains isolated; a real
   production profile requires its own reviewed provenance. Do not convert `production_results`
   into a permissive fallback. Deliver read-only positive/negative evidence tests first.
2. **Isolated public-path wiring.** Connect supported preparation/acquisition, reporting publication,
   executor, both manual closeout writers and promotion to the existing owning cores for the exact
   approved isolated profile. Resolve all public-loader and writer refusals that currently prevent
   that supported path, while preserving neutral/onboarding/unsupported refusals. The profile's
   source, repository, version, target and origin must be independently bound and unable to grant
   authority in the production repository. No general-purpose enrollment bypass or second policy
   inventory. Include the final integration checker, not just local commands. Specify and review
   this narrow binding before implementation; E1 does not invent an already usable switch.
   The approved [binding design](support-depth-isolated-route-design.md) records the exact-host
   boundary, real-packet intake/evidence prerequisites and source increment order. Its IR-P1/IR-P2/
   IR-P3 recommendations were approved on 2026-10-25, including isolated producer-job terminality
   with unchanged pre-MERGE workflow gates. Implementation and host proof remain separate; approval
   alone enables no path.
3. **Run and retain the chain below.** Use the resulting reviewed source unchanged for each step
   and for actual protected-host PR/queue evaluation. If a prerequisite cannot meet the scope or
   a route remains unsupported, record the exact unmet claim and stop dependent acceptance proof.
   Fix demonstrated gaps in a small reviewed change, then rerun affected proof; do not expand to
   every target/agent or replace the queue/validator framework.

Each source increment uses both GPT-6.1 Sol High review lanes at standard/default speed and its
required gates. Split an increment further only if its actual diff or proof boundary needs it.
No production registry declaration changes in either increment. A public API path that still
refuses cannot be claimed exercised positively through a private callback.

## Isolation and authority

Use a disposable GitHub repository and an ephemeral runner/container with only that repository's
working files. A new worktree under the connected Linux host is not a sandbox: it exposes the
host's credentials and sibling checkouts. Do not give the executor production credentials, an
active production worktree, host home, shared writable Git metadata or a writable production
remote. Provision no local daemon, timer or detached agent.

Before E2 host writes, obtain authorization for the exact repository and least-privilege credential
reference; never request token values in chat. Delivery credentials are scoped to that repository
and never passed to the executor; only host-controlled delivery receives them after validation. Resolve host repository
identity independently at each remote mutation; changing a Git URL or receipt field is not proof
of isolation. Keep production pointers/closeouts/registry read-only to the proof and compare their
identities before/after. Stop on any mismatch.

Install/read back the existing desired staging ruleset in the disposable repository, including
bypass actors, effective/inherited rules, legacy overlap, check app and MERGE queue configuration.
Do not alter production settings. Initial establishment of a disposable protected baseline is
separate from any depth-positive candidate. Readiness for a new isolated profile is established
by the approved trust configuration and independent mapping adoption, not by that candidate
endorsing its own producer hash. The current D2 fixed-repository allowlist must not simply be
relaxed to accept receipt-supplied repository names.

## One chain and checkpoints

Use ordinary existing commands/workflows and their admitted lower paths after the prerequisites;
there is no new maintenance orchestration service or mutable status ledger. Record exact commands
and revisions as the proof runs, not fabricated runnable commands for today's refusing CLI.

| Step | Action | Required observable checkpoint |
| --- | --- | --- |
| 0 — baseline | Pin reviewed implementation, isolated host/repository, exact upstream version, Linux artifact/inputs, selector/template/capability mapping and no outstanding debt | Complete profile and source identity; desired host readback; production identities unchanged; isolated producer trust and mapping adoption independently established |
| 1 — intake and first freeze | Open the packet through MA1 and deliver via its real computed path set | Request Event and P match the record. If acquisition is absent, O/E remain absent and readiness remains false. First commit of the selection must be integrated on the isolated branch before treating it as retained |
| 2 — acquisition and O | Run MA2 to a preparation branch; stage acquired snapshots/reports before actual re-freeze; deliver full support bundle and audit result | O derived from actual pinned inputs; request P/O identities agree. Stand-down/token/destination protections apply. A changed O clears E rather than resealing old results. Commit/integrate required baseline before later closeout relies on it |
| 3 — execution and evidence | Run MA4 against an independent candidate using the selected wrapper/adapter; capture authenticated cases, exact outcomes and input/build identities | Only permitted reporting E/work changes delivered, no closeout/entry/request/marker/acceptance mutation. Every due result is independently evaluated or truthfully unverified; skipped/zero/missing/failed cases never become verified |
| 4 — audit and packet freeze | Audit current acquisition, classification, carried work, evidence/reuse and debt. Land the exact-generation stand-down marker on isolated staging before adjudication | No `classification_required`, unresolved due work or unsupported debt edit. Marker precedes new closeout authority on the integration parent; a version-only marker for another generation refuses |
| 5 — manual closeout | Exercise each manual writer on equivalent isolated branches; integrate one resulting closeout candidate through IN1 | Same-batch current-digest closeout entry with exact request/artifact identity; current P/O/E and all due evidence/debt admitted. Failure preserves old artifact or absence. At integration, authority and marker are independently re-established against the exact result |
| 6 — separate promotion | From committed isolated staging, validate exact targets then run MA6; include a replay with later-generation working files replacing the old request/closeout | One coherent pointer/current/metadata/publication/entry/marker-retirement proposal. Retained closeout authority survives working-file replacement but stale bindings/evidence/debt refuse. Validate and merge through the native MERGE queue |
| 7 — final readback | Inspect actual queue commit/base/tree, post-push transition and all derived artifacts | Every record/entry/selection retained; exact owning authority and aggregate freshness; correct promoted pointers and marker retirement; production identities/settings unchanged |

Branches/forks for negative cases start from the named checkpoint, so a rejection is compared with
its own expected-before state. The main positive has no debt transition: reuse conditional
renewal/whole-row-retirement planner tests, which retain unresolved corroboration requirements,
and add a retained-debt negative fork to establish connected refusal.
MA4's current refusal of retained debt inventory edits remains valid; a successful new debt writer
is outside this proof unless separately designed. Debt cannot substitute for depth obligations.

## Minimum rejection and recovery matrix

Reuse existing fixtures and add connected coverage only where the chain introduces a new proof
boundary. Keep the matrix in this plan and raw results beside the run, not a second release policy.

| Cases | Expected result / preservation |
| --- | --- |
| Missing/incorrect Event/P/O, stale acquired bytes, unresolved classification/due work | Refuse readiness/closeout/promotion as appropriate; preserve prior record/work/entries. Changed-request and other-root reporting-only edits meet generation checks |
| Forged receipt/digest/matching entry, wrong producer/source/repository/target, unadopted mapping, skipped/empty/missing/contradictory attempts | Remain unverified/refuse acceptance. Receipt JSON and a successful job alone never create adequacy |
| Dependency-only invalidation, changed implementation/config/build graph, unrelated edit control | Relevant drift invalidates qualification/reuse; truthful downgrade accepted. Unrelated edits preserve applicable evidence only where the actual evaluator establishes reuse |
| Executor nonzero exit, gate/spawn/observation error, out-of-envelope write, request/marker/entry/closeout attempt, another-agent aggregate acceptance | Unconditional guard/finalization; no unauthorized product delivery or final acceptance. Retain attributable diagnostics/recovery candidate |
| Concurrent caller/index/input/output/type/mode change and interruption during delivery/rollback | Preserve externally changed state; report conflict. No destructive snapshot restoration or crash-atomic claim |
| Candidate-only/wrong-generation marker, unsupported lifecycle authority, stale retained closeout, missing own promotion entry, incoherent marker removal | Local owning route and final IN1 both refuse; another path's authority or older entry cannot supply permission |
| Record deletion, entry edit/reorder, selection shrink, omitted/nested/unadopted root; secondary agent aggregate change | All-root IN1/lower guard refuses even when the requested agent/version is ordinary or omitted |
| Every unsupported public route/lower boundary in Annex A | Invoke valid entrypoint fixtures with retained depth (and removed-declaration control); refuse before protected delivery. Ordinary equivalent still works. Source guard inspection is not an executed negative |
| Direct/force/delete ref attempt, wrong-app/missing/failed check, moving integration base, two candidates invalidating the same aggregate | Host refuses or rebuilds/revalidates exact current candidate; native queue supplies conditional final ref update. Reuse existing host-proof cases and rerun only when host/profile/source differences invalidate them |

## Evidence and stopping point

Retain one Markdown run index with source/base/tree/host/repository/profile identities, exact commands
and exit statuses, existing typed records/receipts/entry identities, CI/artifact links and the
negative checkpoints' before/after observations. Reuse Git objects, raw logs and current schemas;
no new proof registry, transaction engine or generic receipt schema is needed.

A complete-chain claim requires all positive checkpoints, the applicable negative/refusal coverage,
independently adequate all-due evidence, true public-route execution and exact host integration.
Record private composition, authenticated single-obligation proof, public refusals and full-chain
proof as different facts. A missing prerequisite stays an unmet claim; do not rename it complete.

After the isolated proof, re-establish inventory/host correspondence for any material source change.
Production enrollment still needs separately reviewed production-origin evidence integration and
explicit per-agent maintainer authorization under Path enablement. Isolated selections/receipts
never qualify a real production promise. Independent onboarding enablement remains separate;
unsupported onboarding routes continue to refuse depth effects.


## Phase C handoff after slice 13

Phase C remains closed. Slice 13 authorizes this documentation reconciliation only.
Before any disposable-host operation, the maintainer must explicitly authorize the exact public
disposable repository, protected installation baseline and credential references. A read-only
credential REFERENCE is required for independent observations of the disposable repository and
canonical protected source. Record a secret-store reference or approved credential handle.
Credential values belong in the authorized host environment. Separate host-controlled delivery
credentials require the existing repository-scoped authorization and stay outside executor reach.
[CP-P5](support-depth-closeout-promotion-design.md#g9--ci-and-host-wiring-of-the-isolated-repository-phase-c),
[OR-P12](support-depth-onboarding-routing-design.md#maintainer-decisions).

### Implementation evidence carried forward

The enforcement revision is `e0d6ffbcd8cce99ccdcdfb448c662622b1e65ddc`.
The completed `e7_connected_onboarding_qualified_chain` consumes public ON1 bootstrap,
reviewed governing adoption, actual B.1 acquisition, ordinary snapshotted metadata and separate
declaration, ON2 freeze/implementation, independently modeled onboarding evidence, ON3 E,
ON4 Publication and prepared/human/final ON5 through real IN1 and modeled readback.
`e7_connected_onboarding_repaired_qualified_chain` covers the public repair sibling.
`e7_connected_truthful_insufficiency_refuses_on4_and_both_on5_entries` covers zero execution
credit and preserved unqualified reporting. `e7_connected_onboarding_release_watch_chain`
retains a pre-freeze authority-setup seam, so its settled ON2→ON5 variant has that bounded claim.
[S12 results and negative map](s12-e7-connected-proof-report.md).

`readonly_gates_two_phases_real_checkers_and_independent_process` and
`readonly_gates_proposal_runs_every_a_checker` passed with actual shared checking code and
independently initialized fake authority in a test-owned child. The full declared-registry
workspace legs retain the lead's execution ownership and their report status. The independent
host run must execute the complete subprocess inventory, including actual wrapper tests.
[S10 results](s10-readonly-gates-report.md#focused-validation).

FR1–FR3 provide independently versioned admission/reporting contexts, exact-job commitments,
complete-tree freeze/acquisition classification and canonical MA2 discovery. The fake-host
proofs retain reporting-only attempts with zero obligation credit and irreversible progression
over higher-stage admissions without producer runs. Real deployed issuance, executable/source
binding, retention and trusted-emitter isolation still need host evidence.
[FR1](fr1-admission-history-report.md), [FR2](fr2-freeze-planner-report.md),
[FR3](fr3-reporting-context-report.md).

### Prerequisites needing review and authorization

| Prerequisite | Required handoff evidence |
| --- | --- |
| Disposable public repository and temporary runner | Exact repository/owner IDs, accepted origin, independent installation baseline and compiled profile binding, protected staging ruleset readback and explicit maintainer authorization. Source installation is independently approved |
| Read-only observation credential reference | Authorized reference and least-privilege scope covering independent disposable/canonical repository observations, with a verified read path in every route consumer |
| Canonical `ci.yml` producer/consumer layout amendment | CP-P5 needs a reviewed invocation graph covering every routed evidence consumer, including the recursive workspace-test leg inside `rust-tests`. The current layout places workspace tests before synthetic collection in that same job. The review must resolve terminal-producer ordering for those consumers and place the trusted reporting branch before candidate execution. Review ordinary scheduling/latency and emitter isolation before landing any amendment |
| Consumer scheduling | Every preflight selection, support/read-only check, per-root `manifest-validate --tip` and integration consumer follows a terminal authenticated `rust-tests` producer through failure-preserving dependencies. Failed/cancelled/missing producers remain observable. The producer never invokes or waits for its own admission. The existing 17-job result policy and terminal gate continue to govern |
| Canonical workflow provenance | Reviewed canonical workflow source is part of the independently approved protected closure. The disposable repository executes those canonical bytes. Any proposed isolated-only divergence requires a separately reviewed contract amendment |
| History and root coverage | Full history in every route-resolving checkout, including post-push validation, accepted `origin`, registry-derived per-root validation loops covering the subject. Canonical root adoption and complete validation are required for every retained root |
| Genuine producer/artifact/adoption population | Real full-schema witnesses, receipts, payloads, reporting contexts and IN1 admission records with independent run/job/attempt/log/artifact commitments. Adoption binds the reviewed onboarding approval Event and exact source. Every applicable attempt and retry is discoverable |

The `ci.yml` layout amendment remains a phase C review prerequisite. Slice 13 leaves its design
and implementation pending. Current source anchors are `.github/workflows/ci.yml:166`, `:248`,
`:345`, `:623` and `.github/workflows/depth-tip.yml:33`. The latter currently fetches a shallow
parent and loops fixed roots. The route's absent live profile is
`crates/xtask/src/isolated_maintenance/mod.rs:47`. Each source/profile/workflow amendment needs
fresh correspondence with the final reviewed installation revision.

### Independent run and retained observations

| Checkpoint | Required proof |
| --- | --- |
| Protected baseline | Independently read effective desired rules, inherited/legacy protections, bypass actors, check application, required checks and native MERGE queue. Retain protected source/repository/profile/tree identities and production before/after identities |
| Real new-entry onboarding | Start from the absent synthetic entry/root. Use public CLI subprocesses for ON1 and each supported ON2–ON5 form over actual preceding outputs. Integrate declaration, freeze, implementation, reporting, Publication and proving checkpoints under their exact independent admission |
| Real checks | Execute support-matrix check, capability check, capability audit and every `make preflight` S/T leaf as unchanged subprocesses over the actual committed candidate. Proposal A checks bind exact scratch bytes. Each process resolves independent installed authority. Preserve frozen O and re-observe candidate/expected-before inputs after gates |
| Producer terminality | Observe the exact existing producer job and successful collection/upload/binding steps. Exercise failed, cancelled, missing and retried attempts with the complete population retained. Evidence may be consumed after producer terminality while unrelated workflow jobs continue. Full workflow/checks/terminal gate must succeed before MERGE |
| Real artifact forms and authority | Retain complete schema-valid witness/receipt/payload, acquisition authority, reporting context and IN1 admission bytes, digests and exact emitting job/attempt/log commitments. Re-authenticate all eleven onboarding obligations and fresh independent adoption. Reporting-only freeze attempts retain zero execution/adoption credit |
| Native queue admission and readback | Queue the actual PR candidate against the pinned current integration base. Run real all-root IN1 and per-root validators on the exact merge-group result. Retain queue/base/head/tree/check identities, conditional protected-ref update and post-push admission/readback of the actual transition. A moved base requires fresh current admission |
| Refusal/recovery | Execute applicable Annex A public-boundary negatives and S12's seven negative groups, including retained/omitted/malformed roots, hand edits, foreign aggregate acceptance, missing evidence, stale currentness and concurrent delivery changes. Retain each case's before/after bytes/modes/kinds and bounded refusal |
| Maintenance control | Run E1 steps 0–7 and its rejection matrix in a separate maintenance-path workspace. Independently prove both MA5 writers, separate MA6 promotion and retained-closeout replay through the native queue. Complete socket-dependent assertions that remain unverified in local reports |

Each retained run index records exact source/installation revisions, repository/owner IDs,
workflow/profile/approval/Event/P/O identities, commands and exits, full-schema artifact
identities, independent adoption, complete producer/admission populations, actual queue/base/tree
and post-tip observations. Links to host artifacts and raw logs accompany the index. Every
positive requires independently adequate evidence for every due obligation. A rejected candidate
preserves concurrent state and receives a retained refusal/recovery record as the existing
protocol requires.

### Per-root adoption and all-root D7

`RULES.json` must name the canonical depth-record path, the root's validator spec must adopt
Annex B, and complete manifest validation must execute for each root holding a record.
`crates/xtask/src/manifest_validate/depth/merge.rs:44` additionally invokes all-root IN1 before
requested-root narrowing. `depth_acceptance/integration.rs:172` compares the old/new registry
union. A named ordinary root therefore still detects a foreign retained-root violation.
All-root D7 and per-root canonical adoption/validation are independently required.
The current fixed Codex/Claude/OpenCode loops need CP-P5's registry-derived coverage for the
synthetic root. `aider` and `gemini_cli` retain their schema-free, unadopted state.

Production path enablement and real-agent qualification each retain their separate authorization
and evidence requirements after the independent machinery run. Same-version onboarding-to-
maintenance transfer remains refused pending its transition design. The current synthetic
positive remains wrapper-only and debt-free.
