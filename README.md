# Unified Agent API (Rust)

This repository provides a unified Rust API over coding-agent CLIs, including Codex, Claude Code, Gemini CLI, and OpenCode.

## Start here

- Operator procedure hub: `docs/cli-agent-onboarding-factory-operator-guide.md`
- Contributing entrypoint: `CONTRIBUTING.md`
- Documentation index: `docs/README.md`
- Normative contract index: `docs/specs/unified-agent-api/README.md`

Use the operator guide for the shipped create-mode onboarding flow, maintenance-mode refresh flow, artifact ownership boundaries, and command sequencing. This README stays as the repo-entry summary rather than a second procedure manual.

## Repo summary

- `crates/agent_api/` - unified API surface and backend harness
- `crates/codex/` - Codex backend crate
- `crates/claude_code/` - Claude Code backend crate
- `crates/gemini_cli/` - Gemini CLI backend crate
- `crates/opencode/` - OpenCode backend crate
- `crates/wrapper_events/` - shared event and adapter utilities
- `crates/xtask/` - repo automation and validation commands
- `cli_manifests/` - committed parity artifacts and publication evidence
- `docs/` - normative specs, ADRs, and operator-facing documentation

## Green gate

The repo green gate is:

```sh
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

The operator guide is the procedural source of truth for when to run that gate in create mode and maintenance mode.

## License

This repository is dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT` and `LICENSE-APACHE`.
