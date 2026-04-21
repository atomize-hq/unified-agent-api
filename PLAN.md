# CLI Agent Onboarding Factory - PLAN

Source: `/Users/spensermcconnell/.gstack/projects/atomize-hq-unified-agent-api/spensermcconnell-main-design-20260420-151505.md`  
Status: Ready for implementation planning  
Last updated (UTC): 2026-04-20

## Purpose
Turn the current post-OpenCode learning into one bounded M1 implementation plan. M1 does not build a universal agent factory. It replaces the two hardcoded control-plane enrollment seams, pins one committed descriptor registry for the three seeded agents, and lands `xtask onboard-agent --dry-run` so an already-approved next agent deterministically produces the next executable artifact set instead of another vague handoff.

## Scope Lock
- Keep M1 focused on the onboarding bridge.
- Keep runtime truth owned by wrapper crates and backend implementations.
- Keep `codex`, `claude_code`, and `opencode` as the only seeded agents in M1.
- Keep `docs/specs/unified-agent-api/support-matrix.md` as the support/publication semantics owner.
- Keep `docs/specs/unified-agent-api/capability-matrix.md` as the capability-advertising projection.
- Keep `crates/xtask` as the control-plane implementation surface.
- Keep recommendation HITL and packet-driven if formalizing it would delay the bridge fix.
- Keep all generated onboarding outputs inside control-plane-owned docs, manifest-root, and release/publication surfaces.

## Success Criteria
- One committed registry artifact exists at `crates/xtask/data/agent_registry.toml`.
- The registry cleanly seeds `codex`, `claude_code`, and `opencode`.
- `cargo run -p xtask -- support-matrix --check` derives enrolled roots from the registry instead of a hardcoded list.
- `cargo run -p xtask -- capability-matrix` derives enrolled backends and canonical target metadata from the registry instead of hardcoded builtin assumptions.
- `cargo run -p xtask -- onboard-agent --dry-run --agent-id <approved-agent>` deterministically previews all generated control-plane outputs without mutating runtime-owned files.
- Registry parity, support publication parity, capability publication parity, and dry-run ownership safety are all covered by automated tests.
- `make preflight` remains the final integration gate.

## What Already Exists
The plan must reuse these surfaces instead of rebuilding them:

- Existing control-plane modules:
  - [crates/xtask/src/main.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/main.rs)
  - [crates/xtask/src/support_matrix.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/support_matrix.rs)
  - [crates/xtask/src/support_matrix/derive.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/support_matrix/derive.rs)
  - [crates/xtask/src/capability_matrix.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/capability_matrix.rs)
  - [crates/xtask/src/wrapper_coverage_shared.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/wrapper_coverage_shared.rs)
- Existing seeded wrappers and backends:
  - `crates/codex/`
  - `crates/claude_code/`
  - `crates/opencode/`
  - `crates/agent_api/src/backends/{codex,claude_code,opencode}/`
- Existing manifest roots:
  - `cli_manifests/codex/`
  - `cli_manifests/claude_code/`
  - `cli_manifests/opencode/`
- Existing publication contracts:
  - [docs/specs/unified-agent-api/support-matrix.md](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/support-matrix.md)
  - [docs/specs/unified-agent-api/capability-matrix.md](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/capability-matrix.md)
  - [docs/crates-io-release.md](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/crates-io-release.md)
- Existing test posture:
  - `crates/xtask/tests/support_matrix_*.rs`
  - `crates/xtask/tests/c8_spec_capability_matrix_*.rs`
  - `make preflight`

## Not In Scope
- Building a universal runtime probe framework.
- Editing runtime/backend behavior in `crates/agent_api/src/backends/**`.
- Generating wrapper surface truth or backend implementation files.
- Replacing backend-owned capability computation with registry-owned logic.
- Formalizing `recommend-agent` if that delays M1.
- Solving already-onboarded-agent lifecycle maintenance beyond parity and dry-run safety.
- Reworking the support-matrix or capability-matrix semantics themselves.

## Control-Plane Flow
```text
approved next agent
        |
        v
xtask onboard-agent --dry-run
        |
        +--> crates/xtask/data/agent_registry.toml
        +--> docs/project_management/next/<agent>-cli-onboarding/**
        +--> cli_manifests/<agent>/**
        +--> release/publication touch points
        |
        v
manual runtime follow-up list

agent_registry.toml
        |
        +--> support-matrix root enrollment
        +--> capability-matrix backend enrollment
        +--> release/publication metadata wiring
```

Runtime truth stays outside the registry:

```text
wrapper crates + backend implementations
        |
        +--> wrapper surface declarations
        +--> runtime capability computation
        +--> probe / preflight / guard logic
```

## Concrete Artifact Decisions
These are part of M1, not open design questions:

| Concern | Decision |
|---|---|
| Registry path | `crates/xtask/data/agent_registry.toml` |
| Registry format | TOML |
| Registry owner | `crates/xtask` control plane |
| Seeded agents | `codex`, `claude_code`, `opencode` |
| Support-matrix consumer | registry-backed enrolled roots |
| Capability-matrix consumer | registry-backed enrolled backends and canonical target metadata |
| New onboarding command | `cargo run -p xtask -- onboard-agent --dry-run --agent-id <approved-agent>` |
| Docs scaffold root | `docs/project_management/next/<agent>-cli-onboarding/` |
| Manifest-root scaffold | `cli_manifests/<agent>/` |
| Final M1 gate | `make preflight` |

## Exact `onboard-agent --dry-run` CLI Contract
M1 must not leave the CLI shape to implementer taste.

### Accepted invocation
```bash
cargo run -p xtask -- onboard-agent --dry-run \
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

### Flag semantics
- `--dry-run` is required in M1. Invocation without `--dry-run` must fail closed with a usage error until M2.
- `--canonical-target` is repeatable and order-preserving.
- `--always-on-capability` is repeatable and order-insensitive in parsing, but must render sorted in preview output.
- `--target-gated-capability` encodes one capability plus one or more targets.
- `--config-gated-capability` encodes one capability, one config key, and optional target intersection. If targets are present, the capability is advertised only when both the config and target gate pass.
- `--backend-extension` is repeatable and reserved for backend-owned extension ids such as `backend.codex.exec_stream`.
- `--support-matrix-enabled` and `--capability-matrix-enabled` are required booleans, not inferred defaults.

### Stdout contract
Successful dry-run output must render these sections in this exact order:

1. `== ONBOARD-AGENT DRY RUN ==`
2. `== INPUT SUMMARY ==`
3. `== REGISTRY ENTRY PREVIEW ==`
4. `== DOCS SCAFFOLD PREVIEW ==`
5. `== MANIFEST ROOT PREVIEW ==`
6. `== RELEASE/PUBLICATION TOUCHPOINTS ==`
7. `== MANUAL FOLLOW-UP ==`
8. `== RESULT ==`

### Exit behavior
- exit `0`: successful dry-run preview
- exit `2`: validation or ownership conflict
- exit `1`: unexpected internal failure

### Dry-run side effects
- no filesystem writes
- no temp-file writes inside the repo
- no edits to wrapper crates, backend implementation files, workflow files, or release scripts
- preview text only

## Registry Contract
The registry is the control-plane source of truth for declaration and enrollment metadata only.

### Registry-owned fields
- agent identity and display metadata
- crate path and manifest-root path
- package and release/publication metadata
- canonical targets
- wrapper-coverage binding metadata
- normalized capability-declaration shape
- onboarding scaffold metadata

### Explicitly not registry-owned
- live capability probes
- final runtime capability values
- wrapper surface contents
- backend execution behavior
- backend-specific guard/preflight logic
- generated support rows or generated capability rows

### Required field schema
| Field | Type | Required | Shape | Notes |
|---|---|---|---|---|
| `agent_id` | string | yes | unique | Control-plane identifier. |
| `display_name` | string | yes | scalar | Human-facing only. |
| `crate_path` | string | yes | repo-relative path | Wrapper crate root. |
| `backend_module` | string | yes | repo-relative path | Backend module path under `crates/agent_api/src/backends/`. |
| `manifest_root` | string | yes | repo-relative path | Root under `cli_manifests/`. |
| `package_name` | string | yes | scalar | Publishable crate package name. |
| `canonical_targets` | array<string> | yes | 1..n | Ordered capability-projection/scaffold target set. |
| `wrapper_coverage.binding_kind` | string | yes | enum | Start with `generated_from_wrapper_crate`. |
| `wrapper_coverage.source_path` | string | yes | repo-relative path | Wrapper-owned source/generator path. |
| `capability_declaration.always_on` | array<string> | yes | 0..n | Always-advertised ids. |
| `capability_declaration.target_gated` | array<table> | yes | 0..n | `capability_id` plus non-empty `targets = []`. |
| `capability_declaration.config_gated` | array<table> | yes | 0..n | `capability_id` plus `config_key`, optional `targets = []` for intersection gating. |
| `capability_declaration.backend_extensions` | array<string> | yes | 0..n | Backend-owned extension ids. |
| `publication.support_matrix_enabled` | bool | yes | scalar | Include in support publication. |
| `publication.capability_matrix_enabled` | bool | yes | scalar | Include in capability publication. |
| `release.docs_release_track` | string | yes | scalar | Start with `crates-io` for seeded crates. |
| `scaffold.onboarding_pack_prefix` | string | yes | scalar | Prefix for generated planning paths. |

Evaluation rules:
- `support_matrix` root enrollment comes from registry membership, but target row derivation still reads each enrolled root's committed `current.json.expected_targets`. `canonical_targets` does not replace manifest-root expected targets for support publication.
- `capability_matrix` uses `canonical_targets` for target-sensitive capability projection.
- Final advertised capability set is the union of `always_on`, matching `target_gated`, and matching `config_gated`, plus `backend_extensions`.
- `backend_extensions` may be emitted only if the owning backend currently advertises them under default config or an explicitly declared config gate.

## Pinned Initial Registry Contents
M1 should start from these exact seeded entries. Workers should not infer or re-derive them from prose.

```toml
[[agents]]
agent_id = "codex"
display_name = "Codex CLI"
crate_path = "crates/codex"
backend_module = "crates/agent_api/src/backends/codex"
manifest_root = "cli_manifests/codex"
package_name = "unified-agent-api-codex"
canonical_targets = ["x86_64-unknown-linux-musl"]

[agents.wrapper_coverage]
binding_kind = "generated_from_wrapper_crate"
source_path = "crates/codex"

[agents.capability_declaration]
always_on = [
  "agent_api.run",
  "agent_api.events",
  "agent_api.events.live",
  "agent_api.control.cancel.v1",
  "agent_api.tools.structured.v1",
  "agent_api.tools.results.v1",
  "agent_api.artifacts.final_text.v1",
  "agent_api.session.handle.v1",
  "agent_api.session.fork.v1",
  "agent_api.session.resume.v1",
  "agent_api.config.model.v1",
  "agent_api.exec.add_dirs.v1",
  "agent_api.exec.non_interactive",
]
backend_extensions = [
  "backend.codex.exec.approval_policy",
  "backend.codex.exec.sandbox_mode",
  "backend.codex.exec_stream",
]

[[agents.capability_declaration.target_gated]]
capability_id = "agent_api.tools.mcp.list.v1"
targets = ["x86_64-unknown-linux-musl"]

[[agents.capability_declaration.target_gated]]
capability_id = "agent_api.tools.mcp.get.v1"
targets = ["x86_64-unknown-linux-musl"]

[[agents.capability_declaration.config_gated]]
capability_id = "agent_api.tools.mcp.add.v1"
config_key = "allow_mcp_write"
targets = ["x86_64-unknown-linux-musl"]

[[agents.capability_declaration.config_gated]]
capability_id = "agent_api.tools.mcp.remove.v1"
config_key = "allow_mcp_write"
targets = ["x86_64-unknown-linux-musl"]

[[agents.capability_declaration.config_gated]]
capability_id = "agent_api.exec.external_sandbox.v1"
config_key = "allow_external_sandbox_exec"

[agents.publication]
support_matrix_enabled = true
capability_matrix_enabled = true

[agents.release]
docs_release_track = "crates-io"

[agents.scaffold]
onboarding_pack_prefix = "codex-cli-onboarding"

[[agents]]
agent_id = "claude_code"
display_name = "Claude Code"
crate_path = "crates/claude_code"
backend_module = "crates/agent_api/src/backends/claude_code"
manifest_root = "cli_manifests/claude_code"
package_name = "unified-agent-api-claude-code"
canonical_targets = ["linux-x64", "darwin-arm64", "win32-x64"]

[agents.wrapper_coverage]
binding_kind = "generated_from_wrapper_crate"
source_path = "crates/claude_code"

[agents.capability_declaration]
always_on = [
  "agent_api.run",
  "agent_api.events",
  "agent_api.events.live",
  "agent_api.control.cancel.v1",
  "agent_api.tools.structured.v1",
  "agent_api.tools.results.v1",
  "agent_api.artifacts.final_text.v1",
  "agent_api.session.handle.v1",
  "agent_api.config.model.v1",
  "agent_api.exec.add_dirs.v1",
  "agent_api.exec.non_interactive",
  "agent_api.session.resume.v1",
  "agent_api.session.fork.v1",
]
backend_extensions = [
  "backend.claude_code.print_stream_json",
]

[[agents.capability_declaration.target_gated]]
capability_id = "agent_api.tools.mcp.list.v1"
targets = ["linux-x64", "darwin-arm64", "win32-x64"]

[[agents.capability_declaration.target_gated]]
capability_id = "agent_api.tools.mcp.get.v1"
targets = ["win32-x64"]

[[agents.capability_declaration.config_gated]]
capability_id = "agent_api.tools.mcp.add.v1"
config_key = "allow_mcp_write"
targets = ["win32-x64"]

[[agents.capability_declaration.config_gated]]
capability_id = "agent_api.tools.mcp.remove.v1"
config_key = "allow_mcp_write"
targets = ["win32-x64"]

[[agents.capability_declaration.config_gated]]
capability_id = "agent_api.exec.external_sandbox.v1"
config_key = "allow_external_sandbox_exec"

[agents.publication]
support_matrix_enabled = true
capability_matrix_enabled = true

[agents.release]
docs_release_track = "crates-io"

[agents.scaffold]
onboarding_pack_prefix = "claude-code-cli-onboarding"

[[agents]]
agent_id = "opencode"
display_name = "OpenCode"
crate_path = "crates/opencode"
backend_module = "crates/agent_api/src/backends/opencode"
manifest_root = "cli_manifests/opencode"
package_name = "unified-agent-api-opencode"
canonical_targets = ["linux-x64", "darwin-arm64", "win32-x64"]

[agents.wrapper_coverage]
binding_kind = "generated_from_wrapper_crate"
source_path = "crates/opencode"

[agents.capability_declaration]
always_on = [
  "agent_api.run",
  "agent_api.events",
  "agent_api.events.live",
  "agent_api.config.model.v1",
  "agent_api.session.resume.v1",
  "agent_api.session.fork.v1",
]
backend_extensions = []

[agents.publication]
support_matrix_enabled = true
capability_matrix_enabled = true

[agents.release]
docs_release_track = "crates-io"

[agents.scaffold]
onboarding_pack_prefix = "opencode-cli-onboarding"
```

## Consumer Cutover Map
| Consumer | Current hardcoded seam | M1 change |
|---|---|---|
| [crates/xtask/src/support_matrix.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/support_matrix.rs) | `CURRENT_AGENT_ROOTS` hardcodes enrolled manifest roots | Replace with registry-backed root enrollment and root metadata loading. |
| [crates/xtask/src/capability_matrix.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/capability_matrix.rs) | builtin backend collection and canonical target assumptions are hardcoded | Load enrolled backends and canonical target metadata from the registry while keeping runtime capability computation backend-owned. |
| [crates/xtask/src/main.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/main.rs) | no onboarding factory command surface exists | Add `OnboardAgent` entrypoint. |
| [docs/crates-io-release.md](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/crates-io-release.md) | release assumptions are tied to the current fixed crate set | Use registry release metadata for new-agent enrollment while preserving current seeded-crate order and rules. |
| `docs/project_management/next/**` | handoff naming and path prefix are manual | Standardize via `scaffold.onboarding_pack_prefix`. |

## Release And Publication Touchpoint Matrix
Workers should not guess which release files are in scope for M1.

| File | M1 status | Exact rule |
|---|---|---|
| [docs/crates-io-release.md](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/crates-io-release.md) | touch only if release docs become stale | Seeded-agent normalization should not rewrite publish order. Dry-run for a new agent may preview a future docs addition, but M1 does not mutate it. |
| [.github/workflows/publish-crates.yml](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.github/workflows/publish-crates.yml) | out of scope | No M1 edits. The workflow already computes publishable crates from cargo metadata. |
| [scripts/publish_crates.py](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/scripts/publish_crates.py) | out of scope | No M1 edits unless M1 proves the workflow cannot model a registry-backed new crate, which is not expected. |
| [scripts/validate_publish_versions.py](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/scripts/validate_publish_versions.py) | out of scope | No M1 edits for seeded normalization. |
| [scripts/check_publish_readiness.py](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/scripts/check_publish_readiness.py) | out of scope | No M1 edits for seeded normalization. |
| `Cargo.toml` | preview-only for new-agent dry-run | Only preview as a future mutation touchpoint when the proposed `crate_path` is a new workspace member not already present. |
| `crates/<agent>/Cargo.toml` | preview-only for new-agent dry-run | Only preview as future manual/runtime-owned work, not a dry-run-generated file. |

Preview contract for W5:
- If the proposed package name matches one of the seeded publishable crates, `== RELEASE/PUBLICATION TOUCHPOINTS ==` must print `NO RELEASE CHANGES`.
- If the proposed package name is new, the section must list exactly which of `Cargo.toml` and `docs/crates-io-release.md` would require future mutation in M2, and must explicitly print that workflow and script files remain unchanged in M1.

## M1 Workstreams
### W1. Registry schema and loader
Goal: add the registry artifact and fail-closed parsing/validation.

Primary work:
- add `crates/xtask/data/agent_registry.toml`
- add registry loader and validation module(s) under `crates/xtask/src/`
- enforce uniqueness for `agent_id`, `crate_path`, `backend_module`, and `manifest_root`
- reject malformed target-gated or config-gated declarations

Acceptance:
- the seeded registry parses cleanly
- malformed or duplicate entries fail closed
- no consumer is cut over yet without parity checks

### W2. Seeded normalization and parity
Goal: seed the three current agents and prove the registry matches existing enrollment before cutover.

Primary work:
- seed `codex`, `claude_code`, and `opencode`
- add parity validation comparing current hardcoded enrollment to registry-derived enrollment
- allow temporary dual-source coexistence only behind explicit parity tests

Acceptance:
- registry-derived enrolled roots match current `support_matrix` enrollment
- registry-derived enrolled backends match current `capability_matrix` enrollment
- CI fails on drift while both sources coexist

### W3. Support-matrix cutover
Goal: move root enrollment to the registry without changing support publication semantics.

Primary work:
- replace `CURRENT_AGENT_ROOTS`
- preserve current publication paths and row semantics
- keep derivation single-pass and deterministic

Primary touchpoints:
- `crates/xtask/src/support_matrix.rs`
- `crates/xtask/src/support_matrix/derive.rs`
- `crates/xtask/tests/support_matrix_*.rs`

Acceptance:
- row set and staleness behavior remain unchanged for seeded agents
- support publication reads enrolled roots from the registry

### W4. Capability-matrix cutover
Goal: move backend enrollment and canonical target metadata to the registry without moving runtime truth into the registry.

Primary work:
- replace builtin backend enrollment assumptions
- source canonical target metadata from the registry
- keep backend capability computation inside backend-owned code

Primary touchpoints:
- `crates/xtask/src/capability_matrix.rs`
- `crates/xtask/tests/c8_spec_capability_matrix_*.rs`

Acceptance:
- seeded-agent capability inventory stays stable unless an intentional backend change occurs
- registry only shapes enrollment and declaration metadata

### W5. `onboard-agent --dry-run`
Goal: land the bridge command without mutation mode.

Primary work:
- add a neutral `OnboardAgent` entrypoint to `xtask`
- preview registry diff, docs scaffold, manifest-root scaffold, and release/publication touch points
- print a deterministic handoff summary naming runtime-owned manual follow-up work
- fail closed on duplicate ids, ambiguous ownership, or pre-existing conflicting targets

Expected dry-run outputs:
- registry entry preview in `crates/xtask/data/agent_registry.toml`
- docs scaffold preview under `docs/project_management/next/<agent>-cli-onboarding/`
- manifest-root scaffold preview under `cli_manifests/<agent>/`
- release/publication touch-point preview
- final handoff summary

Required scaffold file contract:
- `docs/project_management/next/<agent>-cli-onboarding/README.md`
- `docs/project_management/next/<agent>-cli-onboarding/scope_brief.md`
- `docs/project_management/next/<agent>-cli-onboarding/seam_map.md`
- `docs/project_management/next/<agent>-cli-onboarding/threading.md`
- `docs/project_management/next/<agent>-cli-onboarding/review_surfaces.md`
- `docs/project_management/next/<agent>-cli-onboarding/governance/remediation-log.md`
- `docs/project_management/next/<agent>-cli-onboarding/HANDOFF.md`
- `cli_manifests/<agent>/current.json`
- `cli_manifests/<agent>/versions/.gitkeep`
- `cli_manifests/<agent>/pointers/latest_supported/.gitkeep`
- `cli_manifests/<agent>/pointers/latest_validated/.gitkeep`
- `cli_manifests/<agent>/reports/.gitkeep`

Required ownership markers:
- every generated Markdown preview must start with `<!-- generated-by: xtask onboard-agent; owner: control-plane -->`
- `HANDOFF.md` must include a `## Manual Runtime Follow-Up` section
- no ownership markers are required for `.gitkeep`

Acceptance:
- dry-run does not mutate wrapper or backend implementation files
- output order is deterministic
- the command always terminates in a concrete next executable artifact set plus manual follow-up list
- command flags, stdout sections, and exit behavior match the pinned contract above

## Minimal Execution Sequence
```text
W1 registry schema + loader
    |
    v
W2 seeded normalization + parity
    |
    +--> W3 support-matrix cutover
    +--> W4 capability-matrix cutover
    +--> W5 onboard-agent --dry-run
               |
               v
         parity + dry-run safety + preflight
```

Do not reverse this. The registry shape must be pinned before the consumers or the command surface harden around it.

## Test Strategy
### Codepath coverage
| Codepath | What must be proven |
|---|---|
| Registry load | malformed or duplicate entries fail closed |
| Registry parity | seeded registry matches current builtin enrollment before cutover |
| Support-matrix cutover | registry-backed roots publish the same row set and staleness behavior as today |
| Capability-matrix cutover | registry-backed backend enrollment preserves seeded-agent capability inventory |
| Dry-run scaffold | generated outputs are deterministic and do not mutate runtime-owned files |
| Overwrite safety | existing generated targets fail closed unless explicit update mode exists |

### Required test surfaces
- keep `crates/xtask/tests/support_matrix_derivation.rs`
- keep `crates/xtask/tests/support_matrix_consistency.rs`
- keep `crates/xtask/tests/support_matrix_entrypoint.rs`
- keep `crates/xtask/tests/support_matrix_staleness.rs`
- keep `crates/xtask/tests/c8_spec_capability_matrix_paths.rs`
- keep `crates/xtask/tests/c8_spec_capability_matrix_staleness.rs`
- add registry loader and parity tests under `crates/xtask/tests/`
- add dry-run scaffold golden tests under `crates/xtask/tests/`
- add duplicate-id and pre-existing-target conflict tests

### Commands
- `cargo test -p xtask`
- `cargo run -p xtask -- support-matrix --check`
- `cargo run -p xtask -- capability-matrix`
- `cargo run -p xtask -- onboard-agent --dry-run --agent-id <approved-agent>`
- `make preflight`

## Failure Modes
| Codepath | Failure | Guardrail |
|---|---|---|
| registry seed data | duplicate `agent_id` or overlapping roots | loader validation + conflict tests |
| support-matrix cutover | enrolled root omitted and rows silently disappear | parity tests + deterministic row-set checks |
| capability-matrix cutover | canonical target drift changes published inventory unexpectedly | parity tests + staleness tests |
| dry-run scaffold | command touches runtime-owned wrapper/backend files | ownership rules + golden tests |
| release wiring | new crate metadata bypasses the current crates.io release flow | release metadata checks + docs update rules |
| handoff summary | dry-run emits scaffolding but not the next executable artifact | explicit command contract + golden tests |

Critical M1 gap rule:
- M1 is not complete unless registry parity, support publication parity, capability publication parity, and dry-run ownership safety each have automated coverage plus fail-closed behavior.

## Parallelization Strategy
| Lane | Modules touched | Depends on |
|---|---|---|
| A. registry schema + loader | `crates/xtask/data/**`, new registry module(s), `crates/xtask/src/main.rs` | — |
| B. support-matrix cutover | `crates/xtask/src/support_matrix*`, related tests | A |
| C. capability-matrix cutover | `crates/xtask/src/capability_matrix.rs`, related tests | A |
| D. dry-run onboarding command | new onboarding module, `crates/xtask/src/main.rs`, scaffold tests | A |

Execution order:
- launch Lane A first
- once schema validation and seeded entries are pinned, launch B, C, and D in parallel
- merge only after parity and `make preflight` pass

Conflict flags:
- Lanes A and D both touch `crates/xtask/src/main.rs`
- Lanes B and C both depend on the final registry shape
- D should not guess artifact names or path prefixes before A lands

## Follow-On After This Plan
M1 is the current plan-of-record. The follow-on shape stays explicit so implementation does not get clumsy after v1:

- `M2`: add real mutation mode and use the next approved real agent as the first full proving run
- `M3`: formalize recommendation output as a stable approval artifact, either via `xtask recommend-agent` or a deterministic packet generator
- `M4`: add maintenance ergonomics, drift detection, and repeatability hardening after real proving runs expose the clumsy parts
