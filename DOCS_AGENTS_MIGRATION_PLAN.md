# Docs Agents Migration Plan

## Summary

This plan migrates the live agent packet contract from `docs/reports/**` to `docs/agents/**` without changing the meaning of the historical packet artifacts.

Canonical destinations after migration:

- `docs/agents/selection/cli-agent-selection-packet.md`
- `docs/agents/lifecycle/**`

This is a move-and-rewire migration, not a rewrite. Preserve packet history, identifiers, timestamps, findings, hashes, and narrative meaning. Update only path roots, internal references, validators, generated text, and repo guards required to make `docs/agents/**` the only accepted live contract.

## Final Contract

After migration, these are the only canonical live roots for this contract:

- `docs/agents/selection/cli-agent-selection-packet.md`
- `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml`
- `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/proving-run-closeout.json`
- `docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-request.toml`
- `docs/agents/lifecycle/<agent_id>-maintenance/governance/maintenance-closeout.json`

Fully banned old roots after migration:

- `docs/reports/agent-lifecycle/**`
- `docs/reports/verification/**`

Rules:

- Do not leave compatibility copies, fallback acceptance, redirects, or dual-root support.
- No tracked file may remain under either banned root.
- No live code, tests, specs, docs, generated artifacts, or hygiene rules may reference either banned root after migration.
- The only allowed post-migration mentions of banned roots are explicit archival or migration-context documents, such as `.archived/**` and this plan.

## Live Vs Historical Surfaces

Live surfaces:

- `docs/agents/selection/cli-agent-selection-packet.md`
- `docs/agents/lifecycle/**`
- all `xtask` validators/generators that load, validate, render, or point at those paths
- registry data, tests, operator docs, specs, planning docs, changelog entries, and hygiene rules that define or enforce the contract

Historical or archived surfaces:

- the moved packet contents under `docs/agents/lifecycle/**` remain historical proving-run and maintenance evidence, even though they live under the new canonical root
- `.archived/**` remains archival provenance and is not part of the live path contract
- the accidental `docs/reports/verification/*` leftovers are obsolete verification artifacts and must be deleted, not migrated

## Explicit Path Mappings

Root mappings:

| Old path | New path | Action |
| --- | --- | --- |
| `docs/reports/verification/cli-agent-selection/cli-agent-selection-packet.md` | `docs/agents/selection/cli-agent-selection-packet.md` | move |
| `docs/reports/agent-lifecycle/**` | `docs/agents/lifecycle/**` | move subtree, preserve filenames and relative layout |

Lifecycle subtree preservation rules:

- `gemini-cli-onboarding/**` stays `gemini-cli-onboarding/**`
- `opencode-maintenance/**` stays `opencode-maintenance/**`
- every file currently under each packet root keeps the same basename and relative subpath
- `governance/**` stays `governance/**`

Internal reference remaps:

- every `comparison_ref` value must become `docs/agents/selection/cli-agent-selection-packet.md`
- every internal `approval_ref`, `request_ref`, `closeout metadata`, `review surfaces`, `handoff`, and packet-root literal must switch from `docs/reports/agent-lifecycle/...` to `docs/agents/lifecycle/...`

## Exact Deletion List

Move the selection packet, then delete these leftover verification artifacts:

- `docs/reports/verification/changed-paths-2026-02-20.md`
- `docs/reports/verification/cross-doc-agent-api-backend-harness-2026-02-23.md`
- `docs/reports/verification/cross-doc-agent-api-codex-stream-exec-2026-02-20.md`
- `docs/reports/verification/cross-doc-agent-api-codex-stream-exec-2026-02-20.tasks.json`
- `docs/reports/verification/cross-doc-claude-code-live-stream-json-2026-02-18.md`
- `docs/reports/verification/cross-doc-universal-agent-api-session-2026-02-28.md`

Delete empty directories once the move is complete:

- `docs/reports/verification/cli-agent-selection/`
- `docs/reports/verification/`
- `docs/reports/agent-lifecycle/`
- `docs/reports/` if empty

## Risky Invariants And Validators

These are the highest-risk path gates and must be updated in the same change:

- `crates/xtask/src/approval_artifact.rs`
  - `DOCS_NEXT_ROOT` must become `docs/agents/lifecycle`
  - `COMPARISON_REF_PATH` must become `docs/agents/selection/cli-agent-selection-packet.md`
  - `comparison_ref` is a literal equality gate, not a loose reference
- `crates/xtask/src/onboard_agent.rs`
  - help text and path contract comments must switch to `docs/agents/lifecycle`
- `crates/xtask/src/close_proving_run.rs`
  - expected path prefix and validation error text must switch to `docs/agents/lifecycle`
- `crates/xtask/src/agent_maintenance/request.rs`
  - `DOCS_NEXT_ROOT`, `expected_prefix`, and validation error text must switch to `docs/agents/lifecycle`
- `crates/xtask/src/agent_maintenance/closeout.rs`
  - `DOCS_NEXT_ROOT` must switch to `docs/agents/lifecycle`
- `crates/xtask/src/agent_maintenance/closeout/validate.rs`
  - expected path prefix and validation error text must switch to `docs/agents/lifecycle`
- `crates/xtask/src/agent_registry.rs`
  - `approved_agent_descriptor` path validation must switch to `docs/agents/lifecycle/{onboarding_pack_prefix}/governance/approved-agent.toml`
- `crates/xtask/src/onboard_agent/preview/render.rs`
  - rendered output paths must switch to `docs/agents/lifecycle`

Important invariant:

- the lifecycle root change preserves path depth: both old and new lifecycle roots have three path segments before `<pack>`, so validators that enforce `components.len() == 6` and read the pack name from `components[3]` should keep the same shape and only change the expected prefix literals

Selection-path note:

- the selection packet path does not preserve the old `docs/reports/verification/...` depth, but the current hard gate is literal equality in `approval_artifact.rs`, so the required behavior change is the constant and every fixture or artifact that must match it

## Required File Updates

### 1. Move The Canonical Files

Create the new roots if absent:

```sh
mkdir -p docs/agents/selection docs/agents/lifecycle
```

Move with history preservation:

```sh
git mv docs/reports/verification/cli-agent-selection/cli-agent-selection-packet.md docs/agents/selection/cli-agent-selection-packet.md
git mv docs/reports/agent-lifecycle/* docs/agents/lifecycle/
```

Do not rename packet prefixes, filenames, or governance artifact filenames.

### 2. Update Moved Historical Packet Artifacts In Place

Touch every moved file under:

- `docs/agents/lifecycle/gemini-cli-onboarding/**`
- `docs/agents/lifecycle/opencode-maintenance/**`

At minimum update path literals inside:

- `README.md`
- `HANDOFF.md`
- `review_surfaces.md`
- `scope_brief.md`
- `seam_map.md`
- `threading.md`
- `governance/approved-agent.toml`
- `governance/proving-run-closeout.json`
- `governance/maintenance-request.toml` if it embeds moved references
- `governance/maintenance-closeout.json`
- `governance/remediation-log.md`

Preservation rule:

- keep the historical content intact except for path-root rewrites and list entries that must point at the moved files

### 3. Update `xtask` Code And Validators

Required code surfaces:

- `crates/xtask/src/approval_artifact.rs`
- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/close_proving_run.rs`
- `crates/xtask/src/agent_maintenance/request.rs`
- `crates/xtask/src/agent_maintenance/closeout.rs`
- `crates/xtask/src/agent_maintenance/closeout/validate.rs`
- `crates/xtask/src/agent_registry.rs`
- `crates/xtask/src/onboard_agent/preview/render.rs`

Also update any adjacent generator/help-text surfaces returned by repo-wide search, especially:

- preview output text
- dry-run output text
- validation error messages
- doc comments describing accepted repo-relative paths

### 4. Update Registry Data

Required registry artifact:

- `crates/xtask/data/agent_registry.toml`

Required change:

- update every governance-check `path` that currently points at `docs/reports/agent-lifecycle/.../approved-agent.toml` to the new `docs/agents/lifecycle/.../approved-agent.toml` location

### 5. Update Tests And Harnesses

Required test categories:

- `crates/xtask/tests/onboard_agent_entrypoint/**`
- `crates/xtask/tests/onboard_agent_closeout_preview/**`
- `crates/xtask/tests/agent_maintenance_refresh.rs`
- `crates/xtask/tests/agent_maintenance_closeout.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `crates/xtask/tests/support/onboard_agent_harness.rs`
- `crates/xtask/tests/support/agent_maintenance_harness.rs`

Required test updates:

- change all expected lifecycle paths to `docs/agents/lifecycle/...`
- change all expected selection paths to `docs/agents/selection/cli-agent-selection-packet.md`
- update fixture directory creation from `docs/reports/agent-lifecycle` to `docs/agents/lifecycle`
- update any symlink-escape or normalization tests that assert old path strings
- update approval-artifact fixture contents so `comparison_ref` exactly matches the new canonical selection path

### 6. Update Specs, Operator Docs, Planning Docs, And Generated Docs

Required documentation surfaces:

- `docs/specs/agent-registry-contract.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `PLAN.md`
- `TODOS.md`
- `CHANGELOG.md`

Required documentation behavior:

- replace old canonical-root statements with `docs/agents/selection/...` and `docs/agents/lifecycle/...`
- keep historical descriptions intact; only change path contract statements and examples
- update command examples so all `--approval`, `--request`, and `--closeout` arguments use the new roots

Generated and committed artifacts:

- the moved packet docs under `docs/agents/lifecycle/**` are committed generated artifacts and must have their internal references updated
- run a repo-wide search across `docs`, `crates`, `cli_manifests`, `scripts`, `PLAN.md`, `TODOS.md`, and `CHANGELOG.md`; if any additional committed artifact or generated doc references the old roots, update it in the same migration
- current search did not show `cli_manifests/**` hits, but it must still be part of the verification sweep

### 7. Update Hygiene And Guard Rails

Required hygiene file:

- `scripts/check_repo_hygiene.sh`

Required guard changes:

- add `check_no_tracked_under "docs/reports/agent-lifecycle"`
- add `check_no_tracked_under "docs/reports/verification"`
- add tracked-content guards for `docs/reports/agent-lifecycle` and `docs/reports/verification/cli-agent-selection/cli-agent-selection-packet.md`

Guard exclusions:

- exclude `.archived/**`
- exclude `DOCS_AGENTS_MIGRATION_PLAN.md`
- exclude the hygiene script itself if needed for the pattern literal

Do not add a blanket content ban that would fail on legitimate archival or migration records.

## Implementation Sequence

1. Create `docs/agents/selection/` and `docs/agents/lifecycle/`.
2. Move the selection packet and the full lifecycle subtree with `git mv`.
3. Delete the accidental non-selection verification leftovers listed above.
4. Rewrite internal references inside the moved packet artifacts so every path points at `docs/agents/**`.
5. Update `xtask` constants, validators, renderers, help text, and registry validation to accept only `docs/agents/**`.
6. Update `crates/xtask/data/agent_registry.toml`.
7. Update all affected tests and harness fixtures.
8. Update specs, operator docs, planning docs, changelog, and any other committed generated docs/artifacts that still reference the old roots.
9. Add hygiene guards that ban tracked files under the old roots and ban live references to them.
10. Run the validation checklist below.
11. Remove any now-empty `docs/reports/**` directories.

## Validation Checklist

### Path And Deletion Validation

Confirm new canonical files exist:

```sh
test -f docs/agents/selection/cli-agent-selection-packet.md
test -f docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml
test -f docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml
```

Confirm old roots are gone:

```sh
test ! -e docs/reports/verification
test ! -e docs/reports/agent-lifecycle
```

### Search Validation

Repo-wide live-surface search:

```sh
rg -n "docs/reports/agent-lifecycle|docs/reports/verification/cli-agent-selection|docs/reports/verification" docs crates scripts cli_manifests Makefile PLAN.md TODOS.md CHANGELOG.md
```

Expected result:

- no hits outside explicit archival or migration-context files

Focused migration search that intentionally excludes allowed historical contexts:

```sh
rg -n "docs/reports/agent-lifecycle|docs/reports/verification" . \
  --glob '!DOCS_AGENTS_MIGRATION_PLAN.md' \
  --glob '!.archived/**'
```

Expected result:

- zero hits

### Targeted Command Validation

Approval artifact dry run:

```sh
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --dry-run
```

Proving-run closeout validation:

```sh
cargo run -p xtask -- close-proving-run --approval docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml --closeout docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json
```

Maintenance refresh dry run:

```sh
cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --dry-run
```

Maintenance closeout validation:

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json
```

Registry and drift checks:

```sh
cargo run -p xtask -- check-agent-drift --agent gemini_cli
cargo run -p xtask -- check-agent-drift --agent opencode
```

### Test And Repo Gate Validation

Run the full `xtask` suite:

```sh
cargo test -p xtask
```

Run hygiene directly:

```sh
./scripts/check_repo_hygiene.sh
```

Run the repo gate:

```sh
make preflight
```

## Non-Negotiable Decisions

- `docs/agents/selection/cli-agent-selection-packet.md` is the canonical selection packet destination.
- `docs/agents/lifecycle/**` is the canonical lifecycle packet destination.
- `comparison_ref` must point only to `docs/agents/selection/cli-agent-selection-packet.md`.
- `docs/reports/agent-lifecycle/**` and `docs/reports/verification/**` are not legacy aliases; they are banned after migration.
- The moved lifecycle packet docs remain historical evidence, so preserve meaning and metadata while updating only the path contract.
