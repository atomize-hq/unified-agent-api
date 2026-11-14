# Isolated onboarding-proof templates for the synthetic subject

These templates set up the onboarding proof workspace for `depth_fixture` 0.2.0 on
`x86_64-unknown-linux-gnu` (slice 0 of the
[onboarding freeze decisions](../../../../../../docs/agents/lifecycle/support-depth-onboarding-freeze-decisions.md)).
The evidence contract's "Synthetic onboarding proof selection" section governs them. They are
installed only in the disposable isolated repository, never in the canonical repository. This is a
proof workspace, not onboarding path enablement: ON1 to ON5 keep refusing depth.

| Template | Installed at |
| --- | --- |
| `registry-entry-deferred.toml` | appended to `crates/xtask/data/agent_registry.toml`, in place of `../governing/registry-entry.toml` |
| `governance/approved-agent.toml` | `docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml` |
| `governance/depth-acquisition/0.2.0.toml` | `docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/depth-acquisition/0.2.0.toml` |

The manifest-root files are the same as for the maintenance proof (`../governing/manifest-root/`).

- **One path per workspace.** The registry entry enables only the onboarding path and has one
  onboarding enrollment. A workspace holds either this entry or the maintenance entry, never both.
- **Deferred maintenance (OF-P3 A).** The approval states `explicitly_deferred` and the entry has
  no `maintenance.release_watch`. Acquisition uses the input-only writers directly
  (`depth-fixture-snapshot`, `manifest-union`, `depth-fixture-wrapper-coverage`,
  `manifest-report`). The `manifest-acquisition-plan` preview still refuses this entry.
- **Acquisition statement (OF-P4 B).** The statement lists each input O binds, with its SHA-256 and
  producing command. A maintainer commits it to the integration branch, and it counts only from
  there. Its fields are not normative until the charter revision (slice 1).

Placeholders: `{{APPROVAL_COMMIT}}` and `{{APPROVAL_RECORDED_AT}}` in the approval. In the
statement, `{{DEPTH_FIXTURE_BINARY}}` is the path of the acquired release executable that the
snapshot command ran, and each `{{SHA256}}` is that input's digest at the integration tip. The
registry entry has none.

Each statement `command` is the exact invocation, run from the workspace root in the order listed,
that produced its input. `crates/xtask/src/acquisition_statement_cli_tests.rs` parses every
`xtask` command with the real CLI definitions and checks that it writes the input it is listed
for.

The approval descriptor omits `backend_module`, because the subject is a non-runtime entry
(SA-P2). Today's full approval loader still requires that field and refuses the template. Only the
approval's attribution (agent, pack and `[support_depth]` version) parses. A later slice decides
how a non-runtime approval loads.

`crates/xtask/src/isolated_maintenance/tests/onboarding_scope.rs` checks these templates against
the isolated scope.

## Onboarding producer binding

Slice 5 reuses the existing `depth-synthetic-collect` command, `depth_ci_synthetic` case target,
eleven obligations, receipt format, artifact and evidence namespace. The compiled onboarding
binding reads the committed ON2 output at
`cli_manifests/depth_fixture/reports/0.2.0/depth-record.json`, the installed finite approval above,
and its sibling `lifecycle-state.json`. It checks the actual approval Event, frozen lifecycle
P/O object and successful implementation state. The maintenance `../record.json` remains the
maintenance producer input. Each binding authenticates its own Event/P/O and adoption source.

The onboarding graph binds the immutable approval, contract, acquired inputs and complete wrapper
build closure. The record's E/entries and later lifecycle stages are checked as frozen-generation
projections, allowing reporting and publication/proving checkpoints to retain Event/P/O. The
approval-linked staging PR supplies independent adoption for every due mapping. Every matching
`ci.yml` run and every attempt remains applicable regardless of its PR association or witness
availability. Terminal producer evidence can verify while its workflow runs.

`depth_evidence/verdict/github/tests/onboarding.rs` uses private clones of immutable outputs from
the actual public ON1/acquisition/declaration/ON2 chain. Each test owns its mutable fake host and
repository. Source and adoption comparisons execute against their actual committed objects.
Later publication/proving run fixtures exercise population binding, with owning ON3–ON5 delivery
and native protected CI scheduling reserved for their implementation slices and phase C.
