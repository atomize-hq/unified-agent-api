# Changelog

All notable changes to the repository-level Unified Agent API release are documented in this file.

This changelog tracks the root `VERSION` file and uses bare semantic versions (`MAJOR.MINOR.PATCH`).

## [Unreleased]

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
