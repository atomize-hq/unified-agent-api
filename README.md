# Unified Agent API (Rust)

This repository provides a unified Rust API over coding-agent CLIs, starting with:
- OpenAI Codex CLI (`codex`)
- Anthropic Claude Code (`claude`)
- OpenCode CLI (`opencode`)

Codex backend highlights:
- Typed streaming of `codex exec --json` events (`ThreadEvent`)
- Offline parsing of saved JSONL logs into the same `ThreadEvent` model
- Capability probing and compatibility shims for upstream drift
- Tooling and artifacts for release-trailing parity maintenance (`cli_manifests/codex/`)

Claude Code backend highlights (v1):
- Non-interactive `--print` execution
- Tolerant parsing of `--output-format=stream-json` (NDJSON)
- Release-trailing parity lane (`cli_manifests/claude_code/`)

OpenCode backend highlights (v1):
- Canonical `opencode run --format json` execution only
- Incremental JSONL event streaming with bounded parse-failure redaction
- Session resume/fork and working-directory mapping on the supported runtime surface
- Deterministic committed manifest root and validation lane (`cli_manifests/opencode/`)

Published Cargo package names are repo-scoped:
- `unified-agent-api`
- `unified-agent-api-codex`
- `unified-agent-api-claude-code`
- `unified-agent-api-opencode`
- `unified-agent-api-wrapper-events`

Rust library import paths remain:
- `agent_api`
- `codex`
- `claude_code`
- `opencode`
- `wrapper_events`

## License

This repository is dual-licensed under MIT or Apache-2.0, at your option.
See `LICENSE-MIT` and `LICENSE-APACHE`.

## Start here

- Unified Agent API contracts: `docs/specs/unified-agent-api/README.md`
- Support publication contract: `docs/specs/unified-agent-api/support-matrix.md`
- Crates.io release guide: `docs/crates-io-release.md`
- Codex API docs: `crates/codex/README.md`
- Examples index: `crates/codex/EXAMPLES.md`
- Documentation index: `docs/README.md`
- Release metadata: `VERSION`, `CHANGELOG.md`
- Contributing: `CONTRIBUTING.md`

## Repo map

- `crates/agent_api/` — unified API surface and backend harness
- `crates/codex/` — Codex backend crate
- `crates/claude_code/` — Claude Code backend crate
- `crates/opencode/` — OpenCode backend crate
- `docs/` — ADRs, specs, integration notes, project management
- `cli_manifests/codex/` — Codex CLI parity artifacts + ops docs
- `cli_manifests/claude_code/` — Claude Code parity artifacts + ops docs
- `cli_manifests/opencode/` — OpenCode manifest-root artifacts + ops docs

## Operations / parity maintenance

- Ops playbook: `cli_manifests/codex/OPS_PLAYBOOK.md`
- CLI snapshot artifacts: `cli_manifests/codex/README.md`
- Support publication artifact: `cli_manifests/support_matrix/current.json`
- Support publication check: `cargo run -p xtask -- support-matrix --check`
- Decisions (ADRs): `docs/adr/`
- Normative contracts: `docs/specs/`
