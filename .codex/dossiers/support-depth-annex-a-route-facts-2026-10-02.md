# Annex A route facts — first pass (2026-10-02)

Read-only fact base for the support-depth contract's Annex A. Source revision: `staging` `f61534be`
(identical to the candidate branch for all files below). Present behavior only; required behavior
is decided after the P1a review is adjudicated. Not yet reconciled entrypoints-forward.

## Writers by protected output

| Output | Write site (present) | Entrypoints that reach it |
| --- | --- | --- |
| Support matrix JSON + Markdown (all agents, aggregate) | `support_matrix/publication.rs::write_publication_artifacts` | `xtask support-matrix` (called by `parity-acquire.yml:557`, `parity-promote.yml:336`, humans) |
| Same, plus runtime-support data and capability matrix | `publication_refresh.rs::run_in_workspace` via `build_publication_artifact_plan` → `workspace_mutation::apply_mutations` | `xtask refresh-publication` (onboarding path; requires lifecycle `publication_ready`) |
| Embedded runtime-support data `crates/agent_api/src/runtime_support_data.rs` | bundled with support-matrix generation | `support-matrix`, `refresh-publication` |
| Capability matrix `capability-matrix.md` | `capability_matrix.rs:67` | `xtask capability-matrix`, `refresh-publication` |
| Version metadata `versions/<v>.json` | `manifest_version_metadata.rs:302` | `xtask manifest-version-metadata` / `codex-version-metadata` (called by `parity-acquire.yml:556` with `--status reported`, `parity-promote.yml:295`) |
| Pointers `latest_validated.txt`, `pointers/latest_{validated,supported}/<t>.txt` | **shell in `parity-promote.yml:285,286,303,308`**; `manifest_validate/pointers.rs:23`; `manifest_validate/fix_mode.rs:14` writes `none` | promotion workflow; `xtask manifest-validate` (fix mode trigger at `manifest_validate.rs:281`, condition to confirm) |
| `current.json` | `manifest_validate/fix_mode.rs:42` | `manifest-validate` fix mode |
| Stand-down marker removal | **shell `rm -f` in `parity-promote.yml:324-326`** | promotion workflow |
| Snapshot/version directory removal | `manifest_retain.rs:238` (`remove_dir_all`) | `xtask manifest-retain` / `codex-retain` |
| Coverage reports `reports/<v>/coverage.<t>.json` (planned depth-fact carrier) | `manifest_report/util.rs` | `xtask manifest-report` / `codex-report` (`parity-acquire.yml:555`) |
| Maintenance closeout | `agent_maintenance/closeout/write.rs::write_closeout_outputs`; `prepare_closeout.rs` (+ `restore_and_report` at :307) | `close-agent-maintenance`, `prepare-agent-closeout` |
| Proving-run closeout | `close_proving_run.rs`, `prepare_proving_run_closeout.rs` (both via `workspace_mutation`) | `close-proving-run`, `prepare-proving-run-closeout` |
| Lifecycle record stages `lifecycle-state.json` | `agent_lifecycle::write_lifecycle_state` | `prepare-publication` (:431), `close-proving-run` (:476), `repair-runtime-evidence` (:380), `runtime-follow-on` (lifecycle.rs:133), `historical-lifecycle-backfill` (:215, closed_baseline only), `refresh-publication` (via mutation plan) |
| `publication-ready.json` | `prepare_publication.rs` (own writes); `write_publication_ready_packet` | `prepare-publication`, `historical-lifecycle-backfill` (:191) |

## Restoration writers (stale-compensation candidates)

- `publication_refresh.rs:255-266`: `capture_snapshots` → `apply_mutations` → gate → `restore_snapshots` on failure.
- `repair_runtime_evidence.rs:312` `restore_backup`.
- `agent_maintenance/prepare_closeout.rs:307` `restore_and_report`.

## Existing chokepoint

`main.rs:143 run_write_side_lifecycle` wraps only `prepare-agent-maintenance`, `execute-agent-maintenance`,
`refresh-agent`, `close-agent-maintenance`, `prepare-agent-closeout`. Today it carries only the
nested-invocation guard (`XTASK_AGENT_MAINTENANCE_RUN_ID`). No publication or promotion writer
passes through it.

## Observations that bear on the contract text (raise in adjudication)

1. Most protected effects reach `staging` by **PR merge** of a branch carrying generated outputs
   (`parity-acquire.yml:876-913` commits to the packet branch). The merge is a route; its admission
   point is CI validation at merge time, and a PR validated against an older `staging` is the
   stale-admission case. The contract's route definition should name merges explicitly.
2. Coverage reports become the depth-fact carrier under the contract's Publication rule, so report
   writers join the protected set. The Protected effects list does not yet include them.
3. Pointer and marker writes happen in workflow shell, outside any xtask command, so they cannot be
   mediated by a shared command as things stand.
4. Every aggregate generator (support matrix, runtime-support data, capability matrix) rewrites all
   agents' rows whichever agent triggered it: acquisition, promotion and onboarding refresh all do.

## Not protected (checked)

`version_bump.rs` (crate VERSION/CHANGELOG), `agent_maintenance/watch.rs:699` (watch output),
`agent_maintenance/audit_status.rs` (audit JSON projection), `agent_maintenance/execute/packet.rs:309`
(run packet files), snapshot commands (`*_snapshot`), wrapper coverage generators (unless coverage
files become depth carriers; recheck).

## Changes after the round-four fix (candidate `d78cf535`, 2026-10-03)

Read these before writing Annex A; they supersede parts of the first pass above.

1. **Observation 2 is superseded.** Coverage reports are not the depth-fact carrier and are not
   depth-gated outputs. The carrier is the depth record: one committed file per depth-enrolled
   version under `cli_manifests/<agent>/reports/<version>/`. Report generators leave the protected
   set. Anything that writes or deletes `reports/<version>/` wholesale joins it.
2. **`manifest-retain` is a route for the depth record.** `manifest_retain.rs:97-126` deletes
   `snapshots/<v>` and `reports/<v>` for every version outside the keep set (pointer versions plus
   the last `keep_last_validated`). No workflow calls it. Present behavior: deletes the record.
   Required behavior: a depth record is never removed. This closes the D1 deferral's trigger.
3. **Directory iterators must skip the record.** Presence checks look for coverage files, not the
   directory (`agent_maintenance/audit_status/evidence.rs:34-51`), so a record-only directory at
   open does not flip the "reports exist" decision. `list_coverage_report_paths` and
   `support_audit.rs:594-600` need the same care in P2.
4. **Lifecycle record writers** (attributed by the record a value reports, not by the writer):
   `historical_lifecycle_backfill.rs:215`, `prepare_publication.rs:431`,
   `repair_runtime_evidence.rs:380`, `close_proving_run.rs:476`,
   `runtime_follow_on/lifecycle.rs:133,176`, `onboard_agent/lifecycle.rs:95`,
   `publication_refresh.rs` (via planned mutations), `agent_maintenance/closeout/write.rs:63-113`.
5. **D3, decide at the promotion entry.** A later version's request can replace the working files
   between a version's closeout and its promotion. Promotion then has only the record's P and O
   identities and the listed closeout acceptance. Annex A must say whether that is sufficient for
   depth admission or the route refuses.
