# Changelog

All notable changes to the repository-level Unified Agent API release are documented in this file.

This changelog tracks the root `VERSION` file and uses bare semantic versions (`MAJOR.MINOR.PATCH`).

## [Unreleased]

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
