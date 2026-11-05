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
   - Open, outside this subject: in `parity-acquire.yml`, `manifest-validate` runs after
     `manifest-version-metadata --status reported` but before the re-freeze republishes the
     support matrix. Its publication-consistency check then reports `SUPPORT_MATRIX_ROW_MISSING`
     for any agent's newly reported version (reproduced without depth), or a stale depth row once
     MA1 has published one. The isolated proof validates after the re-freeze.

## Open risk

The fixture CLI's surface must be big enough for every resolved CORE obligation (R/V/G/E/A/X,
OUT3 and LC2) to have a substantive case: JSON output, an error path, and a lifecycle end. If the
resolver turns out to need a surface the CLI lacks, add it to the CLI, not as a not-applicable
permission.
