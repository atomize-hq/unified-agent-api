# Support depth: onboarding freeze decisions

Status: OF-P1 to OF-P9 approved by the maintainer on 2026-11-05, after two rounds of
independent review of PR #329. OF-P3 is approved as option (A), and OF-P4 as option (B) together
with its integration-branch-only constraint. Every other row is approved as written. The options
not chosen are kept below for the record. Approval enables no path and installs no profile.
This is roadmap items E.1 and E.3 of [MP-P1](support-depth-machinery-proof-rescope.md).
Source baseline: `feat/depth-isolated-ma4-route` at `f8fcdc8` (PR #323). Line references are to
that tree.
The [support-depth contract](../../specs/support-depth-contract.md), the
[onboarding charter](../../specs/cli-agent-onboarding-charter.md#support-depth), the
[registry contract](../../specs/agent-registry-contract.md#support-depth), the
[maintenance request contract](../../specs/maintenance-request-contract-v1.md#depth-enrolled-generations)
and the [evidence contract](../../specs/support-depth-evidence-v1.md) remain normative and are not
amended here. Every decision below that needs a contract or charter change says so; each such
revision is a separate, later PR. This document changes no code, registry, host setting or
credential, and creates no background job. Nothing in it is specific to one agent.

## Why these decisions are needed

The contract lets the onboarding path be enabled only when the charter "defines where P and O are
frozen on the create lane" and "names the committed artifact that states, beside the depth record,
the P and O identities a create-lane freeze froze, with who writes it and who may replace it"
(`support-depth-contract.md:824-831`). The charter already makes `runtime-follow-on --dry-run` the
freeze (`cli-agent-onboarding-charter.md:317-323`), but leaves open:
- the command that acquires the surface, and what marks a surface as acquired (`:324-331`);
- the committed artifact that states P and O identities (`:332-338`).

The validation test list records the same gaps and adds four more
(`support-depth-validation-test-list.md:435-452`):
- whether a dry run is admitted after `runtime_integrated`;
- what the acquired inputs of an onboarding O are;
- whether a partial single-host union can freeze O;
- why neither the run's input contract nor `written-paths.json` can be trusted.

## Existing mechanisms reused

Each decision below reuses one of these where it can. Where a decision needs something new, the
row says so.

| Mechanism | Where | What it already does | Limit |
| --- | --- | --- | --- |
| Request `[support_depth]` table | `maintenance-request-contract-v1.md:517-562`; `depth_record/request.rs:1-40` | A second committed statement of `policy_identity` and `obligations_identity`. O is absent while O is not frozen. | Maintenance only. |
| Record/request cross-check | `depth_record.rs:80-87`, `:310-322` (`RequestBinding`, `request_mismatch`) | Rejects a record whose Event, P or O is not what the second statement froze. | Takes any `Event`, so the onboarding Event fits unchanged. |
| Freeze and re-freeze rules | `maintenance-request-contract-v1.md:564-587` | The first freeze writes P and the record. A freeze with acquired reports writes O. A re-freeze keeps P and refuses a changed P. | Has no refusal for changed O inputs (test list `:397-399`). |
| Shared O derivation | `support_depth/acquisition.rs:26-160` | Derives O from the bound acquired inputs. Refuses an acquisition with no union, and a selected target with no shard. | Admits `complete:false`. |
| Bound-input reader | `git_inputs.rs:1`, `:108-210` (`GitInputs`, `verify`); used by `support_depth::plan_freeze` (`support_depth/freeze.rs:37`) | Reads only regular stage-0 files whose working bytes equal the staged blob. Refuses untracked, unmerged and divergent inputs. | Proves index consistency, not where the bytes came from. |
| Relay acquired-input check | `depth_acceptance/relay.rs:34-48` | Refuses a run that changed a frozen acquired input ("explicit re-freeze required"). | Runs only in MA4. |
| MA4 independent candidate | `agent_maintenance/execute/candidate.rs:26-75` (`prepare`), `:76` (`observe`), `:101-170` (`check`), `:171-214` (`deliver`) | Captures the caller's before-tree in the controller's memory, never in a file the executor can reach. Runs the executor in a disposable copy. Protects the request, stand-down markers, input contract and prompt. Rejects link, mode and type changes. Re-observes the tree after the validators run. On delivery, refuses if the caller changed. | "Candidate is not an OS sandbox" (contract Annex A.1, MA4). The machinery lives in `agent_maintenance/execute/`. |
| MA4 unconditional finalization | `agent_maintenance/execute/guarded.rs:12-154` | No `?` between execution and observation (`:29-35`). The controller computes written paths from its own observation (`:81-95`). It writes diagnostics after observing (`:96`, `:156`). A failed run delivers only the before-tree plus diagnostics (`:97-101`). The route is rechecked before delivery (`:104-115`). | Same as above. |
| Stand-down marker read from a ref | `agent_maintenance/stand_down.rs:220-223` (`--from-ref`, otherwise the working tree); `depth_acceptance/manual.rs:185-195` (closeout reads the marker from its integration `tip`); `stand_down.rs:430-445` (`require_generation_marker`) | Lets a caller read authority from a named ref instead of the working branch. | The command reads the working tree when `--from-ref` is absent, and it does not check that the ref is a fresh integration tip. A caller that needs integration-branch authority must fetch that ref and name it itself. OF-P4 adopts that as a requirement on the dry run; no earlier rule prescribed it. |
| Isolated route and protected tip | `isolated_maintenance/mod.rs`; `isolated_maintenance/scope.rs:89-91`, `:126`, `:150`; [isolated route design](support-depth-isolated-route-design.md) | Binds the repository, protected tip and baseline for a proof workspace. Production has no route. | Admits only the maintenance path, maintenance Events and maintenance selections. |
| Lifecycle state | `agent_lifecycle.rs:21-27`, `:302-327`; `agent_lifecycle/storage.rs:54-66` | A committed, typed file per onboarding pack. It is bound to the approval's SHA-256, written only through the LW1 lifecycle writer (which refuses depth), and outside the write run's allowed paths. | It does not deny unknown fields (`agent_lifecycle.rs:301-302`). |
| Approval maintenance mode | `cli-agent-onboarding-charter.md:30-33` | An approval freezes `release_watch_enrolled` or `explicitly_deferred`. | — |
| Approval `[support_depth]` loader | `approval_artifact.rs:49`, `:317`, `:459` | Reads the approved exact version. Rejects misspelled or misplaced tables. | — |
| Write-run boundary (create lane) | `runtime_follow_on/codex_exec.rs:35-55`, `:136-163`; `runtime_follow_on.rs:401-490` | Compares the tree before and after the executor. Rejects writes outside the allowed paths and publication-owned manifest writes. | The baseline is read back from the executor-writable `input-contract.json` (`runtime_follow_on.rs:180-181`, `:407-408`). The executor runs in the caller workspace. |
| Runtime-evidence reader | `prepare_publication/runtime_evidence.rs:96-276` | Validates a run's evidence files. Refuses an empty `written-paths.json` (`:241-248`). | Trusts the content of an executor-writable file. |
| Input-only acquisition writers | `support-depth-contract.md` A.2, "Input-only writers" | The per-agent snapshot command, `manifest-union`, the wrapper-coverage generator and `manifest-report`. They change O inputs and grant no acceptance. None of them refuses a depth workspace. | The `manifest-acquisition-plan` preview refuses agents without `maintenance.release_watch` (`manifest_acquisition.rs:159-161`). |

## Approved decisions

Every row was approved by the maintainer on 2026-11-05. Where a row offered options, the chosen
option is marked **Approved** and the others **Not chosen**.

| Id | Decision | Grounds / boundary | Status |
| --- | --- | --- | --- |
| OF-P1 — create-lane freeze point (E.1) | Keep `runtime-follow-on --dry-run` as the only create-lane freeze, as the charter already states. P is frozen at the first dry run where charter rule 3 resolves and rule 4 holds. O is frozen at a dry run that finds an admitted surface (OF-P4). No other create-lane command freezes or re-freezes. | The charter already names it (`cli-agent-onboarding-charter.md:317-323`). It is the first create-lane command that runs after version metadata, `RULES.json` and the validator spec exist (rule 4, `:312-316`), and after the maintainer adds the declaration (rule 2, `:276-280`). It runs before any executor (`runtime_follow_on.rs:165-176`), so O can be frozen before the executor runs ("The executor never changes O", `support-depth-contract.md:420`). | Approved by the maintainer 2026-11-05 |
| OF-P2 — committed identity artifact (E.1) | The onboarding pack's committed `governance/lifecycle-state.json` gains one object, `support_depth`. It holds exactly `policy_identity` and an optional `obligations_identity`, with the shape of the request table. It always states what the version's depth record states, and its states are those of the record:<br>• **No record, no object:** P is not yet frozen. With the declaration and approval in place this is the "declared, no record yet" case: insufficient depth, no acceptance authority, not an error. Declaring the depth enrollment and acquiring the surface happen in this state.<br>• **Record, no object, or object, no record:** a deleted binding, which is an error.<br>• **Record's Event names an earlier approval:** a new generation that has not frozen yet, so insufficient depth. The object keeps stating the earlier generation until this generation's first dry run replaces it.<br>**Writer:** the dry run, in the same invocation that writes or continues the record. **Replacer:** a re-freeze dry run of the same generation, which keeps `policy_identity`, or the first dry run of a later generation that continues the record. **Everyone else:** every other lifecycle writer (ON1 to ON5, NE7) preserves the object byte-for-byte. **Check:** repository validation compares the record with the object using `RequestBinding` / `request_mismatch` with the onboarding Event. | Mirrors the request table and its cross-check, so no new file kind is needed. The states follow contract Bindings rule 8 ("a binding that the record shows its generation has not yet produced is read as such … A missing field … is an error", `support-depth-contract.md:448-451`), B.2 rule 4 (the record is first written when P is frozen) and Bindings rule 9 (a new approval is a new generation). The lifecycle state is committed, typed, bound to the approval's SHA-256 (`agent_lifecycle.rs:306-307`), written only through LW1, and outside the write run's allowed paths. | Approved by the maintainer 2026-11-05 |
| OF-P3 — command that acquires the surface (E.3) | **Approved: (A).** Acquire with the existing input-only writers run directly: the agent's snapshot command, then `manifest-union`, the wrapper-coverage generator and `manifest-report`. This works whether the approval states `release_watch_enrolled` or `explicitly_deferred`. The `manifest-acquisition-plan` preview stays gated on release-watch, unchanged, and is used only when the agent is enrolled. The agent is not enrolled in maintenance automation.<br>**Not chosen: (B).** Require a depth-enrolled onboarding approval to state `release_watch_enrolled`, so the preview and the multi-target entry rule apply. This adds a maintenance-enrollment prerequisite to onboarding depth.<br>No new command is added, `parity-acquire.yml` is not borrowed, and the outputs reach the integration branch in a reviewed change before the dry run. | The preview refuses non-enrolled agents (`manifest_acquisition.rs:159-161`). The charter lets onboarding defer maintenance (`cli-agent-onboarding-charter.md:30-33`) and keeps multi-target acquisition behind release-watch (`:412-424`). Contract A.2 classes the writers as input-only. (A) adds nothing and keeps the deferral. (B) is simpler to preview but changes what onboarding depth requires. MA2 needs `AUTOMATION_TOKEN` (Annex A.1), so borrowing it would borrow maintenance authority. CI stays credential-free. | Approved by the maintainer 2026-11-05: option (A) |
| OF-P4 — what admits a surface (E.3) | Integrity check: every input O would bind (OF-P6) is under the bound-input rule and byte-identical to the same path at an integration ref that the caller fetched and named, as for stand-down markers. On an isolated route, that ref is the route's protected tip. A tree that differs makes the dry run refuse and name the paths. Tip equality is a currentness check only, not provenance.<br>**Provenance:**<br>**Not chosen: (A) Refuse every present surface (no admission rule).** No create-lane artifact today demonstrates where acquired bytes came from, and commit timing cannot: output produced by an earlier executor before the declaration can first be committed after it. So under (A) a dry run that finds acquired inputs refuses to freeze O, and one that finds none leaves O not yet produced. A create-lane generation then cannot reach acceptance for depth-enrolled scope until a later, approved decision adds a demonstrable origin, such as (B). There is no attestation.<br>**Approved: (B) Maintainer acquisition statement.** A maintainer commits `governance/depth-acquisition/<version>.toml` to the integration branch, beside the lifecycle state. It lists each acquired input path, its SHA-256 and the command that produced it, and is modeled on stand-down markers. **Approved constraint:** the statement takes effect only from the integration branch and never from the onboarding branch that uses it. This is an analogy with marker rule 5 (`maintenance-request-contract-v1.md:775-788`), which governs maintenance stand-down markers only; charter rule 10 gives the create lane no packet freeze (`cli-agent-onboarding-charter.md:351-353`), so the constraint is new, adopted by this decision. The dry run requires the statement at the named integration ref, and every bound input to equal both the statement and the ref. Inputs with no statement refuse. The executor's protected paths include the statement.<br>A depth-enrolled write run's allowed paths drop `<manifest_root>/snapshots/**` (`codex_exec.rs:51`, `:155`). Local leftovers from failed or interrupted runs are handled by OF-P9, not by this row. | Charter rule 6 forbids freezing O from the write-run output "of this generation or of an earlier one" (`cli-agent-onboarding-charter.md:324-331`). Byte equality cannot tell an earlier generation's merged executor output from an acquisition, and neither can the order of commits, which the reviewers showed. (A) needs no new artifact and conforms by refusing, but it leaves the create lane with no way to freeze O, so the onboarding path cannot be proven under (A) alone. (B) adds one governance file kind, shaped like an existing one. It makes a maintainer, not the tree, responsible for provenance (as the lifecycle spec has the packet worker declare a stand-down marker) and handles identical re-acquisition. It needs a charter change (slice 1). | Approved by the maintainer 2026-11-05: option (B), with its integration-branch-only constraint |
| OF-P5 — dry run after `runtime_integrated` (E.3) | **Not admitted.** Keep today's refusal (`runtime_follow_on/lifecycle.rs:81-97`). A dry run after a failed write (stage still `enrolled`) is admitted as a re-freeze only under OF-P4, with P unchanged under the request contract's re-freeze rule 3, and only once OF-P9 guarantees that a failed run left no executor output in the caller tree. A generation that reaches `runtime_integrated` with O not yet produced cannot reach acceptance for depth-enrolled scope. Work continues only in a new generation, which needs a new approval artifact (Bindings rule 9). | After a passing write run, the tree holds accepted executor output. A re-freeze over it is the gap the test list records for relay runs (`support-depth-validation-test-list.md:397-399`). Both modes already require `enrolled`. A dry run that leaves O not yet produced must say so, and must say that a passing write run then closes the window. | Approved by the maintainer 2026-11-05 |
| OF-P6 — acquired inputs of an onboarding O (E.3) | Exactly the set the shared derivation binds, through the same `support_depth::acquisition::derive` call: `RULES.json`, `SCHEMA.json`, `snapshots/<version>/union.json` and each target shard, `wrapper_coverage.json`, and `reports/<version>/coverage.*.json` (`support_depth/acquisition.rs:49-160`). The approval artifact, version metadata, validator spec, lifecycle state and run files are not acquired inputs. ON3, ON4 and ON5 explicitly refuse when a frozen acquired input has changed, as the relay does (`depth_acceptance/relay.rs:34-48`). Wrapper coverage is not excluded to avoid that refusal. | One derivation gives O one meaning on both paths (contract Bindings, O row). Version metadata and the validator spec are charter rule 4 preconditions for P (`cli-agent-onboarding-charter.md:312-316`). The approval is the Event. A publication step that regenerates `wrapper_coverage.json` is then refused, and under OF-P5 no re-freeze is left (Q2). | Approved by the maintainer 2026-11-05 |
| OF-P7 — partial single-host union (E.3) | **Conservative: refuse.** On the create lane, a dry run freezes O only from a union whose `complete` is `true` for `union.expected_targets`. A present but incomplete union makes the dry run refuse and name the missing targets. Freezing O for only the declared targets stays open (Q1). | Depth enrollment rule 6: enrollment "MUST NOT remove or narrow … target acquisition completeness" (`support-depth-contract.md:529-530`). Single-host onboarding "can only ever produce a partial, `complete:false` union" (`cli-agent-onboarding-charter.md:414-416`). The shared derivation accepts `complete:false` (`acquisition.rs:147-155`), so this is a create-lane restriction that leaves maintenance unchanged. A union recomputed with more targets can renormalize names on the targets already covered (`manifest_union/intake.rs:70-86`). The synthetic subject declares one expected target. | Approved by the maintainer 2026-11-05 |
| OF-P8 — depth identity in the runtime input contract (E.3) | `InputContract` (`runtime_follow_on/models.rs:7-38`) gains an optional `support_depth` object: `version`, taken from the approval's `[support_depth]` table and equal to the record's selection version, plus `policy_identity` and an optional `obligations_identity` copied from the OF-P2 object. The OF-P2 object itself holds only the two digests. The field is present exactly for depth-enrolled generations, and `workflow_version` is bumped. It is a cross-check, never authority. A write run refuses before execution when the values differ from the lifecycle object or the record, and refuses after execution when either changed. `repair-runtime-evidence` copies the identities from the lifecycle object and never computes them. The input contract cannot authenticate its own saved `baseline`; OF-P9 removes the boundary's dependence on that baseline. | The charter rejects the input contract as the identity artifact (`cli-agent-onboarding-charter.md:332-338`, `:402-408`). As a check, it binds a write run to the freeze it was prepared under, as the request table binds a relay run (`maintenance-request-contract-v1.md:806-812`). | Approved by the maintainer 2026-11-05 |
| OF-P9 — write-run boundary and `written-paths.json` (E.3) | For depth-enrolled generations, run `runtime-follow-on --write` through the MA4 candidate machinery instead of in place, with explicit phases:<br>1. **Trusted preparation (caller).** Load and check the dry-run packet. Record the write attempt in the caller's lifecycle state. The attempt state stays outside candidate delivery.<br>2. **Before-state (controller).** Capture the before-tree in the controller's memory immediately before execution (`Candidate::prepare`). The serialized `baseline` in `input-contract.json` stays reporting-only.<br>3. **Execution (candidate).** The executor runs in the disposable copy. There are three classes of run files:<br>&nbsp;&nbsp;• immutable inputs: the input contract and prompt, protected as MA4 protects them, plus the lifecycle state, the depth record, the approval and any OF-P4(B) statement;<br>&nbsp;&nbsp;• executor outputs: `handoff.json` and the allowed source paths;<br>&nbsp;&nbsp;• controller outputs: stdout, stderr, `codex-execution.json`, the validation report, run status, `written-paths.json` and the summary.<br>4. **Observation (controller).** Observe the candidate and run the create-lane boundary and manifest-split checks against the in-memory before-tree. This happens before any controller output is written, so controller outputs never count as executor changes (`codex_exec.rs:91-94`, `runtime_follow_on.rs:183-186`).<br>5. **Finalization.** There is no early exit between execution and finalization. Controller outputs are written as diagnostics. A failed or interrupted run delivers only the before-tree plus diagnostics. A passing run is delivered through checked expected-before mutations, refused if the caller changed.<br>`written-paths.json` is written by the controller from its own observation and is reporting-only. Publication's non-empty check (`prepare_publication/runtime_evidence.rs:241-248`) is not enough on its own, so OF-P9 adds one more check:<br>&nbsp;&nbsp;**Approved: (i)** the lifecycle state written at success records the SHA-256 of `written-paths.json`, and publication requires a match;<br>&nbsp;&nbsp;**Not chosen: (ii)** publication independently checks that the required runtime outputs exist in the tree: the required `agent_api` test and the backend module.<br>(i) uses a file the executor cannot reach. Non-depth runs keep today's behavior. | Both reviewers showed that an in-place comparison against a saved, executor-writable baseline can be forged, and that narrowing paths would reject legitimate controller writes. MA4 already solves both: a controller-held before-tree (`candidate.rs:26-75`), protected run inputs (`:101-110`), checks after observation (`:101-170`), unconditional finalization (`guarded.rs:29-35`, `:81-101`) and checked delivery (`candidate.rs:171-214`). Because a failed run never reaches the caller tree, the leftovers problem disappears without reading `written-paths.json`. The machinery is "not an OS sandbox" (Annex A.1), which is the same accepted limit as MA4's. | Approved by the maintainer 2026-11-05: as written, with publication check (i) |

Unchanged:
- Annex A remains the only route inventory. ON1 to ON5 keep refusing depth until roadmap E.6
  wires them, and only in the isolated workspace.
- The create lane gets no packet freeze, pointer, `validated` or `supported` route (charter rules
  10 and 11).
- Production keeps refusing depth. No profile, registry table or path enablement is installed.
  The onboarding isolated scope in slice 0 is a proof workspace only. It is not onboarding path
  enablement, which still needs contract "Path enablement" items 1, 3, 4 and 5 and per-agent
  authorization.

## Options considered, risks and proving tests

### OF-P1 — freeze point
- **Options.**
  - `onboard-agent --write`: rejected. It runs before the declaration exists (charter rule 2), and
    it writes no version metadata, `RULES.json` or validator spec
    (`onboard_agent/preview.rs:282-320`).
  - A new freeze command: rejected under the minimum machinery rule
    (`support-depth-contract.md:130`).
  - `prepare-publication`: rejected, because it runs after the executor.
- **Risk.** The dry run becomes a writer of committed files: the record and the lifecycle object.
  Today it writes only ignored run files (`runtime_follow_on.rs:334-399`). It becomes an LW1
  caller, and Annex A's ON2 row must say so.
- **Tests.**
  - A dry run with rule 3 unresolved, or rule 4 failing, refuses and writes nothing.
  - The first admitted dry run writes P and the record.
  - No other create-lane command writes either.

### OF-P2 — identity artifact
- **Options.**
  - The approval artifact: rejected. It is the Event and is committed before the declaration
    exists. Amending an approval is itself open (test list `:462-467`).
  - The input contract: rejected by the charter.
  - The publication packet: rejected, because it is written after execution.
  - A new `governance/depth-freeze.json`: workable, but rejected as an extra mechanism.
- **Risks.**
  - `LifecycleState` does not deny unknown fields, so a writer that predates the field would drop
    it silently. The slice must add the field as typed and reject unknown keys, or bump
    `LIFECYCLE_SCHEMA_VERSION` (`agent_lifecycle.rs:21`). Historical states must still parse.
  - What a new approval does to the existing lifecycle state is part of the open
    approval-amendment question.
  - A hand edit of both the record and the object is not detected in one tree. This is the same
    limit the request table has.
- **Tests.**
  - With the declaration and acquisition in place and no record, the generation reads as
    insufficient depth and is not an error.
  - Record without object, and object without record, are errors.
  - A record/object mismatch is rejected, covering Event, P, and O absent versus present.
  - A new approval with the earlier record keeps the earlier object and reads as insufficient
    depth until its first dry run. That dry run then replaces the object.
  - Every lifecycle writer (ON1 to ON5, NE7) preserves the object.
  - A missing object never reads as "not depth-enrolled".

### OF-P3 — acquisition command
- **Options.** (A) is approved; (B) was not chosen. Both are in the row.
  - The dry run acquires: rejected. A freeze would then run the upstream binary, which cannot be
    credential-free in CI.
  - `onboard-agent` acquires: rejected, because it runs before rule 4 holds.
  - `parity-acquire.yml`: rejected, because it is MA2's maintenance authority.
- **Risks.**
  - (A): without the preview, a maintainer must run the per-target commands correctly. The
    dry-run checks (OF-P4, OF-P6, OF-P7) catch a wrong result, but they do not guide the run.
  - (B), not chosen, would have changed what onboarding depth needs.
  - `manifest-version-metadata` (NE3) refuses depth manifests, which matters for rule 4 (Q3).
- **Tests.**
  - An onboarding subject whose approval states `explicitly_deferred`, with no
    `maintenance.release_watch`: the input-only writers produce an input set that the dry run
    admits, and the preview still refuses.
  - The same writers in a declared depth workspace succeed and grant nothing.
  - A depth-enrolled approval may state either maintenance mode; neither is refused for that
    reason.
  - The synthetic fixture's registry template carries release-watch, so the deferred case needs
    its own fixture variant.

### OF-P4 — admitting a surface
- **Options.** (B) is approved; (A) was not chosen. Both are in the row.
  - Read `written-paths.json`: rejected, because it is executor-writable.
  - Trust the tree: rejected.
  - Tip equality alone (this PR's first draft): rejected by both reviewers, because it is a
    currentness check, not provenance.
  - An acquisition-command stamp: rejected, because the executor can write the same stamp.
- **Risks.**
  - (A), not chosen, would have admitted no surface at all.
  - (B) relies on maintainer diligence, as stand-down markers do. A wrong statement is a maintainer
    error that IN1 cannot detect.
  - Both need a fetched integration ref, so offline dry runs refuse to freeze O, though they can
    still freeze P.
- **Tests.**
  - An earlier generation's executor output already merged to the integration tip, with no
    statement covering it, is refused.
  - **Delayed commit:** executor output produced before the declaration and first committed to the
    integration branch after it is refused unless a statement covers it. Commit timing is never
    consulted.
  - A pre-declaration run's output on the tip, with no statement, is refused.
  - A locally interrupted run leaves no tree change in the caller (OF-P9). A dry run after it is
    admitted only with a matching statement.
  - Byte-identical re-acquisition is admitted with a statement.
  - An input whose bytes differ from the statement, or a statement entry with no matching input,
    refuses.
  - A tree that differs from the ref refuses with the paths named.
  - A statement carried on the onboarding branch does not count before its merge.
  - A depth-enrolled write run that writes `snapshots/` fails the boundary check.
  - On an isolated route, the protected tip is the ref that is read.

### OF-P5 — dry run after integration
- **Options.**
  - Admit a re-freeze after `runtime_integrated`: rejected.
  - Admit it only while O is not yet produced: rejected.
- **Risk.** A generation that misses its O window needs a new approval.
- **Tests.**
  - A dry run at `runtime_integrated` refuses for a depth-enrolled generation.
  - After a failed write, the caller tree equals the before-tree, and a retried dry run is admitted
    only under OF-P4.
  - A changed P refuses.
  - A dry run that leaves O not yet produced prints the acceptance consequence.

### OF-P6 — acquired inputs
- **Options.**
  - A narrower set: rejected, because O would mean different things on the two paths.
  - Excluding `wrapper_coverage.json`: rejected, because it hides a real dependency. Refuse
    instead.
  - Adding version metadata or the validator spec: rejected, because they are P preconditions.
- **Risk.** If publication regenerates wrapper coverage, the generation is refused with no
  re-freeze left (Q2).
- **Tests.**
  - The onboarding O equals the maintenance derivation's O for the same tree.
  - ON3, ON4 and ON5 refuse after a frozen acquired input changes, `wrapper_coverage.json`
    included.

### OF-P7 — partial union
- **Options.**
  - (a) Refuse unless complete. Approved.
  - (b) Freeze for exactly the declared targets when each has a shard. Not chosen; stays open
    (Q1).
  - (c) Freeze for the covered subset by narrowing the declaration: rejected.
- **Risk.** Multi-target agents wait for complete acquisition.
- **Tests.**
  - A `complete:false` union refuses with the missing targets named.
  - A complete single-target union is admitted.
  - The maintenance derivation is unchanged.

### OF-P8 — input contract
- **Options.**
  - No field: cannot detect a write run prepared under an earlier freeze.
  - A full copy of P and O: rejected, because it is a second store.
- **Risk.** `InputContract` does not deny unknown fields, so contracts prepared before the bump
  must be refused for depth-enrolled generations.
- **Tests.**
  - The field is present exactly for depth-enrolled dry runs, with `version` equal to the
    approval's.
  - A write run refuses when the lifecycle object or the record changed after its dry run.
  - `repair-runtime-evidence` copies the identities.
  - A forged saved baseline with unchanged identities does not hide a forbidden write; OF-P9
    catches it.

### OF-P9 — write-run boundary
- **Options.**
  - The in-place narrowed diff from the first draft: rejected. It still compares against an
    executor-writable baseline, rejects controller writes, and self-references the serialized
    contract.
  - Signing the file: rejected, because the executor has the same rights.
  - Committing run directories: rejected.
  - Reusing the MA4 candidate machinery: approved.
- **Risks.**
  - The machinery is in `agent_maintenance/execute/`, so sharing it means extracting a neutral
    module. That touches code another worker currently owns, so the extraction slice needs
    coordination.
  - The candidate is not an OS sandbox.
  - The executor's prompt must say that `handoff.json` is its only run file.
- **Tests.**
  - A successful run with an unchanged fake executor: controller outputs (stdout, stderr,
    `codex-execution.json`, report, status, written paths) cause no rejection.
  - A forged `input-contract.json` baseline with unchanged depth identities does not hide a
    forbidden write.
  - An executor that edits the input contract, prompt, lifecycle state, record or statement is
    rejected.
  - An interruption after the executor tampered with candidate governance delivers nothing but
    diagnostics.
  - A failed run leaves the caller tree equal to the before-tree.
  - Publication refuses a forged non-empty `written-paths.json` whose SHA-256 does not match the
    lifecycle state (check (i)).
  - A non-depth write run is unchanged.

All tests use existing fake fixtures: the fake executor binary and the synthetic subject
templates, with a deferred-maintenance variant. None needs a network or a credential.

## Questions still open

These do not block the slices below. Each slice keeps the refusing behavior until a later decision.

1. **OF-P7(b).** May a single-host union freeze O for exactly the declared targets?
2. **Wrapper coverage after a write run.** OF-P6 refuses a regenerated `wrapper_coverage.json`.
   Should publication be forbidden from regenerating it for a depth-enrolled generation, so the
   refusal never fires?
3. **Rule 4 ordering with NE3.** `manifest-version-metadata` refuses depth manifests. Should
   version metadata be written before the declaration, or only by the E.2 writer?
4. **Scope of OF-P9.** Should candidate execution later apply to non-depth runs as well?

## Implementation slices

Each slice is a reviewed PR into `staging` and assumes the approved choices above: OF-P3(A),
OF-P4(B) and OF-P9 check (i). None enables a path or changes
production refusals. Slices 2 onward are tested on the synthetic subject under `Fixture::install()`
only.

0. **Onboarding isolated scope (prerequisite).** Today the isolated resolver admits only the
   maintenance path, maintenance Events and maintenance selections (`isolated_maintenance/scope.rs:89-91`,
   `:126`, `:150`), and the evidence contract defines `synthetic-wrapper-v1` with a maintenance
   enrollment (`support-depth-evidence-v1.md:443-450`). `Fixture::install()` bypasses none of
   this.
   - First, a bounded evidence-contract amendment adds an onboarding selection for the same
     synthetic subject in the isolated workspace only.
   - Then the scope admits `lifecycle_path = "onboarding"` and the onboarding Event for that
     subject alone.
   - Write scope: `docs/specs/support-depth-evidence-v1.md`, `isolated_maintenance/scope.rs` and
     its tests (not `tests/ma4_route.rs`), and the fixture templates under
     `crates/xtask/tests/fixtures/depth-synthetic/`, including a variant whose approval states
     `explicitly_deferred` with no `maintenance.release_watch` (OF-P3(A)) and a template
     acquisition statement (OF-P4(B)).
   - This is a proof workspace, not onboarding path enablement.
1. **Charter revision (docs).**
   - Name the lifecycle-state object and its states (OF-P2).
   - Name the input-only writers as the acquisition route, with no release-watch prerequisite
     (OF-P3(A)).
   - Add the acquisition statement: path
     `docs/agents/lifecycle/<pack>/governance/depth-acquisition/<version>.toml`, its fields (agent,
     version, and each input's path, SHA-256 and producing command), who writes it (a maintainer,
     by a change to the integration branch), and the rule that it takes effect only from the
     integration branch (OF-P4(B)).
   - Add OF-P5 to OF-P7.
   - Update Present behavior and the test list.
   - Annex A's ON2 row says the dry run is an LW1 caller and the write run uses candidate
     execution.
   - Write scope: `docs/specs/cli-agent-onboarding-charter.md`, `docs/specs/support-depth-contract.md`
     Annex A rows only, and `docs/agents/lifecycle/support-depth-validation-test-list.md`.
2. **Typed lifecycle object (OF-P2).**
   - Add the `support_depth` field, and the field for the SHA-256 of `written-paths.json` that
     OF-P9 check (i) records. Reject unknown keys or bump the schema, reuse `request_mismatch` for
     the onboarding Event, and add the validation cross-check.
   - Add the typed reader for the OF-P4(B) acquisition statement, which rejects unknown keys and
     reads from a named ref, following `stand_down.rs`.
   - Add preservation tests for each writer.
   - Write scope: `agent_lifecycle.rs`, `agent_lifecycle/storage.rs`,
     `depth_record.rs`/`depth_record/`, and the `manifest_validate/depth/` check.
3. **Neutral candidate execution and the create-lane boundary (OF-P9, and OF-P4's `snapshots/**`
   drop).** Slice 3 implemented: shared candidate boundary, isolated create-lane adapter,
   snapshots refusal, and lifecycle-bound written-paths publication check; production stays disabled.
   - Extract the candidate, tree, finalization and delivery pieces from
     `agent_maintenance/execute/` into a neutral module that both MA4 and the create lane call. MA4
     must stay byte-identical in behavior and keep its tests.
   - Route a depth-enrolled `runtime-follow-on --write` through it with the phases in OF-P9.
   - Add the acquisition statement to the executor's protected paths.
   - Record the SHA-256 of `written-paths.json` in the lifecycle state at success, and make
     publication require a match (check (i)).
   - Write scope: the new neutral module, `agent_maintenance/execute/{candidate,guarded,tree,delivery_index}.rs`
     (coordinated with that module's current owner), `runtime_follow_on.rs`,
     `runtime_follow_on/{codex_exec,lifecycle}.rs`, and `prepare_publication/runtime_evidence.rs`.
   - Depends on slice 2 for the protected lifecycle object.
4. **Input contract identity (OF-P8).** Slice 4 implemented: optional version/P/O identity,
   workflow v2, pre-execution and candidate-observation cross-checks, and repair identity copying;
   historical non-depth contracts remain readable and production depth stays disabled.
   - Add the optional field, bump `workflow_version`, add the pre- and post-execution checks, and
     make `repair-runtime-evidence` copy the identities.
   - Write scope: `runtime_follow_on/models.rs`, `runtime_follow_on.rs` and
     `repair_runtime_evidence.rs`.
   - Depends on slice 2, which provides the typed object it copies, and on slice 3, because the
     post-execution check runs in its observation phase.
5. **Dry-run freeze core (OF-P1, OF-P4 to OF-P7).**
   Slice 5 implemented: the private installed-synthetic-route dry run checks charter rules 3/4,
   persists P, the record and lifecycle object with the OF-P8 packet identity, admits the shared
   O derivation only with the integration-tip acquisition statement and identical bound inputs,
   and refuses incomplete unions and post-integration re-freezes. Its narrow non-runtime approval
   projection is private to that dry run; the production approval loader and public ON2 refusal
   are unchanged. End-to-end fixture coverage includes deferred maintenance and refusal rollback.
   Review fixes: statement enumeration, statement blobs and bound inputs use the same hardened
   Git view pinned to the authenticated integration tip; rule 4 admits targets covered by that
   tip's version record, with replacement-ref, graft, alternate and environment regressions.
   - A private, isolated-route-only dry-run path: resolve rule 3, check rule 4, write P, the
     record and the object, read the named integration ref and the acquisition statement there,
     require every bound input to equal both, and call the shared `derive`.
   - Add the complete-union and post-integration refusals.
   - Write the OF-P8 field into the input contract it prepares.
   - The public entry keeps refusing depth until roadmap E.6.
   - Write scope: `runtime_follow_on.rs`, a new `runtime_follow_on/depth.rs`, and
     `support_depth/` call sites only.
   - Depends on slices 0, 2, 3 and 4.
6. **Acquired-input refusal in ON3 to ON5 (OF-P6).**
   - Slice 6 implemented: ON3–ON5 compare the acquired paths and SHA-256 identities recorded
     by the shared O derivation, with OF-P2 record/object binding checks, before any write.
     Installed synthetic-route tests cover each acquired input, deletion, unchanged/P-only
     freezes and orphaned bindings with byte-identical refusal. Public depth guards remain.
   - Mirror the relay's frozen-input check in `prepare-publication`, `refresh-publication` and
     `close-proving-run` for depth-enrolled generations.
   - Write scope: `prepare_publication/`, `publication_refresh.rs` and `close_proving_run.rs`.
   - Depends on slice 5.

Order: 0 and 1 can run in parallel. 2 follows. 3 follows 2. 4 follows 2 and 3. 5 follows 0, 2, 3
and 4. 6 follows 5.
