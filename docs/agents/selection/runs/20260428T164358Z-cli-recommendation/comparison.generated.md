<!-- generated-by: scripts/recommend_next_agent.py generate -->
# Packet - CLI Agent Selection Packet

Status: Generated
Date (UTC): 2026-04-28T16:44:19Z
Owner(s): wrappers team / deterministic runner
Related source docs:
- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/**` for any normative contract this packet cites

Run id: `20260428T164358Z-cli-recommendation`

## 1. Candidate Summary

Provenance: `<committed repo evidence | dated external snapshot evidence | maintainer inference>`

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
- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/templates/agent-selection/cli-agent-selection-packet-template.md`
- `docs/cli-agent-onboarding-factory-operator-guide.md`
- `crates/xtask/src/approval_artifact.rs`

## 3. Selection Rubric

Provenance: `maintainer inference informed by dated external snapshot evidence`

This packet preserves the frozen score dimensions, the 0-3 scale, and the deterministic shortlist sort order. Product-value signals remain primary, while architecture fit and future leverage break ties only after the primary comparison is established.

## 4. Fixed 3-Candidate Comparison Table

Provenance: `dated external snapshot evidence + maintainer inference`

| Candidate | Adoption & community pull | CLI product maturity & release activity | Installability & docs quality | Reproducibility & access friction | Architecture fit for this repo | Capability expansion / future leverage | Notes |
|---|---:|---:|---:|---:|---:|---:|---|
| `openhands` | 3 | 3 | 3 | 2 | 2 | 3 | refs=openhands-doc,openhands-pkg |
| `aider` | 3 | 3 | 3 | 2 | 2 | 2 | refs=aider-doc,aider-pkg |
| `goose` | 2 | 2 | 3 | 2 | 2 | 2 | refs=goose-doc,goose-repo,goose-home |

## 5. Recommendation

Provenance: `maintainer inference grounded in the comparison table`

Recommended winner: `openhands`

`OpenHands` wins on the frozen shortlist ordering with primary score `11`, secondary score `5`, and cited evidence `refs=openhands-doc,openhands-pkg`.

- `aider` lost on the frozen tie-break chain despite refs `refs=aider-doc,aider-pkg`.
- `goose` lost on the frozen tie-break chain despite refs `refs=goose-doc,goose-repo,goose-home`.

Approve recommended agent
Override to shortlisted alternative
Stop and expand research

## 6. Recommended Agent Evaluation Recipe

Provenance: `dated external snapshot evidence + maintainer inference`

reproducible now:
- install paths:
  - `python -m pip install openhands-ai`
  - `uv tool install openhands-ai`
- auth / account / billing prerequisites:
  - Provider credentials may be required for full automation runs after local install.
- runnable commands:
  - install one supported package path above
  - collect local `--help` / `--version` output during maintainer evaluation when the binary is installed
- evidence gatherable without paid or elevated access:
  - official docs refs `openhands-doc, openhands-pkg`
  - package / install refs `openhands-doc, openhands-pkg`
- expected artifacts to save during evaluation:
  - redacted install logs
  - captured help/version output
  - fixture notes describing fake-binary or parser coverage assumptions

blocked until later:
- Full provider-backed automation still needs maintainer credentials after local CLI setup.

## 7. Repo-Fit Analysis

Provenance: `committed repo evidence + maintainer inference`

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
- deliverables: seed snapshot, dossiers, runner outputs, tests, review run, approval draft
- risks: drift, insufficient eligible candidates, approval validation failure
- gates: exactly 3 shortlisted candidates, successful approval dry-run, green validation

## 10. Dated Evidence Appendix

Provenance: `dated external snapshot evidence`

### `openhands`

- Snapshot date: `2026-04-28`
- Official links:
  - `https://docs.all-hands.dev/`
  - `https://pypi.org/project/openhands-ai/`
- Install / distribution:
  - `python -m pip install openhands-ai`
  - `uv tool install openhands-ai`
- Adoption / community:
  - refs `refs=openhands-doc,openhands-pkg`
- Release activity:
  - `openhands-doc` `official_doc` captured `2026-04-28T15:31:24Z`
  - `openhands-repo` `github` captured `2026-04-28T15:31:24Z`
  - `openhands-pkg` `package_registry` captured `2026-04-28T15:31:24Z`
- Access prerequisites:
  - Provider credentials may be required for full automation runs after local install.
- Normalized notes:
  - Local install evidence is public now; provider-backed automation should be proven in onboarding.
- Loser rationale: winner

### `aider`

- Snapshot date: `2026-04-28`
- Official links:
  - `https://aider.chat/`
  - `https://pypi.org/project/aider-chat/`
- Install / distribution:
  - `python -m pip install aider-install`
  - `python -m pip install aider-chat`
- Adoption / community:
  - refs `refs=aider-doc,aider-pkg`
- Release activity:
  - `aider-doc` `official_doc` captured `2026-04-28T15:31:24Z`
  - `aider-repo` `github` captured `2026-04-28T15:31:24Z`
  - `aider-pkg` `package_registry` captured `2026-04-28T15:31:24Z`
- Access prerequisites:
  - Model or provider credentials are required for realistic end-to-end coding sessions.
- Normalized notes:
  - Public docs cover local install now; provider-backed coding behavior should be proven in onboarding.
- Loser rationale: refs=aider-doc,aider-pkg

### `goose`

- Snapshot date: `2026-04-28`
- Official links:
  - `https://goose-docs.ai/docs/category/getting-started`
  - `https://github.com/aaif-goose/goose`
- Install / distribution:
  - `brew install block/goose/goose`
  - `curl -fsSL https://github.com/block/goose/releases/latest/download/download_cli.sh | bash`
- Adoption / community:
  - refs `refs=goose-doc,goose-repo,goose-home`
- Release activity:
  - `goose-doc` `official_doc` captured `2026-04-28T15:31:24Z`
  - `goose-repo` `github` captured `2026-04-28T15:31:24Z`
  - `goose-home` `ancillary` captured `2026-04-28T15:31:24Z`
- Access prerequisites:
  - Provider credentials are required for meaningful model-backed agent runs.
- Normalized notes:
  - Goose is documentable and installable now, but provider-backed behavior belongs to later proving.
- Loser rationale: refs=goose-doc,goose-repo,goose-home

### Strategic Contenders

- `opencode`: agent_id `opencode` already exists in crates/xtask/data/agent_registry.toml and is already onboarded
- `gemini_cli`: agent_id `gemini_cli` already exists in crates/xtask/data/agent_registry.toml and is already onboarded

