# ADR-0023 — CLI manifest coverage for headless automation
#
# Note: Run `make adr-fix ADR=docs/adr/0023-cli-manifest-coverage-for-headless-automation.md`
# after editing to update the ADR_BODY_SHA256 drift guard.

## Status
- Status: Draft
- Date (UTC): 2026-04-06
- Owner(s): spensermcconnell

## Scope

- Extend the existing parity-lane / CLI-manifest system so backend capabilities implemented via
  headless automation are represented as first-class wrapper coverage.
- Cover both built-in parity lanes:
  - `cli_manifests/codex`
  - `cli_manifests/claude_code`
- Define the source-of-truth, generation, report, and validation changes needed in:
  - `crates/codex/src/wrapper_coverage_manifest.rs`
  - `crates/claude_code/src/wrapper_coverage_manifest.rs`
  - `crates/xtask`

## Related Docs

- Backend transport decision:
  - `docs/adr/0022-universal-agent-api-terminal-automation.md`
- Parity-lane foundations:
  - `docs/adr/0001-codex-cli-parity-maintenance.md`
  - `docs/adr/0002-codex-cli-parity-coverage-mapping.md`
  - `docs/adr/0003-wrapper-coverage-auto-generation.md`
  - `docs/adr/0006-agent-wrappers-workspace.md`
- Parity-lane roots:
  - `cli_manifests/codex/README.md`
  - `cli_manifests/claude_code/README.md`
- Wrapper coverage sources:
  - `crates/codex/src/wrapper_coverage_manifest.rs`
  - `crates/claude_code/src/wrapper_coverage_manifest.rs`
- Current `xtask` entrypoints:
  - `crates/xtask/src/main.rs`
  - `crates/xtask/src/codex_wrapper_coverage.rs`
  - `crates/xtask/src/claude_wrapper_coverage.rs`
  - `crates/xtask/src/codex_report.rs`
  - `crates/xtask/src/codex_validate.rs`

## Executive Summary (Operator)

ADR_BODY_SHA256: 0fe7d1ec94c341ea14ab7501f8d564faf8be1c98f92eaa5111ebaffa3d91099b

### Decision (draft)

- Keep `cli_manifests/*` as the evidence and coverage layer for wrapper support.
- Do **not** implement runtime automation in `cli_manifests/*`.
- When a wrapper crate adds headless automation support for a capability not reachable through the
  backend's primary structured transport, the parity lane must represent that capability as covered.
- The authoritative source of that claim remains Rust source in the backend crates; generated JSON
  artifacts remain derived outputs:
  - `crates/codex/src/wrapper_coverage_manifest.rs`
  - `crates/claude_code/src/wrapper_coverage_manifest.rs`
  - `cli_manifests/<agent>/wrapper_coverage.json`
- The `xtask` coverage/report/validate pipeline must be extended so reports can distinguish:
  - uncovered support gaps,
  - intentionally unsupported surfaces,
  - surfaces covered via structured primary transport,
  - surfaces covered via headless automation fallback.

### Why

- Backend implementation alone is not enough; the repo's maintenance system is built around parity
  lanes and generated coverage reports.
- If headless automation closes a real support gap, the parity lane must stop reporting that gap as
  missing.
- The repo already has the right architecture for this:
  wrapper coverage is declared in backend-crate Rust and emitted by `xtask` into deterministic JSON.

## Problem / Context

This repo already uses `cli_manifests/<agent>` as the reviewable source of evidence for wrapper
coverage over upstream CLI surfaces.

Today that system already exists for both built-in lanes:

- snapshots / unions / reports / version metadata under `cli_manifests/*`
- generated wrapper coverage JSON
- `xtask` commands to generate, report, and validate those artifacts

That means a backend-crate-only automation implementation is insufficient. If a capability is now
covered by a secondary automation transport, but the parity lane still treats it as unsupported,
the repo will generate the wrong work queue and the wrong maintenance signal.

The missing piece is not a new runtime layer in `cli_manifests/*`; it is parity metadata and report
support that can honestly say:

- this surface is supported by the wrapper, and
- it is supported through a headless automation transport rather than the backend's primary
  structured transport.

## Goals

- Make automation-backed support visible in parity artifacts and reports.
- Preserve the current "Rust source of truth -> generated JSON" model.
- Keep the parity system deterministic and reviewable.
- Avoid hand-editing generated coverage/report artifacts.

## Non-Goals

- Implementing PTY/background automation in `cli_manifests/*`.
- Replacing upstream snapshot discovery.
- Replacing the existing coverage levels (`explicit`, `passthrough`, `intentionally_unsupported`,
  etc.) with an entirely new classification model.
- Changing `agent_api` capability semantics in this ADR.

## Decision

### 1. `cli_manifests/*` remains the evidence layer, not the runtime layer

Runtime automation is implemented in:

- `crates/codex`
- `crates/claude_code`

The parity lanes under `cli_manifests/*` remain responsible for:

- recording what upstream surfaces exist,
- recording what the wrapper supports,
- generating coverage deltas,
- validating artifact invariants.

### 2. Wrapper coverage remains crate-owned source of truth

The authoritative source for wrapper support claims remains backend-crate Rust:

- `crates/codex/src/wrapper_coverage_manifest.rs`
- `crates/claude_code/src/wrapper_coverage_manifest.rs`

Generated files remain generated:

- `cli_manifests/codex/wrapper_coverage.json`
- `cli_manifests/claude_code/wrapper_coverage.json`

This ADR explicitly rejects hand-maintained JSON as the primary truth for automation-backed support.

### 3. Coverage metadata must distinguish transport provenance

The parity system should distinguish at least two support shapes:

- support via the backend's primary structured transport
- support via headless automation fallback

This ADR does not pin the final schema field names, but it does require that the wrapper coverage
and report pipeline carry machine-readable provenance sufficient to answer:

- which transport family is claiming support, and
- whether that transport is the primary/default path or a fallback path

Examples of transport families that the metadata may need to distinguish for built-in lanes:

- Claude Code:
  - `claude_print_stream_json`
  - `pty_headless_automation`
- Codex:
  - `codex_exec_stream`
  - `codex_app_server_jsonrpc`
  - `pty_headless_automation`

### 4. Support classification remains coverage-level based

Automation-backed support should count as support under the existing coverage/report model when the
wrapper can honestly claim the surface is supported.

That means:

- `explicit` still means supported
- `passthrough` still means weakly supported
- `intentionally_unsupported` still means deliberately waived with rationale

The new transport provenance metadata supplements those coverage levels; it does not replace them.

Concretely:

- a capability covered only through headless automation may still be `explicit`
- reports should stop listing that surface under `missing_*` or `unsupported`
- reports should preserve enough provenance to show that the support came from automation fallback,
  not from the primary structured transport

### 5. `xtask` must be extended, not replaced

The current parity tooling already has the right shape:

- `codex-wrapper-coverage`
- `claude-wrapper-coverage`
- `codex-report`
- `codex-validate`

This ADR requires extending those tools so automation-backed support is propagated through:

- wrapper coverage generation
- report generation
- validation
- optional triad/work-queue generation

The existing deterministic artifact model remains in place.

### 6. Snapshot discovery remains upstream-surface focused

This ADR does not change how upstream CLI surfaces are discovered.

Snapshots and union manifests continue to answer:

- what upstream commands/flags/args exist

Wrapper coverage continues to answer:

- whether and how the wrapper supports them

Headless automation changes the second question, not the first.

## Required Follow-on Work

This ADR implies follow-on spec and implementation updates in the parity lanes:

- lane schemas:
  - `cli_manifests/codex/SCHEMA.json`
  - `cli_manifests/claude_code/SCHEMA.json`
- lane rules:
  - `cli_manifests/codex/RULES.json`
  - `cli_manifests/claude_code/RULES.json`
- validator logic:
  - `crates/xtask/src/codex_validate.rs`
  - supporting validator modules
- report generation:
  - `crates/xtask/src/codex_report.rs`
  - supporting report modules
- wrapper coverage generators:
  - `crates/xtask/src/codex_wrapper_coverage.rs`
  - `crates/xtask/src/claude_wrapper_coverage.rs`

The exact field shapes should be pinned in the lane schema/rules docs rather than left implicit in
the ADR.

## Consequences

### Positive

- Parity reports will reflect real wrapper support once automation-backed transports land.
- Maintainers can see where a capability is covered by a fallback transport instead of assuming all
  support comes from the structured transport.
- The repo keeps one consistent maintenance loop across both built-in agents.

### Negative

- The parity schema and tooling become a bit more complex.
- Report reviewers must reason about transport provenance in addition to simple support/no-support.
- Some existing reports and validation rules will need coordinated updates.

## Rollout Plan

1. Land backend transport decisions per ADR-0022.
2. Extend wrapper coverage source-of-truth declarations in the backend crates.
3. Extend lane schema/rules to carry transport provenance for support claims.
4. Extend `xtask` generation/report/validation accordingly.
5. Regenerate parity artifacts and confirm automation-covered surfaces are no longer reported as
   missing.

## Validation

- `xtask` output must remain deterministic.
- Generated wrapper coverage JSON must remain derived from Rust source, not hand-edited.
- Validation must fail if an automation-backed support claim is malformed or missing required
  provenance metadata once the schema is updated.
- Coverage reports for both built-in lanes must clearly separate:
  - uncovered gaps
  - intentionally unsupported surfaces
  - supported surfaces covered via automation fallback
