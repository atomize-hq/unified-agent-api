<!-- generated-by: xtask refresh-agent; owner: control-plane -->

# Handoff

This file is the canonical contributor execution contract for `codex` maintenance.

## Packet origin

- detected_by: `.github/workflows/agent-maintenance-release-watch.yml`
- current_validated: `0.125.0`
- target_version: `0.128.0`
- latest_stable: `0.129.0`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `openai/codex`
- dispatch_kind: `workflow_dispatch`
- dispatch_workflow: `codex-cli-update-snapshot.yml`
- branch_name: `automation/codex-maintenance-0.128.0`

## Relay contract

- request artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml`
- executor: `codex`
- prompt template path: `cli_manifests/codex/PR_BODY_TEMPLATE.md`
- prompt sha256: `319dc6cc59ee20c9fa5ceb4edee1afeb4e957ef8f7900645eb0be688405323f7`
- canonical handoff: `docs/agents/lifecycle/codex-maintenance/HANDOFF.md`
- derivative pr summary: `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
- exact closeout artifact: `docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json`
- branch linkage: `automation/codex-maintenance-0.128.0`
- manual closeout required: `true`

## Writable surfaces

- `docs/agents/lifecycle/codex-maintenance/**`
- `crates/codex/**`
- `crates/agent_api/**`
- `cli_manifests/codex/artifacts.lock.json`
- `cli_manifests/codex/snapshots/0.128.0/**`
- `cli_manifests/codex/reports/0.128.0/**`
- `cli_manifests/codex/versions/0.128.0.json`
- `cli_manifests/codex/wrapper_coverage.json`
- `cli_manifests/support_matrix/current.json`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/codex-wrapper-coverage-scenarios-v1.md`

## Read-only inputs

- `cli_manifests/codex/OPS_PLAYBOOK.md`
- `cli_manifests/codex/CI_WORKFLOWS_PLAN.md`
- `cli_manifests/codex/PR_BODY_TEMPLATE.md`
- `.github/workflows/codex-cli-update-snapshot.yml`

## Ordered repo commands

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/codex`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Exact green gates

- `cargo fmt --all`
- `cargo run -p xtask -- codex-validate --root cli_manifests/codex`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`

## Recovery

- recreate packet command: `cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --write`
- reopen pr body path: `docs/agents/lifecycle/codex-maintenance/governance/pr-summary.md`
- reopen pr branch: `automation/codex-maintenance-0.128.0`
- notes:
- If PR creation fails after packet generation, rerun packet creation and reopen the PR from the generated pr-summary path.
- If local Codex preflight fails, fix binary/auth and rerun execute-agent-maintenance --dry-run before write mode.

## Exact closeout command

```sh
cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json
```

## Exact coding-agent prompt

```md
# Codex CLI Parity PR Body Template (for `automation/codex-cli-<version>` PRs)

@codex

## Goal

Bring the Rust wrapper (`crates/codex`) into parity with upstream `codex` **rust-v0.128.0** by using the generated parity artifacts in `cli_manifests/codex/`.

This PR already contains:
- pinned upstream release assets (`cli_manifests/codex/artifacts.lock.json`)
- upstream CLI snapshots (`cli_manifests/codex/snapshots/0.128.0/**`)
- deterministic coverage reports (work queue) (`cli_manifests/codex/reports/0.128.0/**`)
- version metadata (`cli_manifests/codex/versions/0.128.0.json`)

Your job is to use those outputs to implement/waive wrapper support until the report no longer contains uncovered surfaces for the required target (and, when union is complete, for all expected targets).

## Where To Look (Source Of Truth)

- Upstream union snapshot: `cli_manifests/codex/snapshots/0.128.0/union.json`
- Coverage work queue: `cli_manifests/codex/reports/0.128.0/coverage.any.json`
- Per-target work queue(s): `cli_manifests/codex/reports/0.128.0/coverage.<target_triple>.json`
- Wrapper coverage manifest (generated): `cli_manifests/codex/wrapper_coverage.json`
- Wrapper coverage source-of-truth (edit this, not the JSON): `crates/codex/src/wrapper_coverage_manifest.rs`
- Wrapper coverage scenario catalog (normative): `docs/specs/codex-wrapper-coverage-scenarios-v1.md`
- Wrapper coverage generator contract (normative): `docs/specs/codex-wrapper-coverage-generator-contract.md`
- Validator contract: `cli_manifests/codex/VALIDATOR_SPEC.md`
- Rules + policy: `cli_manifests/codex/RULES.json`
- Agent runbook: `cli_manifests/codex/CI_AGENT_RUNBOOK.md`

Baseline (previously supported):
- Latest validated version pointer: `cli_manifests/codex/latest_validated.txt`
- Baseline union snapshot: `cli_manifests/codex/snapshots/<latest_validated>/union.json`
- Baseline report: `cli_manifests/codex/reports/<latest_validated>/coverage.any.json`

Pointer policy:
- Do not change `cli_manifests/codex/min_supported.txt` unless maintainers explicitly request a policy bump.

## What To Do (Operational Steps)

1) **Triage the delta for 0.128.0**
- Open `cli_manifests/codex/reports/0.128.0/coverage.any.json`
- Work the lists in this order:
  - `deltas.missing_commands`
  - `deltas.missing_flags`
  - `deltas.missing_args`
  - `deltas.unsupported` (if present)
  - `deltas.passthrough_candidates` (if present; “nice-to-have explicit promotions”)

2) **Classify each missing/unsupported surface**
For each missing unit (command/flag/arg), choose exactly one:
- **Implement support** in `crates/codex` and mark it `explicit` in `crates/codex/src/wrapper_coverage_manifest.rs`, or
- If only safely doable via CLI forwarding, mark it `passthrough`, or
- If we intentionally will not support it, mark it `intentionally_unsupported` **with a non-empty `note`** (required by validator).

Guardrails:
- Do **not** hand-edit `cli_manifests/codex/wrapper_coverage.json` (it is generated).
- Do **not** modify snapshots/reports by hand; re-run generators instead.

3) **Compare 0.128.0 to the current supported baseline**
- Read baseline version: `BASELINE="$(cat cli_manifests/codex/latest_validated.txt)"`
- Determine what’s new/removed at the CLI surface layer by diffing the two union snapshots:
  - New surfaces = present in `snapshots/0.128.0/union.json` but not present in `snapshots/$BASELINE/union.json`
  - Removed surfaces = present in baseline union but not present in 0.128.0 union
  - Treat “new surfaces” as high priority to assess for wrapper support.
  - Treat “removed surfaces” as potential wrapper deprecations (or leave as wrapper-only if still needed).

4) **Regenerate + validate after changes**
Run these from repo root:
- `cargo run -p xtask -- codex-wrapper-coverage --out cli_manifests/codex/wrapper_coverage.json`
- `cargo run -p xtask -- codex-report --version 0.128.0 --root cli_manifests/codex`
- `cargo run -p xtask -- codex-version-metadata --version 0.128.0 --status reported --root cli_manifests/codex`
- `cargo run -p xtask -- codex-validate --root cli_manifests/codex`

Then run wrapper tests (Linux required):
- `cargo test -p codex`
- `cargo test -p codex --examples`
- `CODEX_E2E_BINARY=./codex-x86_64-unknown-linux-musl cargo test -p codex --test cli_e2e -- --nocapture`
- Optional (requires valid auth under `CODEX_E2E_HOME`): `CODEX_E2E_LIVE=1 CODEX_E2E_BINARY=./codex-x86_64-unknown-linux-musl cargo test -p codex --test cli_e2e -- --nocapture`

## Done Criteria

- `cargo run -p xtask -- codex-validate --root cli_manifests/codex` passes.
- For the required target (`x86_64-unknown-linux-musl`), `cli_manifests/codex/reports/0.128.0/coverage.x86_64-unknown-linux-musl.json` has:
  - no missing/unknown/unsupported surfaces after regeneration, OR all remaining gaps are explicitly `intentionally_unsupported` with rationale notes.
- If `snapshots/0.128.0/union.json.complete == true`, meet the same criterion for all expected targets.

```
