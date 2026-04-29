# PLAN - UAA-0022 Runtime Follow-On Codex Runner

Status: ready for implementation
Date: 2026-04-29
Branch: `codex/recommend-next-agent`
Base branch: `main`
Repo: `atomize-hq/unified-agent-api`
Work item: `uaa-0022`

## Source Inputs

- Design artifact:
  - `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-design-20260429-131949.md`
- CEO plan:
  - `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/ceo-plans/2026-04-29-runtime-follow-on-codex-lane.md`
- Eng review test-plan artifact:
  - `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-codex-recommend-next-agent-eng-review-test-plan-20260429-135452.md`
- Canonical repo surfaces:
  - `docs/backlog/uaa-0022-runtime-follow-on-codex-runner.md`
  - `docs/backlog.json`
  - `docs/cli-agent-onboarding-factory-operator-guide.md`
  - `docs/specs/cli-agent-onboarding-charter.md`
  - `docs/adr/0013-agent-api-backend-harness.md`
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/onboard_agent/preview/render.rs`
  - `crates/xtask/data/agent_registry.toml`
  - `crates/codex/src/wrapper_coverage_manifest.rs`
  - `crates/claude_code/src/wrapper_coverage_manifest.rs`
  - `crates/agent_api/src/backends/opencode/**`
  - `crates/opencode/**`
  - `crates/agent_api/src/backends/gemini_cli/**`
  - `crates/gemini_cli/**`

## Outcome

Land one repo-owned runtime lane after `onboard-agent --write` and `scaffold-wrapper-crate --write`.

The lane must start from approval and registry truth, invoke Codex through a bounded repo command, write only runtime-owned surfaces, emit reviewable artifacts, and hand off cleanly into the later publication-refresh and proving-run-closeout lane.

This does not reopen the recommendation lane. It does not merge runtime implementation into `onboard-agent`. It does not silently absorb publication refresh into the same milestone.

## Problem Statement

The front half of onboarding is now boring in the good way.

Recommendation can end in a valid `approved-agent.toml`. `onboard-agent` can enroll the control plane. `scaffold-wrapper-crate` can create the wrapper shell at the registry-owned `crate_path`.

The remaining gap is the middle seam. A maintainer still has to translate repo patterns into real wrapper code, real `agent_api` backend code, real wrapper-coverage source updates, real `agent_api` tests, and real manifest evidence. Today that step depends on memory, taste, and archaeology across existing agents.

That is the bottleneck this plan removes.

## Scope Lock

### In scope

- Add one new repo-owned `xtask` subcommand for the runtime follow-on lane.
- Make that command the only normative host surface for the new Codex-driven runtime runner.
- Default the lane to `opencode`-level support.
- Allow `gemini_cli`-level support only as an explicit `minimal` exception with required justification.
- Allow richer patterns from `codex` and `claude_code` only through explicit opt-in fields.
- Define the exact machine-owned inputs, scratch artifacts, write allowlist, forbidden writes, validation checks, and handoff artifact.
- Require `agent_api` backend tests as part of the default-tier runtime lane.
- Keep the operator guide as the canonical shipped procedure until one real proving run validates this lane.

### Out of scope

- Folding runtime generation into `onboard-agent`.
- Making generated onboarding packet markdown an executable source of truth.
- Automating publication refresh in the same command.
- Automating proving-run closeout in the same command.
- Redefining capability promotion policy or the onboarding charter.
- Adding a new artifact-publishing pipeline or a new end-user binary outside `xtask`.
- Broad portfolio policy for which agents should or should not be onboarded.

## Step 0 Scope Challenge

### What already exists

| Sub-problem | Existing surface to reuse | Reuse decision |
| --- | --- | --- |
| Control-plane enrollment | `cargo run -p xtask -- onboard-agent --approval ... --write` | Reuse unchanged. Do not collapse runtime generation into it. |
| Wrapper shell creation | `cargo run -p xtask -- scaffold-wrapper-crate --agent <id> --write` | Reuse unchanged. It remains the hard boundary before runtime work starts. |
| Approval truth and path ownership | `crates/xtask/src/approval_artifact.rs`, `crates/xtask/data/agent_registry.toml`, `crates/xtask/src/agent_registry.rs` | Reuse as the machine-owned truth for crate path, backend path, manifest root, and wrapper-coverage source path. |
| Runtime handoff wording | `crates/xtask/src/onboard_agent/preview/render.rs` | Reuse as reviewer evidence only. Do not promote generated markdown into executable authority. |
| Default implementation baseline | `crates/agent_api/src/backends/opencode/**`, `crates/opencode/**` | Reuse as the default runtime template. |
| Minimal exception baseline | `crates/agent_api/src/backends/gemini_cli/**`, `crates/gemini_cli/**` | Reuse only when `minimal` is explicitly requested and justified. |
| Feature-rich references | `crates/agent_api/src/backends/codex/**`, `crates/codex/**`, `crates/agent_api/src/backends/claude_code/**`, `crates/claude_code/**` | Reuse only for explicit richer-surface opt-ins. |
| Wrapper coverage source-of-truth pattern | `crates/codex/src/wrapper_coverage_manifest.rs`, `crates/claude_code/src/wrapper_coverage_manifest.rs` | Reuse directly. Runtime lane edits source-of-truth Rust, never generated `wrapper_coverage.json`. |
| Path-jail and safe mutation patterns | `crates/xtask/src/workspace_mutation.rs` | Reuse the same discipline for runner-managed writes and validations. |

### Minimum change set

Keep the control-plane boundary intact and add the smallest complete runtime lane:

1. Add a new `xtask` subcommand:
   - `cargo run -p xtask -- runtime-follow-on --approval <path> --dry-run`
   - `cargo run -p xtask -- runtime-follow-on --approval <path> --write`
2. Add a runtime prompt payload that the command bakes in or explicitly loads from a repo-owned path.
3. Add deterministic scratch artifacts under one `.uaa-temp` root.
4. Add exact boundary validation for allowed writes and required outputs.
5. Add runner contract tests plus default-tier `agent_api` onboarding test requirements.
6. Add a thin repo-local skill wrapper that invokes the command but does not own the contract.

No new crate. No new service. No new external workflow runner.

### Complexity check

This will touch more than 8 files and more than 2 modules. That is acceptable here.

This milestone spans:
- `xtask` command wiring
- prompt payload ownership
- backend/template selection logic
- runtime-owned manifest evidence rules
- `agent_api` onboarding test requirements
- one proving-run target

Trying to fake a smaller seam would only hide the real work and create plan debt.

### TODOS cross-reference

`TODOS.md` already captures the publication-refresh follow-on after this runtime lane.

This plan does not add a new strategic TODO. It closes the current runtime-runner planning gap and explicitly hands off to the already-captured publication-refresh follow-on.

### Completeness check

The shortcut version would be:
- tell Codex to "copy opencode"
- let it write anywhere in the repo
- trust a prose summary
- clean up by hand afterward

That saves almost no time with Codex and guarantees weak reviewability.

The complete version is still a boilable lake:
- one command
- one machine-owned input contract
- one write allowlist
- one explicit tier policy
- one structured output contract
- one proving target

### Distribution check

No new end-user distribution artifact is introduced.

This milestone ships an internal repo workflow surface through `xtask`. The only required "distribution" work is:
- add the command to `xtask`
- document it in the repo
- prove it on one real onboarding target

## Decisions Locked

### 1. Host surface

The normative host surface is a repo-owned `xtask` command that invokes `codex exec`.

Exact command family:

```sh
cargo run -p xtask -- runtime-follow-on --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --dry-run
cargo run -p xtask -- runtime-follow-on --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --write
```

The thin skill wrapper exists for ergonomics only. The command owns the contract.

### 2. Machine truth precedence

Executable truth comes from:

1. `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`
2. `crates/xtask/data/agent_registry.toml`
3. the scaffolded wrapper crate root under the registry-owned `crate_path`

Generated packet docs under `docs/agents/lifecycle/<pack>/**` remain reviewer evidence only.

The runner must reject the approval artifact before any write if these fields do not exactly match the registry entry for the same `agent_id`:

- `agent_id`
- `crate_path`
- `backend_module`
- `manifest_root`
- `wrapper_coverage_binding_kind`
- `wrapper_coverage_source_path`
- `onboarding_pack_prefix`

The runner may surface `display_name`, `package_name`, and capability declarations in summaries, but path ownership and write allowlist decisions come only from the exact-match fields above.

### 3. Tier policy

- `default`: `opencode`-level baseline, required unless explicitly overridden.
- `minimal`: `gemini_cli`-level exception path, allowed only when the runner receives explicit justification and follow-up work to reach `default`.
- `feature-rich`: additive opt-in path that may borrow selected patterns from `codex` and `claude_code`.

### 4. Write boundary

Allowed write targets for `--write`:

- `crates/<agent_id>/**`
- `crates/agent_api/src/backends/<agent_id>/**`
- `crates/agent_api/tests/**` for target-specific onboarding tests
- the registry-owned wrapper coverage source path resolved from `agent_registry.toml`
- `cli_manifests/<agent_id>/snapshots/**`
- `cli_manifests/<agent_id>/supplement/**`
- scratch artifacts under `docs/agents/.uaa-temp/runtime-follow-on/runs/<run_id>/`

Everything else is read-only for this milestone.

### 5. Manifest split

Runtime-owned manifest evidence is in scope. Publication-owned manifest state is not.

Runtime-owned examples:
- `cli_manifests/<agent_id>/snapshots/**`
- `cli_manifests/<agent_id>/supplement/**`

This milestone does not introduce any new runner-owned manifest subtree beyond `snapshots/**` and `supplement/**`.

Publication-owned and therefore forbidden in this milestone:
- `cli_manifests/<agent_id>/current.json`
- `cli_manifests/<agent_id>/latest_validated.txt`
- `cli_manifests/<agent_id>/min_supported.txt`
- `cli_manifests/<agent_id>/pointers/**`
- `cli_manifests/<agent_id>/reports/**`
- `cli_manifests/<agent_id>/versions/**`
- `cli_manifests/<agent_id>/wrapper_coverage.json`
- `cli_manifests/support_matrix/**`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/unified-agent-api/capability-matrix.md`

### 6. Dry-run and write semantics

- `--dry-run` resolves machine truth, validates input consistency, materializes scratch packet artifacts, prints the exact planned write boundary, and does not invoke `codex exec`.
- `--write` does everything from `--dry-run`, invokes `codex exec`, validates produced outputs, records scratch artifacts, and exits non-zero if the run violates boundary or output contract rules.

No automatic repo rollback is attempted after a failed `--write`. The runner preserves the failure summary and offending-path list for review.

### 7. Handoff contract

The runtime lane is complete only when it emits a machine-readable handoff artifact that tells the next lane:
- what runtime evidence now exists
- what publication-refresh commands remain
- whether the lane is ready for publication refresh
- what blockers remain if it is not ready

## Architecture

### End-to-end flow

```text
approved-agent.toml + agent_registry.toml
                │
                ├──────────────▶ path / tier / capability truth
                │
                ▼
      runtime-follow-on input assembly
                │
                ├──────────────▶ operator guide + charter + ADR-0013
                │
                ├──────────────▶ baked-in runtime prompt payload
                │
                ▼
      xtask runtime-follow-on --write
                │
                ▼
            codex exec
                │
     ┌──────────┼──────────┬──────────────┬────────────────────┐
     │          │          │              │                    │
     ▼          ▼          ▼              ▼                    ▼
crates/<id>  crates/agent_api/   crates/agent_api/   wrapper coverage   cli_manifests/<id>/
runtime      src/backends/<id>/  tests/**            source path        runtime-owned evidence
code         backend code        target onboarding   Rust source        only
                                 tests
     │
     ▼
run-status.json + run-summary.md + handoff.json + validation-report.json
     │
     ▼
publication refresh / validation / closeout
(next milestone, separate owner seam)
```

### Input contract

Required reads:

- `docs/agents/lifecycle/<pack>/governance/approved-agent.toml`
- `crates/xtask/data/agent_registry.toml`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/adr/0013-agent-api-backend-harness.md`
- `crates/xtask/src/onboard_agent/preview/render.rs`
- `crates/xtask/templates/runtime_follow_on_codex_prompt.md`
- `crates/agent_api/src/backends/opencode/**`
- `crates/opencode/**`

Conditional reads:

- `crates/agent_api/src/backends/gemini_cli/**`
- `crates/gemini_cli/**`
- `crates/agent_api/src/backends/codex/**`
- `crates/codex/**`
- `crates/agent_api/src/backends/claude_code/**`
- `crates/claude_code/**`
- wrapper coverage source files in existing wrapper crates when richer coverage declarations are needed

Command inputs:

- `--approval <repo-relative-path>` required
- `--requested-tier default|minimal|feature-rich` optional, default `default`
- `--minimal-justification-file <repo-relative-path>` required when `--requested-tier minimal`
- `--allow-rich-surface <surface>` repeatable, optional
- `--run-id <id>` optional, default timestamped
- exactly one of `--dry-run` or `--write`

`--minimal-justification-file` must point to:

- `docs/agents/lifecycle/<pack>/governance/minimal-tier-justification.json`

Required JSON fields:

```json
{
  "schema_version": 1,
  "agent_id": "example_agent",
  "reason_default_is_not_viable_yet": "one short paragraph",
  "blocking_gaps": ["gap 1", "gap 2"],
  "required_follow_up_to_reach_default": ["step 1", "step 2"],
  "publication_refresh_blocked": true
}
```

The runner must reject:

- missing file
- malformed JSON
- `agent_id` mismatch with the approval artifact
- `publication_refresh_blocked = false`
- empty `blocking_gaps`
- empty `required_follow_up_to_reach_default`

Allowed richer surfaces for `--allow-rich-surface`:

- `add_dirs`
- `mcp_management`
- `external_sandbox_policy`
- `advanced_session_controls`

Unknown richer-surface values fail closed.

### Output contract

Scratch root:

`docs/agents/.uaa-temp/runtime-follow-on/runs/<run_id>/`

Required scratch artifacts for both `--dry-run` and `--write`:

- `input-contract.json`
- `codex-prompt.md`
- `run-status.json`
- `run-summary.md`
- `validation-report.json`
- `written-paths.json`
- `handoff.json`

Required additional artifacts for `--write`:

- `logs/codex-exec.stdout.log`
- `logs/codex-exec.stderr.log`

`run-status.json` minimum schema:

```json
{
  "run_id": "20260429T000000Z-example",
  "workflow_version": "runtime_follow_on_v1",
  "status": "dry_run_ok | write_ok | write_failed_validation | write_failed_exec",
  "approval_artifact_path": "docs/agents/lifecycle/example/governance/approved-agent.toml",
  "agent_id": "example_agent",
  "host_surface": "xtask.runtime-follow-on",
  "loaded_prompt_source": "embedded",
  "tier_requested": "default",
  "tier_achieved": "default",
  "primary_template": "opencode",
  "written_paths": [],
  "validation_checks": [],
  "deferred_richer_surfaces": [],
  "handoff_ready": false,
  "errors": []
}
```

`handoff.json` minimum schema:

```json
{
  "agent_id": "example_agent",
  "manifest_root": "cli_manifests/example_agent",
  "runtime_lane_complete": false,
  "publication_refresh_required": true,
  "required_commands": [
    "cargo run -p xtask -- support-matrix --check",
    "cargo run -p xtask -- capability-matrix --check",
    "cargo run -p xtask -- capability-matrix-audit",
    "make preflight"
  ],
  "blockers": []
}
```

### Tier policy details

#### Default

`default` means the run must land all of:

- wrapper runtime code under `crates/<agent_id>/`
- harnessed backend integration under `crates/agent_api/src/backends/<agent_id>/`
- target onboarding tests under `crates/agent_api/tests/**`
- wrapper coverage source-of-truth updates at the registry-owned source path
- runtime-owned manifest evidence under `cli_manifests/<agent_id>/snapshots/**` and `cli_manifests/<agent_id>/supplement/**`
- achieved-tier summary and green-lane handoff artifact

Primary template: `opencode`

#### Minimal

`minimal` is allowed only when the run also records:

- explicit reason `default` was not chosen
- exact missing work to reach `default`
- confirmation that publication refresh is blocked on those missing pieces

Primary template: `gemini_cli`

#### Feature-rich

`feature-rich` is additive. It may borrow selective surfaces from `codex` and `claude_code`, but only for the opt-ins requested through `--allow-rich-surface`.

Primary template: `opencode` plus explicit richer references as needed.

### Wrapper coverage source-path resolution

Current registry entries use:

- `wrapper_coverage.binding_kind = "generated_from_wrapper_crate"`
- `wrapper_coverage.source_path = "crates/<agent_id>"`

For this binding kind, the runner must interpret the registry-owned coverage source-of-truth as:

- writable Rust source only under `<source_path>/src/**`
- preferred canonical file: `<source_path>/src/wrapper_coverage_manifest.rs`
- creating that file is allowed if the target wrapper shell does not yet contain it

The runner must reject:

- direct edits to `cli_manifests/<agent_id>/wrapper_coverage.json`
- direct edits outside `<source_path>/src/**` that are justified only as wrapper-coverage work

### Write allowlist

The runner must validate every changed path after `codex exec` and reject any path outside this allowlist:

- `crates/<agent_id>/`
- `crates/agent_api/src/backends/<agent_id>/`
- `crates/agent_api/tests/`
- registry-owned wrapper coverage source path
- `cli_manifests/<agent_id>/snapshots/`
- `cli_manifests/<agent_id>/supplement/`
- `docs/agents/.uaa-temp/runtime-follow-on/runs/<run_id>/`

Special rule:
- generated onboarding packet docs are never writable by this command
- generated publication outputs are never writable by this command
- generated `wrapper_coverage.json` is never writable by this command
- `versions/**` is never writable by this command

### Failure and rerun semantics

- Every invocation gets a fresh `run_id`.
- Prior scratch runs remain untouched.
- A failed `--write` preserves logs and summary artifacts under its own scratch root.
- The runner exits non-zero for any boundary violation, missing required output, missing test output, or missing summary field.
- Reruns do not delete previous scratch runs and do not silently overwrite prior evidence.

## NOT in scope

- Extending `onboard-agent` to write runtime code.
  Rationale: breaks the control-plane/runtime boundary.
- Treating generated packet docs as executable input.
  Rationale: creates dual truth systems.
- Writing publication-owned manifest pointers, current snapshots, or reports.
  Rationale: belongs to the next lane.
- Regenerating support or capability matrices inside this runner.
  Rationale: publication refresh remains deferred.
- Closing the proving run from this command.
  Rationale: closeout remains a later gate after green validation.
- Auto-choosing richer surfaces from `codex` or `claude_code` without explicit opt-in.
  Rationale: that is how a bounded lane turns back into a scavenger hunt.

## What already exists

- `onboard-agent` already owns approval-linked control-plane enrollment.
- `scaffold-wrapper-crate` already owns publishable wrapper-crate shell creation.
- `onboard_agent` preview renderers already describe the remaining runtime checklist.
- `agent_registry.toml` already pins `crate_path`, `backend_module`, `manifest_root`, and wrapper-coverage source ownership.
- `opencode` already provides the default baseline for wrapper + backend harness integration.
- `gemini_cli` already shows the reduced exception shape.
- `codex` and `claude_code` already show richer optional surfaces.
- wrapper coverage source-of-truth is already a Rust-owned pattern, not a handwritten JSON pattern.

The new lane should reuse all of that. It should not invent parallel truth surfaces.

## Workstreams

### Workstream 1 - Runner entrypoint and contract assembly

Modules:

- `crates/xtask/src/main.rs`
- `crates/xtask/src/runtime_follow_on.rs`
- `crates/xtask/src/runtime_follow_on/input.rs`
- `crates/xtask/src/runtime_follow_on/output.rs`

Tasks:

1. Register `RuntimeFollowOn` in `xtask`.
2. Parse `--approval`, `--requested-tier`, `--minimal-justification-file`, `--allow-rich-surface`, `--run-id`, `--dry-run`, and `--write`.
3. Resolve approval artifact and registry truth into one normalized `input-contract.json`.
4. Fail fast on path mismatches, missing scaffold roots, or invalid tier requests.

Acceptance gate:

- `--dry-run` produces a complete `input-contract.json` and planned write set without invoking Codex.

### Workstream 2 - Embedded prompt payload and thin skill wrapper

Modules:

- `crates/xtask/src/runtime_follow_on/prompt.rs`
- `.codex/skills/runtime-follow-on/SKILL.md`

Tasks:

1. Store the canonical runtime prompt payload at `crates/xtask/templates/runtime_follow_on_codex_prompt.md` and embed it into the runner with compile-time inclusion.
2. Ensure the command writes the exact resolved prompt payload to `codex-prompt.md`.
3. Add a thin skill wrapper that calls the command and does not duplicate the contract.

Acceptance gate:

- the runner works even if no ambient local runtime-follow-on skill is discoverable
- the exact prompt payload used in a run is reviewable on disk

### Workstream 3 - Write boundary, manifest split, and validation

Modules:

- `crates/xtask/src/runtime_follow_on/validate.rs`
- `crates/xtask/src/runtime_follow_on/write_boundary.rs`
- `crates/xtask/src/runtime_follow_on/manifest_split.rs`

Tasks:

1. Enforce the allowed write boundary.
2. Reject generated `wrapper_coverage.json` edits and require source-path Rust edits instead.
3. Enforce runtime-owned vs publication-owned manifest split.
4. Emit `validation-report.json` and `written-paths.json`.

Acceptance gate:

- any write outside the boundary fails the run with offending paths listed explicitly

### Workstream 4 - Summary and handoff artifacts

Modules:

- `crates/xtask/src/runtime_follow_on/summary.rs`

Tasks:

1. Emit `run-status.json`.
2. Emit `run-summary.md`.
3. Emit `handoff.json`.
4. Require `tier_achieved`, `primary_template`, deferred richer surfaces, and blocker lists.

Acceptance gate:

- no successful run exits without all three artifacts present and schema-valid

### Workstream 5 - Tests

Modules:

- `crates/xtask/tests/runtime_follow_on_entrypoint.rs`
- `crates/xtask/tests/runtime_follow_on_contract.rs`
- `crates/xtask/tests/runtime_follow_on_boundary.rs`
- `crates/agent_api/tests/**`

Tasks:

1. Add entrypoint tests for dry-run/write mode parsing.
2. Add contract tests for approval/registry mismatch rejection.
3. Add boundary tests for forbidden writes and manifest split enforcement.
4. Add tests proving explicit prompt embedding or loading.
5. Add default-tier onboarding tests under `agent_api`.

Required naming convention for new target-level `agent_api` tests:

- primary test file: `crates/agent_api/tests/c1_<agent_id>_runtime_follow_on.rs`
- optional helper module directory: `crates/agent_api/tests/c1_<agent_id>_runtime_follow_on/`

Acceptance gate:

- the runner contract is testable without a real CLI and default-tier onboarding proves the required `agent_api` test posture

### Workstream 6 - Proving run on one real target

Modules:

- `crates/<target_agent>/`
- `crates/agent_api/src/backends/<target_agent>/`
- `crates/agent_api/tests/**`
- `cli_manifests/<target_agent>/snapshots/**`
- `cli_manifests/<target_agent>/supplement/**`

Tasks:

1. Use the exact agent referenced by the approval artifact passed to `runtime-follow-on`.
2. For the first real proving run after `uaa-0022` lands, that approval artifact must reference the next newly approved real target agent, not an already-onboarded built-in agent.
3. If no newly approved real target exists yet, stop after contract tests and fixture validation. Do not promote this lane into the operator guide.
4. Run `onboard-agent --write`.
5. Run `scaffold-wrapper-crate --write`.
6. Run `runtime-follow-on --dry-run`, then `--write`.
7. Review boundary output, summary artifacts, and produced runtime evidence.

Acceptance gate:

- one real target proves the lane end to end without using generated packet docs as machine truth

## Test Review

### Code path coverage diagram

```text
NEW OPERATOR FLOWS
===========================
[+] Runtime runner after onboard + scaffold
    ├── [GAP] Happy path default-tier onboarding
    ├── [GAP] Minimal-tier request rejected without justification
    ├── [GAP] Rerun after partial failure preserves prior summary
    └── [GAP] Handoff artifact marks publication lane readiness

NEW DATA FLOWS
===========================
[+] approval artifact + registry -> runner input assembly
    ├── [GAP] Path mismatch rejected
    ├── [GAP] Pack-prefix mismatch rejected
    └── [GAP] Capability and tier contract materialized into summary

[+] runner -> codex exec with embedded prompt payload
    ├── [GAP] Prompt payload is persisted to scratch artifacts
    └── [GAP] Ambient local skill discovery is not required

[+] runner -> runtime-owned writes
    ├── [GAP] wrapper crate writes allowed
    ├── [GAP] backend adapter writes allowed
    ├── [GAP] `agent_api` onboarding tests allowed
    ├── [GAP] wrapper coverage source path allowed
    └── [GAP] publication-owned files rejected

NEW CODEPATHS / BRANCHES
===========================
[+] Tier policy
    ├── [GAP] default -> opencode template
    ├── [GAP] minimal -> explicit exception
    └── [GAP] feature-rich -> explicit opt-in only

[+] Manifest evidence split
    ├── [GAP] runtime-owned evidence accepted
    └── [GAP] publication-owned state rejected

NEW ERROR / RESCUE PATHS
===========================
[+] MissingInput
    └── [GAP] exact missing-path error
[+] WriteBoundaryViolation
    └── [GAP] offending-path rejection
[+] TierPolicyViolation
    └── [GAP] missing-justification rejection
[+] CoverageSourceViolation
    └── [GAP] generated JSON edit rejection
[+] SummaryContractViolation
    └── [GAP] incomplete summary rejection
```

### Required tests

Unit tests:

- approval artifact path resolution
- registry/approval mismatch rejection
- tier-policy parsing
- richer-surface allowlist parsing
- scratch artifact schema validation

Integration tests:

- dry-run emits all required scratch artifacts
- write boundary rejects out-of-scope paths
- manifest split rejects publication-owned writes
- embedded prompt payload is written to `codex-prompt.md`
- `minimal` fails without justification
- `default` requires target onboarding tests
- target onboarding tests follow the `c1_<agent_id>_runtime_follow_on.rs` naming rule

Target onboarding tests:

- backend contract behavior
- event mapping
- completion behavior
- validation boundary behavior
- redaction and bounded output behavior

## Failure Modes Registry

| Codepath | Failure mode | Rescued? | Test required? | User sees | Logged? |
| --- | --- | ---: | ---: | --- | ---: |
| runner input assembly | missing approval artifact | Y | Y | exact missing path error | Y |
| runner input assembly | approval artifact and registry disagree on path ownership | Y | Y | machine-truth precedence error | Y |
| tier policy | `minimal` requested without justification | Y | Y | explicit policy rejection | Y |
| prompt loading | runner depends on ambient local skill discovery | Y | Y | explicit prompt-source failure | Y |
| write boundary | Codex writes outside allowed runtime targets | Y | Y | offending-path rejection | Y |
| coverage update | generated `wrapper_coverage.json` edited instead of source Rust | Y | Y | source-path violation | Y |
| test posture | default-tier run omits required `agent_api` tests | Y | Y | lane incomplete, not handoff-ready | Y |
| manifest evidence | publication-owned manifest state edited by runtime lane | Y | Y | seam-violation error | Y |
| summary emission | missing `run-status.json`, `run-summary.md`, or `handoff.json` | Y | Y | unreviewable-run failure | Y |
| proving-run promotion | no newly approved real target exists yet | Y | Y | plan stays backlog-only, no operator-guide promotion | Y |

No silent partial success is accepted.

## Success Metrics

This milestone is only successful if it improves throughput, not just wording.

Track in `run-status.json` or related runner metrics:

- time from `approved-agent.toml` + scaffold to first runnable runtime packet
- maintainer review time for one runtime-runner output
- number of boundary violations caught before review
- number of reruns needed before a handoff-ready output
- whether the first proving target reached `default`, `minimal`, or `feature-rich`
- whether publication refresh was blocked by missing runtime evidence

Success bar for the first proving run:

- zero silent boundary violations
- one reviewable runtime summary with exact deferred surfaces
- no archaeology needed to start the publication-refresh follow-on

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Runner entrypoint and contract assembly | `crates/xtask/src/`, `crates/xtask/tests/` | — |
| 2. Embedded prompt payload and thin skill wrapper | `crates/xtask/src/`, `.codex/skills/` | 1 |
| 3. Write boundary, manifest split, and validation | `crates/xtask/src/`, `crates/xtask/tests/` | 1 |
| 4. Summary and handoff artifacts | `crates/xtask/src/`, `crates/xtask/tests/` | 1, 3 |
| 5. Default-tier onboarding tests | `crates/agent_api/tests/`, `crates/agent_api/src/backends/` | 3 |
| 6. Real target proving run | `crates/<target>/`, `crates/agent_api/src/backends/<target>/`, `cli_manifests/<target>/` | 2, 4, 5 |
| 7. Operator-guide promotion after proof | `docs/cli-agent-onboarding-factory-operator-guide.md`, `docs/backlog/` | 6 |

### Parallel lanes

- Lane A: step 1 -> step 3 -> step 4
  Core runner lane. Keep all `xtask` contract and validation work together because these modules will overlap heavily.
- Lane B: step 2
  Prompt-payload and skill-wrapper lane. It can run once the command surface from step 1 is named and stable.
- Lane C: step 5
  Test lane. It can start after step 3 locks the boundary and tier rules.
- Lane D: step 6
  Proving lane. Wait for A, B, and C.
- Lane E: step 7
  Procedure-promotion lane. Wait for the proving lane.

### Execution order

1. Launch lane A first.
2. Once the command name and scratch schema are stable, launch lane B in parallel.
3. Once boundary and manifest-split rules are stable, launch lane C in parallel with late lane-A cleanup.
4. Launch lane D only after A, B, and C merge cleanly.
5. Launch lane E last if the proving run validates the lane.

### Conflict flags

- Steps 1, 3, and 4 all touch `crates/xtask/src/runtime_follow_on*` and `crates/xtask/tests/**`. Keep them in one lane.
- Step 5 touches `crates/agent_api/tests/**` and may also touch backend modules for shared test helpers. Do not overlap it with proving-run backend edits unless ownership is explicit.
- Step 6 will touch one real target backend plus `cli_manifests/<target>/`. Do not start it until the runner boundary is final.

If only one engineer is available, follow the same order sequentially.

## Verification Commands

Run in this order:

```sh
cargo test -p xtask runtime_follow_on -- --nocapture
cargo test -p agent_api --all-features
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --write
cargo run -p xtask -- scaffold-wrapper-crate --agent <agent_id> --write
cargo run -p xtask -- runtime-follow-on --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --dry-run
cargo run -p xtask -- runtime-follow-on --approval docs/agents/lifecycle/<pack>/governance/approved-agent.toml --write
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

If the runtime lane is not yet supposed to touch publication refresh, the last four commands are still required for the proving run. They are not part of the runner's write set.

## Acceptance Criteria

This plan is done only when all of the following are true:

1. `xtask` exposes a new `runtime-follow-on` command with `--dry-run` and `--write`.
2. The command defaults to `default` tier and uses `opencode` as the baseline template.
3. `minimal` is rejected unless explicit justification is provided.
4. Richer `codex` and `claude_code` surfaces are used only through explicit opt-ins.
5. Approval artifact and registry truth are the only executable machine-owned inputs.
6. The runtime prompt payload is embedded or explicitly loaded by the runner and written to scratch artifacts for review.
7. The runner enforces the exact write allowlist and rejects any out-of-scope path.
8. The runner rejects generated `wrapper_coverage.json` edits and requires source-of-truth Rust edits instead.
9. The runner writes only `cli_manifests/<agent_id>/snapshots/**` and `cli_manifests/<agent_id>/supplement/**`, and rejects publication-owned manifest state changes.
10. A successful run emits `input-contract.json`, `run-status.json`, `run-summary.md`, `validation-report.json`, `written-paths.json`, and `handoff.json`.
11. Default-tier runs include target onboarding tests under `agent_api`.
12. One real onboarding target proves the lane end to end.
13. The operator guide remains the shipped truth until the proving run succeeds and the lane is intentionally promoted.

## Completion Summary

- Step 0: Scope Challenge - accepted; the runtime seam is the right seam, but it now has an exact host surface and boundary contract
- Architecture Review: resolved around one `xtask` command, machine-truth inputs, and scratch artifacts
- Code Quality Review: dual-authority drift removed, prompt ownership pinned, summary schema pinned
- Test Review: coverage diagram produced, required unit/integration/onboarding tests named
- Performance Review: bounded and acceptable; maintainer-time waste is the main thing to avoid
- NOT in scope: written
- What already exists: written
- Failure modes: written, no silent partial-success path accepted
- Parallelization: 5 lanes total, 3 technical lanes before proving, 1 proving lane, 1 procedure-promotion lane
- Lake Score: complete version selected over the shortcut at every major decision point

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | CEO | Keep the seam after `onboard-agent` and `scaffold-wrapper-crate` | mechanical | P2 + P3 | preserves the existing boundary while fixing the actual bottleneck | end-to-end create-lane rewrite |
| 2 | CEO | Use a repo-owned `xtask` runner that invokes `codex exec` | taste | P5 | contract surfaces belong in repo-owned code, not only in prompt text | skill-only orchestrator |
| 3 | CEO | Treat `opencode` as the default implementation baseline | mechanical | P1 + P5 | it exercises the backend harness without dragging in optional richness | defaulting to `codex` or `claude_code` |
| 4 | CEO | Keep publication refresh out of this milestone but require a handoff artifact | mechanical | P1 + P2 | keeps the seam bounded without creating an automation island | bundling refresh into the same runner |
| 5 | Eng | Use approval artifact and registry as executable truth | mechanical | P4 + P5 | avoids dual-authority drift with generated packet markdown | generated packet docs as machine input |
| 6 | Eng | Expand the write boundary to include `crates/agent_api/tests/**` | mechanical | P1 | default-tier onboarding is incomplete without required tests | runtime code only |
| 7 | Eng | Split runtime-owned manifest evidence from publication-owned state | mechanical | P1 + P5 | keeps the seam explicit and reviewable | whole-root manifest writes |
| 8 | Eng | Require machine-readable scratch artifacts and handoff output | mechanical | P5 | review and replay must be deterministic | freeform prose summary only |
