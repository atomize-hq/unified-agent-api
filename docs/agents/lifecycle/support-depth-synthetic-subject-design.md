# Support depth: synthetic machinery-proof subject design

Status: SA-P1, SA-P2 and SA-P3 approved by the maintainer on 2026-11-02. This is roadmap items B.1 and B.2 of
[MP-P1](support-depth-machinery-proof-rescope.md). Implementation proceeds in the
order below; approval enables no path and installs no profile.
Source baseline: staging `0cb08241` (PR #309).
The [depth contract](../../specs/support-depth-contract.md), [registry contract](../../specs/agent-registry-contract.md#support-depth)
and [evidence contract](../../specs/support-depth-evidence-v1.md) remain normative. This document
changes no code, registry, host setting or credential, and creates no background job.

## What the subject must exercise

The E1 chain must run for real, through the public entrypoints:
1. A new upstream release is noticed.
2. Its assets are acquired and snapshotted.
3. P/O are frozen from the request.
4. Packet evidence is produced and authenticated.
5. Audit and stand-down.
6. Manual closeout through IN1.
7. Separate promotion through the native merge queue.

Each step consumes exact repository data:
- a registry entry and `RULES.json` with an `acquisition` block;
- an xtask snapshot command named by that block;
- version metadata and a baseline snapshot under the manifest root;
- a wrapper crate that yields wrapper coverage;
- an evidence profile whose cases run in CI.

## Facts found while designing

- **The isolated profile is already repository-generic.** `isolated_maintenance/mod.rs` binds
  repository ids, the protected tip, the baseline and an approved closure of compiled producer
  bytes. Only `isolated_maintenance/scope.rs` fixes Codex: the agent, `0.156.1`, musl,
  `codex exec` and CORE. Producer source must be byte-identical to canonical-approved source, so
  the subject's code must land in this repository through reviewed PRs.
- **Acquisition is generic.** `parity-acquire.yml` has no per-agent branching. A
  `github_releases` source with a `tar_gz` archive fits a small Linux binary. Snapshot commands
  are per-agent xtask subcommands, so the subject needs one.
- **The registry assumes every agent is a runtime built-in.** As a test, an extra registry entry
  with publication disabled passes `support-matrix --check`, `capability-matrix --check` and
  `capability-matrix-audit`. It fails `runtime-advertising-model --check` with "unsupported
  built-in inventory": `runtime_advertising/projection.rs:98` requires the registry to equal the
  fixed `AGENTS` list, and `backend_module` is a required registry field. The onboarding gap memo
  records the same assumption, as the hard-coded backend list.

## Approved decisions

| Id | Recommendation | Reason / boundary |
| --- | --- | --- |
| SA-P1 — where the subject lives | **Code in this repository, data only in the isolated one.** Add one unpublished workspace crate, `crates/depth_fixture` (`publish = false`, excluded from release and publication). It holds a tiny fixture CLI binary, a wrapper API over it, and its wrapper tests. Add one xtask `depth-fixture-snapshot` subcommand. The canonical registry never lists the agent. The isolated repository's protected baseline adds the governing data: the registry entry with a `support_depth` declaration, RULES and its acquisition block, version metadata, and the baseline snapshot. Upstream releases are GitHub releases of the fixture CLI in the isolated repository: a baseline `0.1.0`, then `0.2.0` as the new release. | Producer bytes must be canonical-approved (IR-P1 closure), so the code has to be here. Keeping the agent out of the canonical registry means production support claims, matrices and maintenance never see it. Using the isolated repository as its own upstream needs no outside account and exercises `maintenance-watch` for real. The CLI's whole API is known, so "synthetic proof certifies no nonexistent API". |
| SA-P2 — non-runtime registry agents | Let a registry entry declare that it is **not an `agent_api` built-in**: `backend_module` becomes optional, and it is required exactly when the agent is a runtime backend. Runtime advertising and backend feature checks apply only to built-ins. A non-built-in entry MUST NOT carry any capability claim. The parser rejects a non-empty capability declaration, capability publication, and any depth capability mapping or `mapped` / `proposed_extension` operation on such an entry. Built-in membership comes from the fixed compiled backend inventory, never from the entry, so an actual built-in cannot declare itself exempt. Wrapper coverage, acquisition, maintenance and depth work unchanged. This is amended in the [registry contract](../../specs/agent-registry-contract.md) first. | This is a general fix, not a fixture hack. It is the hard-coded backend assumption the onboarding gap memo already names (roadmap E.5), and onboarding needs it too, for an agent between wrapper scaffolding and runtime follow-on. Because claims are rejected explicitly, contract rule 5 (complete mapping) cannot be triggered by a non-built-in. |
| SA-P3 — evidence profile and binding | Amend the evidence contract to add one more compiled isolated profile, **`synthetic-wrapper-v1`**. It covers the fixture agent's single `run` operation with template `CORE` and `shared_mapping = wrapper_only`, on the `x86_64-unknown-linux-gnu` target (corrected on 2026-11-02 with maintainer approval from `linux-x64`: the evidence contract forbids cross-target evidence, and the wrapper builds on the gnu tuple), with every resolved obligation mapped to cases in `crates/depth_fixture`. The cases run in the isolated repository's `rust-tests` job under IR-P3 producer terminality. Its trust revision, executor and receipt paths are named distinctly from CORE. Change `isolated_maintenance/scope.rs` to select between compiled subjects. The Codex subject keeps its current constants and stays a Codex qualification subject. | Adds one profile only from compiled source, as the contract already requires. Profiles never come from a receipt, an environment value or a flag, and unsupported selections still refuse. A wrapper-only operation creates no adapter obligation (contract, "Shared mapping"), so no `agent_api` change is needed. Adequacy mappings still need independent review and maintainer adoption, and the record is labeled machinery proof. |

Unchanged:
- `LIVE_PROFILE` stays uninstalled until the disposable repository, its baseline and its credential
  references are separately authorized (roadmap C).
- Production registry parsing still refuses `support_depth` tables.
- `production_results` stays empty.
- No new job, required check or service is added to the canonical repository.

## Implementation order after approval

Each step is a reviewed PR into `staging`.
1. **SA-P2: amend the registry contract, then change the registry.** The contract defines runtime
   membership and the restrictions on non-runtime entries. The change ships positive and negative
   tests, including a rejected capability claim on a non-built-in and a built-in that tries to
   declare itself non-runtime. All existing agents and matrices stay byte-identical.
2. **Fixture crate and snapshot command (SA-P1).** Tests run in the existing `rust-tests` job. The
   isolated repository's governing data is kept here only as test-fixture templates. It is
   installed nowhere, and the canonical registry never lists the agent.
   The templates are in `crates/xtask/tests/fixtures/depth-synthetic/governing/`, and
   `xtask depth-fixture-wrapper-coverage` generates the wrapper coverage. A test runs the real
   acquisition-plan, snapshot, union, coverage, report, version-metadata and validate commands
   over them. Release-watch for `github_releases` supports only `latest_stable_minus_one`, so the
   isolated repository also publishes `v0.3.0`; only then does watch target the enrolled `0.2.0`.
3. **Evidence-contract amendment, then the `synthetic-wrapper-v1` profile and subject selection
   (SA-P3).** Ships read-only positive and negative evaluation tests, in the same shape as the CORE
   bridge.
4. Continue with roadmap B.3–B.8 against this subject.
   - B.3 connects the public `prepare-agent-maintenance` route, which is MA1 and MA2's re-freeze,
     through `isolated_maintenance::route`. While no profile is installed it returns nothing and
     the route keeps its production refusals. A plan applies only on the route and protected tip it
     was built under.
   - Depth records live only in published roots, so the isolated support matrix publishes the
     subject. The capability matrix never does.
   - Promotion still loads the production registry (B.6). The isolated baseline therefore promotes
     `0.1.0` before the depth table is installed in a later protected commit.
   - The first real O keeps every owned `run` surface `classification_required`. It is not
     cleared to make the chain look ready: resolving it is a maintainer change to P (contract,
     classification rule 6).
   - Maintainer-approved 2026-11-03:
     - `run --help` is a per-unit parity exclusion (`dispatch`), like the root's help and version
       flags, and not an intentionally-unsupported waiver. The support audit carries any waived
       unit as required work, and that work survives into every later generation. So the
       exclusion is in the templates before any O freezes.
     - Each of the five `run` surfaces (`run`, `--help`, `--json`, `--model`, `PROMPT`) gets a
       classification row that selects only `CORE`, with the acquired `classification_basis`.
       Preparation prints that basis (`classification_basis: operation=… sha256=…`) for the
       maintainer to review. P changes only in a new generation, so the rows enter under a new
       Event. A stale basis refuses.
     - The isolated scope admits classification rows only for the synthetic subject, and only
       when every row selects `CORE` alone. A retained earlier generation may differ from the
       declaration only in those rows and their basis.
   - B.3 part 2b routes MA2's other read routes through `isolated_maintenance::registry`:
     `manifest-acquisition-plan`, `maintenance-stand-down-check`, `manifest-validate`'s depth
     checks, `support-matrix` and `maintenance-audit-status`. A resolved isolated route yields its
     rechecked scoped registry. Otherwise each keeps the production load and its `support_depth`
     refusal, and a failing installed route errors instead of falling back. Audit-status is not a
     depth-gated output (contract, route inventory), so it stays clean while O is unclassified;
     MA5 closeout gates depth readiness.
   - Resolved, outside this subject: `parity-acquire.yml` ran `manifest-validate` after
     `manifest-version-metadata --status reported` but before the support matrix was published.
     Its publication-consistency check then refused any agent's newly reported version
     (`SUPPORT_MATRIX_ROW_MISSING`, reproduced without depth) or reported a stale depth row once
     MA1 had published one. Validation now runs in its own step after publication and before
     delivery; the isolated proof asserts the pre-publication refusal.

   - B.4 (MA4) decisions, approved by the maintainer on 2026-11-03:
     - E is written by a new xtask command that the MA4 executor runs inside the candidate. It
       works only under an installed isolated route, and production keeps refusing it. The relay
       delivers only that reporting E, and the audit re-evaluates it independently.
     - The green gates keep `codex-validate --root <manifest_root>`. It is the agent-general
       validator under its old name, and the request loader requires every frozen gate list to
       equal the current contract, so a rename would invalidate the committed Codex, Claude Code
       and OpenCode requests. The rename is later cleanup.
   - B.4 part 1 routes `execute-agent-maintenance` through the isolated context. It resolves the
     route once, in the caller workspace, and keeps a `CapturedRegistry` that only
     `Route::capture` mints. Every later request reload uses it. The independent candidate cannot
     resolve the route, so while the guarded run checks the candidate, the captured registry is
     bound to that candidate's exact root for in-process checks. Binding requires an independent
     clone whose HEAD descends from the caller's captured HEAD and whose registry equals the
     captured one; the guard is thread-bound. The relay still refuses any registry change, and the
     captured route is rechecked against the caller before delivery. Capability publication now
     loads through `isolated_maintenance::registry` too. Without a route, every load keeps the
     production `support_depth` refusal.
   - B.4 part 2 writes reporting E from authenticated isolated evidence (design option A).
     - Under the captured route only, the guarded MA4 host gets the evidence host from that
       route. It authenticates the complete applicable run and attempt population with the
       synthetic profile's resolver. It then stages a bounded, read-only bundle (E plus every
       referenced receipt and payload) in a private temporary directory outside the candidate.
       Production has no route and stages nothing.
     - The executor runs `xtask depth-evidence-assemble` inside the candidate. With no network
       and no credentials, it writes E and its files only into the packet's normal reporting
       destination, `<manifest_root>/reports/<version>/`. Its record must hold the same Event,
       P and O. The E and file assembly is shared with the `prove` diagnostic, which still
       previews in a scratch clone.
     - The live invocation is the only route to the bundle. The host serves it through a Unix
       socket in that private directory, for its own candidate root only, while the executor
       runs. The executor gets just the socket's path, as
       `XTASK_AGENT_MAINTENANCE_EVIDENCE_CHANNEL`. The command checks the bundle's digest and
       invocation id against what the socket answers. When the executor exits, the socket and
       the bundle are removed. A copied bundle, a path, the variable alone or a restored origin
       is therefore never a route. `CapturedRegistry` is never serialized.
     - The assembler's channel check is fail-closed convenience; authority is the host's
       captured route plus byte binding at delivery; a forged standalone run can write only an
       undeliverable local E. The command writes nothing without a live socket that answers, but
       it does not authenticate the server: any check of a socket's listener is forgeable by a
       same-user process, and an identity check would also reject a legitimately built
       `cargo run -p xtask -- depth-evidence-assemble`.
     - The host answers only a peer whose process (read from the kernel with SO_PEERCRED)
       descends from the executor it spawned, and only when the request carries the
       invocation's random nonce. The host passes the nonce to the executor alone, as
       `XTASK_AGENT_MAINTENANCE_EVIDENCE_NONCE`; green gates never inherit it. The requested
       root must equal the host's own canonical candidate root; it is never resolved. Platforms
       other than Linux have no such anchor, so the host refuses every client. The host revokes
       the channel as soon as the executor process exits, on every path and before any gate
       runs. Each request has an absolute deadline, and nothing is answered after revocation.
     - The executor and the green gates never inherit host delivery credentials (`GH_TOKEN`,
       `GITHUB_TOKEN`, `AUTOMATION_TOKEN` and the related GitHub, registry and Actions token
       variables). The executor's own home and model login are left as they are.
     - E remains untrusted. The relay checks are unchanged. One delivery change: support
       publication reads bound evidence only from the Git index. So the relay validates against
       the caller index plus the new files that a changed record's E binds and that are
       byte-identical to files the host itself staged. Copy-back adds exactly those files to the
       caller index, as blobs written from the validated bytes (never re-read from the working
       tree). The index entries are published before any file is written, so an index failure
       writes nothing; if a file write then fails, exactly the owned index entries are removed
       and a concurrent change to one of them is preserved and reported. Publication and
       rollback each follow Git's lockfile protocol: the relay creates `index.lock`
       exclusively before it compares anything, works on a private copy of the index, and
       installs that copy with an atomic rename while it still holds the lock. No other Git
       process can change the index between the comparison and the installation, and an
       existing lock refuses with the index unchanged. This byte binding is the authority over
       the channel, and it binds by path class, whether or not any depth record changed. The
       class is every file under any evidence destination (`.../reports/<version>/evidence/`),
       every receipt any E binds, and every payload those receipts reference (toolchain,
       environment, dependency graph, raw output), wherever it lies. Each snapshot, before and
       after the run, is traversed on its own and the results are unioned; every receipt is
       read, including those found in an evidence destination. A receipt's source inputs
       (covered inputs, provenance, assertion sources) are product files the run may change, so
       they are not in the class.
       Every file the candidate adds, changes or removes in that class must be byte-identical to
       the host-staged bundle file at that path; anything else refuses the whole delivery,
       before publication and again at copy-back, with caller files and index unchanged. In
       production nothing is staged, so any change in the class refuses. Matching new files,
       payloads included, are what gain index identities. E from a forged channel, a payload
       substituted behind unchanged receipts (also on a rerun over committed evidence), a
       removed payload, an added unreferenced file, a substituted payload outside `evidence/`
       or a receipt newly referencing another file are therefore refused. Bound evidence may
       never live in an MA4 run directory (`docs/agents/.uaa-temp/agent-maintenance/runs/`),
       whose diagnostics are copied back even when a run fails: any E or receipt reference
       there refuses the whole delivery.
     - Failure delivery copies back no file the candidate holds. The host serializes every run
       diagnostic itself, in its own private directory, from what it holds in memory: its
       validation report, run status, summary, the executor's exit record and its stdout and
       stderr as the host received them, and the recovery pointer. A failed candidate's own
       report is never read back. No diagnostic is written over a path bound evidence names.
     - Every capture used for binding covers the caller's tracked paths together with the
       candidate's, so a tracked file under a directory capture otherwise skips (such as
       `target/`) is always classified.
     - Containment, at MA4 delivery only: every E reference must lie in its own record's
       `<manifest_root>/reports/<version>/evidence/`, and every payload a receipt references in
       that receipt's evidence destination. Anything else refuses the delivery. Contract
       revision 1 allows any repository-relative path, and evaluation is unchanged. The
       maintainer approved keeping this containment on 2026-11-05; the
       [evidence format](../../specs/support-depth-evidence-v1.md#boundary) states it. A completely unchanged
       MA4 rerun is still refused by the existing zero-change validator; that is unchanged. Production stages nothing, so no new E-bound
       file is ever admitted. A receipt that only the candidate made still refuses as an
       untracked bound input. `evaluate_isolated_record` now covers the
       synthetic profile through the installed binding. It, and every other consumer,
       re-authenticates the full population rather than trusting a bundle.
   - B.5b connects both MA5 writers to the B.5a owning authority (G2–G4, CP-P4 of the
     [closeout/promotion design](support-depth-closeout-promotion-design.md)). Both the closeout
     planner/writer and `prepare-agent-closeout` capture the resolved route's proposal before
     preparing outputs. A failed installed route refuses; an absent route keeps production
     behavior and its depth refusal. The request validator, findings, support-audit truth, drift
     readers and maintenance-adoption validator use the routed registry. The proposal rechecks
     its captured caller and host before delivery and MA5 re-authenticates the full applicable
     evidence population for the exact record, rather than trusting E's labels.
     - Under the route, preparation reads preflight from the authenticated isolated repository
       of its captured context through `Proposal::preflight_repository`, a read accessor that
       exposes no context, evidence host or admission authority. Neither a flag nor the spelling
       of `origin` selects it. Production keeps the canonical repository. Run selection, attempt
       handling and whole-workflow
       terminality are unchanged, including in preview.
     - `isolated_maintenance/tests/ma5_closeout.rs` exercises each writer on the classified
       synthetic MA1 → acquisition → MA2 → MA4 → audit chain, with the exact-generation marker
       on the fake protected tip. Refusals cover production/no route, captured-route drift,
       missing or wrong-generation markers, changed generation/P/O and an unverified population
       despite E's verified labels. The socket-free checks also cover preflight policy and routed
       preview/readers with a missing-E refusal. Full connected chains require Unix sockets and
       must run outside the restricted sandbox; the MA4 green gates remain stubbed.
     - Publication is unchanged. **Qualified publication not exercised**: CP-P3 option A belongs
       to B.7b, which must rerun the complete chains. MA6 wiring belongs to B.6.
   - B.5c connects IN1 to route-selected owning authority (G8, CP-P1 of the
     [closeout/promotion design](support-depth-closeout-promotion-design.md)). The repository-wide
     `check_integration` resolves the route before enumerating roots. An absent route retains
     production semantics; a failed installed route refuses. Per-root `manifest-validate --tip`
     uses the routed registry and route-bound baseline/comparison readers, retaining the existing
     distinction between a production registry failure and an isolated route refusal.
     - The fully qualified local `refs/remotes/origin/staging` must equal authenticated protected
       tip T; similarly named tags or local branches supply no authority. With H the checkout
       HEAD and P the supplied parent, admission requires P = T, H != T and local Git ancestry
       from P to H. Observation requires H = T and P equal to H's immediate first parent, read
       from T's raw commit headers with replacement objects disabled; message encoding is
       irrelevant. P, H and T are pinned to object IDs once. Both run full all-root generation,
       closeout and promotion admission for that exact P → H, then recheck the captured context,
       HEAD and fully qualified local tip. A moving caller parent ref cannot change the comparison.
       Caller labels, flags and event payloads confer no authority. Equal H/P, stacked bases,
       unsupported parentage and observation across several first-parent commits refuse.
     - `isolated_maintenance/tests/in1_integration.rs` covers binding decisions without sockets,
       including distinct merge parents, a multi-commit first-parent chain, forged or unrelated
       parents, shadow tags/branches, non-UTF-8 commit messages, a parent-ref race with a forged
       closeout entry, local-tip mismatch, authenticated-tip drift and unchanged ordinary production
       behavior, plus per-root `--tip` validation of a frozen P through the routed readers.
       The connected Linux tests take each B.5b MA5 writer's output through admission,
       post-push observation and per-root `--tip` validation, and refuse forged entries and an
       unverified population despite verified E labels. They also retain the production depth
       refusal and check that validation preserves caller files and index. These full chains
       require Unix sockets and must run outside the restricted sandbox; publication remains
       unchanged pending B.7b. No workflow is changed (CP-P5).
   - B.6b connects MA6 to the caller's captured route (G5, G6, G10 of the
     [closeout/promotion design](support-depth-closeout-promotion-design.md)). `build_plan`
     resolves once and captures the registry; its proposal shares that same `Arc<Route>`.
     `Proposal::isolated_view` and B.6a's `CapturedRegistry::bind_view` give scratch renderers
     and canonical validation the captured registry and pinned, read-only caller Git inputs.
     Scratch never resolves authority. The ordinary, absent-route path retains its production
     registry and proposal.
     - Under a route, validation must name HEAD, the local integration tip and the authenticated
       protected tip. Planning and apply use `Proposal::authority` to run the existing promotion
       predicate through B.5a's isolated host. Apply rechecks the captured route immediately before
       delivery. No context is serialized and no new public authority is introduced.
     - `isolated_maintenance/tests/ma6_promotion.rs` covers both MA5 alternatives through IN1
       admission and observation, then MA6's pointers, metadata, publication outputs, first
       `Promotion` entry and marker retirement, followed by IN1 admission and observation.
       The A.5 replay forks from the committed, IN1-admitted MA5 checkpoint before any MA6;
       an ordinary non-depth `0.3.0` request and its own closeout replace the working files.
       MA6 for `0.2.0` then relies on the retained closeout entry alone. A sibling fork changes
       the wrapper's prompt limit, invalidating the earlier evidence, and must refuse at the
       producer-closure approval check (`crates` differs from the retained producer build).
       Each replay advance asserts a clean committed tree, local integration ref and
       authenticated protected tip; ordinary generation replacement preserves the producer tree.
       Refusals also cover production/no route, local/authenticated-tip mismatch, route drift
       before apply and unbound or forged scratch. The production predicate refusal proposes the
       pointer advance directly: a refused apply leaves new version metadata absent in the caller.
       Socket-free tests count one route resolution
       and exercise the ordinary replay generation separately; the full MA4-connected chains
       require the lead's run outside the socket sandbox.
     - Publication remains on production verdicts and unqualified throughout this slice. The
       option A ordering and qualified publication belong to B.7b; no workflow changes here.
   - Open, before the isolated run (C): green gates run as subprocesses in the candidate, so they
     see neither the route nor the bound registry. `capability-matrix --check` and
     `support-matrix --check` would then refuse the depth subject. The in-process proof stubs the
     gates; the real run needs a decided gate route.

## Open risk

The fixture CLI's surface must be big enough for every resolved CORE obligation (R/V/G/E/A/X,
OUT3 and LC2) to have a substantive case: JSON output, an error path, and a lifecycle end. If the
resolver turns out to need a surface the CLI lacks, add it to the CLI, not as a not-applicable
permission.
