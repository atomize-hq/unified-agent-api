# ADR 0006: Unified Agent API Workspace (Multi-Agent CLI Support + Parity Lanes)

Date: 2026-02-12  
Status: Accepted

## Context

This repository started as a single-purpose Rust integration around one upstream CLI agent (Codex).
We want to expand the repo into a stable home for *multiple* CLI agent backends while preserving:

- A consistent, testable wrapper API shape per agent
- Deterministic “snapshot → union → report → validate” parity artifacts
- Highly-automated maintenance for upstream releases (goal: ~90–95% automated)
- A mechanical “trigger new work” mechanism when coverage deltas appear (triad scaffolds)

We also want repo-level naming to reflect the new scope: “Unified Agent API”.

## Decision

### 1) Treat the repository as the **Unified Agent API** workspace

The repo is the canonical workspace for multiple agent backends. The repo-level identity is
“Unified Agent API” / `unified-agent-api` (docs and repo identity), even if internal crate and
workflow identifiers continue evolving over time.

### 2) Use a **crate-per-agent** layout + shared automation

Workspace crates:

- `crates/codex` — Codex CLI wrapper crate (existing)
- `crates/claude_code` — Claude Code CLI wrapper crate (new)
- `crates/xtask` — deterministic tooling shared across all parity lanes (new work continues here)

Published crates share a single workspace version. `xtask` remains non-published.

### 3) Create a **parity lane root per agent** under `cli_manifests/<agent>/`

Each agent backend has a dedicated parity root directory that contains:

- Schemas and merge/comparison rules (`SCHEMA.json`, `RULES.json`, `VERSION_METADATA_SCHEMA.json`)
- Pins (`artifacts.lock.json`)
- Pointers (`min_supported.txt`, `latest_validated.txt`, plus per-target pointer files)
- Generated artifacts (snapshots, union snapshots, reports, version metadata)
- Runbooks (ops playbook, CI plan, PR body template, agent runbook)

For Claude Code this root is:
- `cli_manifests/claude_code/`

### 4) Mirror the Codex maintenance framework for Claude Code (and future agents)

For each agent, we maintain a consistent automation loop:

1. **Release watch** detects a new upstream version (per lane policy).
2. **Update snapshot** workflow:
   - downloads/verifies binaries (integrity pinned + locked)
   - generates per-target snapshots
   - generates union snapshot on the required target
   - generates wrapper coverage JSON from the crate source-of-truth
   - generates reports + version metadata
   - validates lane invariants
   - generates a triad scaffold pack from the coverage delta
   - opens/updates a PR branch `automation/<agent>-<version>`
3. **Promote** workflow (manual gate) updates pointers and `current.json` for the lane.

### 5) Establish a single source of truth for backend coverage per agent crate

Each backend crate owns a deterministic coverage declaration file (Rust source) that is the
authoritative input to generate `cli_manifests/<agent>/wrapper_coverage.json`.

For Claude Code:
- Source-of-truth: `crates/claude_code/src/wrapper_coverage_manifest.rs`
- Generated output: `cli_manifests/claude_code/wrapper_coverage.json`

## Consequences

### Positive

- Adding a new agent backend becomes repeatable: add `crates/<agent>`, add `cli_manifests/<agent>`,
  add per-agent workflows, and reuse `xtask` machinery.
- Parity artifacts remain deterministic and reviewable (diff-first) across all agents.
- Automation can mechanically generate new work packs (triads) whenever coverage deltas appear.

### Tradeoffs / costs

- `xtask` becomes a central dependency and must stay deterministic and stable across lanes.
- Each new agent lane adds CI surface area (workflows, pins, docs), which increases maintenance
  overhead but is offset by shared tooling and templates.

## Implementation Notes (normative)

- Agent backend crates must avoid runtime downloads/updates of upstream CLIs; acquisition is done in
  CI workflows and pinned via `cli_manifests/<agent>/artifacts.lock.json`.
- Lane rules define the required target and expected targets; union and validation are anchored on
  the required target.
- “Trigger new work” is implemented as deterministic triad scaffolding generated from coverage
  deltas (no LLM decisions in the generator).

## Out of Scope (for this ADR)

- Supporting interactive/tui surfaces for Claude Code wrapper APIs (v1 focuses on headless
  `--print`).
- Adding additional agents beyond Codex and Claude Code (the framework supports it; lanes can be
  added incrementally).
