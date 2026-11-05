# Support depth: onboarding freeze decisions (proposal)

Status: OF-P1 to OF-P9 proposed, awaiting maintainer approval. Nothing here is approved.
This is roadmap items E.1 and E.3 of [MP-P1](support-depth-machinery-proof-rescope.md).
Source baseline: `feat/depth-isolated-ma4-route` at `f8fcdc8` (PR #323). Line references are to
that tree.
The [support-depth contract](../../specs/support-depth-contract.md), the
[onboarding charter](../../specs/cli-agent-onboarding-charter.md#support-depth), the
[registry contract](../../specs/agent-registry-contract.md#support-depth) and the
[maintenance request contract](../../specs/maintenance-request-contract-v1.md#depth-enrolled-generations)
remain normative and are not amended here. Every decision below that needs a charter change says
so; the charter revision is a separate, later PR. This document changes no code, registry, host
setting or credential, and creates no background job. Nothing in it is specific to one agent.

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

Each decision below reuses one of these instead of adding a new store, file kind or command.

| Mechanism | Where | What it already does |
| --- | --- | --- |
| Request `[support_depth]` table | `maintenance-request-contract-v1.md:517-562`; `depth_record/request.rs:1-40` | A second committed statement of `policy_identity` and `obligations_identity`, absent O while O is not frozen. |
| Record/request cross-check | `depth_record.rs:80-87`, `:310-322` (`RequestBinding`, `request_mismatch`) | Rejects a record whose Event, P or O is not what the second statement froze. |
| Freeze and re-freeze rules | `maintenance-request-contract-v1.md:564-587` | First freeze writes P and the record. A freeze with acquired reports writes O. A re-freeze keeps P and refuses a changed P. |
| Shared O derivation | `support_depth/acquisition.rs:26-160` | Derives O from the bound acquired inputs. Refuses an acquisition that has no union, and a selected target that has no shard. |
| Bound-input rule | `support_depth/freeze.rs:37`; `maintenance-request-contract-v1.md:589-600` | Binds only stage-0 Git files whose working bytes equal the staged blob. Refuses missing, untracked and divergent inputs. |
| Relay acquired-input check | `depth_acceptance/relay.rs:34-48` | Refuses a run that changed a frozen acquired input ("explicit re-freeze required"). |
| Stand-down marker read from a ref | `agent_maintenance/stand_down.rs:198-223`, `:313`, `:430-445` | Reads authority from the integration branch, never from the working branch. |
| Isolated route and protected tip | `isolated_maintenance/mod.rs`; [isolated route design](support-depth-isolated-route-design.md) | Binds the repository, the protected tip and the baseline for a proof workspace. Production has no route. |
| Lifecycle state | `agent_lifecycle.rs:21-27`, `:302-327`; `agent_lifecycle/storage.rs:54-66` | A committed, typed file per onboarding pack. Bound to the approval's SHA-256. Written only through the LW1 lifecycle writer, which already refuses depth. |
| Approval `[support_depth]` loader | `approval_artifact.rs:49`, `:317`, `:459` | Reads the approved exact version and rejects misspelled or misplaced tables. |
| Write-run boundary | `runtime_follow_on/codex_exec.rs:35-55`, `:136-163`; `runtime_follow_on.rs:401-490` | Compares the tree before and after the executor. Rejects writes outside the allowed paths and publication-owned manifest writes. |
| Input-only acquisition writers | `support-depth-contract.md` A.2, "Input-only writers"; [charter multi-target entry](../../specs/cli-agent-onboarding-charter.md#multi-target-parity-acquisition-when-a-new-agent-joins-it) | The per-agent snapshot command, `manifest-union`, the wrapper-coverage generator and `manifest-report`. Each changes O inputs and grants no acceptance. |

## Proposed decisions

Every row is **proposed, awaiting maintainer approval**.

| Id | Recommendation | Grounds / boundary | Status |
| --- | --- | --- | --- |
| OF-P1 — create-lane freeze point (E.1) | Keep `runtime-follow-on --dry-run` as the only create-lane freeze, as the charter already states. P is frozen at the first dry run where charter rule 3 resolves and rule 4 holds. O is frozen at a dry run that finds an acquired surface (OF-P4). No other create-lane command freezes or re-freezes. | The charter already names it (`cli-agent-onboarding-charter.md:317-323`). It is the first create-lane command that runs after version metadata, `RULES.json` and the validator spec exist (rule 4, `:312-316`), and after the maintainer adds the declaration (rule 2, `:276-280`). It runs before any executor does (`runtime_follow_on.rs:165-176`), so O can be frozen before the executor runs, which the contract requires ("The executor never changes O", `support-depth-contract.md:420`). | Proposed, awaiting maintainer approval |
| OF-P2 — committed identity artifact (E.1) | The onboarding pack's committed `governance/lifecycle-state.json` gains one object, `support_depth`, with exactly `policy_identity` and an optional `obligations_identity`. It has the shape and rules of the request table: present exactly for a depth-enrolled create-lane generation, `obligations_identity` absent until O is frozen, each a 64-hex SHA-256 as Annex B defines it. **Writer:** the dry run, in the same invocation that writes or continues the depth record. **Replacer:** only a re-freeze dry run of the same generation. It keeps `policy_identity` and may restate `obligations_identity` only under OF-P4 and OF-P5. **Everyone else:** every other lifecycle writer preserves the object byte-for-byte. These are ON1 to ON5 and NE7. **Check:** repository validation compares the record with the object, using the existing `RequestBinding` / `request_mismatch` with the onboarding Event. A missing object on a depth-enrolled generation is an error, never "not depth-enrolled". | Mirrors the request table (`maintenance-request-contract-v1.md:517-562`) and its cross-check (`depth_record.rs:310-322`), so no new file kind is needed. The lifecycle state is already committed, typed and bound to the approval's SHA-256 (`agent_lifecycle.rs:306-307`). It is written only through the LW1 writer (`agent_lifecycle/storage.rs:54-66`). It is outside the write run's allowed paths (`codex_exec.rs:35-55`, `:136-157`), so an executor change to it fails the boundary check. Charter rule 7 is satisfied once the charter names this artifact (`cli-agent-onboarding-charter.md:332-338`). | Proposed, awaiting maintainer approval |
| OF-P3 — command that acquires the surface (E.3) | No new command. The surface is acquired by the existing input-only writers that the agent's `RULES.json` `acquisition` block names, previewed with `manifest-acquisition-plan`: the per-target snapshot command, then `manifest-union`, the wrapper-coverage generator and `manifest-report`. A maintainer runs them ahead of the dry run, and the outputs land on the integration branch in a reviewed change before the dry run (OF-P4). The create lane gains no workflow, and `parity-acquire.yml` is not borrowed. | Contract A.2 classes these writers as input-only: they "can invalidate O/E" and "do not grant acceptance". The charter's multi-target entry rule names the same acquisition block and preview (`cli-agent-onboarding-charter.md:412-445`). `manifest-union` and `manifest-report` have no depth refusal, so they work in a depth workspace. MA2 is a maintenance route that needs `release_watch` enrollment and `AUTOMATION_TOKEN` (Annex A.1), so using it here would borrow maintenance authority. Required CI stays credential-free: no acquisition runs in CI. | Proposed, awaiting maintainer approval |
| OF-P4 — what marks a surface as acquired (E.3) | A surface counts as **acquired** for a dry run only when both hold:<br>(a) every input that O would bind (OF-P6) is in the working tree under the bound-input rule, and is byte-identical to the same path at the integration-branch tip. On an isolated route, that tip is the route's protected tip.<br>(b) the integration branch's lifecycle state for this approval records no write attempt that came before the O freeze (OF-P9 item 3).<br>When (a) fails because the tree differs, the dry run refuses and names the paths. When nothing is acquired, O is left not yet produced (charter rule 6). In addition, the write run of a depth-enrolled generation drops `<manifest_root>/snapshots/**` from its allowed paths, so a write run can no longer produce the inputs O binds. | The tree cannot tell an acquired surface from executor output, because the executor is the only create-lane writer of `snapshots/` (`codex_exec.rs:51`, `:154`). So the authority has to come from somewhere the executor cannot write. That is the integration branch, the same source the stand-down check reads markers from (`stand_down.rs:221`, `:313`) and the same tip that commits a generation (contract depth enrollment rule 7). A failed or interrupted run's leftovers differ from the tip, so they are refused without reading `written-paths.json`. Dropping `snapshots/**` for depth-enrolled generations applies "the executor never changes O" directly. Non-depth runs keep today's paths. | Proposed, awaiting maintainer approval |
| OF-P5 — dry run after `runtime_integrated` (E.3) | **Not admitted.** Keep today's refusal (`runtime_follow_on/lifecycle.rs:81-97`). A dry run after a failed write (stage still `enrolled`) is admitted as a re-freeze only under OF-P4, with P unchanged under the request contract's re-freeze rule 3. A generation that reaches `runtime_integrated` with O not yet produced cannot reach acceptance for depth-enrolled scope. Work continues only in a new generation, which means a new approval artifact (Bindings rule 9). | After a passing write run, the tree holds accepted executor output. A re-freeze over it is the gap the test list records for relay runs (`support-depth-validation-test-list.md:397-399`). Refusing needs no code change, because both modes already require `enrolled`. A dry run that leaves O not yet produced must say so, and must say that a passing write run then closes the window. | Proposed, awaiting maintainer approval |
| OF-P6 — acquired inputs of an onboarding O (E.3) | Exactly the set that the shared derivation binds, through the same `support_depth::acquisition::derive` call, with no onboarding-specific list: `RULES.json`, `SCHEMA.json`, `snapshots/<version>/union.json` and each target shard, `wrapper_coverage.json`, and `reports/<version>/coverage.*.json` (`support_depth/acquisition.rs:49-160`). The approval artifact, version metadata, validator spec, lifecycle state and run files are not acquired inputs. | One derivation means one meaning of O on both paths (contract Bindings, O row). Version metadata and the validator spec are charter rule 4 preconditions for P (`cli-agent-onboarding-charter.md:312-316`), not O inputs. The approval is the Event. ON3/ON4/ON5 refuse a changed frozen acquired input, as the relay does (`depth_acceptance/relay.rs:34-48`). | Proposed, awaiting maintainer approval |
| OF-P7 — partial single-host union (E.3) | **Conservative: refuse.** On the create lane, a dry run freezes O only from a union whose `complete` is `true` for `union.expected_targets`. A present but incomplete union makes the dry run refuse with the missing targets named. It does not leave O silently not yet produced. Freezing O for only the targets the union covers is left as an open question (Q2). | Depth enrollment rule 6: enrollment "MUST NOT remove or narrow ... target acquisition completeness" (`support-depth-contract.md:529-530`). The charter states that single-host onboarding "can only ever produce a partial, `complete:false` union" (`cli-agent-onboarding-charter.md:414-416`). The shared derivation refuses a selected target with no shard (`acquisition.rs:147-155`) but accepts `complete:false`, so this adds a create-lane refusal and does not change maintenance. A union recomputed with more targets can renormalize names on the covered targets (`manifest_union/intake.rs:70-86`), and OF-P5 allows no later re-freeze. Refusing is loud. Leaving O unproduced would let the generation run on with acceptance already lost. The synthetic subject declares one expected target, so the proof is unaffected. | Proposed, awaiting maintainer approval |
| OF-P8 — depth identity in the runtime input contract (E.3) | `InputContract` (`runtime_follow_on/models.rs:7-38`) gains an optional `support_depth` object: `version`, `policy_identity` and an optional `obligations_identity`. The dry run copies it from the OF-P2 object it just wrote. It is present exactly for depth-enrolled generations, and `workflow_version` is bumped. It is a cross-check, never authority. A write run refuses before it starts the executor when the prepared values differ from the lifecycle-state object or the record (a stale dry run, or a re-freeze in between). It refuses afterwards when either changed. `repair-runtime-evidence` copies the object from the lifecycle state and refuses when it is missing for a depth-enrolled generation. It never computes identities. | The charter rejects the input contract as the identity artifact (`cli-agent-onboarding-charter.md:332-338`, `:402-408`): run directories are ignored by git and executor-writable. Carrying identities as a check is still useful: it binds a write run to the freeze it was prepared under, as the request's table binds a relay run (`maintenance-request-contract-v1.md:806-812`). | Proposed, awaiting maintainer approval |
| OF-P9 — `written-paths.json` integrity (E.3) | Make `written-paths.json` reporting-only. No freeze, admission or validation reads it, and OF-P4 does not need it. For depth-enrolled generations:<br>1. Take the write run's comparison baseline after the dry run has written its files, not in `build_context` before them (`runtime_follow_on.rs:321`, `:407`). A record or lifecycle object the dry run wrote then is not counted as a write-run change.<br>2. Narrow the ignored and executor-writable run area from the whole runs root (`runtime_follow_on.rs:320`; `codex_exec.rs:53`) to this run's `handoff.json`. Then an executor that rewrites its input contract, its prompt or another run's files fails the boundary check.<br>3. Before starting the executor, record a write-attempt transition in the lifecycle state through the existing writer. Today only a completed failure is recorded (`runtime_follow_on/lifecycle.rs:141-182`). An interrupted run then leaves a trace for OF-P4(b), and a completed run that erases the trace fails the boundary check, because the lifecycle state is outside its allowed paths. | The three weaknesses the charter lists are that the executor can rewrite the file, an interrupted run writes none, and no dry run reads it (`cli-agent-onboarding-charter.md:393-401`; `runtime_follow_on.rs:196-221`). Item 2 addresses the first, item 3 the second, and OF-P4 makes the third harmless. Each item reuses the existing snapshot diff, boundary check or lifecycle writer. Scoping them to depth-enrolled generations leaves production behavior unchanged. | Proposed, awaiting maintainer approval |

Unchanged:
- Annex A remains the only route inventory. ON1 to ON5 keep refusing depth until roadmap E.6
  wires them, and only in the isolated workspace.
- The create lane gets no packet freeze, pointer, `validated` or `supported` route (charter rules
  10 and 11).
- Production keeps refusing depth. No profile, registry table or path enablement is installed.

## Options considered, risks and proving tests

### OF-P1 — freeze point
- **Options.**
  - `onboard-agent --write`: rejected. It runs before the declaration exists (charter rule 2), and
    it writes no version metadata, `RULES.json` or validator spec
    (`onboard_agent/preview.rs:282-320`), so charter rule 4 cannot hold.
  - A new freeze command: rejected under the minimum machinery rule (`support-depth-contract.md:130`).
  - `prepare-publication`: rejected, because it runs after the executor.
- **Risk.** The dry run becomes a writer of committed files (the record and the lifecycle object)
  where today it writes only ignored run files (`runtime_follow_on.rs:334-399`). That makes it an
  LW1 caller, and it must be admitted as such in Annex A's ON2 row.
- **Tests.**
  - A dry run with rule 3 unresolved, or rule 4 failing, refuses and writes nothing.
  - The first admitted dry run writes P and the record.
  - No other create-lane command writes either, checked for each of ON1 and ON3 to ON5.

### OF-P2 — identity artifact
- **Options.**
  - The approval artifact: rejected. It is the Event and is committed before the declaration
    exists. Changing it changes the approval SHA-256 that the lifecycle state and input contract
    bind, and amending an approval is itself an open question (test list `:462-467`).
  - The input contract: rejected by the charter.
  - The publication packet: rejected. It is written after execution (test list `:447-452`).
  - A new `governance/depth-freeze.json`: a workable alternative, but rejected as an extra
    mechanism, since the lifecycle state already has the needed properties.
- **Risks.**
  - `LifecycleState` does not deny unknown fields (`agent_lifecycle.rs:301-302`). Any writer that
    deserializes and reserializes would silently drop an unknown `support_depth` key. The slice must
    add the field as typed and reject unknown keys, or bump `LIFECYCLE_SCHEMA_VERSION`
    (`agent_lifecycle.rs:21`). Historical lifecycle states must still parse.
  - Many commands rewrite this file, so each one needs a preservation test.
  - A hand edit of both the record and the object is not detected in one tree, which is the same
    limit the request table has. IN1 still checks committed generations against the tip.
- **Tests.**
  - The object is present exactly when the generation is depth-enrolled.
  - A record/object mismatch is rejected by validation, covering Event, P, O, and O absent versus
    present.
  - Every lifecycle writer (ON1 to ON5, NE7) preserves the object.
  - A write run that edits the lifecycle state fails the boundary check.
  - A missing object reads as an error, not as "not depth-enrolled".

### OF-P3 — acquisition command
- **Options.**
  - The dry run acquires: rejected. A freeze would then execute the upstream binary, which mixes
    acquisition with freezing and cannot run credential-free in CI.
  - `onboard-agent` acquires: rejected, because it runs before rule 4 holds.
  - `parity-acquire.yml`: rejected for now, because it is the MA2 route with maintenance
    authority and a token.
- **Risk.** `manifest-version-metadata` (NE3) refuses a depth manifest
  (`manifest_version_metadata.rs`). The rule 4 version metadata must therefore be written before the
  declaration is added, or by the E.2 writer. That ordering belongs to E.2 (Q4).
- **Tests.**
  - In the synthetic fixture test harness (`crates/xtask/tests/fixtures/depth-synthetic/`), the
    existing acquisition-plan, snapshot, union, coverage and report commands produce an input set
    that the dry run accepts as acquired.
  - The same commands run against a declared depth workspace succeed and grant nothing: the record
    has no acceptance entry.

### OF-P4 — acquired marker
- **Options.**
  - Read `written-paths.json`: rejected, because the executor can rewrite it.
  - Hash snapshots at the dry run and trust the tree: rejected, because the tree is exactly what a
    failed run leaves behind.
  - A provenance stamp written by the acquisition commands: rejected, because the executor can
    write the same stamp.
  - The integration-branch tip (recommended): the executor cannot write it.
- **Risks.**
  - The rule needs a reachable integration ref. Offline dry runs refuse to freeze O, though they
    can still freeze P.
  - A surface written by an earlier, pre-declaration write run and merged into the integration
    branch passes (a). Item (b) catches it only if that run's attempt is in the lifecycle state.
    See Q1.
  - Dropping `snapshots/**` stops a depth-enrolled write run from refreshing snapshots of other
    versions.
- **Tests.**
  - The dry run refuses when one bound input differs from the tip, with the path named.
  - It refuses when an input is untracked or unstaged.
  - It leaves O not yet produced when no input exists.
  - It freezes O when every input equals the tip.
  - A failed write run's leftover snapshot makes a retried dry run refuse.
  - A depth-enrolled write run that writes `snapshots/` fails the boundary check; a non-depth run
    is unchanged.
  - On an isolated route, the protected tip is the ref that is read.

### OF-P5 — dry run after integration
- **Options.**
  - Admit a re-freeze after `runtime_integrated`: rejected, because the executor output is in the
    tree.
  - Admit it only while O is not yet produced: rejected for the same reason.
- **Risk.** A generation that misses its O window needs a new approval, and approval amendment
  is still open (test list `:462-467`).
- **Tests.**
  - A dry run at `runtime_integrated` refuses. This exists today; extend it to a depth-enrolled
    generation.
  - A dry run after a failed write is admitted only under OF-P4.
  - A re-freeze that sees a changed P refuses.
  - A dry run that leaves O not yet produced prints the acceptance consequence.

### OF-P6 — acquired inputs
- **Options.**
  - An onboarding-specific narrower set (snapshots only): rejected, because it would give the two
    paths two meanings of O.
  - Adding version metadata or the validator spec: rejected, because those are P preconditions.
- **Risk.** `wrapper_coverage.json` is generated from wrapper source that the write run may change
  (`codex_exec.rs:50`, `:153`). If publication regenerates it, a frozen acquired input changes and
  acceptance is refused, with no re-freeze left under OF-P5. See Q3.
- **Tests.**
  - For the same tree, the onboarding dry run's O equals the maintenance derivation's O.
  - ON3, ON4 and ON5 refuse after a frozen acquired input changes.

### OF-P7 — partial union
- **Options.**
  - (a) Refuse unless the union is complete (recommended).
  - (b) Freeze O when every declared target has a shard, with the record's targets equal to the
    declaration's.
  - (c) Freeze for the covered subset and narrow the declaration: rejected, because it narrows a
    maintainer's selection.
- **Risk.** Under (a), multi-target agents cannot freeze O on the create lane until acquisition
  covers every expected target.
- **Tests.**
  - The dry run refuses a `complete:false` union and names the missing targets.
  - It accepts a complete single-target union, which is the synthetic subject.
  - The maintenance derivation is unchanged for `complete:false`.

### OF-P8 — input contract
- **Options.**
  - No field, relying only on the lifecycle object: workable, but it cannot detect a write run that
    was prepared under an earlier freeze.
  - A full copy of P and O: rejected, because it is a second store.
- **Risk.** `InputContract` does not deny unknown fields either. The slice must handle input
  contracts prepared before the bump (refuse them for depth-enrolled generations).
- **Tests.**
  - The field is present exactly for depth-enrolled dry runs.
  - A write run refuses when the lifecycle object or the record changed after its dry run, both
    before and after execution.
  - `repair-runtime-evidence` copies the object, and refuses when the object is missing.

### OF-P9 — written paths
- **Options.**
  - Sign or hash the file: rejected, because the executor runs with the same rights.
  - Commit run directories: rejected, because they contain executor output and churn.
  - Delete the file: rejected, because it is useful reporting evidence.
- **Risk.** Item 2 changes what an executor may write in its run directory. The prompt must say
  that `handoff.json` is its only run file.
- **Tests.**
  - A depth-enrolled write run with a dry-run-written record and lifecycle object passes the
    manifest check.
  - An executor that rewrites `input-contract.json` or another run's file fails the boundary check.
  - An interrupted run, simulated by stopping after the attempt is recorded, leaves the transition
    and makes OF-P4(b) refuse.
  - A non-depth write run keeps today's baseline and paths.

All tests use existing fake fixtures (the fake executor binary and the synthetic subject
templates). None needs a network or a credential.

## Open questions for the maintainer

1. **Earlier write-run surfaces on the integration branch.** Charter rule 6 forbids freezing O
   from any generation's write-run output. OF-P4(b) catches only attempts that the lifecycle state
   records. Conservative default: refuse to freeze O for a version whose acquired inputs were on
   the integration branch before its declaration, until they are re-acquired in a change after the
   declaration. Re-acquisition that produces identical bytes cannot be shown in git, so this
   default may block forever. Is a maintainer attestation in the declaring change acceptable
   instead?
2. **OF-P7(b).** May a single-host union freeze O for exactly the declared targets when every
   declared target has a shard, while undeclared expected targets stay missing?
3. **Wrapper coverage after a write run.** Should onboarding O bind `wrapper_coverage.json`
   (OF-P6) when publication may regenerate it, or should publication be refused from regenerating
   it for a depth-enrolled generation?
4. **Rule 4 ordering with NE3.** `manifest-version-metadata` refuses depth manifests. Should
   version metadata be written before the declaration is added, or only by the E.2 `onboard-agent`
   writer?
5. **Scope of OF-P9.** Items 1 and 2 also make non-depth runs safer. Should they apply to every
   run in a later change?

## Implementation slices

Each slice is a reviewed PR into `staging`, after approval and in this order. None enables a path,
installs a profile or changes production refusals. Every slice is tested on the synthetic subject
under `Fixture::install()` only.

1. **Charter revision (docs).** Name the lifecycle-state `support_depth` object as the rule 7
   artifact, with its writer and replacer (OF-P2). Name the acquisition writers and the
   integration-branch marker (OF-P3, OF-P4). State OF-P5 to OF-P7, and update the
   [Present behavior](../../specs/cli-agent-onboarding-charter.md#present-behavior) bullets and the
   test list's open items. Annex A's ON2 row is updated to say the dry run is an LW1 caller.
2. **Typed lifecycle `support_depth` object (OF-P2).** Add the field, reject unknown keys or bump
   the schema version, reuse `RequestBinding`/`request_mismatch` for the onboarding Event, and add
   the validation cross-check. Add preservation tests for every lifecycle writer. There are no
   behavior changes while no declaration exists.
3. **Write-run hardening for depth-enrolled generations (OF-P9, the OF-P4 boundary change).**
   - baseline after the dry run's files;
   - narrowed runs area;
   - `snapshots/**` dropped from the allowed paths;
   - write-attempt transition.
   Non-depth runs are byte-identical.
4. **Input contract identity (OF-P8).** Add the optional field, bump `workflow_version`, add the
   pre- and post-execution checks, and make `repair-runtime-evidence` copy the object.
5. **Dry-run freeze core (OF-P1, OF-P4 to OF-P7).** A private, isolated-route-only dry-run path
   that resolves rule 3, checks rule 4, writes P, the record and the object, reads the
   integration-branch tip for acquired inputs, and calls the shared `derive`. It covers the
   complete-union and post-integration refusals. The public entry keeps refusing depth until
   roadmap E.6.
6. **Acquired-input refusal in ON3 to ON5 (OF-P6).** Mirror the relay's frozen-input check in
   `prepare-publication`, `refresh-publication` and `close-proving-run` for depth-enrolled
   generations.

Slices 2 to 4 are independent of each other. Slice 5 depends on slices 2 and 3, and slice 6 on
slice 5.
