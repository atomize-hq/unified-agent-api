<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/feat-fill-trust-gap-autoplan-restore-20260423-151422.md -->
# CLI Agent Onboarding Factory - PLAN

Source:
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-design-20260420-151505.md`
- `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md`
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md`
- `docs/project_management/next/gemini-cli-onboarding/HANDOFF.md`

Status: M1 through M4 landed on `feat/cli-agent-onboarding-factory`; M5 is the active plan-of-record on `staging`, and W4 is the narrow docs closeout lane
Last updated (UTC): 2026-04-23

## Post-M4 Roadmap
This file still contains the full M4 plan-of-record because that milestone just landed and remains the design basis for the maintenance lane. M5 is now the active implementation and closeout milestone, and M4 is no longer the next step.

Correct gstack workflow from here:
- CEO review is done for this slice. Scope is no longer the open question.
- The right posture for M5 is engineering planning and hardening, basically `/plan-eng-review` mode rather than more scope exploration.
- M5 should make the current factory truthful and boring before the repo takes on the next ownership-boundary change in M6.

### M5. Factory Truth Hardening
Status:
- active implementation milestone on `staging`
- 2026-04-23 `/autoplan` premise gate approved; the accepted M5 review decisions are folded into this section so M5 reads as one plan-of-record instead of a roadmap block plus a late addendum
- design review skipped because M5 has no UI scope

Goal:
- make the control plane trustworthy at head so maintainers can believe the factory's green checks, generated publications, and agent-scoped drift output

#### Purpose
M4 proved the separate maintenance lane on `opencode`.

M5 makes that lane truthful. The current trust gap is not missing command surface, it is that publication truth, drift truth, and maintenance-closeout truth can still disagree about what built-in backends advertise by default. If local green and CI green are different predicates, the factory is lying. M5 fixes that before M6 widens ownership boundaries.

#### Problem Statement
The checked-in capability matrix is already correct for the current repo state, but the control plane can still reason about that truth in two different ways:

- `crates/xtask/src/capability_matrix.rs` models published default capability truth from the registry, built-in backend defaults, and manifest availability.
- `crates/xtask/src/agent_maintenance/drift/shared.rs` separately models capability truth for `check-agent-drift`, and today that path can overclaim default-off config-gated capabilities for seeded agents such as `codex` and `claude_code`.
- `crates/xtask/src/agent_maintenance/closeout/validate.rs` trusts the live drift report, so any semantic fork in drift truth leaks directly into maintenance closeout truth.
- `make preflight` currently does not enforce capability publication freshness while CI still runs `cargo run -p xtask -- capability-matrix` plus `cargo run -p xtask -- capability-matrix-audit`, so maintainers cannot fully trust local green.

M5 is therefore not a two-file hotfix. It is a factory-truth hardening milestone that removes duplicated capability semantics and makes local and CI green mean the same thing.

#### Landed Baseline
These are already true on `staging` and are not M5 work:

- `crates/xtask/data/agent_registry.toml` seeds `codex`, `claude_code`, `opencode`, and `gemini_cli`, and all four are enrolled in capability publication.
- `crates/xtask/src/capability_matrix.rs` already regenerates the checked-in `docs/specs/unified-agent-api/capability-matrix.md` without drift.
- `crates/xtask/src/agent_maintenance/drift/shared.rs` already collects per-agent publication truth and is the current source of the false-positive capability drift.
- `crates/xtask/src/agent_maintenance/closeout/validate.rs` already re-checks live drift truth as part of maintenance closeout validation.
- `Makefile` already exposes the repo's check-only publication pattern through `cargo run -p xtask -- support-matrix --check`.
- `.github/workflows/ci.yml` already runs capability publication generation and `cargo run -p xtask -- capability-matrix-audit`.
- The current regression surfaces already exist in `crates/xtask/tests/c8_capability_matrix_unit.rs`, `crates/xtask/tests/agent_maintenance_drift.rs`, `crates/xtask/tests/agent_maintenance_closeout.rs`, and `crates/xtask/tests/agent_registry.rs`.

#### Step 0. Scope Challenge
- Existing code leverage: the repo already has the publication generator, the maintenance drift/closeout path, the registry validator, the `support-matrix --check` pattern, and the targeted regression suites. M5 should reuse those surfaces, not invent a second factory.
- Minimum complete change: one shared capability projection contract, one explicit primary publication-target contract, one check-only capability publication gate, and one regression suite that covers both publication and maintenance truth. No new crate, no new lifecycle command family, no runtime-owned code changes.
- Complexity check: the blast radius is expected to be 8-10 files across `xtask`, tests, `Makefile`, CI, and one or two narrow spec notes. That is acceptable because every touched file sits inside one control-plane seam. Widening into M6 wrapper scaffolding would be the real overbuild.
- Search check: [Layer 1] reuse the repo's existing `support-matrix --check` pattern and current CI audit shape instead of inventing a second gate mechanism. [Layer 3] remove the hidden `canonical_targets[0]` ordering contract by making the primary capability-publication target explicit in registry-owned truth.
- Completeness check: do the whole fix now. Patching `check-agent-drift` alone would still leave closeout truth and local-vs-CI gate divergence intact.
- Distribution check: M5 does not introduce a new binary, package, container, or release rail. No distribution work is needed beyond keeping the existing preflight and CI contract truthful.

#### Scope Lock
In scope:
- extract one shared capability projection path so publication truth, drift truth, and closeout truth stop re-deciding config-gated semantics per caller
- make the primary capability-publication target explicit in registry-owned truth instead of relying on ordering in `canonical_targets`
- validate registry `config_key` values against an explicit built-in allowlist; the current intended keys are `allow_mcp_write` and `allow_external_sandbox_exec`
- keep config-gated capability advertising default-off in published truth unless the projection explicitly opts in, matching the seeded backend defaults already exposed by `crates/agent_api/src/backends/codex/backend.rs` and `crates/agent_api/src/backends/claude_code/backend.rs`
- add a first-class `cargo run -p xtask -- capability-matrix --check` path, or an equivalent check-only entrypoint wired through the same command, so local preflight and CI use the same freshness contract
- treat `cargo run -p xtask -- capability-matrix-audit` as the secondary but still-required semantic guard; freshness belongs to `--check`, orthogonality belongs to the audit, and both must run the same way locally and in CI
- add seeded-agent parity tests and targeted registry/publication regressions so the fifth enrolled agent cannot silently reopen this bug class
- update `PLAN.md` plus the narrow normative notes in `docs/specs/agent-registry-contract.md` and `docs/specs/unified-agent-api/capabilities-schema-spec.md` only where M5 changes the authoritative truth or gate definition
- remove stale status language that still implies M4 is the next milestone

#### Not In Scope
- widening `xtask onboard-agent` into wrapper-crate scaffolding or any other M6 ownership shift
- mutating runtime-owned wrapper or backend code to paper over control-plane drift
- replacing `docs/specs/unified-agent-api/capability-matrix.md` with a new structured artifact in M5
- full documentation cleanup, operator-guide consolidation, or historical doc gardening outside the narrow truth-surface notes M5 changes

#### Success Criteria
M5 is complete only when all of these are true:

- every `capability_matrix_enabled` agent is drift-clean under one canonical projection contract; the current seeded expectation is `codex`, `claude_code`, `opencode`, and `gemini_cli`
- `cargo run -p xtask -- check-agent-drift --agent codex` exits `0` on a clean repo
- `cargo run -p xtask -- check-agent-drift --agent claude_code` exits `0` on a clean repo
- `cargo run -p xtask -- check-agent-drift --agent opencode` and `cargo run -p xtask -- check-agent-drift --agent gemini_cli` remain clean
- capability truth is modeled once and reused by capability publication, drift inspection, and maintenance closeout validation instead of being duplicated per caller
- the primary capability-publication target is explicit in registry-owned truth and target-order churn cannot silently change publication output
- `cargo run -p xtask -- capability-matrix --check` exists and fails on stale publication without mutating the worktree
- `make preflight` and CI both run the same capability-publication freshness and audit contract
- regression coverage proves config-gated capabilities stay default-off in publication truth, malformed `config_key` values fail closed, missing primary publication targets fail closed, and target-order churn cannot silently change publication truth
- `PLAN.md` and the touched normative notes no longer claim M4 is the next milestone or leave the M5 green gate ambiguous

#### What Already Exists
| Sub-problem | Existing code to reuse | Why it matters |
|---|---|---|
| published capability truth | `crates/xtask/src/capability_matrix.rs` | already models the correct checked-in default publication truth and should stay the render/check entrypoint |
| drift inspection | `crates/xtask/src/agent_maintenance/drift/shared.rs`, `crates/xtask/src/agent_maintenance/drift/publication.rs` | already collects per-agent drift and is the current false-positive source |
| closeout truth | `crates/xtask/src/agent_maintenance/closeout/validate.rs` | already consumes live drift truth, so projection bugs leak directly into maintenance closure |
| registry validation | `crates/xtask/src/agent_registry.rs`, `crates/xtask/data/agent_registry.toml` | already validates path and shape constraints and is the right place to make publication-target and `config_key` truth explicit |
| local gate pattern | `Makefile` | already demonstrates the repo's intended check-only generator pattern through `support-matrix --check` |
| CI gate pattern | `.github/workflows/ci.yml` | already shows the stricter capability publication contract that local preflight must match |
| regression harness | `crates/xtask/tests/c8_capability_matrix_unit.rs`, `crates/xtask/tests/agent_maintenance_drift.rs`, `crates/xtask/tests/agent_maintenance_closeout.rs`, `crates/xtask/tests/agent_registry.rs` | the repo already has the right test homes; M5 should extend them instead of inventing new harnesses |

#### Chosen Approach
M5 hardens one control-plane seam:

1. define one shared capability projection contract
2. route both publication and maintenance truth through it
3. make the capability freshness gate check-only and identical locally and in CI
4. pin the whole thing with seeded-agent and registry regression coverage

That is the smallest complete fix. Anything smaller leaves the lying-factory problem intact. Anything bigger turns M5 into M6 scope creep.

#### Dream State Delta
```text
CURRENT
capability_matrix.rs decides published default truth
drift/shared.rs decides maintenance truth separately
closeout validation inherits drift semantics
local preflight and CI disagree about what "green" means

M5
one shared capability projection contract
one explicit primary publication target
one check-only capability freshness gate
one seeded-agent parity suite proving publication and maintenance truth match

12-MONTH IDEAL
new enrolled agents consume the same projection contract automatically
publication truth cannot fork from maintenance truth without a failing test
local and CI green are the same predicate
```

#### Architecture Review
##### Preferred Module Shape
Keep M5 inside the existing `xtask` crate and make the new code boring:

- `crates/xtask/src/main.rs`
  - thin CLI routing only; extend the existing `CapabilityMatrix` command surface rather than adding a second entrypoint
- `crates/xtask/src/agent_registry.rs`
  - own the explicit primary publication-target contract and `config_key` fail-closed validation
- one small shared capability-projection helper inside `crates/xtask/src/`
  - reusable by publication, drift, and closeout truth; one implementation, no per-caller forks
- `crates/xtask/src/capability_matrix.rs`
  - keep markdown rendering plus the new check-only freshness path
- `crates/xtask/src/agent_maintenance/drift/shared.rs`
  - consume the shared projection helper instead of restating config-gated semantics
- `crates/xtask/src/agent_maintenance/closeout/validate.rs`
  - continue to validate live drift truth, but only after M5 removes the semantic fork upstream

If the shared helper stays small, one file is enough. Do not build a mini-framework around capability truth.

##### Dependency Graph
```text
crates/xtask/data/agent_registry.toml
                |
                v
       crates/xtask/src/agent_registry.rs
                |
                v
  shared capability projection contract
  - declared registry truth
  - explicit primary publication target
  - built-in default runtime truth
  - manifest availability truth
                |
        +-------+-------------------+
        |                           |
        v                           v
crates/xtask/src/            crates/xtask/src/agent_maintenance/
capability_matrix.rs         drift/shared.rs
render + --check             check-agent-drift
        |                           |
        v                           v
docs/specs/unified-agent-api/   closeout/validate.rs
capability-matrix.md            validate_live_drift_truth
        |
        v
Makefile preflight + CI capability gate
```

##### Architecture Decisions
- One shared capability projection contract is the whole game. Publication truth, drift truth, and closeout truth must stop carrying duplicate policy.
- The primary capability-publication target must become explicit registry-owned truth. Depending on `canonical_targets[0]` is a hidden control-plane contract and is no longer acceptable after M5.
- `cargo run -p xtask -- capability-matrix --check` owns freshness. `cargo run -p xtask -- capability-matrix-audit` remains a second semantic guard, not a replacement for freshness checks.
- Local verification must be check-only. Any write-then-diff design would self-heal drift and make local green less trustworthy than CI.
- M5 must not mutate runtime-owned code to make drift disappear. If runtime and control-plane truth disagree, the control plane must fail closed and say so.

#### Code Quality Guardrails
- Keep `main.rs` as routing glue. Real logic belongs in registry, projection, publication, and maintenance modules.
- Do not keep two projection helpers. If two callers need the same capability truth, extract one small shared helper and delete the duplicate policy.
- Keep the new publication-target and `config_key` validation explicit and fail closed. Silent fallback is the bug class M5 is supposed to remove.
- Compare generated capability publication in memory for `--check`; do not write files during verification just to discover staleness.
- Extend the existing targeted xtask regression files before creating new ones. Minimal diff and obvious test ownership win here.
- If M5 changes any nearby ASCII diagram comments in code or docs, update them in the same change. Stale diagrams are worse than no diagrams.

#### Workstreams
##### W1. Canonical Capability Projection Contract
Goal: remove semantic duplication at the source.

Deliverables:
- one shared capability projection helper reused by publication, drift, and closeout truth
- one explicit primary capability-publication target contract in registry-owned data
- one explicit `config_key` allowlist for config-gated capability declarations

Primary modules:
- `crates/xtask/src/agent_registry.rs`
- `crates/xtask/data/agent_registry.toml`
- one shared projection helper under `crates/xtask/src/`
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/tests/c8_capability_matrix_unit.rs`
- `crates/xtask/tests/agent_registry.rs`

Exit criteria:
- publication truth no longer depends on hidden target ordering
- registry declarations cannot silently introduce unknown config gates

##### W2. Maintenance Drift + Closeout Parity
Goal: make maintenance truth consume the same projection contract as publication truth.

Deliverables:
- `check-agent-drift` stops overclaiming default-off config-gated capabilities for seeded agents
- maintenance closeout continues to re-check live drift truth, but now through the unified projection model
- seeded parity coverage proves `codex`, `claude_code`, `opencode`, and `gemini_cli` are all clean on a clean repo

Primary modules:
- `crates/xtask/src/agent_maintenance/drift/shared.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`
- `crates/xtask/src/agent_maintenance/closeout/validate.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `crates/xtask/tests/agent_maintenance_closeout.rs`

Exit criteria:
- maintenance truth and publication truth agree for all seeded agents under default built-in configs

##### W3. Gate Unification
Goal: make local and CI green mean the same thing.

Deliverables:
- `cargo run -p xtask -- capability-matrix --check`
- `make preflight` wiring for capability publication freshness
- CI wiring that uses the same freshness command and keeps `capability-matrix-audit` as the semantic companion gate

Primary modules:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/capability_matrix.rs`
- `Makefile`
- `.github/workflows/ci.yml`
- `crates/xtask/tests/c8_spec_capability_matrix_paths.rs`

Exit criteria:
- maintainers can trust local green because it now evaluates the same capability-publication contract as CI

##### W4. Regression Coverage + Operator Truth
Goal: make this bug class hard to reintroduce and easy to explain.

Deliverables:
- regression tests for default-off config-gated publication, missing primary publication target, unknown `config_key`, target-order churn, and stale publication checks
- one updated M5 plan-of-record plus narrow normative notes that name the authoritative truth sources and the authoritative green gate

Primary modules:
- `PLAN.md`
- `docs/specs/agent-registry-contract.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `crates/xtask/tests/c8_capability_matrix_unit.rs`
- `crates/xtask/tests/agent_registry.rs`
- `crates/xtask/tests/agent_maintenance_drift.rs`
- `crates/xtask/tests/agent_maintenance_closeout.rs`

Exit criteria:
- the next enrolled agent cannot reopen this bug class without failing an obvious test or contract check

#### Implementation Sequence
##### Phase 1. Projection Contract Lock
Outputs:
- shared capability projection helper
- explicit primary capability-publication target contract
- explicit `config_key` allowlist validation

Modules touched:
- `crates/xtask/src/agent_registry.rs`
- `crates/xtask/data/agent_registry.toml`
- one shared projection helper under `crates/xtask/src/`
- `crates/xtask/src/capability_matrix.rs`
- targeted registry and capability-matrix tests

Implementation notes:
- lock the truth model first because every later phase consumes it
- prefer explicit registry-owned metadata over clever inference from target ordering
- keep this phase inside the control-plane seam only

Exit gate:
- publication truth is defined once and can be reused without call-site policy forks

##### Phase 2. Maintenance Truth Parity
Outputs:
- drift/shared parity fix
- closeout validation still grounded in live drift truth
- seeded-agent clean proof for all four current capability-matrix-enrolled agents

Modules touched:
- `crates/xtask/src/agent_maintenance/drift/shared.rs`
- `crates/xtask/src/agent_maintenance/drift/publication.rs`
- `crates/xtask/src/agent_maintenance/closeout/validate.rs`
- maintenance drift and closeout tests

Implementation notes:
- delete the duplicate config-gated policy instead of teaching it the same rule twice
- keep the proving signal user-facing: a maintainer should be able to run `check-agent-drift --agent <id>` and believe the result

Exit gate:
- maintenance truth and publication truth no longer disagree on seeded default capability advertising

##### Phase 3. Gate Parity
Outputs:
- `capability-matrix --check`
- `make preflight` capability freshness gate
- CI capability freshness gate aligned with local preflight

Modules touched:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/capability_matrix.rs`
- `Makefile`
- `.github/workflows/ci.yml`
- capability-matrix entrypoint/path tests

Implementation notes:
- compare generated output in memory and fail on drift; do not write during verification
- keep freshness and orthogonality separate, but run both in both environments

Exit gate:
- local green and CI green are the same predicate for capability publication

##### Phase 4. Regression Coverage + Narrow Contract Notes
Outputs:
- final regression suite
- updated plan/status language
- updated narrow normative notes for registry and capability publication truth

Modules touched:
- targeted xtask tests
- `PLAN.md`
- `docs/specs/agent-registry-contract.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`

Implementation notes:
- land docs after the code contract is final so the notes name the real gate and the real truth sources
- keep the documentation touch small; M7 owns broad cleanup

Exit gate:
- a new maintainer can explain the authoritative capability truth and the authoritative green gate without reading repo history

#### Error & Rescue Registry
| Method / Codepath | What can go wrong | Failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| registry parse | primary publication target missing or invalid | validation error | yes | reject registry load before generation or drift work | explicit validation failure |
| registry config-gated declaration | unknown `config_key` is accepted | schema drift | yes | fail closed on registry validation | explicit validation failure |
| projection helper | config-gated capability is treated as always-on in one caller only | semantic fork | yes | route all callers through one helper and fail tests if they diverge | false clean or false drift disappears |
| `capability-matrix --check` | stale markdown is silently rewritten during verification | gate integrity bug | yes | compare in memory and fail without mutation | non-zero check-only failure |
| `check-agent-drift --agent` | seeded agent reports false-positive capability drift | maintenance truth bug | yes | reuse shared projection helper and keep targeted seeded parity tests | non-zero drift report on a clean repo |
| maintenance closeout | resolved findings still match live drift truth | closeout truth bug | yes | reject closeout until live drift is actually clean | closeout validation failure |
| local vs CI gate | one environment runs freshness and the other does not | operator trust gap | yes | wire the same freshness plus audit contract into both | green in one place, red in the other |

#### Test Strategy
##### Test Diagram
```text
CANONICAL CAPABILITY PROJECTION
==============================
[+] registry declaration
    |
    +--> explicit primary publication target
    +--> declared registry truth
    +--> built-in default runtime truth
    +--> manifest availability truth
    |
    +--> [GAP -> validation] unknown config_key fails closed
    +--> [GAP -> validation] missing primary publication target fails closed
    +--> [GAP -> regression] target-order churn cannot silently change published truth

PUBLICATION FRESHNESS
=====================
[+] capability-matrix --check
    |
    +--> [GAP -> regression] stale markdown fails without mutating the worktree
    +--> [GAP -> integration] make preflight and CI both call the same freshness rule
    +--> [GAP -> integration] capability-matrix-audit stays paired with freshness locally and in CI

MAINTENANCE TRUTH
=================
[+] check-agent-drift --agent codex / claude_code / opencode / gemini_cli
    |
    +--> [GAP -> regression] seeded agents are clean on a clean repo
    +--> [GAP -> regression] codex and claude_code do not advertise default-off write/sandbox capability drift

[+] validate_live_drift_truth
    |
    +--> [GAP -> regression] closeout validation consumes the same projection semantics as publication and drift
```

##### Required Test Surfaces
- Extend `crates/xtask/tests/c8_capability_matrix_unit.rs`
  - explicit primary publication target selection drives projection
  - config-gated MCP write and external sandbox capabilities stay default-off in publication truth
  - target-order churn does not change the published result
- Extend `crates/xtask/tests/agent_registry.rs`
  - unknown `config_key` is rejected
  - missing or invalid primary publication target is rejected
- Extend `crates/xtask/tests/agent_maintenance_drift.rs`
  - `check_agent_drift_reports_clean_agent` covers `codex`, `claude_code`, `opencode`, and `gemini_cli`
  - false-positive capability drift for config-gated capability ids is gone
- Extend `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `validate_live_drift_truth` rejects stale truth using the unified projection model
- Extend `crates/xtask/tests/c8_spec_capability_matrix_paths.rs`
  - `capability-matrix --check` fails on stale markdown and does not rewrite it

##### Verification Commands
- `cargo run -p xtask -- check-agent-drift --agent codex`
- `cargo run -p xtask -- check-agent-drift --agent claude_code`
- `cargo run -p xtask -- check-agent-drift --agent opencode`
- `cargo run -p xtask -- check-agent-drift --agent gemini_cli`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `cargo test -p xtask --test c8_capability_matrix_unit`
- `cargo test -p xtask --test agent_maintenance_drift`
- `cargo test -p xtask --test agent_maintenance_closeout`
- `cargo test -p xtask --test agent_registry`
- `make preflight`

##### Test Plan Artifact
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-staging-test-plan-20260423-065703.md`

#### Failure Modes Registry
| Codepath | Failure mode | Test required? | Error handling required? | User sees | Logged? |
|---|---|---|---|---|---|
| projection helper | config-gated capability is published as always-on | yes | yes | false clean or false drift | yes |
| registry validation | typoed or unsupported `config_key` is accepted | yes | yes | control plane claims a gate runtime does not own | yes |
| publication target selection | target ordering changes truth without an explicit contract change | yes | yes | silent publication churn | yes |
| local preflight | freshness verification rewrites files instead of only checking | yes | yes | self-healing green gate | yes |
| maintenance closeout | live drift re-check accepts stale capability truth | yes | yes | maintenance pack can close while factory still lies | yes |
| CI/local mismatch | CI remains stricter than local after M5 | yes | yes | maintainers cannot trust local green | yes |

Critical gap rule:
- if capability publication, drift truth, and closeout truth still carry separate semantics after M5, the milestone failed
- if local and CI capability publication checks still differ after M5, the milestone failed

#### Security Review
- registry-owned capability gates must fail closed; unknown `config_key` values are not harmless metadata
- check-only publication verification must not mutate the worktree, because self-healing verification hides drift
- maintenance closeout must continue to distrust packet prose over live control-plane truth
- M5 must not create a path where runtime-owned code is mutated to make control-plane drift disappear

#### Performance Review
- capability publication freshness should compare generated output in memory, not write and re-read files
- drift and closeout validation should reuse the same projection helper instead of re-parsing policy in multiple places
- seeded parity coverage should stay targeted so the repo gets signal quickly without turning M5 into a full-workspace retest tax

#### Worktree Parallelization Strategy
##### Dependency Table
| Step | Modules touched | Depends on |
|---|---|---|
| W1. projection contract | `crates/xtask/src/agent_registry.rs`, `crates/xtask/data/agent_registry.toml`, one shared projection helper under `crates/xtask/src/`, `crates/xtask/src/capability_matrix.rs`, targeted registry/publication tests | — |
| W2. maintenance parity | `crates/xtask/src/agent_maintenance/drift/**`, `crates/xtask/src/agent_maintenance/closeout/validate.rs`, maintenance parity tests | W1 |
| W3. gate parity | `crates/xtask/src/main.rs`, `crates/xtask/src/capability_matrix.rs`, `Makefile`, `.github/workflows/ci.yml`, capability-matrix path tests | W1 |
| W4. docs and contract notes | `PLAN.md`, `docs/specs/agent-registry-contract.md`, `docs/specs/unified-agent-api/capabilities-schema-spec.md` | W2, W3 |

##### Parallel Lanes
Lane A: W1 -> W3
Core publication lane. Keep these sequential because both steps touch `crates/xtask/src/capability_matrix.rs` and the command-entry surface.

Lane B: W2
Maintenance parity lane. Launch this after W1 freezes the shared helper signature and registry truth shape.

Lane C: W4
Docs lane. Run this only after W2 and W3 merge so the written contract names the final gate and the final truth sources.

##### Execution Order
1. Launch Lane A first. W1 defines the contract every later step consumes.
2. After W1 merges, launch Lane B and the W3 part of Lane A in parallel worktrees.
3. Merge W2 and W3.
4. Run Lane C last as a docs-only closeout once the code contract is stable.

##### Conflict Flags
- W1 and W3 both touch `crates/xtask/src/capability_matrix.rs`. Do not parallelize them.
- W2 must stay inside `agent_maintenance/**` plus its tests after W1 lands. If W2 starts editing publication entrypoints again, it will collide with Lane A.
- W4 must stay docs-only. If the docs lane starts changing code, it loses its parallel safety.

#### Completion Summary
- Step 0: scope challenge accepted as-is, M6 kept separate, and the explicit primary publication-target fix is now in scope.
- Architecture review: one projection contract, one explicit publication-target contract, one freshness gate, and no runtime-owned mutation.
- Code quality review: fail-closed registry validation, no duplicate policy helpers, and check-only verification are non-negotiable.
- Test review: diagram produced, 8 required coverage points locked across projection truth, maintenance parity, and gate parity.
- Performance review: in-memory freshness comparison and helper reuse keep the change boring and cheap.
- Not in scope: written.
- What already exists: written.
- Failure modes: two critical gates remain non-negotiable, semantic duplication must disappear and local green must equal CI green.
- Parallelization: three lanes total, one real parallel window after W1, docs stay last by design.

#### Deferred To TODOS.md
- explicit follow-on decision about whether `docs/specs/unified-agent-api/capability-matrix.md` should remain the long-term canonical published truth surface after M5 lands

### M6. Separate Wrapper Scaffold Command
Status:
- next implementation milestone after M5 lands cleanly and the factory-truth gate stays green
- no UI scope; design review stays skipped unless the operator workflow later gains a human-facing surface

Goal:
- turn "approved and enrolled agent" into a publishable wrapper-crate shell through one explicit runtime-lane command, without changing what `xtask onboard-agent` means

#### Purpose
M5 makes the control plane trustworthy. M6 should make the first runtime step boring.

Right now `xtask onboard-agent` can enroll the registry entry, docs pack, manifest root, workspace membership, and release-touchpoint docs, then the workflow drops the maintainer into manual wrapper-crate bootstrapping. Gemini already showed the cost of that gap: the control plane can be green while the wrapper crate still misses crate-local packaging surfaces such as `README.md`, `LICENSE-APACHE`, `LICENSE-MIT`, and `readme = "README.md"` in `Cargo.toml`.

M6 closes that gap, but it should not do it by quietly widening `onboard-agent` into an everything command. The repo already encodes a control-plane versus runtime-owned boundary, and M6 should preserve that boundary while making the runtime lane start from a valid, publishable shell instead of an empty directory.

#### Problem Statement
The current repo state already tells us where the missing step lives:

- `crates/xtask/src/onboard_agent.rs` owns control-plane enrollment and today writes the registry/docs/manifest/release surfaces only.
- `crates/xtask/src/onboard_agent/preview/render.rs` and `crates/xtask/src/onboard_agent/preview.rs` explicitly describe the next runtime step as "implement the runtime-owned wrapper crate" and still treat crate-local publishability files as later manual follow-up.
- `crates/xtask/tests/onboard_agent_entrypoint/help_and_preview.rs` explicitly guards against `onboard-agent` preview text becoming "Create the wrapper crate".
- `crates/xtask/tests/onboard_agent_entrypoint/write_mode.rs` locks in the current control-plane write set and the current `15 total planned` replay semantics.
- `docs/project_management/next/cli-agent-onboarding-charter.md` still starts the onboarding checklist with manual wrapper-crate creation.

So the actual M6 problem is not "should the repo scaffold wrapper shells?" The answer is yes. The problem is command shape and ownership: how do we add deterministic wrapper-shell scaffolding without collapsing approval artifacts, control-plane writes, runtime-owned code, and publishability templates into one overloaded `onboard-agent` contract?

#### Landed Baseline
These are already true on `feat/fill-trust-gap` and are not M6 work:

- `xtask onboard-agent` exists as the create-mode control-plane bridge for new agents.
- `onboard-agent` already inserts the wrapper crate path into workspace membership and release docs, and it already emits the onboarding pack that points the maintainer at the runtime-owned next step.
- `crates/xtask/src/workspace_mutation.rs` already gives the repo jailed, replay-safe mutation primitives with fail-closed semantics.
- `crates/xtask/src/agent_registry.rs` already owns the canonical `crate_path`, `package_name`, release-track, and onboarding-pack metadata that a scaffold command should consume.
- `crates/gemini_cli/`, `crates/opencode/`, `crates/codex/`, and `crates/claude_code/` already show the target wrapper-crate package surfaces that a minimal scaffold should match.
- publish guards already exist and now fail earlier, so M6 can target the missing crate shell instead of adding yet another late-stage publish check.

#### Step 0. Scope Challenge
- Existing code leverage: M6 should reuse `main.rs` command routing, `agent_registry` as the read-only input contract, `workspace_mutation` for bounded writes, and the existing onboard-agent preview/render surfaces for downstream packet wording. It should not invent a second registry, a second approval artifact, or an ad hoc file writer.
- Minimum complete change: one new `xtask` subcommand, one narrow runtime file write set at the registry-owned `crate_path` under `crates/`, one docs wording update so the onboarding packet points at the new command, and one test harness proving replay, divergence protection, and workspace validity.
- Complexity check: the likely blast radius is about 8 to 12 files across `crates/xtask/src/**`, `crates/xtask/tests/**`, `PLAN.md`, and the onboarding charter. That is acceptable because every touched file belongs to one onboarding-to-runtime seam.
- Search check: reuse the repo's existing `onboard-agent` preview/write split and `workspace_mutation` rollback semantics instead of inventing bespoke template plumbing. Use the existing wrapper crates as template evidence instead of introducing a generic package DSL.
- Completeness check: a partial M6 that only writes README/license files is not enough. The scaffold must also write `src/lib.rs` and a valid `Cargo.toml`, or the workspace remains structurally broken after `onboard-agent` has already added the member to root `Cargo.toml`.
- Distribution check: M6 adds no new binary and no new release rail. It is a subcommand inside `xtask`.

#### Scope Lock
In scope:
- add a separate `xtask` subcommand for wrapper-shell scaffolding, with `--dry-run` and `--write` modes
- make the command registry-driven: input should be the enrolled agent id, not a second descriptor CLI surface and not a second approval-artifact parser
- generate the minimal publishable wrapper-crate shell at the registry-owned `crate_path` under `crates/`:
  - `Cargo.toml`
  - `README.md`
  - `LICENSE-APACHE`
  - `LICENSE-MIT`
  - `src/lib.rs`
- reuse repo-owned license text and current wrapper-crate conventions so the shell matches the already-landed crates closely enough to be boring
- fail closed on unknown agents, path escapes, partial writes, or divergent pre-existing runtime files
- update onboarding packet preview/render wording so the "next executable runtime step" points at the new scaffold command instead of manual crate creation
- update `docs/project_management/next/cli-agent-onboarding-charter.md` so the onboarding checklist reflects the new explicit wrapper-shell step
- add fixture-driven tests for preview, write, identical replay, divergent replay, and workspace-valid generated shells

#### Not In Scope
- widening `xtask onboard-agent` to write runtime-owned wrapper files directly
- backend module scaffolding under `crates/agent_api/src/backends/<agent>/`
- manifest evidence generation under `cli_manifests/<agent>/**`
- runtime probe logic, backend-specific CLI quirks, or fake-binary implementation work
- maintenance-lane changes under `check-agent-drift`, `refresh-agent`, or `close-agent-maintenance`
- registry schema expansion unless a concrete missing field proves it is necessary
- automatic chaining where `onboard-agent` implicitly invokes the new scaffold command

#### Success Criteria
M6 is complete only when all of these are true:

- `cargo run -p xtask -- scaffold-wrapper-crate --agent <agent> --dry-run` exists, reads the enrolled agent from `crates/xtask/data/agent_registry.toml`, and previews the exact wrapper-shell file set without mutating the worktree
- `cargo run -p xtask -- scaffold-wrapper-crate --agent <agent> --write` writes the wrapper-shell file set at the enrolled agent's registry-owned `crate_path` and is idempotent on identical replay
- replaying the command against divergent pre-existing wrapper-shell files fails without partial writes
- the scaffold command rejects unknown agents, missing registry-owned paths, and symlink/path-escape attempts with validation exit `2`
- the scaffold command does not mutate registry/docs/manifest/publication surfaces owned by `onboard-agent`
- the generated `Cargo.toml` includes the minimum publishability metadata needed to avoid the Gemini failure class, including `readme = "README.md"` and crate-local dual-license surfaces
- the generated wrapper shell is structurally valid Rust workspace content, including `src/lib.rs`, so targeted `cargo check -p <package>` can succeed once the shell lands
- `crates/xtask/src/onboard_agent/preview.rs`, `crates/xtask/src/onboard_agent/preview/render.rs`, and their tests no longer describe manual wrapper-crate creation as the immediate next step
- `docs/project_management/next/cli-agent-onboarding-charter.md` reflects the new explicit wrapper-shell step without rewriting the broader control-plane/runtime boundary

#### What Already Exists
| Sub-problem | Existing code to reuse | Why it matters |
|---|---|---|
| control-plane enrollment | `crates/xtask/src/onboard_agent.rs` | already owns registry/docs/manifest/release writes and defines the boundary M6 must not silently widen |
| path-jail and rollback-safe writes | `crates/xtask/src/workspace_mutation.rs` | already gives the repo the right fail-closed mutation primitive for runtime-shell scaffolding |
| enrolled agent lookup | `crates/xtask/src/agent_registry.rs` | already owns `crate_path`, `package_name`, release track, and scaffold metadata, so M6 should read from it instead of re-asking the operator |
| onboarding packet wording | `crates/xtask/src/onboard_agent/preview.rs`, `crates/xtask/src/onboard_agent/preview/render.rs` | already define the maintainer-facing next-step contract and must be updated so the workflow stays truthful |
| wrapper package examples | `crates/gemini_cli/**`, `crates/opencode/**`, `crates/codex/**`, `crates/claude_code/**` | already show the minimal crate layout, README shape, dual-license surfaces, and publishable package metadata |
| entrypoint test harness | `crates/xtask/tests/onboard_agent_entrypoint/*.rs`, `crates/xtask/tests/support/onboard_agent_harness.rs` | already prove dry-run/write/replay/divergence semantics and can be extended instead of inventing a new test style |
| onboarding workflow charter | `docs/project_management/next/cli-agent-onboarding-charter.md` | already captures the operator workflow and is the narrow doc surface that must stop implying manual shell creation |

#### Chosen Approach
M6 should add one explicit command:

`cargo run -p xtask -- scaffold-wrapper-crate --agent <agent_id> --dry-run|--write`

That is the smallest complete fix.

It preserves `onboard-agent` as the create-mode control-plane enrollment step. It gives the repo one place to own publishable wrapper-shell scaffolding. It avoids threading crate template choices back through approval artifacts or widening the current `15 total planned` control-plane mutation contract into a mixed control-plane/runtime write set.

The command should be registry-driven, not descriptor-driven. Once the agent is enrolled, the control plane already knows the `crate_path`, `package_name`, release track, and pack prefix. Re-asking for those fields would duplicate truth and create a second drift surface.

#### Dream State Delta
```text
CURRENT
onboard-agent enrolls control-plane surfaces
packet tells maintainer to create wrapper crate manually
crate-local publishability files can be forgotten until late

M6
scaffold-wrapper-crate writes a minimal publishable wrapper shell from registry truth
onboarding packet points at the new command
runtime lane starts from a valid package shell instead of a blank directory

12-MONTH IDEAL
approved agent
  -> onboard-agent
  -> scaffold-wrapper-crate
  -> backend implementation
  -> manifest evidence generation
  -> make preflight
no ownership confusion and no late publish-surface surprises
```

#### Architecture Review
##### Preferred Module Shape
Keep M6 inside the existing `xtask` crate and keep the code boring:

- `crates/xtask/src/main.rs`
  - add the new subcommand and route it like the other `xtask` entrypoints
- `crates/xtask/src/lib.rs`
  - export the new scaffold module for library-driven tests
- `crates/xtask/src/wrapper_scaffold.rs`
  - own command args, registry lookup, write planning, and top-level run logic
- optional narrow helper splits under `crates/xtask/src/wrapper_scaffold/`
  - `preview.rs` for dry-run rendering
  - `validation.rs` for fail-closed agent/path/file checks
- `crates/xtask/src/agent_registry.rs`
  - remain the read-only source of enrolled-agent metadata
- `crates/xtask/src/workspace_mutation.rs`
  - remain the only write primitive for runtime-shell file creation
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/src/onboard_agent/preview/render.rs`
  - update packet wording so the next runtime step is "run scaffold-wrapper-crate, then implement backend/runtime details"

If one file is enough for the command, keep it in one file. Do not build a mini templating framework for five files.

##### Dependency Graph
```text
crates/xtask/data/agent_registry.toml
                |
                v
     crates/xtask/src/agent_registry.rs
                |
                v
  scaffold-wrapper-crate command input
  - crate_path
  - package_name
  - display_name
  - release metadata
                |
                v
     crates/xtask/src/wrapper_scaffold.rs
                |
        +-------+------------------+
        |                          |
        v                          v
workspace_mutation.rs      preview/validation helpers
        |                          |
        v                          v
registry-owned crate_path/**          stdout preview + fail-closed checks
                |
                v
onboard-agent packet wording updates
```

##### Architecture Decisions
- `onboard-agent` remains control-plane-only. M6 must not quietly rewrite its meaning.
- the new scaffold command should take `--agent <agent_id>`, not a second descriptor surface and not `--approval`
- registry metadata is enough input for M6; the scaffold command should not grow new committed truth unless the current registry is provably insufficient
- `src/lib.rs` is required, not optional. A crate shell that still leaves the workspace structurally invalid is not a real scaffold
- M6 should not touch backend modules, manifest evidence, or publication artifacts. Those stay downstream runtime work
- dry-run and write mode semantics must mirror the rest of `xtask`: preview first, no hidden writes, deterministic replay, fail closed on divergence

#### Code Quality Guardrails
- keep the file templates explicit and small; five obvious files beat a generic package-scaffold abstraction
- read from registry truth once and pass a narrow typed scaffold plan through the command
- fail closed on divergent files instead of trying to merge maintainer-written runtime code
- keep output ownership obvious: the new command owns only the registry-owned `crate_path/**` package-shell files, and `onboard-agent` keeps owning registry/docs/manifest/release surfaces
- update preview/help tests in the same change as the packet wording so the repo never publishes stale workflow instructions

#### Error & Rescue Registry
| Condition | Detection surface | User-visible failure | Auto-recovery | Maintainer action |
|---|---|---|---|---|
| agent id is not enrolled | registry lookup in scaffold command | validation error, exit `2` | no | run `onboard-agent` first or fix the agent id |
| `crate_path` escapes the workspace or hits a symlink | `workspace_mutation` path jail | validation error, exit `2` | no | fix registry/path ownership before retrying |
| wrapper-shell file already exists with divergent contents | planned mutation compare | divergent replay error, no writes | no | inspect runtime-owned edits, then rerun only if overwrite is intentional |
| scaffold writes README/licenses but not valid Rust sources | targeted `cargo check -p <package>` or fixture validation | broken workspace/package shell | no | M6 must include `src/lib.rs` and valid `Cargo.toml`; otherwise the milestone failed |
| onboarding packet still says "implement the wrapper crate manually" | preview/help tests and packet diff review | stale operator workflow | yes, via same M6 change | update `onboard-agent` render/preview text and tests |
| scaffold command starts mutating registry/docs/publication surfaces | write-mode tests | ownership-boundary regression | yes, via test failure | keep those writes in `onboard-agent`; remove them from scaffold command |

#### Test Review
##### Test Diagram
```text
WRAPPER SCAFFOLD COMMAND
========================
[+] cargo run -p xtask -- scaffold-wrapper-crate --agent <id> --dry-run
    |
    +--> registry lookup
    |    +--> [GAP -> validation] unknown agent exits 2
    |    \--> [GAP -> validation] missing or invalid crate_path exits 2
    |
    +--> scaffold plan build
    |    +--> [GAP -> preview] exact five-file plan renders in stable order
    |    \--> [GAP -> validation] path-jail and symlink rejection happen before any writes
    |
    \--> preview output only
         \--> [GAP -> regression] worktree stays unchanged

WRITE MODE
==========
[+] cargo run -p xtask -- scaffold-wrapper-crate --agent <id> --write
    |
    +--> apply the same in-memory plan used by --dry-run
    |    +--> [GAP -> regression] first write creates Cargo.toml, README.md, dual licenses, and src/lib.rs
    |    +--> [GAP -> regression] identical replay is a no-op
    |    \--> [GAP -> regression] divergent existing file fails without partial writes
    |
    \--> ownership boundary stays narrow
         \--> [GAP -> regression] no writes escape the registry-owned crate_path/**

WORKFLOW RE-CONTRACT
====================
[+] onboard-agent preview + charter
    |
    +--> [GAP -> regression] preview and handoff now point at scaffold-wrapper-crate
    \--> [GAP -> regression] onboarding charter names the explicit wrapper-shell step

STRUCTURAL VALIDITY
===================
[+] generated wrapper shell
    |
    +--> [GAP -> integration] Cargo.toml includes readme and dual-license metadata
    +--> [GAP -> integration] src/lib.rs exists and compiles as a minimal crate shell
    \--> [GAP -> integration] targeted cargo check -p <package> succeeds in a fixture workspace
```

##### Required Test Surfaces
- Add `crates/xtask/tests/wrapper_scaffold_entrypoint.rs`
  - `scaffold_wrapper_crate_dry_run_previews_exact_file_set`
  - `scaffold_wrapper_crate_write_creates_minimal_publishable_shell`
  - `scaffold_wrapper_crate_replay_is_noop`
  - `scaffold_wrapper_crate_divergent_file_fails_without_partial_writes`
- Add `crates/xtask/tests/wrapper_scaffold_validation.rs`
  - `scaffold_wrapper_crate_rejects_unknown_agent`
  - `scaffold_wrapper_crate_rejects_path_escape_or_symlinked_crate_path`
- Update `crates/xtask/tests/onboard_agent_entrypoint/help_and_preview.rs`
  - assert the next runtime step names `scaffold-wrapper-crate`
  - keep the guard that `onboard-agent` itself does not create the wrapper crate
- Add one fixture-backed structural validity test
  - either in `wrapper_scaffold_entrypoint.rs` or a narrow companion file
  - assert the generated shell passes targeted `cargo check -p <package>` without requiring backend implementation

##### Verification Commands
- `cargo test -p xtask --test wrapper_scaffold_entrypoint`
- `cargo test -p xtask --test wrapper_scaffold_validation`
- `cargo test -p xtask --test onboard_agent_entrypoint`
- `cargo check -p xtask`
- `make preflight`

##### Test Plan Artifact
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-fill-trust-gap-eng-review-test-plan-20260423-152745.md`

#### Workstreams
##### W1. Scaffold Command Skeleton
Goal: add the command without changing current control-plane ownership.

Deliverables:
- new `xtask` subcommand wired through `crates/xtask/src/main.rs`
- registry-driven agent lookup and validation
- dry-run and write mode surfaces with deterministic preview text

Primary modules:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/wrapper_scaffold.rs`
- optional `crates/xtask/src/wrapper_scaffold/{preview,validation}.rs`

Exit criteria:
- command exists
- unknown agents fail closed
- dry-run does not write

##### W2. Runtime Shell File Generation
Goal: generate the minimal publishable wrapper shell and nothing more.

Deliverables:
- `Cargo.toml`
- `README.md`
- `LICENSE-APACHE`
- `LICENSE-MIT`
- `src/lib.rs`

Primary modules:
- `crates/xtask/src/wrapper_scaffold.rs`
- `crates/xtask/src/workspace_mutation.rs`
- wrapper-shell fixture tests

Exit criteria:
- identical replay is a no-op
- divergent replay fails without partial writes
- generated shell is structurally valid for targeted `cargo check -p <package>`

##### W3. Workflow Re-Contracting
Goal: make the operator-facing workflow truthful once the new command exists.

Deliverables:
- onboarding packet preview/handoff wording that points at `scaffold-wrapper-crate`
- onboarding charter checklist update
- tests that guard the new wording and preserve the control-plane/runtime boundary

Primary modules:
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/src/onboard_agent/preview/render.rs`
- `crates/xtask/tests/onboard_agent_entrypoint/help_and_preview.rs`
- `docs/project_management/next/cli-agent-onboarding-charter.md`

Exit criteria:
- no onboarding preview text still implies the first runtime step is manual crate creation
- the charter shows the explicit wrapper-shell step without widening `onboard-agent`

#### Implementation Sequence
##### Phase 1. Command Contract Lock
Outputs:
- `scaffold-wrapper-crate` CLI surface wired through `crates/xtask/src/main.rs`
- registry-driven agent lookup and validation contract
- deterministic `--dry-run` preview text

Modules touched:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/lib.rs`
- `crates/xtask/src/wrapper_scaffold.rs`
- optional `crates/xtask/src/wrapper_scaffold/{preview,validation}.rs`

Implementation notes:
- lock the command shape first so later work does not relitigate ownership
- keep the command registry-driven and fail closed on unknown agents or invalid crate paths
- make `--dry-run` the authoritative render path; `--write` should only apply that exact plan

Exit gate:
- the repo has one explicit runtime-shell scaffold entrypoint and it still preserves the `onboard-agent` boundary

##### Phase 2. Shell Generation + Replay Safety
Outputs:
- the minimal five-file wrapper shell
- identical replay no-op
- divergent replay failure without partial writes

Modules touched:
- `crates/xtask/src/wrapper_scaffold.rs`
- `crates/xtask/src/workspace_mutation.rs`
- `crates/xtask/tests/wrapper_scaffold_entrypoint.rs`
- `crates/xtask/tests/wrapper_scaffold_validation.rs`

Implementation notes:
- keep file templates explicit and small
- use one mutation plan for both preview and write mode
- validate all file targets before the first write so replay failures stay atomic

Exit gate:
- a newly enrolled agent can receive a valid package shell without risking runtime-owned overwrite

##### Phase 3. Workflow Re-Contract + Proof
Outputs:
- updated onboarding preview and handoff wording
- updated onboarding charter checklist
- fixture proof that the generated shell passes targeted `cargo check -p <package>`

Modules touched:
- `crates/xtask/src/onboard_agent/preview.rs`
- `crates/xtask/src/onboard_agent/preview/render.rs`
- `crates/xtask/tests/onboard_agent_entrypoint/help_and_preview.rs`
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- the fixture-backed xtask test selected in Phase 2

Implementation notes:
- land wording updates only after the command contract is stable
- keep the docs touch narrow and ownership-specific
- use targeted fixture compilation as the proof that M6 closes the Gemini publishability gap instead of only changing prose

Exit gate:
- the operator-facing workflow is truthful and the scaffolded shell is mechanically valid

#### Performance Review
- Build the scaffold plan once per invocation and share it across `--dry-run` and `--write`. Two render trees for five files would be self-inflicted drift.
- Keep templates explicit and local. M6 should not copy whole example crates or walk the workspace beyond registry lookup and jailed file validation.
- Run targeted `cargo check -p <package>` in a fixture workspace for structural proof instead of full-workspace compilation inside every new test.
- Reuse `workspace_mutation` transaction semantics so divergence is detected before the first file write, not after half a crate shell lands.

#### Worktree Parallelization Strategy
##### Dependency Table
| Step | Modules touched | Depends on |
|---|---|---|
| W1. command contract lock | `crates/xtask/src/main.rs`, `crates/xtask/src/lib.rs`, `crates/xtask/src/wrapper_scaffold*` | — |
| W2. shell generation + replay safety | `crates/xtask/src/wrapper_scaffold*`, `crates/xtask/src/workspace_mutation.rs`, `crates/xtask/tests/wrapper_scaffold_*.rs` | W1 |
| W3. workflow re-contract | `crates/xtask/src/onboard_agent/preview*`, `crates/xtask/tests/onboard_agent_entrypoint/**`, `docs/project_management/next/cli-agent-onboarding-charter.md` | W1 |
| W4. final fixture proof | `crates/xtask/tests/wrapper_scaffold_*.rs`, fixture support under `crates/xtask/tests/support/**` | W2, W3 |

##### Parallel Lanes
Lane A: W1 -> W2
Core scaffold-command lane. Keep these sequential because W2 depends on the exact contract and file-plan shape W1 defines.

Lane B: W3
Workflow wording lane. Launch after W1 lands so docs point at the final command name and argument shape, not a moving target.

Lane C: W4
Final proof lane. Run this only after W2 and W3 merge because it validates both the generated shell and the operator-facing workflow.

##### Execution Order
1. Launch Lane A first. W1 sets the command seam and failure posture every later step consumes.
2. After W1 merges, launch W2 and Lane B in parallel worktrees.
3. Merge W2 and W3.
4. Run Lane C last as the fixture-backed proof pass against the merged command and wording.

##### Conflict Flags
- W1 and W2 both touch `crates/xtask/src/wrapper_scaffold*`. Do not parallelize them.
- W2 and W4 both likely touch `crates/xtask/tests/wrapper_scaffold_*.rs`. Split test ownership early if two worktrees are used.
- W3 must stay limited to `onboard_agent` preview surfaces plus the charter. If it starts changing scaffold command behavior, it collides with Lane A.

#### Failure Modes Registry
| Surface | Failure | Prevent in M6? | Detect in tests? | User impact | Blocker? |
|---|---|---|---|---|---|
| command boundary | `onboard-agent` silently starts writing runtime crate files | yes | yes | approval/control-plane semantics become ambiguous | yes |
| runtime shell | generated crate misses `src/lib.rs` or valid package metadata | yes | yes | workspace stays broken or publish checks fail late | yes |
| replay safety | scaffold overwrites maintainer-edited runtime files | yes | yes | user loses runtime work | yes |
| path safety | scaffold follows a symlink or escapes `crate_path` | yes | yes | unsafe write outside repo boundary | yes |
| operator docs | onboarding packet still points at manual wrapper creation | yes | yes | maintainers follow stale workflow | yes |
| scope creep | scaffold command starts owning backend or manifest work | yes | yes | one command becomes another overloaded lifecycle tool | yes |

#### Decision Audit Trail
| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | CEO | keep M6 as a separate scaffold command | mechanical | explicit over clever | the repo already encodes `onboard-agent` as control-plane-only and tests guard that boundary | widen `onboard-agent` |
| 2 | CEO | make the command registry-driven with `--agent` input | mechanical | DRY | enrolled metadata already exists in the registry and should not be re-entered | second descriptor CLI surface |
| 3 | Eng | require `src/lib.rs` in the scaffolded shell | mechanical | completeness | README/license files alone do not restore workspace validity | docs-only or package-only shell |
| 4 | Eng | keep backend modules and manifest evidence out of M6 | mechanical | boil lakes, not oceans | M6 should close the wrapper-shell gap, not absorb the entire runtime lane | backend scaffold or manifest generation in the same milestone |

#### Completion Summary
| Dimension | Verdict | Notes |
|---|---|---|
| Scope | ready | M6 is now bounded to one runtime-shell seam after M5, not a generic onboarding rewrite |
| Architecture | ready | separate `scaffold-wrapper-crate` command is the clean seam and preserves existing ownership |
| Tests | ready with explicit gate | dry-run/write/replay/divergence/path-safety, workflow wording coverage, and targeted `cargo check` are the minimum proof set |
| Performance | ready | one shared scaffold plan plus targeted fixture compilation keeps the lane boring and cheap |
| Docs | ready with narrow re-contract | only packet wording and the onboarding charter need contract updates if M6 stays registry-driven |
| Parallelization | ready | one real parallel window opens after W1, then fixture proof runs last |
| Deferred | none beyond M7 | broader docs cleanup and operator-guide consolidation remain M7 work |

### M7. Documentation And Legacy Cleanup
Status:
- next documentation milestone after M6 lands cleanly
- no UI scope; design review remains skipped

Goal:
- make the factory understandable to a maintainer who did not live through M1 through M6
- replace repo-archaeology onboarding with one canonical operator guide plus narrow cleanup of stale entry docs

#### Purpose
M5 makes factory truth reliable.
M6 makes the first runtime step boring.

After those land, the next bottleneck is not command surface. It is operator comprehension.

The repo already has the real workflow:
- `onboard-agent` for control-plane enrollment
- `scaffold-wrapper-crate` for the runtime-owned wrapper shell
- `check-agent-drift`, `refresh-agent`, and `close-agent-maintenance` for the maintenance lane
- `support-matrix --check`, `capability-matrix --check`, and `make preflight` for the green gate

What it does not yet have is one human-readable place that explains that chain end to end.

Right now the truth is scattered across `README.md`, `CONTRIBUTING.md`, `docs/README.md`, the onboarding charter, generated packet outputs, and historical planning packs. The commands are real, but the maintainer still has to reconstruct the workflow from repo history. That is the whole M7 problem.

#### Problem Statement
The current repo state is internally consistent enough to build and maintain agents, but not yet legible enough to operate without context from earlier milestones.

Concrete examples already visible on this branch:

- `crates/xtask/src/main.rs` exposes `onboard-agent`, `scaffold-wrapper-crate`, `check-agent-drift`, `refresh-agent`, and `close-agent-maintenance`, but the repo entry docs do not present them as one lifecycle.
- `docs/project_management/next/cli-agent-onboarding-charter.md` now correctly names `scaffold-wrapper-crate`, but it is a charter, not the day-to-day operator manual.
- `README.md` and `CONTRIBUTING.md` still mention `support-matrix --check`, but they do not explain the full green gate or the new factory command chain.
- `docs/README.md` still lacks a pointer to any canonical factory/operator guide.
- historical planning roots such as `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md` remain discoverable and useful as provenance, but they can still read like current operating procedure if they are not explicitly framed as historical.
- `PLAN.md` itself currently has stale M4 maintenance content pasted under the M7 heading, which is a plan-integrity bug, not just a writing problem.

M7 is therefore not "write nicer docs." It is the milestone that turns a working factory into a legible one.

#### Landed Baseline
These are already true on `feat/fill-trust-gap` and are not M7 work:

- `crates/xtask/src/main.rs` already exposes the full factory command surface:
  - `onboard-agent`
  - `scaffold-wrapper-crate`
  - `check-agent-drift`
  - `refresh-agent`
  - `close-agent-maintenance`
  - `support-matrix`
  - `capability-matrix`
- `docs/project_management/next/cli-agent-onboarding-charter.md` already reflects the M6 ownership boundary: `onboard-agent` does not create the wrapper crate and `scaffold-wrapper-crate` does.
- `docs/project_management/next/gemini-cli-onboarding/**` already provides one closed proving-run onboarding packet.
- `docs/project_management/next/opencode-maintenance/**` already provides one real maintenance packet and closeout example.
- `docs/specs/agent-registry-contract.md` and `docs/specs/unified-agent-api/capabilities-schema-spec.md` already pin the narrow normative truth M5 needed.
- `docs/crates-io-release.md` already contains the generated publish-order block emitted by the control plane.

M7 should document this state clearly. It should not reopen any code-path or contract decisions from M5 or M6.

#### Step 0. Scope Challenge
- Existing doc leverage: the repo already has the right raw inputs in `README.md`, `CONTRIBUTING.md`, `docs/README.md`, `docs/project_management/next/cli-agent-onboarding-charter.md`, generated onboarding/maintenance packets, and the command surface in `crates/xtask/src/main.rs`. M7 should consolidate and point, not invent a second explanation tree.
- Minimum complete change: one canonical operator guide, one repo-entry cleanup pass, one historical/provenance framing pass for the most discoverable stale planning docs, and one `PLAN.md` correction so the roadmap itself is truthful.
- Complexity check: the likely blast radius is 6 to 10 documentation files plus `PLAN.md`. That is acceptable because the whole milestone is about reducing confusion in one narrow factory seam.
- Search check: quote the exact commands and artifact paths already present in `xtask`, the charter, and the generated packets. Do not invent a "friendly" workflow that diverges from the repo.
- Completeness check: a partial M7 that only adds a new guide but leaves `README.md`, `CONTRIBUTING.md`, and `docs/README.md` pointing maintainers at older fragments is not enough. The guide must become the center of gravity.
- Distribution check: M7 does not add a binary, release rail, or runtime artifact. It is docs-only, but it changes where maintainers learn the workflow, so precision matters.

#### Scope Lock
In scope:
- add one canonical operator guide at `docs/cli-agent-onboarding-factory-operator-guide.md`
- make that guide the source of truth for:
  - the create-mode onboarding sequence
  - the runtime-shell scaffold step
  - the maintenance lane
  - the green-gate verification sequence
  - ownership boundaries between control-plane surfaces, runtime-owned code, generated publication, and historical packets
- update `README.md`, `CONTRIBUTING.md`, and `docs/README.md` so they point to the operator guide and stop acting like partial command references
- update `docs/project_management/next/cli-agent-onboarding-charter.md` so it stays normative and links to the operator guide for operational procedure instead of trying to be both charter and manual
- add narrow "historical only" or "provenance" framing to the highest-confusion hand-authored legacy planning surfaces that remain discoverable after M6, starting with:
  - `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md`
  - any nearby hand-authored README/handoff surface that still reads like current execution guidance after the new operator guide exists
- repair `PLAN.md` so M7 is a real docs milestone instead of copied M4 maintenance text

#### Not In Scope
- changing command behavior, flags, exit codes, or generated packet templates
- editing generated onboarding or maintenance packet files purely for tone cleanup
- moving historical planning packs out of `docs/project_management/next/`
- rewriting normative specs to duplicate operator guidance
- creating a second operator guide under `docs/project_management/`
- broad documentation gardening outside the factory/operator seam

#### Success Criteria
M7 is complete only when all of these are true:

- `docs/cli-agent-onboarding-factory-operator-guide.md` exists and is the clear center of gravity for the factory workflow
- the guide documents both shipped operator paths with exact commands and exact artifact roots:
  - create-mode:
    - approval artifact
    - `onboard-agent`
    - `scaffold-wrapper-crate`
    - runtime implementation follow-on
    - manifest evidence + publication refresh
  - maintenance-mode:
    - `check-agent-drift`
    - maintenance request artifact
    - `refresh-agent`
    - `close-agent-maintenance`
- the guide explicitly names the authoritative green gate:
  - `cargo run -p xtask -- support-matrix --check`
  - `cargo run -p xtask -- capability-matrix --check`
  - `cargo run -p xtask -- capability-matrix-audit`
  - `make preflight`
- `README.md`, `CONTRIBUTING.md`, and `docs/README.md` all point maintainers at the operator guide instead of leaving them to reconstruct the workflow from scattered docs
- the onboarding charter remains the normative checklist/contract surface, but no longer has to double as the only discoverable operator manual
- the most discoverable hand-authored legacy planning docs that could be mistaken for live procedure are explicitly framed as historical provenance and point to the operator guide
- `PLAN.md` no longer contains pasted M4 maintenance content under the M7 heading

#### What Already Exists
| Sub-problem | Existing surface to reuse | Why it matters |
|---|---|---|
| command inventory | `crates/xtask/src/main.rs` | already exposes the real command set M7 must document exactly |
| runtime-step ownership boundary | `docs/project_management/next/cli-agent-onboarding-charter.md`, `crates/xtask/src/onboard_agent/preview.rs`, `crates/xtask/src/onboard_agent/preview/render.rs` | already encode the M6 split between `onboard-agent` and `scaffold-wrapper-crate` |
| proving-run example | `docs/project_management/next/gemini-cli-onboarding/**` | already shows the create-mode packet/output shape |
| maintenance example | `docs/project_management/next/opencode-maintenance/**` | already shows the maintenance lane and closeout shape |
| green-gate contract | `docs/specs/unified-agent-api/capabilities-schema-spec.md`, `README.md`, `CONTRIBUTING.md`, `docs/crates-io-release.md` | already contain fragments of the verification story M7 must unify |
| repo entrypoints | `README.md`, `CONTRIBUTING.md`, `docs/README.md` | already act as discoverability surfaces and therefore must stop being partially stale |
| historical provenance docs | `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md` and nearby hand-authored planning docs | still useful as source provenance, but need explicit framing so operators do not mistake them for current procedure |

#### Chosen Approach
M7 should use a hub-and-spokes documentation model:

1. one canonical operator guide
2. thin entry docs that point to it
3. normative docs that stay normative
4. historical planning docs that are clearly labeled as provenance only

That is the smallest complete fix.

Anything smaller leaves the workflow discoverable only by archaeology.
Anything bigger turns M7 into a repo-wide docs rewrite and misses the point.

#### Dream State Delta
```text
CURRENT
commands exist
generated packets exist
charter knows the M6 boundary
entry docs still tell only pieces of the story
historical planning docs can still read like current procedure

M7
one canonical operator guide
README / CONTRIBUTING / docs index all point to it
charter stays normative, not operationally overloaded
historical planning docs are explicitly framed as provenance
maintainer can run the factory without reconstructing milestone history

12-MONTH IDEAL
new maintainer joins
reads one guide
runs one create-mode or maintenance-mode workflow correctly
does not need private branch lore to understand what is current
```

#### Documentation Architecture Review
##### Preferred Document Topology
Keep the docs structure boring:

- `README.md`
  - short repo overview
  - pointer to the operator guide
- `CONTRIBUTING.md`
  - contributor workflow and verification entrypoint
  - pointer to the operator guide
- `docs/README.md`
  - docs index
  - pointer to the operator guide
- `docs/cli-agent-onboarding-factory-operator-guide.md`
  - canonical human operator manual
- `docs/project_management/next/cli-agent-onboarding-charter.md`
  - normative onboarding charter
  - points to the operator guide for procedure
- historical planning docs under `docs/project_management/next/**`
  - remain source provenance
  - gain narrow supersession or provenance framing where needed

Do not create parallel "quick start", "playbook", and "operator guide" variants for the same workflow. One manual is enough.

##### Dependency Graph
```text
crates/xtask/src/main.rs
        |
        v
docs/cli-agent-onboarding-factory-operator-guide.md
        |
   +----+--------------------+-------------------+
   |                         |                   |
   v                         v                   v
README.md              CONTRIBUTING.md      docs/README.md
   |                         |                   |
   +----------- pointers ----+-------------------+
        |
        v
docs/project_management/next/cli-agent-onboarding-charter.md
        |
        v
historical planning / proving-run / maintenance packet docs
as examples and provenance, not the primary manual
```

##### Documentation Decisions
- The operator guide should be procedural, not normative. It says "what to run" and "how the lifecycle fits together."
- The charter should remain normative. It says "what the repo requires from onboarding."
- Generated packets stay examples and evidence, not the only way to learn the process.
- Historical planning docs should stay in the repo, but operators must not have to guess whether they are current. If a document is provenance-only, say so explicitly.
- Entry docs should summarize and point. They should not each become their own partial command reference.

#### Code Quality Guardrails For Docs
- Use exact command lines from `xtask` and exact file paths from the repo. No paraphrased command names.
- Keep the operator guide authoritative for procedure. If an entry doc needs more than a short summary, link instead of duplicating.
- Distinguish four kinds of truth explicitly:
  - normative spec
  - operator procedure
  - generated packet/example output
  - historical provenance
- If a doc is historical only, say so at the top instead of burying that fact halfway through the file.
- Do not rewrite generated packet outputs by hand just to make the docs read cleaner. Fix the hand-authored docs around them.

#### Workstreams
##### W1. Canonical Operator Guide
Goal: give the repo one operator manual that matches the shipped command surface.

Deliverables:
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- create-mode workflow section
- maintenance-mode workflow section
- green-gate section
- ownership-boundaries section

Primary modules:
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `crates/xtask/src/main.rs`
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/gemini-cli-onboarding/**`
- `docs/project_management/next/opencode-maintenance/**`

Exit criteria:
- a maintainer can follow one doc to understand the full factory lifecycle

##### W2. Entry-Doc Cleanup
Goal: make the repo entrypoints point to the canonical manual instead of each telling fragments of the story.

Deliverables:
- `README.md` update
- `CONTRIBUTING.md` update
- `docs/README.md` update
- green-gate command list corrected where needed

Primary modules:
- `README.md`
- `CONTRIBUTING.md`
- `docs/README.md`

Exit criteria:
- maintainers entering from the repo root or docs index land on the operator guide quickly

##### W3. Normative And Historical Framing
Goal: stop the charter and old planning docs from competing with the operator guide.

Deliverables:
- charter pointer to operator guide
- historical/provenance framing notes on the highest-confusion hand-authored planning docs
- stale status cleanup in `PLAN.md`

Primary modules:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md`
- any nearby hand-authored planning README/handoff file that still reads like live procedure
- `PLAN.md`

Exit criteria:
- operators can distinguish current procedure from historical planning evidence without guesswork

##### W4. Cross-Doc Verification Pass
Goal: prove the docs tell the same story as the shipped repo surfaces.

Deliverables:
- command-to-doc verification matrix
- final contradiction pass over entry docs, operator guide, charter, and the key legacy pointer docs

Primary modules:
- `README.md`
- `CONTRIBUTING.md`
- `docs/README.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- selected historical pointer docs

Exit criteria:
- no doc still implies the pre-M6 manual wrapper-creation flow or hides the maintenance lane

#### Implementation Sequence
##### Phase 1. Operator Guide Skeleton
Outputs:
- guide path chosen and created
- create-mode and maintenance-mode outlines
- ownership-boundary section

Modules touched:
- `docs/cli-agent-onboarding-factory-operator-guide.md`

Implementation notes:
- start with the real command chain, not repo history
- treat generated packets as examples, not as the outline itself
- keep commands and artifact paths exact

Exit gate:
- one draft guide exists that already reflects the shipped `xtask` surface

##### Phase 2. Entry-Doc Repointing
Outputs:
- `README.md`, `CONTRIBUTING.md`, and `docs/README.md` point to the guide
- green-gate references corrected and aligned

Modules touched:
- `README.md`
- `CONTRIBUTING.md`
- `docs/README.md`

Implementation notes:
- summarize, then point
- remove partial command storytelling where the guide now owns it

Exit gate:
- repo entrypoints no longer force maintainers to bounce between stale fragments

##### Phase 3. Charter And Legacy Framing
Outputs:
- charter links to the guide for procedure
- selected historical planning docs get explicit provenance framing
- `PLAN.md` M7 section becomes truthful

Modules touched:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- selected hand-authored historical docs
- `PLAN.md`

Implementation notes:
- keep historical docs in place
- add the minimum framing needed so they stop reading like current instructions
- do not hand-edit generated packet files

Exit gate:
- maintainers can tell "current manual" from "historical evidence" on first read

##### Phase 4. Verification Pass
Outputs:
- cross-doc contradiction check complete
- final command/reference cleanup

Modules touched:
- all touched docs from Phases 1 through 3

Implementation notes:
- verify every command named in the docs exists in `xtask`
- verify every cited path exists or is intentionally planned
- verify entry docs do not contradict the guide

Exit gate:
- no remaining top-level doc contradicts the post-M6 factory workflow

#### Error & Rescue Registry
| Surface | What can go wrong | Failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| operator guide | guide drifts from the real `xtask` command set | documentation drift | yes | validate against `crates/xtask/src/main.rs` and key generated packets | operator follows a command that does not exist |
| README / CONTRIBUTING | entry docs keep partial stale instructions | discoverability bug | yes | reduce them to summary-plus-pointer | maintainer follows an incomplete workflow |
| charter | charter becomes duplicate procedural manual | ownership drift | yes | keep charter normative and link to guide for procedure | two docs disagree about the flow |
| historical planning docs | old handoff still reads like current procedure | provenance ambiguity | yes | add explicit historical/provenance framing | operator uses a superseded path |
| PLAN.md | roadmap still contains pasted M4 content under M7 | plan integrity bug | yes | rewrite M7 as an actual docs milestone | maintainers mistrust the roadmap itself |

#### Documentation Verification Strategy
##### Verification Matrix
```text
CREATE-MODE WORKFLOW
====================
[+] approval artifact / enrolled agent
    |
    +--> onboard-agent
    +--> scaffold-wrapper-crate
    +--> runtime/backend implementation
    +--> manifest evidence + publication refresh
    \--> make preflight

[GAP -> docs]
- one canonical guide must explain the full sequence
- README / CONTRIBUTING / docs index must point to that guide

MAINTENANCE-MODE WORKFLOW
=========================
[+] already-onboarded agent
    |
    +--> check-agent-drift --agent <id>
    +--> maintenance-request.toml
    +--> refresh-agent --dry-run / --write
    \--> close-agent-maintenance

[GAP -> docs]
- guide must explain this lane clearly
- historical proving-run docs must be examples, not the only explanation

GREEN GATE
==========
[+] support-matrix --check
[+] capability-matrix --check
[+] capability-matrix-audit
[+] make preflight

[GAP -> docs]
- entry docs must stop mentioning only part of the gate
```

##### Required Verification Surfaces
- `README.md`
  - points to the operator guide
  - does not imply only the support-matrix check matters
- `CONTRIBUTING.md`
  - points to the operator guide
  - lists the same green gate the guide uses
- `docs/README.md`
  - indexes the operator guide
- `docs/cli-agent-onboarding-factory-operator-guide.md`
  - matches `crates/xtask/src/main.rs`
  - names the create-mode and maintenance-mode artifact roots correctly
- `docs/project_management/next/cli-agent-onboarding-charter.md`
  - remains normative
  - points to the guide for procedure
- selected historical planning docs
  - explicitly framed as provenance or historical guidance

##### Verification Commands
- `cargo run -p xtask -- --help`
- `cargo run -p xtask -- onboard-agent --help`
- `cargo run -p xtask -- scaffold-wrapper-crate --help`
- `cargo run -p xtask -- check-agent-drift --help`
- `cargo run -p xtask -- refresh-agent --help`
- `cargo run -p xtask -- close-agent-maintenance --help`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix --check`
- `cargo run -p xtask -- capability-matrix-audit`
- `make preflight`
- `rg -n "scaffold-wrapper-crate|check-agent-drift|refresh-agent|close-agent-maintenance|capability-matrix --check|support-matrix --check" README.md CONTRIBUTING.md docs/README.md docs/cli-agent-onboarding-factory-operator-guide.md docs/project_management/next/cli-agent-onboarding-charter.md docs/project_management/next`

#### Failure Modes Registry
| Surface | Failure mode | Prevent in M7? | Detect in verification? | User impact | Blocker? |
|---|---|---|---|---|---|
| operator guide | says to run a command that does not exist or omits one that does | yes | yes | maintainer loses trust in docs immediately | yes |
| entry docs | still point operators into stale fragments instead of the guide | yes | yes | repo root remains confusing | yes |
| historical planning docs | still look like live instructions | yes | yes | maintainers follow old workflow | yes |
| normative docs | charter/specs duplicate or contradict procedure | yes | yes | operator cannot tell contract from how-to guidance | yes |
| roadmap | M7 text still contains maintenance-lane content from M4 | yes | yes | roadmap stops being credible | yes |

Critical gap rule:
- if the operator guide does not become the obvious center of gravity, M7 failed
- if a new maintainer can still land on a legacy planning doc and reasonably mistake it for the current operating procedure, M7 failed

#### Performance Review
- Keep the guide centralized so future command changes touch one procedural doc instead of five near-duplicates.
- Keep entry docs thin. Long duplicated command references become drift surfaces the day after they land.
- Prefer explicit provenance notes over wide doc moves or archive churn. The goal is clarity, not a renaming project.

#### Worktree Parallelization Strategy
##### Dependency Table
| Step | Modules touched | Depends on |
|---|---|---|
| W1. operator guide | `docs/cli-agent-onboarding-factory-operator-guide.md` | — |
| W2. entry-doc cleanup | `README.md`, `CONTRIBUTING.md`, `docs/README.md` | W1 |
| W3. charter + legacy framing | charter, selected historical planning docs, `PLAN.md` | W1 |
| W4. verification pass | all touched docs | W2, W3 |

##### Parallel Lanes
Lane A: W1 -> W2
Canonical guide plus repo-entry cleanup. These should stay sequential because entry docs need the final guide path and framing.

Lane B: W3
Charter plus historical framing lane. This can start once W1 fixes the guide path and final terminology.

Lane C: W4
Final contradiction sweep. Run only after Lane A and Lane B merge.

##### Execution Order
1. Write the operator guide first.
2. Update repo entry docs next.
3. In parallel, add charter and historical-provenance framing once the guide path is fixed.
4. Run one contradiction sweep last.

##### Conflict Flags
- W2 and W4 both touch `README.md`, `CONTRIBUTING.md`, and `docs/README.md`. Do not overlap them.
- W3 must stay on hand-authored docs only. If it starts editing generated packet files, it has left M7 scope.
- `PLAN.md` belongs to W3. Do not let the verification pass re-open milestone scope.

#### Decision Audit Trail
| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | CEO | keep M7 docs-only | mechanical | boil lakes, not oceans | the code/command surfaces already exist; the problem is discoverability and stale framing | reopening command or contract work |
| 2 | CEO | create one canonical operator guide under `docs/` | mechanical | explicit over clever | maintainers need one obvious procedure hub, not another planning artifact | spreading procedure across README, charter, and packets |
| 3 | Eng | keep generated packet files out of manual cleanup scope | mechanical | DRY | hand-editing generated outputs would create immediate drift with `xtask` | tone-cleaning generated packet docs by hand |
| 4 | Eng | treat historical planning docs as provenance, not procedure | mechanical | pragmatic | these docs still have value, but they should stop competing with the live manual | deleting or relocating the planning history wholesale |

#### Completion Summary
- Scope: M7 is now a real docs milestone, not copied M4 maintenance text.
- Architecture: one operator guide becomes the center of gravity, entry docs point to it, normative docs stay normative, historical docs get provenance framing.
- Verification: command references are validated against `xtask`, and the green gate is documented as one contract instead of scattered fragments.
- Deferred: broad repo-wide docs gardening remains out of scope; M7 only fixes the factory/operator seam.

#### Deferred To TODOS.md
- consider a generated command-reference appendix only if the operator guide starts drifting despite the centralized model
- consider archiving older planning roots into a clearer historical namespace only after the lightweight provenance framing proves insufficient

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 1 | issues_open via `/autoplan` (incorporated) | M5 must be canonical capability-projection hardening, not a narrow drift hotfix, and M6 wrapper scaffolding stays out of scope |
| Codex Review | `codex exec` | Independent 2nd opinion | 1 | codex-only via `/autoplan` | outside voice pushed the same two high-signal requirements now folded into the plan: delete duplicate capability semantics and make local green equal CI green |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | issues_open via `/autoplan` (incorporated) | one shared projection helper, one explicit primary publication-target contract, one check-only capability freshness gate, and seeded regression coverage are required |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | skipped | no UI scope in M5 |

**CODEX:** The outside voice agreed with the local read that the real problem is duplicated control-plane semantics, not stale checked-in markdown.
**CROSS-MODEL:** No contradiction remained after consolidation. The only initially open scope edge was whether to make the publication target explicit now, and that choice is now accepted into M5.
**UNRESOLVED:** 0 after folding the approved explicit publication-target decision into the M5 scope.
**VERDICT:** CEO + ENG review outcomes are incorporated into the active M5 plan-of-record; W1-W3 carry the implementation contract and W4 is the docs closeout against that merged truth.
