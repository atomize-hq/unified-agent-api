<!-- generated-by: scripts/recommend_next_agent.py generate -->
# Packet — CLI Agent Selection Recommendation

Status: Generated
Date (UTC): 2026-04-28T03:35:33Z
Run id: `20260428T033528Z-cli-recommendation`
Related source docs:
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

## 1. Candidate Summary

Provenance: `dated external snapshot evidence + maintainer inference encoded by the deterministic runner`

Shortlisted candidates:
- `openhands`
- `aider`
- `goose`

Why these 3:
- they are the highest-ranked eligible candidates under the frozen shortlist algorithm

Recommendation in one sentence:
- `OpenHands` (`openhands`) ranks first under the deterministic shortlist contract.

## 2. What Already Exists

Provenance: `committed repo evidence`

- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `crates/xtask/src/approval_artifact.rs`

## 3. Selection Rubric

Provenance: `maintainer inference informed by dated external snapshot evidence`

This packet uses the frozen score buckets and the deterministic shortlist sort order. It does not publish a weighted total column.

## 4. Fixed 3-Candidate Comparison Table

Provenance: `dated external snapshot evidence + deterministic runner scoring`

| Candidate | Adoption & community pull | CLI product maturity & release activity | Installability & docs quality | Reproducibility & access friction | Architecture fit for this repo | Capability expansion / future leverage | Notes |
|---|---:|---:|---:|---:|---:|---:|---|
| `openhands` | 3 | 3 | 3 | 2 | 1 | 1 | stars=72210, installs=2, docs=1, auth_hits=3, architecture_hits=4, leverage_hits=2 |
| `aider` | 3 | 3 | 3 | 1 | 1 | 0 | stars=44029, installs=2, docs=1, auth_hits=5, architecture_hits=5, leverage_hits=1 |
| `goose` | 3 | 0 | 3 | 2 | 1 | 0 | stars=43414, installs=2, docs=2, auth_hits=3, architecture_hits=4, leverage_hits=1 |

## 5. Recommendation

Provenance: `maintainer inference grounded in the comparison table`

Recommended winner: `openhands`

`OpenHands` ranks first after deterministic tie-break ordering.

## 6. Recommended Agent Evaluation Recipe

Provenance: `dated external snapshot evidence + seed inputs`

Recommended agent: `OpenHands`

Install paths:
- `python -m pip install openhands-ai`
- `uv tool install openhands-ai`

Auth / access notes:
- Provider credentials may be required for full automation runs; install and local startup documentation are public.

## 7. Repo-Fit Analysis

Provenance: `committed repo evidence + deterministic descriptor derivation`

- crate path: `crates/openhands`
- backend module: `crates/agent_api/src/backends/openhands`
- manifest root: `cli_manifests/openhands`
- package name: `unified-agent-api-openhands`

## 8. Required Artifacts

Provenance: `committed repo evidence + maintainer inference`

- canonical comparison packet
- approval artifact draft
- committed review run artifacts
- wrapper/backend follow-on surfaces after approval

## 9. Workstreams, Deliverables, Risks, And Gates

Provenance: `maintainer inference grounded in repo constraints`

- workstreams: contract, runner, validation, proving, integration
- deliverables: seed file, skill, runner, tests, review run, approval draft
- risks: source drift, insufficient eligible candidates, approval validation failure
- gates: exactly 3 shortlisted candidates, successful approval dry-run, green validation

## 10. Dated Evidence Appendix

Provenance: `dated external snapshot evidence`

### `openhands`

- display name: `OpenHands`
- `github_repo` `https://github.com/All-Hands-AI/OpenHands` fetched `2026-04-28T03:35:37Z`
- `generic_page` `https://docs.all-hands.dev/` fetched `2026-04-28T03:35:38Z`
- `pypi_package` `https://pypi.org/project/openhands-ai/` fetched `2026-04-28T03:35:39Z`

### `aider`

- display name: `aider`
- `github_repo` `https://github.com/Aider-AI/aider` fetched `2026-04-28T03:35:33Z`
- `generic_page` `https://aider.chat/` fetched `2026-04-28T03:35:33Z`
- `pypi_package` `https://pypi.org/project/aider-chat/` fetched `2026-04-28T03:35:33Z`

### `goose`

- display name: `Goose`
- `github_repo` `https://github.com/block/goose` fetched `2026-04-28T03:35:35Z`
- `generic_page` `https://goose-docs.ai` fetched `2026-04-28T03:35:36Z`
- `generic_page` `https://goose-docs.ai/docs/` fetched `2026-04-28T03:35:36Z`

## 11. Acceptance Checklist

Provenance: `deterministic runner output`

- [x] The packet compares exactly 3 candidates.
- [x] The packet names one deterministic recommendation.
- [x] The appendix preserves dated source provenance for each shortlisted candidate.
