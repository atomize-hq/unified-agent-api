<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/feat-cli-agent-onboarding-factory-autoplan-restore-20260420-223712.md -->
# CLI Agent Onboarding Factory - PLAN

Source: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-design-20260420-151505.md`  
Status: M1 landed on `feat/cli-agent-onboarding-factory`; M2 ready for implementation planning  
Last updated (UTC): 2026-04-21

## Purpose
M2 turns the onboarding bridge from rehearsal into execution.

M1 proved the repo can hold one committed agent registry, derive support enrollment from it, derive capability-matrix enrollment from it, and preview the next control-plane packet with `xtask onboard-agent --dry-run`. That work is now landed on this branch.

The next milestone is not more preview polish. The next milestone is one safe control-plane mutation path plus one real approved-agent proving run. The outcome that matters is simple: after an agent is approved, maintainers stop hand-editing control-plane files one by one, the repo mutates only files it owns, runtime truth stays backend-owned, and the first real onboarding closes with an unmistakable next executable artifact instead of another OpenCode-style stall.

## Landed M1 Baseline
These are already in the branch and are no longer plan items:

- `crates/xtask/data/agent_registry.toml` exists and seeds `codex`, `claude_code`, and `opencode`.
- `crates/xtask/src/agent_registry.rs` parses and validates the registry with fail-closed uniqueness checks.
- `crates/xtask/src/support_matrix/derive.rs` now enrolls roots from the registry instead of `CURRENT_AGENT_ROOTS`.
- `crates/xtask/src/capability_matrix.rs` now enrolls capability-matrix backends from the registry and applies canonical-target MCP projection.
- `crates/xtask/src/onboard_agent.rs` and `crates/xtask/src/onboard_agent/preview.rs` implement `xtask onboard-agent --dry-run`.
- `scripts/publish_planner.py`, `scripts/publish_crates.py`, and `.github/workflows/publish-crates.yml` now handle new crates through the existing crates.io publish flow.
- `crates/xtask/tests/agent_registry.rs` and `crates/xtask/tests/onboard_agent_entrypoint.rs` cover the seeded registry and dry-run preview surface.

M2 must build on this exact repo state. It is not a greenfield M1 rewrite.

## Premise Challenge
| Premise | Verdict | Why |
|---|---|---|
| The next bottleneck is manual control-plane mutation after approval, not candidate recommendation. | Accept | The branch already has dry-run preview machinery. The remaining gap is converting that preview into a safe write path and landing a real agent without repo archaeology. |
| Runtime truth must remain owned by wrapper crates, backend implementations, and committed manifest evidence. | Accept | `docs/specs/unified-agent-api/support-matrix.md` keeps support truth crate-first and evidence-first. M2 must not move that truth into the registry. |
| The first real proving run is mandatory in M2. | Accept | Without a real approved agent run, the factory is still a preview tool. |
| A fully data-driven backend registry is required now. | Reject | Current residual manual runtime registration is real, but solving it with a framework-scale abstraction now is ocean-boiling. M2 should prove the control-plane mutation slice first. |
| Recommendation formalization belongs in M2. | Reject | Recommendation remains packet-driven and HITL until the onboarding bridge stops being the bottleneck. |

## Scope Lock
- Keep M2 focused on safe mutation of control-plane-owned artifacts plus one real proving run.
- Keep runtime/backend behavior owned by `crates/<agent>/` and `crates/agent_api/src/backends/<agent>/`.
- Keep `docs/specs/unified-agent-api/support-matrix.md` authoritative for support publication semantics.
- Keep `docs/specs/unified-agent-api/capability-matrix.md` authoritative for capability-advertising projection semantics.
- Keep the first mutation slice explicit and conservative: no hidden overwrite mode, no best-effort partial writes.
- Keep the first proving run centered on one already-approved real agent, not a synthetic fixture.
- Keep the current registry schema for compatibility, but validate it against runtime and manifest truth instead of trusting it over either one.

## Success Criteria
M2 is complete only when all of these are true:

- `cargo run -p xtask -- onboard-agent --write --agent-id <approved-agent> ...` exists as an explicit mutation mode beside `--dry-run`.
- The write mode mutates only control-plane-owned surfaces:
  - `crates/xtask/data/agent_registry.toml`
  - `docs/project_management/next/<prefix>/**`
  - `cli_manifests/<agent>/**` control-plane skeleton files
  - root `Cargo.toml` workspace membership when `crate_path` is a new member
  - the generated publishable-crate block inside `docs/crates-io-release.md`
- On validation failure or write failure, the command leaves the repo unchanged.
- Re-running the same approved descriptor against an already-generated identical control-plane state is a deterministic no-op, not a duplicate-entry failure.
- M2 defines one explicit canonical-target rule for capability-matrix target-sensitive projection and one explicit parity rule between registry `canonical_targets` and manifest-root `current.json.expected_targets`.
- The first real approved-agent proving run lands through this sequence:
  - `onboard-agent --dry-run`
  - `onboard-agent --write`
  - manual runtime-owned wrapper/backend implementation
  - committed manifest evidence population
  - regenerated support/capability publication artifacts
  - `make preflight`
- Outcome metrics are recorded for the proving run:
  - manual control-plane file edits by maintainers: `0`
  - partial-write incidents: `0`
  - ambiguous ownership incidents: `0`
  - approved-agent to repo-ready control-plane mutation time: recorded
  - proving-run closeout passes `make preflight`

## What Already Exists
M2 must reuse these surfaces instead of inventing new ones:

- Control-plane entrypoints:
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/onboard_agent.rs`
  - `crates/xtask/src/onboard_agent/preview.rs`
  - `crates/xtask/src/agent_registry.rs`
- Publication and evidence consumers:
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/support_matrix/derive.rs`
  - `crates/xtask/src/capability_matrix.rs`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- Release/publication rails:
  - `Cargo.toml`
  - `docs/crates-io-release.md`
  - `scripts/publish_planner.py`
  - `scripts/publish_crates.py`
  - `.github/workflows/publish-crates.yml`
- Existing test posture:
  - `crates/xtask/tests/agent_registry.rs`
  - `crates/xtask/tests/onboard_agent_entrypoint.rs`
  - `crates/xtask/tests/support_matrix_*.rs`
  - `crates/xtask/tests/c8_spec_capability_matrix_*.rs`

## Not In Scope
- Generating wrapper crate source files or backend implementation files.
- Auto-implementing runtime capability computation for the new agent.
- Replacing explicit backend registration with a plugin system or universal backend factory.
- Formalizing `xtask recommend-agent` or recommendation artifacts before the proving run.
- Adding update mode for already-onboarded agents.
- Reworking support-matrix semantics, capability-matrix semantics, or release workflow semantics.
- Making `onboard-agent` infer runtime truth from upstream CLIs.

## Dream State Delta
```text
CURRENT STATE
approved agent
    |
    +--> dry-run preview
    +--> human copies paths and edits control-plane files manually
    +--> runtime work
    +--> regenerate artifacts

M2
approved agent
    |
    +--> onboard-agent --dry-run
    +--> onboard-agent --write
    +--> runtime-owned implementation lane
    +--> regenerate evidence + publication artifacts
    +--> preflight closeout

12-MONTH IDEAL
approved recommendation artifact
    |
    +--> replay-safe control-plane mutation
    +--> well-bounded runtime implementation lane
    +--> drift checks for already-onboarded agents
    +--> boring, repeatable onboarding closeout
```

M2 does not reach the 12-month ideal. It closes the most expensive remaining gap between approval and executable repo state.

## Implementation Alternatives
### Approach A: Keep Dry-Run, Do The Proving Run Mostly By Hand
Summary: leave `onboard-agent` as preview-only and use the first real agent to manually apply the preview output.

Effort: S  
Risk: High

Pros:
- smallest diff
- avoids write-transaction design work
- lets the team land the next agent quickly if urgency is extreme

Cons:
- preserves the exact manual-control-plane bottleneck M2 is supposed to remove
- yields no replay or rollback semantics
- teaches the repo nothing durable about safe mutation

Reuses:
- current dry-run output
- existing manual runtime follow-up

### Approach B: Safe Control-Plane Mutation + First Real Proving Run
Summary: add one explicit write mode for control-plane-owned files, make it replay-safe and rollback-safe, then run one approved real agent through the full flow.

Effort: M  
Risk: Medium

Pros:
- fixes the actual remaining bottleneck
- keeps runtime truth ownership intact
- generates concrete proof about what still feels clumsy after a real run

Cons:
- requires explicit transaction semantics and path-jailing
- still leaves runtime lane partially manual
- requires a real agent decision and closeout discipline

Reuses:
- current dry-run renderers
- current support/capability publication rails
- current publish planner/workflow

### Approach C: Full Data-Driven Onboarding Runtime
Summary: use M2 to eliminate explicit backend registration, derive everything from registry metadata, and add update mode immediately.

Effort: XL  
Risk: High

Pros:
- closest to a long-term factory ideal
- would remove more residual manual runtime registration
- creates one stronger abstraction story

Cons:
- reopens runtime-truth and authority boundaries that the specs just locked
- likely invents a generic framework before the second real use is proven
- increases blast radius far beyond the next bottleneck

Reuses:
- M1 registry and preview scaffolding, but stretches them into a new abstraction layer

**Recommendation:** Choose Approach B. It fixes the actual remaining bottleneck without pretending the repo is ready for a universal runtime framework.

## Mode Selection
Auto-decided mode: `SELECTIVE EXPANSION`.

Reasoning:
- the repo is on an existing feature iteration, not a greenfield concept
- the current plan already overshot into now-landed M1 details
- M2 needs a complete bounded milestone, not a bigger platform rewrite

Accepted expansion:
- add outcome metrics for the proving run so M2 measures lead-time reduction instead of only artifact existence

Deferred expansions:
- backend-registry abstraction
- update mode for already-onboarded agents
- recommendation artifact formalization

## M2 Plan Of Record
### Goal
Turn `xtask onboard-agent` into a safe control-plane mutator and prove the flow on one real approved agent without moving runtime truth into the registry.

### Milestone Outcome
M2 is one bounded milestone, not a loose bundle of improvements. At the end of this milestone:

- maintainers can take an approved agent descriptor and materialize every control-plane-owned artifact without hand-editing those files
- the command fails closed on ownership violations, divergent generated state, and mid-transaction write failures
- the remaining manual runtime lane is explicit, bounded, and attached to a real proving run instead of implied by dry-run output
- the proving run ends with regenerated publication artifacts, `make preflight`, and recorded outcome metrics

### Execution Shape
```text
approved descriptor
        |
        v
shared render plan
        |
        v
transactional owned writes
        |
        +--> release/workspace owned surfaces
        +--> target/capability parity guards
        |
        v
real-agent proving run
        |
        v
generators + preflight + metrics
```

### Command Contract
M2 keeps `--dry-run` and adds one explicit write mode:

```bash
cargo run -p xtask -- onboard-agent --write \
  --agent-id <agent_id> \
  --display-name <display_name> \
  --crate-path <repo-relative-path> \
  --backend-module <repo-relative-path> \
  --manifest-root <repo-relative-path> \
  --package-name <crate-package-name> \
  --canonical-target <target> \
  [--canonical-target <target> ...] \
  --wrapper-coverage-binding-kind <binding-kind> \
  --wrapper-coverage-source-path <repo-relative-path> \
  --always-on-capability <capability-id> \
  [--always-on-capability <capability-id> ...] \
  [--target-gated-capability '<capability-id>:<target>[,<target>...]' ...] \
  [--config-gated-capability '<capability-id>:<config-key>[:<target>[,<target>...]]' ...] \
  [--backend-extension <capability-id> ...] \
  --support-matrix-enabled <true|false> \
  --capability-matrix-enabled <true|false> \
  --docs-release-track <track> \
  --onboarding-pack-prefix <prefix>
```

Rules:
- `--dry-run` and `--write` are mutually exclusive.
- `--write` uses the exact same render plan as `--dry-run`; no hidden write-only behavior.
- M2 has no update mode. Existing divergent generated files fail closed.
- Exact-byte replay against already-generated identical outputs is a success no-op.
- Stdout section order stays the same as dry-run, plus one final mutation summary line under `== RESULT ==`.

### Controlled Write Set
The first write mode is intentionally narrow. If a surface is not in this table, `onboard-agent --write` does not touch it.

| Surface | Owner | M2 write mode |
|---|---|---|
| `crates/xtask/data/agent_registry.toml` | control plane | write |
| `docs/project_management/next/<prefix>/**` | control plane | write |
| `cli_manifests/<agent>/current.json` and empty skeleton dirs | control plane | write |
| root `Cargo.toml` workspace `members` entry | control plane | write when new |
| generated publishable-crate block in `docs/crates-io-release.md` | control plane | write when new |
| `crates/<agent>/**` | runtime owner | never |
| `crates/agent_api/src/backends/<agent>/**` | runtime owner | never |
| `scripts/publish_*.py` and workflow files | release rails | never in M2 |
| support/capability generated outputs | existing generators | regenerated after runtime evidence exists |

### Milestone Invariants
These are the non-negotiable contracts that make M2 implementable instead of hand-wavy:

1. Render-plan parity: dry-run and write are two execution modes over the same in-memory mutation plan.
2. Transactional writes: validation runs before mutation, writes stage safely, and any failure leaves the repo unchanged.
3. Truth ownership stays split correctly: registry metadata can enroll and project, but runtime wrappers, backends, and committed manifests still own executable truth.
4. Proving-run closure is part of the milestone: M2 is not done when `--write` exists, it is done when one real agent makes it through closeout with zero manual control-plane edits.

### Capability And Target Authority Rules
- Support publication remains driven by committed manifest evidence under `cli_manifests/<agent>/`.
- Runtime backend capabilities remain authoritative for the capability matrix.
- Registry capability declarations are treated as projection metadata and must validate against runtime backend capabilities. They are never trusted over runtime output.
- `capability_matrix` keeps its backend-global table shape in M2. For target-sensitive MCP projection, the first item in `canonical_targets` is the one canonical comparison target.
- Once runtime evidence exists for the new agent, the first canonical target must appear in `current.json.expected_targets`.
- If registry canonical targets and manifest expected targets diverge after the proving run, `preflight` must fail closed.

## Architecture
```text
approved real agent
        |
        v
xtask onboard-agent --dry-run
        |
        v
xtask onboard-agent --write
        |
        +--> registry append / replay check
        +--> docs pack materialization
        +--> manifest-root skeleton materialization
        +--> Cargo.toml workspace member insertion (if new)
        +--> docs/crates-io-release.md generated block refresh (if new)
        |
        v
manual runtime-owned lane
        |
        +--> crates/<agent>/**
        +--> crates/agent_api/src/backends/<agent>/**
        +--> explicit backend registration touchpoints
        +--> manifest evidence population
        |
        v
existing generators + gates
        |
        +--> cargo run -p xtask -- support-matrix --check
        +--> cargo run -p xtask -- capability-matrix
        +--> make preflight
```

## Workstreams
### W1. Shared Render Plan + Explicit Write Mode
Goal: make dry-run and write two views over the same execution contract.

Deliverables:
- one shared in-memory mutation plan inside `onboard-agent`
- `--write` CLI plumbing that consumes that plan without introducing write-only render logic
- replay-safe identical-state detection for already-generated control-plane outputs

Primary touchpoints:
- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/tests/onboard_agent_entrypoint.rs`

Exit criteria:
- every file write mode would materialize is already named in dry-run
- dry-run and write remain byte-aligned for the same descriptor
- identical replay succeeds as a no-op, not a duplicate-entry failure

### W2. Path Jailing, Overwrite Policy, And Rollback
Goal: turn write mode into a transaction instead of a best-effort file copier.

Deliverables:
- workspace-root canonicalization for every candidate path
- explicit rejection of symlink escapes and out-of-root resolved paths
- staged write + rollback behavior for created files and directories
- explicit overwrite policy of absent-or-identical only

Primary touchpoints:
- `crates/xtask/src/onboard_agent.rs`
- `crates/xtask/tests/onboard_agent_entrypoint.rs`

Exit criteria:
- validation failures leave zero repo diffs
- injected fs-write failures leave zero partial outputs
- ownership violations fail before the first write

### W3. Release And Workspace Mutation Slice
Goal: include every control-plane-owned repo mutation needed for the proving run, not just the registry and packet.

Deliverables:
- deterministic insertion of a new workspace member in root `Cargo.toml` when `crate_path` is new
- deterministic refresh of the generated publishable-crate block in `docs/crates-io-release.md`
- tests that prove write mode can materialize these owned surfaces without touching release scripts or workflows

Primary touchpoints:
- `Cargo.toml`
- `docs/crates-io-release.md`
- `crates/xtask/tests/onboard_agent_entrypoint.rs`
- `scripts/publish_planner.py` tests only if generator assumptions need new assertions

Exit criteria:
- write mode updates the same release/workspace surfaces that dry-run previews
- workflow and publish script files remain unchanged
- generated release-doc output stays deterministic and reviewer-friendly

### W4. Target Parity And Capability Ownership Hardening
Goal: prevent the registry, manifests, and runtime backend capability truth from silently drifting apart.

Deliverables:
- one explicit primary canonical-target rule for multi-target capability projection
- parity validation between registry `canonical_targets` and manifest `current.json.expected_targets`
- subset validation showing registry-declared capabilities do not outrun runtime backend capability truth
- proving-run docs that name remaining runtime-owned backend registration touchpoints exactly once

Primary touchpoints:
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/src/agent_registry.rs`
- `crates/xtask/tests/agent_registry.rs`
- `crates/xtask/tests/c8_spec_capability_matrix_*.rs`

Exit criteria:
- target mismatches fail before publication artifacts drift
- capability declaration drift fails closed
- the plan no longer pretends explicit backend registration is already automated

### W5. First Real Approved-Agent Proving Run
Goal: prove the bridge on one real approved agent, then capture what is still manual and why.

Deliverables:
- one approved real agent descriptor exercised through `--dry-run` and `--write`
- one bounded runtime-owned implementation lane
- committed manifest evidence plus regenerated support/capability publication outputs
- proving-run packet closeout notes with actual metrics and residual friction

Primary touchpoints:
- `docs/project_management/next/<prefix>/HANDOFF.md`
- `cli_manifests/<agent>/**`
- runtime-owned wrapper/backend files for the chosen agent
- final proving-run verification notes

Exit criteria:
- zero manual control-plane edits outside the command
- the handoff after `--write` names the exact next executable runtime step
- the proving run closes with `make preflight`

## Execution Sequence
This is the concrete order of operations. If any earlier step is still unstable, do not start the next one.

### Phase 1. Transaction Foundation
Scope:
- complete W1
- pin the mutation-plan shape, dry-run/write parity, and replay semantics

Outputs:
- shared render-plan implementation
- `--write` command surface
- entrypoint coverage for parity and replay

Exit gate:
- dry-run output is sufficient to predict all owned writes
- replay-safe no-op behavior is proven in tests

### Phase 2. Safety Hardening
Scope:
- complete W2 on top of the W1 contract

Outputs:
- path jailing
- explicit overwrite policy
- rollback coverage for validation and io failures

Exit gate:
- no partial writes survive any injected failure path
- ownership violations fail before mutation begins

### Phase 3. Control-Plane Completion
Scope:
- complete W3 and W4 against the now-stable write transaction

Outputs:
- release/workspace mutation support
- target/capability parity guards
- documentation of remaining explicit runtime registration touchpoints

Exit gate:
- all control-plane-owned surfaces required by the proving run are command-owned
- publication parity fails closed on target or capability drift

### Phase 4. Real-Agent Proving Run
Scope:
- execute W5 only after Phases 1-3 are merged or otherwise stable in one branch

Outputs:
- one real agent run through the full flow
- regenerated publication artifacts
- recorded metrics and residual friction list

Exit gate:
- `make preflight` passes
- proving-run packet closes with no ambiguous next step

## Minimal Execution Sequence
```text
Phase 1: W1 shared render plan + --write
    |
    v
Phase 2: W2 path jail + rollback
    |
    +--> Phase 3A: W3 release/workspace mutation slice
    +--> Phase 3B: W4 target parity + capability ownership hardening
                |
                v
         Phase 4: W5 first real approved-agent proving run
                |
                v
      support-matrix check + capability-matrix + preflight
```

## Error & Rescue Registry
| Method / Codepath | What can go wrong | Exception / failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| `onboard-agent` argument parse | invalid flag combination | usage error | yes | clap exits with usage text | exit `2` + clear stderr |
| descriptor normalization | malformed target or capability gate | validation error | yes | reject before planning writes | exit `2` |
| path canonicalization | symlink escape or non-workspace target | ownership violation | yes | reject before any write | exit `2` |
| registry replay check | duplicate divergent entry | conflict | yes | reject, print owning surface | exit `2` |
| mutation staging | temp write failure | io error | yes | abort and clean staged outputs | exit `1` |
| atomic apply | rename/create_dir failure mid-transaction | io error | yes | rollback created paths, surface exact file | exit `1` |
| proving-run publication | canonical target mismatch or stale manifest evidence | parity failure | yes | fail the proving-run gate before publish claims drift | failing test / check output |
| proving-run closeout | runtime-owned lane incomplete | preflight failure | yes | keep packet open, do not declare success | failing verification output |

## Test Diagram
```text
NEW CONTROL-PLANE FLOWS
=======================
[+] onboard-agent --write
    |
    ├── [GAP -> unit/integration] shared render plan equals dry-run render plan
    ├── [GAP -> integration] absent-path write succeeds and materializes all owned outputs
    ├── [GAP -> integration] identical replay is a no-op
    ├── [GAP -> integration] divergent generated file fails closed with no writes
    └── [GAP -> integration] injected write failure rolls back everything

[+] path safety
    |
    ├── [GAP -> unit] symlink escape via crate_path is rejected
    ├── [GAP -> unit] symlink escape via backend_module is rejected
    └── [GAP -> unit] symlink escape via manifest_root is rejected

[+] target/capability parity
    |
    ├── [GAP -> unit] primary canonical target rule is explicit for multi-target agents
    ├── [GAP -> integration] registry canonical_targets mismatch current.json.expected_targets
    └── [GAP -> integration] registry-declared capability id missing from runtime backend capabilities

[+] release/workspace mutation
    |
    ├── [GAP -> integration] new workspace member is inserted deterministically
    ├── [GAP -> integration] release-doc generated block refresh is deterministic
    └── [GAP -> regression] workflow and publish scripts remain untouched

PROVING-RUN FLOW
================
[+] approved agent -> dry-run -> write -> runtime lane -> generators -> preflight
    |
    ├── [GAP -> system] dry-run and write materialize identical control-plane intent
    ├── [GAP -> system] handoff still names the exact next executable artifact after write
    ├── [GAP -> system] support-matrix check passes after runtime evidence lands
    ├── [GAP -> system] capability-matrix output includes the real agent under the pinned target rule
    └── [GAP -> system] make preflight passes for the proving run branch
```

## Required Test Surfaces
- extend `crates/xtask/tests/onboard_agent_entrypoint.rs` for:
  - `--write` happy path
  - identical replay no-op
  - divergent generated file rejection
  - rollback on injected write failure
  - `Cargo.toml` mutation
  - `docs/crates-io-release.md` generated block mutation
- extend `crates/xtask/tests/agent_registry.rs` for capability-projection subset validation
- add parity tests under `crates/xtask/tests/c8_spec_capability_matrix_*.rs` for primary canonical target and target drift
- add a proving-run checklist doc test or packet validation step for post-write handoff completeness

## Commands
- `cargo test -p xtask`
- `cargo run -p xtask -- onboard-agent --dry-run --agent-id <approved-agent> ...`
- `cargo run -p xtask -- onboard-agent --write --agent-id <approved-agent> ...`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix`
- `make preflight`

## Failure Modes Registry
| Codepath | Failure mode | Rescued? | Test? | User sees | Logged? |
|---|---|---|---|---|---|
| write transaction | partial file set lands before a failing write | must be | add integration rollback test | explicit failure, zero repo diff | yes |
| path resolution | generated path resolves outside workspace via symlink | must be | add unit + integration test | explicit validation failure | yes |
| target parity | capability matrix claims a target that support evidence does not | must be | add parity regression | failing check before publication | yes |
| replay | identical rerun duplicates registry entry | must be | add replay regression | success no-op | yes |
| proving run | write mode succeeds but packet still does not name the next executable runtime step | must be | add system checklist test | failing closeout checklist | yes |
| runtime capability drift | registry capability declaration outgrows runtime backend capability set | must be | add subset validation test | failing generator/check | yes |

Critical gap rule:
- If any failure mode above has no test and no fail-closed behavior, M2 is not ready.

## Security Review
- Path ownership is the main new attack surface. M2 must canonicalize and jail every candidate path before touching disk.
- No new secrets or external credentials are introduced by `onboard-agent`.
- `Cargo.toml` and release-doc mutation must be scoped to deterministic generated blocks only. No arbitrary text rewriting.
- Runtime-owned surfaces stay out of the write set, which keeps the blast radius bounded.

## Performance Review
- The current dry-run conflict scan walks `reports/**` repeatedly. M2 should avoid multiplying that cost during write-mode validation.
- Release-doc generation should derive from existing publishable-package discovery once per invocation, not re-scan cargo metadata per file.
- Replay detection should hash or compare rendered outputs in memory once, not through repeated read/parse loops for each file.

## What The Implementer Needs To Know
### Hour 1
- M2 is a rebaseline, not a continuation of greenfield M1.
- The write set is only control-plane-owned files.
- Replay-safe and rollback-safe behavior are non-negotiable.

### Hour 2-3
- The command must use the same render plan for dry-run and write.
- Canonical target authority must be explicit because capability-matrix output is backend-global, not target-expanded.

### Hour 4-5
- The proving run is where residual manual runtime registration surfaces get named, not abstracted away.
- `docs/crates-io-release.md` needs one deterministic generated block if the new crate is publishable.

### Hour 6+
- Capture actual proving-run metrics and residual friction.
- Do not silently broaden scope into update mode, recommendation formalization, or generic backend registries.

## Parallelization Strategy
Parallelization only starts after the mutation-plan contract is stable enough that other lanes are not guessing at file shape or error behavior.

| Lane | Scope | Modules touched | Start gate | Must hand off |
|---|---|---|---|---|
| A. transaction core | shared render plan, `--write`, replay semantics, rollback contract | `crates/xtask/src/onboard_agent*`, `crates/xtask/tests/onboard_agent_entrypoint.rs` | — | stable mutation-plan API, stderr/result contract, fixture shape |
| B. release/workspace mutation | `Cargo.toml` member insertion, release-doc generated block, related tests | repo root `Cargo.toml`, `docs/crates-io-release.md`, onboard-agent tests | A | deterministic generated-block behavior and test fixtures |
| C. target/capability parity hardening | canonical-target rule, parity validation, capability subset validation | `crates/xtask/src/capability_matrix.rs`, `crates/xtask/src/agent_registry.rs`, related tests | A | pinned target rule, failing-check coverage, proving-run parity checklist |
| D. proving-run closeout | approved-agent packet, runtime lane, manifest evidence, final closeout | `docs/project_management/next/<prefix>/`, `cli_manifests/<agent>/`, runtime-owned files | A, B, C | final proving-run metrics and residual friction notes |

Recommended execution:
1. Land Lane A first. This is the contract all later work assumes.
2. Run Lanes B and C in parallel once A's mutation-plan shape and fixture conventions are pinned.
3. Rebase and merge B and C before starting Lane D.
4. Run Lane D sequentially. It is the integration proof, not a speculative implementation lane.

Coordination rules:
- Lane A owns shared `onboard_agent` test fixtures. B and C should add new fixtures only after A's fixture format is stable.
- Lane B owns the release/workspace generated-output assertions. Lane C owns target/capability parity assertions.
- Lane D should not invent missing control-plane behavior. If D discovers that B or C left a gap, stop and fix the upstream lane first.
- If staffing is one engineer, follow the same order serially: A -> B -> C -> D.

## Deferred To TODOS.md
- Formalize recommendation approval artifacts after one successful proving run. Reason: still upstream of the current bottleneck.
- Add update mode for already-onboarded agents after replay-safe write mode proves insufficient. Reason: first prove create-mode and replay semantics.
- Revisit backend registration abstraction only if the proving run shows explicit runtime registration is the dominant residual manual step. Reason: avoid ocean-boiling before the second real use is proven.

## Completion Summary
- Step 0: Scope challenge — rebaselined from stale M1 plan to landed-M1-plus-M2 plan-of-record
- CEO review: 5 issues found, all resolved in the rewrite by narrowing M2 to safe mutation + proving run
- CEO voices: Codex high-level reframe matched Claude subagent on the main point
- Design review: skipped, no UI scope
- Eng review: 5 issues found, all resolved in the rewrite as explicit M2 requirements
- Eng voices: Codex and Claude subagent both required atomic mutation, explicit target authority, and a real proving run
- M2 execution threading: unified around milestone outcome, invariants, phased execution, and schedulable lane handoffs
- Test review: diagram produced, gaps enumerated, proving-run artifact required
- Performance review: report-tree scan and replay cost called out
- Not in scope: written
- What already exists: written
- Failure modes: written with critical gap rule
- Parallelization: 4 lanes, 2 parallel before proving-run closeout

## Cross-Phase Themes
- The plan had become stale because it still talked like pre-landing M1. Both CEO and Eng voices pushed to rebaseline around current repo truth.
- The durable milestone is safe mutation plus one real proving run, not more preview ceremony.
- Runtime truth must stay backend/evidence-owned. The registry can project and enroll, but it cannot become a second executable truth store.

## Decision Audit Trail
| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | CEO | Rebaseline the document around landed M1 and actionable M2 | mechanical | explicit over clever | The branch already contains M1 code, so planning against a future-M1 fiction would mislead implementation | keep stale greenfield-M1 framing |
| 2 | CEO | Make safe mutation plus one proving run the M2 goal | mechanical | choose completeness | This is the smallest milestone that removes the remaining bottleneck | more preview-only work |
| 3 | CEO | Keep recommendation formalization out of M2 | mechanical | pragmatic | It is upstream of the current bottleneck | pull `recommend-agent` into M2 |
| 4 | CEO | Add proving-run outcome metrics to success criteria | taste | boil lakes | Artifact existence alone does not prove the workflow got better | artifact-only success criteria |
| 5 | Eng | Add explicit `--write` beside `--dry-run` | mechanical | explicit over clever | Separate mode flags are easier to reason about and test than implicit mutation | implicit default write mode |
| 6 | Eng | Make replay-safe identical reruns succeed as no-ops | taste | pragmatic | It prevents duplicate-entry churn without opening update mode | fail every rerun |
| 7 | Eng | Keep overwrite policy at absent-or-identical only | mechanical | explicit over clever | First mutation slice should be conservative and fail closed | generalized update mode |
| 8 | Eng | Pin first canonical target as the capability-matrix comparison target | mechanical | explicit over clever | Current output shape is backend-global, so the target rule must be singular and visible | union/intersection target inference |
| 9 | Eng | Validate registry capability declarations against runtime backend capabilities | mechanical | DRY | Runtime truth stays backend-owned, registry declarations must not drift away from it | trusting registry declarations over runtime |
| 10 | Eng | Defer backend-registry abstraction to later | taste | pragmatic | The proving run should tell us if explicit backend registration is the real next bottleneck | full data-driven backend factory now |
| 11 | Eng | Rewrite the M2 core as one phased execution plan with explicit lane handoffs | mechanical | explicit over clever | The milestone was already correct, but the document still made implementers stitch sequencing and ownership together from separate sections | keep the same content fragmented across review-shaped sections |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 1 | clear via `/autoplan` | rebaseline to landed M1, make M2 the safe-mutation + proving-run milestone |
| Codex Review | `codex exec` | Independent 2nd opinion | 2 | clear via `/autoplan` | both runs pushed on stale framing, outcome metrics, atomic mutation, and truth ownership |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | clear via `/autoplan` | atomic write contract, replay policy, target parity, path jailing, and proving-run closeout pinned |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | no UI scope |

**CODEX:** Both Codex passes converged on the same correction. Stop treating this like pre-landing M1 and make M2 prove safe mutation on a real agent.
**CROSS-MODEL:** Claude subagents and Codex agreed on the main direction. The plan needed rebaselining, a real proving run, rollback-safe mutation, and a stricter runtime-truth boundary.
**UNRESOLVED:** 0
**VERDICT:** CEO + ENG CLEARED — M2 is concrete enough to implement.
