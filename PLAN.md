# PLAN — Recommendation Lane For The Next CLI Agent

Status: ready for implementation  
Date: 2026-04-27  
Branch: `staging`  
Repo: `atomize-hq/unified-agent-api`

## Source Inputs

- Approved design artifact: `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-design-20260427-151419.md`
- Eng-review test artifact: `~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-eng-review-test-plan-20260427-153026.md`
- Normative contracts:
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `crates/xtask/src/approval_artifact.rs`

## Outcome

Build the missing pre-create recommendation lane for onboarding a new CLI agent.

The lane starts from maintainer seed hints, gathers dated evidence, hard-rejects ineligible candidates, compares exactly 3 eligible candidates, promotes one canonical selection packet, drafts `approved-agent.toml`, then stops for maintainer approve-or-override before the existing `cargo run -p xtask -- onboard-agent --approval ...` lane begins.

This plan does not reopen the shipped create-mode factory. It feeds it.

## Scope Lock

### In scope

- One committed repo-local skill at `.codex/skills/recommend-next-agent/SKILL.md`
- One deterministic runner at `scripts/recommend_next_agent.py`
- One committed Python test module for runner logic and golden rendering
- One committed validation path that proves the generated approval draft satisfies the existing Rust approval-artifact contract
- One canonical promotion flow into `docs/agents/selection/cli-agent-selection-packet.md`
- One approval-draft flow that ends at `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml`
- One proving run against a real candidate set

### Out of scope

- `xtask recommend-agent`
- Wrapper or `agent_api` implementation for the eventual winning agent
- Generalized cross-repo recommendation tooling
- Dynamic candidate-count configuration
- Upgrade-lane or maintenance-lane redesign

## Step 0: Scope Challenge

### What already exists

The repo already owns the post-approval create lane.

- `docs/cli-agent-onboarding-factory-operator-guide.md` is the live operator procedure.
- `docs/specs/cli-agent-onboarding-charter.md` defines the onboarding gates.
- `docs/agents/selection/cli-agent-selection-packet.md` is the canonical comparison packet path.
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md` already fixes the packet shape around exactly 3 candidates.
- `crates/xtask/src/approval_artifact.rs` already defines the approval-artifact contract and hard-codes `comparison_ref = "docs/agents/selection/cli-agent-selection-packet.md"`.
- `crates/xtask/src/onboard_agent.rs` and `crates/xtask/src/onboard_agent/approval.rs` already own the post-approval control-plane enrollment.
- `crates/xtask/src/wrapper_scaffold.rs` already owns the runtime shell step after onboarding.

### Minimum diff that achieves the goal

The smallest complete version is:

1. Add one repo-local skill.
2. Add one script-first deterministic runner.
3. Reuse the existing canonical packet path and approval-artifact schema.
4. Validate the generated approval draft by running the existing `xtask` approval flow in dry-run mode.

Anything bigger is premature.

### Complexity call

This plan intentionally stays below a new-control-plane threshold.

- New primary implementation surfaces: `.codex/skills/`, `scripts/`, and a narrow set of docs/test touchpoints
- No new crate
- No new `xtask` command
- No `agent_api` churn

That keeps the blast radius small enough to land as one focused slice.

## Locked Decisions

| Decision | Why it is locked |
| --- | --- |
| V1 compares exactly 3 candidates | The canonical packet template already assumes exactly 3 rows. |
| Eligibility gating happens before scoring | Ranking infeasible candidates creates fake confidence and wasted maintainer review. |
| Run-local artifacts and canonical promotion are separate steps | `approved-agent.toml` points at one canonical packet path, so promotion must be explicit. |
| The runner is script-first, not `xtask`-first | External evidence collection will change faster than the stable control-plane contract. |
| The lane must emit an approval draft, not just a packet | The create-mode input is `approved-agent.toml`, not a narrative summary. |
| Maintainer override is first-class | Override is part of the product, not an error condition. |
| The skill is the orchestration surface and the script is the deterministic engine | This preserves discoverability while keeping evidence capture replayable. |

## Architecture

```text
maintainer seed / optional shortlist hints
                |
                v
  .codex/skills/recommend-next-agent/SKILL.md
                |
                v
     scripts/recommend_next_agent.py
                |
                +------------------------------+
                |                              |
                v                              v
       discovery candidate pool         dated raw source capture
                |                              |
                +--------------+---------------+
                               |
                               v
                      hard eligibility gate
                               |
                  +------------+------------+
                  |                         |
                  v                         v
        rejected candidates log      eligible candidates
                                            |
                                            v
                                 fixed exactly-3 evaluation
                                            |
                                            v
                            run-local dossiers + scorecard + packet
                                            |
                                  explicit promotion step
                                            |
                    +-----------------------+----------------------+
                    |                                              |
                    v                                              v
  docs/agents/selection/cli-agent-selection-packet.md   approval-draft.generated.toml
                                                                    |
                                                                    v
                                       docs/agents/lifecycle/<pack>/governance/approved-agent.toml
                                                                    |
                                                                    v
                                         maintainer approve or override
                                                                    |
                                                                    v
                         cargo run -p xtask -- onboard-agent --approval ...
```

## Artifact Contract

### Committed source surfaces

| Path | Owner | Purpose |
| --- | --- | --- |
| `.codex/skills/recommend-next-agent/SKILL.md` | repo | operator workflow and maintainer decision framing |
| `scripts/recommend_next_agent.py` | repo | deterministic evidence capture, gating, scoring, rendering, promotion |
| `scripts/test_recommend_next_agent.py` | repo | unit and golden coverage for runner behavior |
| `docs/agents/selection/candidate-seed.toml` | repo | exact candidate pool plus descriptor defaults and per-candidate overrides |
| `docs/cli-agent-onboarding-factory-operator-guide.md` | repo | operator-procedure update for the new pre-create lane |

### Scratch outputs

Root: `~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id>/`

| Path | Purpose |
| --- | --- |
| `candidate-pool.json` | full discovered pool, including rejected candidates and rejection reasons |
| `eligible-candidates.json` | candidates that survive hard gating |
| `candidate-dossiers/<agent_id>.json` | normalized per-candidate evidence |
| `scorecard.json` | fixed-dimension scores for the exactly-3 comparison set |
| `sources.lock.json` | dated source provenance and fetch metadata |
| `comparison.generated.md` | run-local comparison packet render |
| `approval-draft.generated.toml` | run-local approval artifact draft before canonical promotion |
| `run-summary.md` | human-readable summary of what happened and why |

### Committed review outputs

Root: `docs/agents/selection/runs/<run_id>/`

This root exists only for the promoted run that supports the merged recommendation.

- copy of `candidate-pool.json`
- copy of `eligible-candidates.json`
- copy of `candidate-dossiers/**`
- copy of `scorecard.json`
- copy of `sources.lock.json`
- copy of `comparison.generated.md`
- copy of `approval-draft.generated.toml`
- copy of `run-summary.md`

### Canonical promoted outputs

| Path | Purpose |
| --- | --- |
| `docs/agents/selection/cli-agent-selection-packet.md` | one canonical comparison packet referenced by approval artifacts |
| `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml` | maintainer-approved create-mode input |

### Commit policy

The commit policy is strict.

1. Scratch runs under `~/.gstack/projects/**` are never committed.
2. `docs/agents/selection/runs/<run_id>/` is committed only for the one promoted run that backs the current recommendation.
3. Unpromoted or superseded repo-local run directories must be deleted before merge.
4. A PR for this slice must contain exactly one canonical packet update and exactly one matching committed review run directory.
5. The committed review run directory, canonical packet update, and approval draft must all describe the same `recommended_agent_id`.

## Runner CLI Contract

The runner uses two subcommands. No hidden modes.

### Generate

```sh
python3 scripts/recommend_next_agent.py generate \
  --seed-file docs/agents/selection/candidate-seed.toml \
  --run-id <timestamp>-<shortlist_slug> \
  --scratch-root ~/.gstack/projects/<repo-slug>/recommend-next-agent-runs
```

Behavior:

- reads one seed file
- performs discovery and evidence capture
- applies the hard eligibility gate
- scores only eligible candidates
- selects exactly 3 candidates
- writes a complete scratch run under `~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id>/`
- does not mutate any repo-tracked file

Exit rules:

- exit `0` only when exactly 3 eligible candidates were selected and all scratch artifacts were written
- non-zero if fewer than 3 eligible candidates remain after gating
- non-zero if any required source capture or normalization step fails

### Promote

```sh
python3 scripts/recommend_next_agent.py promote \
  --run-dir ~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id> \
  --repo-run-root docs/agents/selection/runs \
  --approved-agent-id <agent_id> \
  --onboarding-pack-prefix <kebab-case-pack-prefix> \
  [--override-reason "<required when approved agent differs from recommended>"]
```

Behavior:

- reads one previously generated scratch run
- copies that run into `docs/agents/selection/runs/<run_id>/`
- promotes `comparison.generated.md` into `docs/agents/selection/cli-agent-selection-packet.md`
- writes `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml`
- validates the generated approval artifact by running `cargo run -p xtask -- onboard-agent --approval <path> --dry-run`

Promotion guards:

- fail if `docs/agents/selection/runs/<run_id>/` already exists
- fail if the scratch run is missing any required artifact
- fail if `approved_agent_id` is not one of the 3 shortlisted candidates
- fail if `approved_agent_id != recommended_agent_id` and `--override-reason` is absent
- fail if the approval dry-run validation fails

Write ordering:

1. copy scratch run into repo review root
2. write approval artifact to a temp path
3. validate approval artifact with `xtask --dry-run`
4. replace canonical packet and final approval artifact atomically via rename

This keeps canonical surfaces unchanged when validation fails.

### Seed file contract

Path:

- `docs/agents/selection/candidate-seed.toml`

The seed file is required. It provides the exact candidate pool plus descriptor defaults that research cannot infer safely.

Required top-level tables:

- `[defaults.descriptor]`
- `[candidate.<agent_id>]` for each seed candidate

Required `[defaults.descriptor]` keys:

- `canonical_targets = ["darwin-arm64"]`
- `wrapper_coverage_binding_kind = "generated_from_wrapper_crate"`
- `always_on_capabilities = ["agent_api.config.model.v1", "agent_api.events", "agent_api.events.live", "agent_api.run"]`
- `target_gated_capabilities = []`
- `config_gated_capabilities = []`
- `backend_extensions = []`
- `support_matrix_enabled = true`
- `capability_matrix_enabled = true`
- `capability_matrix_target = ""`
- `docs_release_track = "crates-io"`

Required `[candidate.<agent_id>]` keys:

- `display_name`
- `research_urls`
- `install_channels`
- `auth_notes`

Allowed optional `[candidate.<agent_id>]` override keys:

- `crate_path`
- `backend_module`
- `manifest_root`
- `package_name`
- `canonical_targets`
- `wrapper_coverage_binding_kind`
- `wrapper_coverage_source_path`
- `always_on_capabilities`
- `target_gated_capabilities`
- `config_gated_capabilities`
- `backend_extensions`
- `support_matrix_enabled`
- `capability_matrix_enabled`
- `capability_matrix_target`
- `docs_release_track`
- `onboarding_pack_prefix`

Derived defaults when omitted:

- `crate_path = "crates/<agent_id>"`
- `backend_module = "crates/agent_api/src/backends/<agent_id>"`
- `manifest_root = "cli_manifests/<agent_id>"`
- `package_name = "unified-agent-api-<agent_id with underscores replaced by hyphens>"`
- `onboarding_pack_prefix = "<agent_id with underscores replaced by hyphens>-onboarding"`

## Eligibility Gate

Every candidate must pass these checks before it can enter the 3-row comparison:

1. It exposes a plausible deterministic non-interactive CLI surface.
2. It has a credible offline parser, fixture, or fake-binary strategy.
3. It can fit the repo's redaction posture without raw backend leakage.
4. It supports a crate-first onboarding path without forcing `agent_api` design churn up front.
5. Its external evidence is inspectable and reproducible enough for future maintainers.

Candidates that fail are recorded in `candidate-pool.json` with rejection reasons. They never appear in the final comparison table.

## Scoring Contract

Only eligible candidates are scored.

### Fixed dimensions

Primary dimensions:

- `Adoption & community pull`
- `CLI product maturity & release activity`
- `Installability & docs quality`
- `Reproducibility & access friction`

Secondary dimensions:

- `Architecture fit for this repo`
- `Capability expansion / future leverage`

### Score buckets

- `0` = weak, missing, or materially blocked
- `1` = partial, with notable caveats
- `2` = solid, usable with caveats
- `3` = strong, clearly favorable

### Deterministic shortlist algorithm

If more than 3 candidates survive eligibility gating, sort eligible candidates by this exact tuple:

1. primary-dimension sum, descending
2. `Architecture fit for this repo`, descending
3. `Reproducibility & access friction`, descending
4. secondary-dimension sum, descending
5. `CLI product maturity & release activity`, descending
6. `Adoption & community pull`, descending
7. `agent_id`, ascending lexical

Take the first 3 after sorting.

### Recommendation algorithm

The `recommended_agent_id` is candidate rank 1 from the deterministic shortlist algorithm above.

The published packet must not show a weighted total column, but the runner may store per-candidate aggregate numbers in `scorecard.json` for deterministic ordering and replay.

### Tie policy

If two candidates tie across all ordering fields above, lexical `agent_id` order wins. No human tie-break at generation time.

## Workstreams

### Workstream 1: Lock the contract surfaces

Modules touched:

- `.codex/skills/`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/agents/selection/`

Deliverables:

- Skill contract committed at `.codex/skills/recommend-next-agent/SKILL.md`
- Seed-file contract committed at `docs/agents/selection/candidate-seed.toml`
- Operator guide updated to describe the pre-create recommendation lane
- Artifact-root rules documented, including run-local root versus canonical promoted outputs

Acceptance:

- A maintainer can read one committed skill and understand when to run it, what artifacts it produces, and where approve-or-override happens.

### Workstream 2: Implement deterministic runner core

Modules touched:

- `scripts/`

Deliverables:

- `scripts/recommend_next_agent.py`
- explicit typed data model for candidate pool, dossier, scorecard, and source lock
- deterministic ordering and stable serialization
- hard eligibility gate
- fixed exactly-3 shortlist selection

Implementation notes:

- Keep the runner explicit and boring. Plain Python, standard library where possible.
- Use one normalization path for all candidates. No candidate-specific ad hoc formatting branches unless proven necessary.
- Fail closed when required evidence is missing.

Acceptance:

- Re-running the same inputs against the same source snapshots produces byte-stable JSON and markdown outputs.

### Workstream 3: Render, promote, and draft approval artifacts

Modules touched:

- `scripts/`
- `docs/agents/selection/`
- `docs/agents/lifecycle/`

Deliverables:

- canonical packet renderer that preserves the 3-candidate contract
- explicit promotion step from `comparison.generated.md` to `docs/agents/selection/cli-agent-selection-packet.md`
- approval-draft renderer that emits a valid `approved-agent.toml` shape
- override path that requires `override_reason` when `approved_agent_id != recommended_agent_id`

Implementation notes:

- The approval draft must reuse the real repo rules from `crates/xtask/src/approval_artifact.rs`.
- Do not duplicate the Rust schema by hand in multiple places if one shared output builder can keep fields aligned.

Acceptance:

- `cargo run -p xtask -- onboard-agent --approval <generated approval path> --dry-run` succeeds on the proving-run output.

### Approval artifact field mapping

The generated `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml` must map fields exactly as follows:

| Field | Source |
| --- | --- |
| `artifact_version` | literal `"1"` |
| `comparison_ref` | literal `"docs/agents/selection/cli-agent-selection-packet.md"` |
| `selection_mode` | literal `"factory_validation"` for v1 |
| `recommended_agent_id` | rank-1 candidate from the deterministic shortlist |
| `approved_agent_id` | `--approved-agent-id` from `promote` |
| `approval_commit` | `git rev-parse HEAD` at promotion time |
| `approval_recorded_at` | UTC RFC3339 timestamp at promotion time |
| `override_reason` | required `--override-reason` when `approved_agent_id != recommended_agent_id`; otherwise omitted |
| `descriptor.agent_id` | `approved_agent_id` |
| `descriptor.display_name` | `[candidate.<approved_agent_id>].display_name` from seed file |
| `descriptor.crate_path` | `[candidate.<approved_agent_id>].crate_path` if present, else default derived path |
| `descriptor.backend_module` | `[candidate.<approved_agent_id>].backend_module` if present, else default derived path |
| `descriptor.manifest_root` | `[candidate.<approved_agent_id>].manifest_root` if present, else default derived path |
| `descriptor.package_name` | `[candidate.<approved_agent_id>].package_name` if present, else default derived package name |
| `descriptor.canonical_targets` | `[candidate.<approved_agent_id>].canonical_targets` if present, else `[defaults.descriptor].canonical_targets` |
| `descriptor.wrapper_coverage_binding_kind` | candidate override if present, else `[defaults.descriptor].wrapper_coverage_binding_kind` |
| `descriptor.wrapper_coverage_source_path` | `[candidate.<approved_agent_id>].wrapper_coverage_source_path` if present, else `descriptor.crate_path` |
| `descriptor.always_on_capabilities` | candidate override if present, else `[defaults.descriptor].always_on_capabilities` |
| `descriptor.target_gated_capabilities` | candidate override if present, else `[defaults.descriptor].target_gated_capabilities` |
| `descriptor.config_gated_capabilities` | candidate override if present, else `[defaults.descriptor].config_gated_capabilities` |
| `descriptor.backend_extensions` | candidate override if present, else `[defaults.descriptor].backend_extensions` |
| `descriptor.support_matrix_enabled` | candidate override if present, else `[defaults.descriptor].support_matrix_enabled` |
| `descriptor.capability_matrix_enabled` | candidate override if present, else `[defaults.descriptor].capability_matrix_enabled` |
| `descriptor.capability_matrix_target` | candidate override if present; else omit when empty-string default is supplied |
| `descriptor.docs_release_track` | candidate override if present, else `[defaults.descriptor].docs_release_track` |
| `descriptor.onboarding_pack_prefix` | `--onboarding-pack-prefix` from `promote` |

Validation rules:

- `descriptor.agent_id` must equal `approved_agent_id`
- `descriptor.onboarding_pack_prefix` must equal the `<onboarding_pack_prefix>` path segment in the output path
- `comparison_ref` must point to the canonical packet path and that file must exist
- when `approved_agent_id != recommended_agent_id`, `override_reason` is required and must be non-empty

### Workstream 4: Add proving-run and validation coverage

Modules touched:

- `scripts/`
- `crates/xtask/tests/`
- `docs/agents/selection/`

Deliverables:

- Python unit and golden tests for gating, scoring, ordering, rendering, and promotion guards
- Rust-side validation or integration coverage that proves generated approval drafts satisfy the real approval contract
- one real proving run committed as reviewable evidence

Acceptance:

- The proving run leaves one timestamped run directory, one canonical packet update, and one valid approval draft.

## Code Quality Guardrails

- Keep discovery and evaluation separate. Discovery may inspect a wider seed set. Evaluation is the narrow exactly-3 path.
- Do not split the scoring rubric across the skill and the runner. The skill orchestrates. The runner computes.
- Keep output formats textual and diff-friendly. No binary captures.
- Keep the first version script-first. If the proving run reveals stable repetition, then consider `xtask recommend-agent`.
- Reuse existing packet/docs contracts instead of creating a second comparison or approval shape.

## Performance And Reliability

- Candidate count is intentionally small. Latency is dominated by external evidence capture, not local computation.
- Prefer bounded or serial fetches over complex async orchestration. This lane is review-path tooling, not a throughput system.
- Persist `sources.lock.json` so the run can explain why repeated executions differ after upstream drift.
- Time out evidence collection per source and fail closed with a clear rejection or incomplete-run error.
- Promotion must be atomic enough that the repo never claims a canonical packet refresh when only run-local artifacts were written.

## Test Strategy

### Primary commands

- `python3 scripts/recommend_next_agent.py generate --seed-file docs/agents/selection/candidate-seed.toml --run-id <timestamp>-<shortlist_slug> --scratch-root ~/.gstack/projects/<repo-slug>/recommend-next-agent-runs`
- `python3 scripts/recommend_next_agent.py promote --run-dir ~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id> --repo-run-root docs/agents/selection/runs --approved-agent-id <agent_id> --onboarding-pack-prefix <pack-prefix> [--override-reason "..."]`
- `python3 -m unittest scripts.test_recommend_next_agent`
- targeted Rust validation coverage for generated approval artifacts
- `cargo run -p xtask -- onboard-agent --approval <generated approval path> --dry-run`
- `make preflight` before merge

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] scripts/recommend_next_agent.py
    │
    ├── parse_seed_input()
    │   └── unit: stable defaults, explicit shortlist hints, bad input rejection
    │
    ├── capture_sources()
    │   └── integration: dated source capture + reproducible sources.lock.json
    │
    ├── apply_eligibility_gate()
    │   └── unit: reject missing non-interactive, fixture, or evidence story
    │
    ├── select_exactly_three()
    │   └── unit: deterministic ordering, hard fail on 2-or-4 candidate output
    │
    ├── render_candidate_dossiers()
    │   └── unit/golden: stable dossier JSON shape
    │
    ├── render_comparison_packet()
    │   └── golden: markdown matches canonical 3-row packet contract
    │
    ├── promote_canonical_packet()
    │   └── integration: writes docs/agents/selection/cli-agent-selection-packet.md explicitly
    │
    └── render_approval_draft()
        └── integration: generated approved-agent.toml passes xtask dry-run validation
```

### Maintainer-flow coverage

```text
USER FLOW COVERAGE
===========================
[+] Clean recommendation run
    ├── seed -> discovery -> gate -> shortlist -> score -> packet -> approval draft
    └── integration: one end-to-end dry run with durable artifacts

[+] Candidate rejection path
    ├── ineligible candidate recorded with rejection reason
    └── unit/integration: rejection reason survives into candidate-pool.json and run-summary.md

[+] Canonical promotion path
    ├── run-local artifacts exist
    └── integration: canonical packet refresh is explicit, reproducible, and never implicit

[+] Maintainer override path
    ├── approved candidate differs from recommended candidate
    └── integration: override_reason required and copied into approval artifact
```

### Required test additions

- `scripts/test_recommend_next_agent.py`
  - seed parsing defaults
  - hard gate rejection reasons
  - deterministic exactly-3 ordering
  - golden render for `comparison.generated.md`
  - promotion guard when canonical target is missing or stale
  - override draft generation requiring `override_reason`
- Rust validation coverage in `crates/xtask/tests/`
  - generated approval draft accepted by the existing approval loader
  - generated override artifact rejected when `override_reason` is absent
  - generated comparison ref must match `docs/agents/selection/cli-agent-selection-packet.md`
- proving-run validation
  - one committed end-to-end run against real candidate inputs
  - `cargo run -p xtask -- onboard-agent --approval ... --dry-run` passes

### QA handoff artifact

Primary QA input for this slice already exists at:

`~/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-eng-review-test-plan-20260427-153026.md`

The implementation should keep that artifact in sync if the execution flow changes materially.

## Failure Modes Registry

| Codepath | Likely failure | Required test | Required handling | Visible outcome |
| --- | --- | --- | --- | --- |
| Discovery | upstream page or API shape drift | integration fixture for source parsing | fail closed with source citation | clear "source changed" error |
| Eligibility gate | ineligible candidate reaches scoring | unit per rejection reason | reject before scoring | clear rejection summary |
| Exactly-3 selection | runner emits 2 or 4 comparison rows | unit on count invariant | hard fail before render | clear invariant failure |
| Dossier normalization | per-candidate JSON shapes drift | golden tests | fail before packet render | clear normalization failure |
| Packet rendering | packet shape drifts from canonical template | golden tests | block promotion | clear render-drift failure |
| Canonical promotion | run-local packet exists but canonical packet stays stale | integration on target-path write | fail before approval draft finalization | clear stale-canonical failure |
| Approval draft | descriptor fields wrong or incomplete | integration through real approval loader | block handoff | clear validation error |
| Override path | approved candidate differs without rationale | integration on override branch | block artifact finalization | clear override-contract error |

Critical gap rule:

Any path that can silently produce a stale canonical packet or an invalid approval artifact is a release blocker.

## Acceptance Gates

The slice is complete only when all of the following are true:

1. The skill exists at `.codex/skills/recommend-next-agent/SKILL.md`.
2. The seed file exists at `docs/agents/selection/candidate-seed.toml`.
3. The runner exists at `scripts/recommend_next_agent.py`.
4. `generate` writes the full scratch artifact set under `~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id>/`.
5. `promote` writes exactly one committed review run under `docs/agents/selection/runs/<run_id>/`.
6. The final comparison packet contains exactly 3 candidates.
7. Rejected candidates are preserved with rejection reasons.
8. The canonical packet update is explicit and reproducible.
9. The generated approval artifact passes the real `xtask` approval dry run.
10. The override path is supported and requires `override_reason`.
11. Python unit/golden tests pass.
12. Targeted Rust validation coverage passes.
13. `make preflight` passes before merge.

## NOT In Scope

- `xtask recommend-agent`
  The contract needs one proving run before it deserves a new Rust command surface.
- Runtime onboarding for the selected agent
  This slice ends before wrapper/backend implementation begins.
- Generalized recommendation-service work
  The problem is repo-local approvability, not global discovery infrastructure.
- Candidate-count configurability
  V1 stays fixed at 3 to match the canonical packet shape.
- Maintenance or upgrade-lane automation
  That is a later lifecycle concern.

## Worktree Parallelization Strategy

This plan has real parallelization value once the artifact contract is fixed.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Lock skill + operator contract | `.codex/skills/`, `docs/cli-agent-onboarding-factory-operator-guide.md`, `docs/agents/selection/` | — |
| Build runner core | `scripts/` | contract lock |
| Add packet promotion + approval drafting | `scripts/`, `docs/agents/selection/`, `docs/agents/lifecycle/` | runner core |
| Add proving-run validation | `scripts/`, `crates/xtask/tests/`, `docs/agents/selection/` | runner core + promotion/drafting |

### Parallel lanes

- Lane A: lock skill contract and operator-doc updates
- Lane B: runner core and normalization schema
- Lane C: promotion and approval drafting, after Lane B
- Lane D: proving-run validation, after Lanes B and C

### Execution order

1. Launch Lane A and Lane B in parallel worktrees.
2. Merge Lane B first because it establishes the real runner contract.
3. Run Lane C next.
4. Run Lane D last because it depends on the real runner outputs and the real promoted packet path.

### Conflict flags

- Lanes A and C both touch docs under `docs/agents/selection/`, so do not run them in parallel.
- Lanes B and C both touch `scripts/`, so they must stay sequential.
- Lane D will read real outputs from B and C, so it should not start until both are merged.

## Definition Of Done

- Maintainer can run one repo-local workflow and get:
  - one timestamped run directory with durable evidence
  - one exactly-3 comparison packet
  - one canonical packet promotion step
  - one valid `approved-agent.toml` draft
  - one explicit approve-or-override decision point
- The existing `onboard-agent` lane starts unchanged after approval.
- No new control-plane abstraction was added prematurely.
