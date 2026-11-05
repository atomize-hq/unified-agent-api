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

Placeholders: `{{APPROVAL_COMMIT}}` and `{{APPROVAL_RECORDED_AT}}` in the approval, and each
`{{SHA256}}` in the statement. The registry entry has none.

The approval descriptor omits `backend_module`, because the subject is a non-runtime entry
(SA-P2). Today's full approval loader still requires that field and refuses the template. Only the
approval's attribution (agent, pack and `[support_depth]` version) parses. A later slice decides
how a non-runtime approval loads.

`crates/xtask/src/isolated_maintenance/tests/onboarding_scope.rs` checks these templates against
the isolated scope.
