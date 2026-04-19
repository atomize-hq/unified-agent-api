# Changelog

All notable changes to the repository-level Unified Agent API release are documented in this file.

This changelog tracks the root `VERSION` file and uses bare semantic versions (`MAJOR.MINOR.PATCH`).

## [Unreleased]

## [0.2.3] - 2026-04-18

### Added

- Added the first OpenCode integration surfaces across the workspace: `crates/opencode/` for canonical `opencode run --format json` flows, `cli_manifests/opencode/` for committed manifest-root evidence, and the `crates/agent_api` OpenCode backend with resume, fork, model, and redaction coverage.
- Added `cargo run -p xtask -- support-matrix` plus committed support-matrix publication artifacts so maintainers can see manifest support, backend support, unified support, and passthrough visibility without inferring it from scattered reports.
- Added dedicated regression coverage for OpenCode wrapper parsing/streaming behavior, backend mapping and fail-closed boundaries, support-matrix derivation/publication, and agent-api backend type leak detection.

### Changed

- Aligned the manifest docs, validator specs, runbooks, execution packs, and UAA spec text around one target-first support contract that keeps manifest support, backend support, unified support, and passthrough visibility separate while keeping OpenCode promotion out of scope by default.
- Extracted wrapper-coverage normalization and root-intake logic into shared `xtask` modules so Codex, Claude Code, and OpenCode intake and publication checks use the same layout and drift rules.
- Hardened CI and release wiring so `make preflight`, the artifact validation jobs, and the publish workflow all understand the new OpenCode root and crate alongside the existing Codex and Claude Code release surfaces.

## [0.2.2] - 2026-04-15

## [0.2.1] - 2026-04-14

### Added

- Added `cargo run -p xtask -- version-bump <new-version>` to update the root `VERSION`, workspace package version, and exact inter-crate publish pins in one pass.
- Added `xtask` integration coverage for the new version bump flow so invalid semver input and release-surface rewrites are tested directly.

### Changed

- Aligned `xtask` itself with the workspace version source of truth to avoid a separate tool-only version drift surface.
- Synced the publishable workspace crates and exact sibling dependency pins to the `0.2.1` release line.
- Hardened publish readiness bootstrap handling so dependent crates do not fail local readiness checks before newly bumped leaf crate versions are visible on crates.io.
- Restricted manual crates.io publish workflow dispatches to `main` so maintainers cannot publish unmerged branch content by mistake.

## [0.2.0] - 2026-04-14

### Added

- Established the repository-level Unified Agent API surface in `crates/agent_api/`, spanning Codex and Claude Code backends behind one shared contract.
- Added core orchestration features now present on `staging`, including session resume and fork flows, explicit cancellation, MCP management, `add_dirs`, and backend model selection.
- Added contract-first documentation under `docs/specs/unified-agent-api/` plus ADR coverage for the backend harness, session extensions, external sandbox policy, model selection, and terminal automation surfaces.

### Changed

- Aligned the repo identity and entrypoint docs around the Unified Agent API naming instead of the older wrapper-only framing.
- Hardened parity and regression coverage for backend capability publication, runtime rejection handling, and CLI drift detection across the workspace.

### Notes

- This is the first root-level changelog entry. It captures the current `staging` baseline at the point the repository-level `VERSION` and `CHANGELOG.md` files were introduced.
