<!-- generated-by: scripts/recommend_next_agent.py generate -->
# Packet — CLI Agent Selection Recommendation

Status: Generated
Date (UTC): 2026-04-28T00:00:11Z
Run id: `20260428T000011Z-cli-recommendation`
Related source docs:
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`

## 1. Candidate Summary

Provenance: `dated external snapshot evidence + maintainer inference encoded by the deterministic runner`

Shortlisted candidates:
- `aider`
- `opencode`
- `gemini_cli`

Why these 3:
- they are the highest-ranked eligible candidates under the frozen shortlist algorithm

Recommendation in one sentence:
- `aider` (`aider`) ranks first under the deterministic shortlist contract.

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
| `aider` | 3 | 3 | 3 | 1 | 1 | 0 | stars=44022, installs=2, docs=1, auth_hits=5, architecture_hits=5, leverage_hits=1 |
| `opencode` | 3 | 3 | 3 | 1 | 1 | 0 | stars=150652, installs=3, docs=2, auth_hits=4, architecture_hits=5, leverage_hits=1 |
| `gemini_cli` | 3 | 3 | 1 | 1 | 1 | 1 | stars=102566, installs=2, docs=0, auth_hits=4, architecture_hits=5, leverage_hits=3 |

## 5. Recommendation

Provenance: `maintainer inference grounded in the comparison table`

Recommended winner: `aider`

`aider` ranks first after deterministic tie-break ordering.

## 6. Recommended Agent Evaluation Recipe

Provenance: `dated external snapshot evidence + seed inputs`

Recommended agent: `aider`

Install paths:
- `python -m pip install aider-install`
- `python -m pip install aider-chat`

Auth / access notes:
- Requires model/provider credentials or service access for realistic evaluation. Local install and help surfaces are available before paid provider validation.

## 7. Repo-Fit Analysis

Provenance: `committed repo evidence + deterministic descriptor derivation`

- crate path: `crates/aider`
- backend module: `crates/agent_api/src/backends/aider`
- manifest root: `cli_manifests/aider`
- package name: `unified-agent-api-aider`

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

### `aider`

- display name: `aider`
- `github_repo` `https://github.com/Aider-AI/aider` fetched `2026-04-28T00:00:11Z`
- `generic_page` `https://aider.chat/` fetched `2026-04-28T00:00:12Z`
- `pypi_package` `https://pypi.org/project/aider-chat/` fetched `2026-04-28T00:00:12Z`

### `opencode`

- display name: `OpenCode`
- `github_repo` `https://github.com/sst/opencode` fetched `2026-04-28T00:00:13Z`
- `generic_page` `https://opencode.ai/docs/` fetched `2026-04-28T00:00:13Z`
- `generic_page` `https://opencode.ai/docs/cli/` fetched `2026-04-28T00:00:14Z`
- `npm_package` `https://www.npmjs.com/package/opencode-ai` fetched `2026-04-28T00:00:14Z`

### `gemini_cli`

- display name: `Gemini CLI`
- `github_repo` `https://github.com/google-gemini/gemini-cli` fetched `2026-04-28T00:00:12Z`
- `npm_package` `https://www.npmjs.com/package/@google/gemini-cli` fetched `2026-04-28T00:00:12Z`
- `github_repo` `https://github.com/google-github-actions/run-gemini-cli` fetched `2026-04-28T00:00:13Z`

## 11. Acceptance Checklist

Provenance: `deterministic runner output`

- [x] The packet compares exactly 3 candidates.
- [x] The packet names one deterministic recommendation.
- [x] The appendix preserves dated source provenance for each shortlisted candidate.
