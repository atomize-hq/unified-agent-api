<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/staging-autoplan-restore-20260423-065640.md -->
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
- planned follow-on after M5 lands cleanly

Current direction:
- prefer a separate wrapper scaffold command over widening `xtask onboard-agent`

Why this is the likely shape:
- it preserves the current control-plane versus runtime-owned boundary
- it avoids silently changing `onboard-agent` from “control-plane enrollment” into “create the wrapper crate too”
- it gives the repo a place to add package-surface scaffolding like `README.md`, `LICENSE-*`, and other publishability requirements without rewriting the meaning of the onboarding packet or closeout flow

Initial M6 scope note:
- scaffold the minimal wrapper-crate package surfaces needed after agent approval and control-plane enrollment
- keep the command separate from maintenance and separate from create-mode onboarding
- re-contract ownership language only where the new scaffold step must be documented explicitly

### M7. Documentation And Legacy Cleanup
Status:
- tentative follow-on after M5 and M6 land cleanly

Intent:
- clean up outdated, duplicated, or confusing onboarding/maintenance docs
- create one detailed reference for the new system with commands, artifact shapes, typical workflows, and operator guidance

Candidate deliverables:
- one canonical “CLI agent onboarding factory” operator guide
- command reference for `onboard-agent`, `check-agent-drift`, `refresh-agent`, `close-agent-maintenance`, and the planned wrapper scaffold command
- migration notes for old packet assumptions and legacy docs that are now historical only
- cleanup of stale status language and superseded handoff notes where they still create ambiguity

## Purpose
M4 turns post-onboarding maintenance from repo archaeology into a separate, repeatable lifecycle.

M1 created the reproducible onboarding bridge.
M2 added write-mode and proved the bridge on one real agent.
M3 formalized comparison -> approval -> proving-run closeout governance.

That leaves the next bottleneck. Once an agent is already in the repo, maintainers still have to discover drift manually across:
- `crates/xtask/data/agent_registry.toml`
- `cli_manifests/<agent>/**`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/unified-agent-api/capability-matrix.md`
- `docs/crates-io-release.md`
- closed onboarding and implementation packet docs

`onboard-agent` is not the answer to that problem. It is the create-mode bridge for new agents. M4 needs a separate maintenance lane for already-onboarded agents.

## Problem Statement
If an onboarded agent changes upstream or repo truth drifts, the current repo has no single maintenance entrypoint.

OpenCode already showed the failure shape. The landing itself succeeded, but `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md` records that `docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md` understates the landed OpenCode capability set versus:
- `crates/agent_api/src/backends/opencode/backend.rs`
- `docs/specs/opencode-agent-api-backend-contract.md`
- `docs/specs/unified-agent-api/capability-matrix.md`

That is exactly the class of bug M4 should eliminate:
- landed runtime/backend truth says one thing
- generated publication says one thing
- closed packet/governance docs say something else
- the operator has to manually rediscover the right repair path

The repo can now onboard a new agent with governed create-mode. It still cannot repair an existing agent boringly once drift appears. M4 must fix that.

## Landed Baseline
These are already true in this branch and are not M4 work:

- `crates/xtask/data/agent_registry.toml` seeds `codex`, `claude_code`, `opencode`, and `gemini_cli`.
- `crates/xtask/src/onboard_agent.rs` implements `--dry-run`, `--write`, and `--approval` for new-agent control-plane mutation.
- `crates/xtask/src/approval_artifact.rs` and `crates/xtask/src/proving_run_closeout.rs` validate approval and closeout truth.
- `crates/xtask/src/close_proving_run.rs` refreshes onboarding packet docs from a validated proving-run closeout artifact.
- `crates/xtask/src/support_matrix.rs`, `crates/xtask/src/support_matrix/derive.rs`, and `crates/xtask/src/support_matrix/consistency.rs` already derive and fail closed on support-publication drift.
- `crates/xtask/src/capability_matrix.rs` already derives capability publication from registry enrollment plus runtime/backend truth.
- `docs/project_management/next/gemini-cli-onboarding/**` is the first closed factory-backed proving-run packet.
- OpenCode is already landed, and `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md` documents a real post-onboarding drift issue to use as M4 input.

M4 builds on this exact repo state. It is not M2 or M3 cleanup, and it is not permission to widen `onboard-agent` into an everything command.

## Scope Lock
In scope:
- define a separate post-onboarding maintenance lifecycle for already-onboarded agents
- add agent-scoped drift detection across registry truth, manifest evidence, publication outputs, release docs, and packet/governance docs
- add a separate maintenance packet root under `docs/project_management/next/<agent>-maintenance/`
- add separate maintenance request and maintenance closeout artifacts
- add separate refresh ergonomics for control-plane-owned maintenance work
- keep maintenance writes bounded to control-plane-owned and generated surfaces
- use OpenCode as the first maintenance proving run because it has a real documented post-onboarding drift issue
- make reopen and closeout rules explicit so closed onboarding packets stay immutable

## Not In Scope
- adding update mode to `xtask onboard-agent`
- changing the recommendation, approval, or new-agent onboarding flow from M3
- generating or mutating runtime-owned wrapper/backend code under `crates/<agent>/` or `crates/agent_api/src/backends/<agent>/`
- mutating raw manifest evidence under `cli_manifests/<agent>/current.json`, `versions/`, `pointers/`, or `reports/` from the control plane
- collapsing recommendation, onboarding, proving-run closeout, and maintenance into one universal lifecycle command family
- changing support-matrix or capability-matrix semantics
- automating candidate research or building `recommend-agent`
- building a framework-scale runtime abstraction because one agent drifted

## Success Criteria
M4 is complete only when all of these are true:

- `xtask onboard-agent` remains create-only for new agents. Already-onboarded maintenance does not flow through it.
- `cargo run -p xtask -- check-agent-drift --agent <agent_id>` exists and:
  - exits `0` when the agent is clean
  - exits `2` when drift or validation problems are found
  - emits explicit drift categories instead of a generic failure blob
- `cargo run -p xtask -- refresh-agent --request <path> --dry-run` exists for already-onboarded agents.
- `cargo run -p xtask -- refresh-agent --request <path> --write` exists and shares the exact same render plan as `--dry-run`.
- `cargo run -p xtask -- close-agent-maintenance --request <path> --closeout <path>` exists and validates maintenance closure truth.
- Maintenance write mode mutates only:
  - `docs/project_management/next/<agent>-maintenance/**`
  - generated publication outputs from existing generators
  - the generated block inside `docs/crates-io-release.md` when it drifted
- Maintenance write mode never mutates:
  - `crates/<agent>/**`
  - `crates/agent_api/src/backends/<agent>/**`
  - raw manifest evidence files under `cli_manifests/<agent>/**`
  - historical onboarding packet roots such as `docs/project_management/next/<agent>-cli-onboarding/**`
- Closed onboarding packets remain immutable. Maintenance history is recorded in the separate maintenance pack.
- OpenCode is used as the proving run and the known stale capability claim in `docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md` becomes legible, repairable, and closeable through the M4 flow.
- The M4 test plan exists and remains current at `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-cli-agent-onboarding-factory-test-plan-20260421-233454.md`.

## What Already Exists
M4 must reuse these surfaces instead of inventing a second factory:

- Registry and path truth:
  - `crates/xtask/data/agent_registry.toml`
  - `crates/xtask/src/agent_registry.rs`
- Existing control-plane mutation primitives:
  - `crates/xtask/src/onboard_agent.rs`
  - `crates/xtask/src/onboard_agent/preview.rs`
  - `crates/xtask/src/onboard_agent/preview/render.rs`
  - `crates/xtask/src/onboard_agent/validation.rs`
  - `crates/xtask/src/workspace_mutation.rs`
- Existing proving-run governance primitives:
  - `crates/xtask/src/approval_artifact.rs`
  - `crates/xtask/src/proving_run_closeout.rs`
  - `crates/xtask/src/close_proving_run.rs`
- Existing drift-sensitive publication surfaces:
  - `crates/xtask/src/support_matrix.rs`
  - `crates/xtask/src/support_matrix/derive.rs`
  - `crates/xtask/src/support_matrix/consistency.rs`
  - `crates/xtask/src/capability_matrix.rs`
  - `docs/specs/unified-agent-api/support-matrix.md`
  - `docs/specs/unified-agent-api/capability-matrix.md`
- Existing release/doc generation surface:
  - `docs/crates-io-release.md`
- Historical maintenance input:
  - `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md`
  - `docs/project_management/next/opencode-implementation/**`
  - `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-test-outcome-20260420-091704.md`

## Chosen Approach
M4 is a separate maintenance lane, not onboarding scope creep.

The repo already has a governed create flow:
- comparison
- approval
- create-mode onboarding
- proving-run closeout

The missing lifecycle is what happens after that when an onboarded agent drifts. The smallest complete M4 is:
- one drift detector
- one maintenance request artifact
- one bounded control-plane refresh command
- one maintenance closeout artifact
- one real proving run on OpenCode

Anything bigger is ocean-boiling. Anything smaller leaves the repo in the same archaeological post-onboarding posture that OpenCode exposed.

## Dream State Delta
```text
CURRENT STATE
already-onboarded agent
    |
    +--> drift appears in docs, publication, or governance truth
    +--> maintainer manually compares registry, manifests, runtime code, and packet docs
    +--> repair path is rediscovered from repo history

M4
already-onboarded agent
    |
    +--> check-agent-drift --agent <id>
    +--> maintenance-request.toml
    +--> refresh-agent --dry-run / --write
    +--> runtime/evidence follow-up when required
    +--> close-agent-maintenance

12-MONTH IDEAL
already-onboarded agent
    |
    +--> boring per-agent drift checks
    +--> boring maintenance packets
    +--> boring refresh/closeout loop
    +--> no reopen requires conversation archaeology
```

## M4 Plan Of Record
### Goal
Make already-onboarded agents repairable without reopening new-agent onboarding.

### Milestone Outcome
At the end of M4:

- maintainers can detect drift for one onboarded agent in one command
- maintainers can open one bounded maintenance packet for that agent
- control-plane-owned repair steps are previewable and replay-safe
- runtime/evidence follow-up stays explicit and separate
- maintenance closure records exactly what was resolved, what was deferred, and whether `make preflight` passed
- OpenCode proves the workflow on a real post-onboarding drift case

### Maintenance Chain
```text
drift trigger or stale proof
        |
        v
check-agent-drift --agent <agent_id>
        |
        v
maintenance-request.toml
        |
        v
refresh-agent --dry-run / --write
        |
        +--> control-plane-owned refreshes
        +--> explicit runtime/evidence follow-up list
        |
        v
close-agent-maintenance
        |
        v
closed maintenance pack + reopen trigger record
```

### Step 0. Scope Challenge
M4 should extend the existing `xtask` control plane, not create a second factory.

Existing code already solves most of the hard parts:
- path jailing, symlink rejection, identical-write detection, and rollback already live in `crates/xtask/src/workspace_mutation.rs`
- bounded onboarding preview and write planning already exist in `crates/xtask/src/onboard_agent.rs`
- generated publication truth already exists in `crates/xtask/src/support_matrix.rs` and `crates/xtask/src/capability_matrix.rs`
- closeout-style artifact validation already exists in `crates/xtask/src/approval_artifact.rs` and `crates/xtask/src/proving_run_closeout.rs`

Minimum complete change set:
- add one maintenance namespace under `crates/xtask/src/` for drift, refresh, and closeout logic
- wire three new subcommands in `crates/xtask/src/main.rs`
- reuse existing generator and mutation primitives instead of cloning onboarding logic
- add maintenance-specific integration tests and one maintenance packet root per agent

Complexity guardrail:
- no new crate
- no lifecycle umbrella abstraction
- no runtime-owned writes
- no file-by-file special cases in `main.rs` beyond thin command routing

## Artifact Contract
### 1. Maintenance request artifact
Path: `docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml`
Format: TOML
Owner: maintainer workflow

Required fields:
- `artifact_version`
- `agent_id`
- `trigger_kind`
- `basis_ref`
- `opened_from`
- `requested_control_plane_actions`
- `runtime_followup_required`
- `request_recorded_at`
- `request_commit`

Rules:
- must reference an already-onboarded agent in `agent_registry.toml`
- must not be used to create a new agent
- must live under the maintenance pack root, not the onboarding pack root
- `requested_control_plane_actions` may include only bounded maintenance actions:
  - `packet_doc_refresh`
  - `support_matrix_refresh`
  - `capability_matrix_refresh`
  - `release_doc_refresh`

Exact schema rules:
- TOML root only. Unknown top-level keys fail validation.
- `artifact_version = "1"`
- `agent_id = "<registry agent id>"`, non-empty, must already exist in `crates/xtask/data/agent_registry.toml`
- `trigger_kind` is one of:
  - `drift_detected`
  - `manual_reopen`
  - `post_release_audit`
- `basis_ref` is a repo-relative path to the evidence that triggered maintenance. It must resolve inside the workspace and may point to:
  - an existing governance closeout doc
  - a committed spec or publication doc
  - a committed maintenance packet doc created during the same maintenance run
- `opened_from` is a repo-relative path to the document or artifact where the maintainer initiated the maintenance run
- `requested_control_plane_actions` is a non-empty array of unique strings, sorted in file order as written by the maintainer; all values must come from the allowed action set above
- `runtime_followup_required` is a table with exact fields:
  - `required = true|false`
  - `items = ["..."]`
- `runtime_followup_required.required = false` requires `items = []`
- `runtime_followup_required.required = true` requires `items` to be a non-empty array of non-blank strings
- `request_recorded_at` must be RFC3339 UTC
- `request_commit` must be 7-40 lowercase hex characters

Example:
```toml
artifact_version = "1"
agent_id = "opencode"
trigger_kind = "drift_detected"
basis_ref = "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md"
opened_from = "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md"
requested_control_plane_actions = [
  "packet_doc_refresh",
  "capability_matrix_refresh",
]
request_recorded_at = "2026-04-22T01:15:00Z"
request_commit = "1adb8f1"

[runtime_followup_required]
required = false
items = []
```

### 2. Maintenance closeout artifact
Path: `docs/project_management/next/<agent>-maintenance/governance/maintenance-closeout.json`
Format: JSON
Owner: maintenance closeout

Required fields:
- `request_ref`
- `request_sha256`
- `resolved_findings`
- exactly one of:
  - `deferred_findings`
  - `explicit_none_reason`
- `preflight_passed`
- `recorded_at`
- `commit`

Rules:
- closeout must fail validation if it cannot link back to the request artifact
- closeout must state what was resolved and whether anything remains deferred
- maintenance closure must not mutate historical onboarding packet docs directly

Exact schema rules:
- JSON object root only. Unknown fields fail validation.
- `request_ref` is the exact repo-relative path to `governance/maintenance-request.toml`
- `request_sha256` must be 64 lowercase hex characters and must match the current bytes of `request_ref`
- `resolved_findings` is a non-empty array of objects with exact fields:
  - `category_id`
  - `summary`
  - `surfaces`
- `deferred_findings`, when present, is a non-empty array of the same object shape as `resolved_findings`
- every `category_id` in `resolved_findings` or `deferred_findings` must be one of the drift category IDs defined in the drift output contract below
- every `surfaces` value is a non-empty array of repo-relative paths
- exactly one of:
  - `deferred_findings`
  - `explicit_none_reason`
- `explicit_none_reason` must be non-empty and is allowed only when `deferred_findings` is absent
- `preflight_passed` is boolean
- `recorded_at` must be RFC3339 UTC
- `commit` must be 7-40 lowercase hex characters

Example:
```json
{
  "request_ref": "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
  "request_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "resolved_findings": [
    {
      "category_id": "governance_doc_drift",
      "summary": "SEAM-2 closeout now matches the landed capability advertisement boundary.",
      "surfaces": [
        "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
        "docs/project_management/next/opencode-maintenance/HANDOFF.md"
      ]
    }
  ],
  "explicit_none_reason": "No deferred maintenance findings remain after publication and packet refresh.",
  "preflight_passed": true,
  "recorded_at": "2026-04-22T01:45:00Z",
  "commit": "4adefdf"
}
```

## Command Contract
M4 adds a separate maintenance command set.

### Drift detection
```bash
cargo run -p xtask -- check-agent-drift --agent <agent_id>
```

Rules:
- read-only
- agent must already exist in `crates/xtask/data/agent_registry.toml`
- exit `0` means no drift
- exit `2` means drift or maintenance preconditions failed
- exit `1` means internal or IO failure
- output categories must include, when present:
  - registry versus manifest evidence drift
  - runtime/backend versus capability publication drift
  - support publication drift
  - release/doc generated-block drift
  - closed packet/governance doc drift

Stable drift category IDs:
- `registry_manifest_drift`
- `capability_publication_drift`
- `support_publication_drift`
- `release_doc_drift`
- `governance_doc_drift`

Exact stdout contract:
- human-readable summary on stdout
- deterministic block order:
  1. header
  2. agent id
  3. status
  4. zero or more drift records sorted by `category_id`
- exact header line:
  - `== AGENT DRIFT REPORT ==`
- exact status line:
  - clean run: `status: clean`
  - drift run: `status: drift_detected`

Exact drift record shape on stdout:
```text
category_id: <stable id>
summary: <single line>
surfaces:
  - <repo-relative path>
  - <repo-relative path>
```

Exact clean-run example:
```text
== AGENT DRIFT REPORT ==
agent_id: opencode
status: clean
```

Exact drift example:
```text
== AGENT DRIFT REPORT ==
agent_id: opencode
status: drift_detected

category_id: governance_doc_drift
summary: maintenance-relevant closeout prose no longer matches landed capability truth.
surfaces:
  - docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md
  - docs/specs/unified-agent-api/capability-matrix.md
```

M4 deliberately does not add JSON output for `check-agent-drift`. The stdout contract above is the only drift-detector output in this milestone.

### Maintenance refresh
```bash
cargo run -p xtask -- refresh-agent --request docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml --dry-run
cargo run -p xtask -- refresh-agent --request docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml --write
```

Rules:
- `--dry-run` and `--write` are mutually exclusive
- dry-run and write must share one in-memory render plan
- request artifact path is jailed and maintenance-root validated
- unknown agent ids fail closed
- request actions that imply runtime-owned mutations fail closed
- exact-byte replay after an identical write is a success no-op
- exit `0` means success, including identical no-op replay
- exit `2` means validation or ownership failure
- exit `1` means internal or IO failure

### Maintenance closeout
```bash
cargo run -p xtask -- close-agent-maintenance --request docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml --closeout docs/project_management/next/<agent>-maintenance/governance/maintenance-closeout.json
```

Rules:
- validates request linkage and request hash
- requires explicit resolved findings plus either deferred findings or `explicit_none_reason`
- refreshes maintenance packet docs only
- does not reopen or rewrite the historical onboarding packet root
- exit `0` means validated closure
- exit `2` means validation or unresolved-maintenance failure
- exit `1` means internal or IO failure

## Controlled Write Set
The maintenance lane is intentionally narrow.

| Surface | Owner | M4 write mode |
|---|---|---|
| `docs/project_management/next/<agent>-maintenance/**` | maintenance control plane | write |
| `docs/specs/unified-agent-api/support-matrix.md` and `cli_manifests/support_matrix/current.json` via existing generator | generated publication | write |
| `docs/specs/unified-agent-api/capability-matrix.md` via existing generator | generated publication | write |
| generated block in `docs/crates-io-release.md` between `<!-- generated-by: xtask onboard-agent; section: crates-io-release -->` and `<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->` | generated publication | write when drifted |
| `crates/xtask/data/agent_registry.toml` | registry truth | read-only in M4 unless explicit follow-on reopening is approved |
| `cli_manifests/<agent>/current.json`, `versions/`, `pointers/`, `reports/` | manifest evidence | never |
| `crates/<agent>/**` | runtime owner | never |
| `crates/agent_api/src/backends/<agent>/**` | runtime owner | never |
| `docs/project_management/next/<agent>-cli-onboarding/**` | historical onboarding packet | never |

Exact maintenance packet docs surface:
- `docs/project_management/next/<agent>-maintenance/README.md`
- `docs/project_management/next/<agent>-maintenance/scope_brief.md`
- `docs/project_management/next/<agent>-maintenance/seam_map.md`
- `docs/project_management/next/<agent>-maintenance/threading.md`
- `docs/project_management/next/<agent>-maintenance/review_surfaces.md`
- `docs/project_management/next/<agent>-maintenance/HANDOFF.md`
- `docs/project_management/next/<agent>-maintenance/governance/remediation-log.md`
- `docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml`
- `docs/project_management/next/<agent>-maintenance/governance/maintenance-closeout.json`

Command ownership:
- maintainer-created only:
  - `docs/project_management/next/<agent>-maintenance/governance/maintenance-request.toml`
- `refresh-agent` may write only:
  - `README.md`
  - `scope_brief.md`
  - `seam_map.md`
  - `threading.md`
  - `review_surfaces.md`
  - `HANDOFF.md`
  - `governance/remediation-log.md`
- `close-agent-maintenance` may write only:
  - `governance/maintenance-closeout.json`
  - `HANDOFF.md`
  - `governance/remediation-log.md`

No maintenance command may write any other docs path.

## Architecture Review
### Preferred module shape
Keep M4 inside the existing `xtask` crate and make the new code boring:
- `crates/xtask/src/main.rs`
  - thin CLI routing only
- `crates/xtask/src/lib.rs`
  - export the maintenance namespace only if tests or shared helpers need it
- `crates/xtask/src/agent_maintenance/`
  - `drift.rs`
  - `request.rs`
  - `refresh.rs`
  - `closeout.rs`
  - `docs.rs` only if packet rendering grows past one file

If the code stays small, `agent_maintenance.rs` plus a couple of sibling files is better than premature submodule fan-out. The rule is explicit over clever.

### Dependency graph
```text
xtask main.rs
    |
    +--> check-agent-drift
    |      |
    |      +--> agent_registry::AgentRegistry
    |      +--> support_matrix::{derive, consistency}
    |      +--> capability_matrix runtime/publication truth
    |      \--> maintenance packet drift inspector
    |
    +--> refresh-agent
    |      |
    |      +--> maintenance request validator
    |      +--> workspace_mutation::{WorkspacePathJail, apply_mutations}
    |      +--> support_matrix generator reuse
    |      +--> capability_matrix generator reuse
    |      \--> crates-io generated-block refresh
    |
    \--> close-agent-maintenance
           |
           +--> maintenance closeout validator
           +--> request hash/linkage verifier
           +--> maintenance packet doc refresh
           \--> workspace_mutation::{WorkspacePathJail, apply_mutations}
```

### Architecture decisions
- `check-agent-drift` is a read-only aggregation layer. It should call existing truth producers and validators, then classify mismatches by agent instead of inventing a second source of truth.
- `refresh-agent` is the only write-capable maintenance command. It should build one in-memory mutation plan, print it in `--dry-run`, and apply that exact plan in `--write`.
- `close-agent-maintenance` is a closeout validator plus maintenance-doc refresher. It should look more like `close_proving_run` than `onboard_agent`.
- Global generated outputs remain global. Agent scoping happens at the operator contract, not by forking support or capability generators into per-agent implementations.

## Code Quality Guardrails
- Keep `main.rs` as routing glue. Real logic lives in maintenance modules.
- Model drift categories as explicit enums/structs, not ad hoc strings. The command output can still render readable prose.
- Keep request and closeout artifact parsing symmetrical with existing approval/closeout validators: parse, validate path ownership, validate schema, then execute.
- Reuse `workspace_mutation` for every write. No direct `fs::write` calls in maintenance commands.
- Reuse existing generators for support/capability/release refreshes. Do not duplicate their derivation logic inside maintenance code.
- Keep maintenance docs rendering deterministic and side-effect free before write mode.
- If two codepaths need the same validation, extract one small helper. If only one codepath needs it, keep it local. Minimal diff over speculative abstraction.

## Workstreams
### W1. Agent-Scoped Drift Detection
Goal: stop making operators manually discover which truth surfaces disagree.

Deliverables:
- `check-agent-drift` entrypoint
- one explicit drift category taxonomy
- agent-scoped output that aggregates existing validators instead of duplicating them

Primary modules:
- `crates/xtask/src/main.rs`
- `crates/xtask/src/agent_registry.rs`
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/support_matrix/derive.rs`
- `crates/xtask/src/support_matrix/consistency.rs`
- `crates/xtask/src/capability_matrix.rs`
- new maintenance drift module(s) under `crates/xtask/src/`

Exit criteria:
- one command can tell maintainers whether an onboarded agent is clean or which surfaces drifted

### W2. Maintenance Request + Refresh
Goal: add a separate operator path for already-onboarded agents.

Deliverables:
- `maintenance-request.toml` schema
- `refresh-agent --dry-run`
- `refresh-agent --write`
- maintenance pack scaffold under `docs/project_management/next/<agent>-maintenance/`

Primary modules:
- `crates/xtask/src/main.rs`
- new maintenance request and refresh module(s) under `crates/xtask/src/`
- `crates/xtask/src/workspace_mutation.rs`
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/capability_matrix.rs`

Exit criteria:
- already-onboarded maintenance no longer requires `onboard-agent`
- refresh writes stay bounded to maintenance and generated publication surfaces

### W3. Maintenance Closeout + Reopen Rules
Goal: close repairs deterministically without mutating historical onboarding truth.

Deliverables:
- `maintenance-closeout.json` schema
- `close-agent-maintenance` entrypoint
- explicit reopen rules for recurring drift

Primary modules:
- `crates/xtask/src/main.rs`
- new maintenance closeout module(s) under `crates/xtask/src/`
- `crates/xtask/src/workspace_mutation.rs`
- maintenance packet docs under `docs/project_management/next/<agent>-maintenance/**`

Exit criteria:
- closed maintenance runs are explicit and replay-safe
- reopening uses the maintenance lane, not edits to closed onboarding packets

### W4. OpenCode Maintenance Proving Run
Goal: prove the maintenance lane on a real post-onboarding drift issue.

Deliverables:
- `docs/project_management/next/opencode-maintenance/**`
- a maintenance request that cites the stale capability claim
- refreshed maintenance packet truth
- validated maintenance closeout

Primary modules:
- `docs/project_management/next/opencode-implementation/**`
- `docs/specs/opencode-agent-api-backend-contract.md`
- `docs/specs/unified-agent-api/capability-matrix.md`

Exit criteria:
- the repo can repair the OpenCode stale closeout claim through the new M4 flow without conversation archaeology

### OpenCode source-of-truth precedence
To remove ambiguity in the proving run, maintenance must resolve conflicting claims in this order:
1. landed runtime/backend behavior in `crates/agent_api/src/backends/opencode/**`
2. canonical spec contract in `docs/specs/opencode-agent-api-backend-contract.md`
3. generated publication outputs derived from the landed runtime/spec truth
4. historical governance and closeout docs under `docs/project_management/next/opencode-implementation/**`

Implications:
- if historical governance prose disagrees with runtime/spec truth, governance docs are repaired
- if generated capability publication disagrees with runtime/spec truth, the generator output is refreshed
- M4 does not mutate runtime code or raw manifest evidence to make the docs “look consistent”
- the OpenCode proving run is blocked only if runtime behavior and canonical spec disagree with each other, because that is a pre-existing truth conflict outside the M4 write set

## Implementation Sequence
### Phase 1. Drift Contract Lock
Outputs:
- drift taxonomy
- `check-agent-drift`
- OpenCode proving-run target confirmed

Modules touched:
- `crates/xtask/src/main.rs`
- maintenance drift module(s)
- `crates/xtask/src/agent_registry.rs`
- existing support/capability matrix readers

Implementation notes:
- define the drift categories first, because every later artifact needs those names
- keep this phase read-only
- prove the OpenCode stale capability claim appears as one of the categories instead of a one-off doc complaint

Exit gate:
- one command can expose the OpenCode stale capability claim as maintenance drift

### Phase 2. Maintenance Request + Refresh
Outputs:
- maintenance request schema
- maintenance pack scaffold
- `refresh-agent --dry-run/--write`

Modules touched:
- `crates/xtask/src/main.rs`
- maintenance request / refresh module(s)
- `crates/xtask/src/workspace_mutation.rs`
- existing support/capability/release refresh call sites

Implementation notes:
- keep one request artifact as the source of truth for both dry-run and write
- scaffold the maintenance packet root in this phase so later closeout work never has to infer paths
- reject runtime-owned actions at request-validation time, not halfway through mutation planning

Exit gate:
- maintenance writes are bounded and replay-safe

### Phase 3. Maintenance Closeout
Outputs:
- closeout schema
- `close-agent-maintenance`
- reopen rules

Modules touched:
- `crates/xtask/src/main.rs`
- maintenance closeout module(s)
- maintenance packet doc rendering helpers

Implementation notes:
- mirror the validation posture of `close_proving_run`
- closeout only succeeds when request linkage, resolved findings, and deferred-or-none truth all line up
- keep reopen rules documentary and explicit, not implicit via edits to historical onboarding docs

Exit gate:
- maintenance history closes without mutating closed onboarding packets

### Phase 4. OpenCode Proving Run
Outputs:
- OpenCode maintenance pack
- repaired capability-claim truth
- validated closeout

Modules touched:
- `docs/project_management/next/opencode-maintenance/**`
- `docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md`
- generated publication outputs touched by the repair

Implementation notes:
- treat OpenCode as a proving run, not a bespoke exception path
- the proving run is only valid if a new maintainer can reproduce the repair from the maintenance packet and commands alone
- if the proving run needs manual runtime or evidence follow-up, record it explicitly in the maintenance pack instead of hiding it in the closeout prose

Exit gate:
- the repo can repair one already-onboarded agent boringly

## Error & Rescue Registry
| Method / Codepath | What can go wrong | Failure class | Rescued? | Rescue action | User sees |
|---|---|---|---|---|---|
| `check-agent-drift --agent` | unknown or non-onboarded agent id | validation error | yes | reject before comparison work | exit `2` |
| drift aggregation | one source loads, another source fails | partial truth | yes | fail closed with category-specific error | explicit drift/load failure |
| maintenance request parse | malformed TOML or invalid `requested_control_plane_actions` | validation error | yes | reject before refresh plan build | exit `2` |
| maintenance request path | request artifact escapes maintenance root | ownership violation | yes | reject before artifact load | exit `2` |
| refresh write plan | request implies runtime-owned mutation | scope violation | yes | reject before any writes | exit `2` |
| refresh apply | one generated surface diverges mid-transaction | mutation error | yes | rollback staged writes | repo unchanged |
| maintenance closeout | request linkage missing or hashes do not match | validation error | yes | reject closeout | exit `2` |
| OpenCode proving run | stale claim cannot be reconciled to runtime/spec truth | needs-context | no | block closeout until maintainer decides source of truth | blocked docs update |

## Test Strategy
### Test Diagram
```text
POST-ONBOARDING MAINTENANCE
===========================
[+] already-onboarded agent -> check-agent-drift
    |
    ├── [GAP -> validation] unknown agent fails closed
    ├── [GAP -> aggregation] support publication drift is surfaced per agent
    ├── [GAP -> aggregation] capability/runtime drift is surfaced per agent
    └── [GAP -> aggregation] governance packet drift is surfaced per agent

[+] maintenance-request.toml -> refresh-agent --dry-run / --write
    |
    ├── [GAP -> validation] request outside maintenance root fails
    ├── [GAP -> validation] runtime-owned actions are rejected
    ├── [GAP -> integration] dry-run and write share the same plan
    ├── [GAP -> integration] historical onboarding packet remains untouched
    └── [GAP -> regression] identical replay is a no-op

[+] maintenance-closeout.json -> close-agent-maintenance
    |
    ├── [GAP -> validation] request hash/linkage is required
    ├── [GAP -> validation] resolved plus deferred/explicit-none truth is required
    └── [GAP -> integration] maintenance packet docs refresh without touching onboarding packet docs

OPENCODE PROVING RUN
====================
[+] stale `SEAM-2` capability claim -> maintenance request -> refresh -> closeout
    |
    ├── [GAP -> docs/validation] stale capability claim becomes explicit maintenance drift
    └── [GAP -> regression] repair path is reproducible without conversation history
```

### Required Test Surfaces
- Add `crates/xtask/tests/agent_maintenance_drift.rs`
  - `check_agent_drift_reports_clean_agent`
  - `check_agent_drift_rejects_unknown_agent`
  - `check_agent_drift_reports_support_publication_mismatch`
  - `check_agent_drift_reports_capability_truth_mismatch`
  - `check_agent_drift_reports_governance_doc_mismatch`
- Add `crates/xtask/tests/agent_maintenance_refresh.rs`
  - `refresh_agent_dry_run_matches_write_plan`
  - `refresh_agent_rejects_request_outside_maintenance_root`
  - `refresh_agent_rejects_runtime_owned_actions`
  - `refresh_agent_does_not_touch_onboarding_packet_root`
  - `refresh_agent_replay_is_noop`
- Add `crates/xtask/tests/agent_maintenance_closeout.rs`
  - `close_agent_maintenance_requires_request_linkage`
  - `close_agent_maintenance_requires_resolved_and_deferred_truth`
  - `close_agent_maintenance_rejects_symlinked_output`
  - `opencode_maintenance_proving_run_fixes_stale_capability_claim`

### Verification Commands
- `cargo run -p xtask -- check-agent-drift --agent opencode`
- `cargo run -p xtask -- refresh-agent --request docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml --dry-run`
- `cargo run -p xtask -- refresh-agent --request docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml --write`
- `cargo run -p xtask -- close-agent-maintenance --request docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml --closeout docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix`
- `cargo test -p xtask`
- `make preflight`

### Test Plan Artifact
- `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-feat-cli-agent-onboarding-factory-test-plan-20260421-233454.md`

## Failure Modes Registry
| Codepath | Failure mode | Test required? | Error handling required? | User sees | Logged? |
|---|---|---|---|---|---|
| drift detection | known drift exists but stays hidden in repo-wide generators only | yes | yes | false clean state | yes |
| maintenance scope | maintenance path widens into new-agent onboarding or runtime mutation | yes | yes | unsafe write rejection | yes |
| packet immutability | refresh mutates historical onboarding packets | yes | yes | validation failure | yes |
| publication repair | agent-scoped repair misses global generated outputs | yes | yes | stale support/capability docs remain | yes |
| governance truth | maintenance closeout claims clean state while deferrals still exist | yes | yes | closeout rejected | yes |
| OpenCode proving run | stale capability claim remains unrepairable without archaeology | yes | yes | blocked proving run | yes |

Critical gap rule:
- if maintenance can mutate runtime-owned code or historical onboarding packet roots, M4 is not ready
- if OpenCode cannot prove the repair lane on a real drift case, M4 is not ready

## Security Review
- maintenance request and closeout artifacts are new trust boundaries and must be path-jailed
- `refresh-agent` must never infer permission to mutate runtime-owned code from a maintenance request
- agent-scoped drift checks must not trust packet docs over runtime/spec truth
- global generated outputs must refresh deterministically or fail closed
- the maintenance lane should reuse the same symlink and rollback protections that M2/M3 added for onboarding

## Performance Review
- `check-agent-drift` should aggregate existing support/capability validators instead of re-implementing them
- `refresh-agent` should batch planned writes into one transaction instead of re-running file updates per surface
- maintenance should stay agent-scoped at the operator layer even when publication outputs are global files

## Worktree Parallelization Strategy
### Dependency Table
| Step | Modules touched | Depends on |
|---|---|---|
| W1. drift detection | `crates/xtask/src/main.rs`, maintenance drift module(s), `support_matrix/**`, `capability_matrix.rs`, tests | — |
| W2. request + refresh | `crates/xtask/src/main.rs`, maintenance request/refresh module(s), `workspace_mutation.rs`, tests | W1 |
| W3. closeout + reopen rules | `crates/xtask/src/main.rs`, maintenance closeout module(s), maintenance docs templates, tests | W2 |
| W4. OpenCode maintenance pack scaffold | `docs/project_management/next/opencode-maintenance/**`, related governance docs | W1, W2 |
| W5. OpenCode proving run execution | generated publication outputs, maintenance packet closeout docs | W3, W4 |

### Parallel Lanes
Lane A: W1 -> W2 -> W3
Core command lane. This stays sequential because all three steps touch `crates/xtask/src/main.rs` and the same maintenance command namespace.

Lane B: W4
Docs scaffold lane. This can start after W2 freezes the request schema and maintenance pack root shape.

Lane C: W5
Final proving-run lane. This starts only after Lane A and Lane B merge, because it consumes the final command contract plus the concrete OpenCode maintenance packet.

### Execution Order
1. Launch Lane A alone. W1 must land first because it defines the taxonomy and exit codes the rest of the milestone depends on.
2. After W2 stabilizes the request schema, launch W3 and W4 in separate worktrees only if W4 stays docs-only.
3. Merge W3 first so the final closeout contract is fixed.
4. Run W5 last in the main integration worktree using the merged command surface and packet docs.

### Conflict Flags
- W1, W2, and W3 all touch `crates/xtask/src/main.rs`. Do not parallelize those.
- W2 and W3 both touch the maintenance module namespace and `crates/xtask/tests/**`. Split test files early if two worktrees are used after W2.
- W4 must stay packet-doc scoped. If it starts changing command behavior, it no longer belongs in a parallel lane.
- W5 is integration-only. If earlier lanes are still moving, W5 becomes churn and should wait.

## Completion Summary
- Step 0: Scope challenge, separate maintenance lane confirmed. No onboarding scope creep, no new crate, no lifecycle umbrella.
- Architecture review: one `xtask` maintenance namespace, shared generators and mutation helpers reused, global publications remain global.
- Code quality review: explicit artifacts, explicit exit codes, deterministic dry-run/write parity, and reuse of existing validation/mutation primitives required.
- Test review: diagram produced, 14 required coverage points identified across drift detection, refresh, closeout, and OpenCode proving-run regression coverage.
- Performance review: aggregation reuse, batched writes, and agent-scoped operator semantics locked.
- Not in scope: written.
- What already exists: written.
- Failure modes: two critical gates remain non-negotiable, runtime-owned mutation must stay impossible and OpenCode must prove the lane on a real drift case.
- Parallelization: three lanes total, one narrow docs-only parallel window after W2, core command work remains sequential by design.

## Deferred To TODOS.md
- automate maintenance-request generation from upstream release scans only after two successful maintenance cycles prove the shape
- add manifest-evidence refresh helpers only after the repo proves it can keep manifest evidence ownership separate from control-plane refresh
- consider batched multi-agent maintenance scheduling only after per-agent maintenance is boring

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
