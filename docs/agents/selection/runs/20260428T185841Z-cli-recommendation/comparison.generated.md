<!-- generated-by: scripts/recommend_next_agent.py generate -->
# Packet - CLI Agent Selection Packet

Status: Generated
Date (UTC): 2026-04-28T19:05:58Z
Owner(s): wrappers team / deterministic runner
Related source docs:
- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/**` for any normative contract this packet cites

Run id: `20260428T185841Z-cli-recommendation`

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
| `goose` | 3 | 3 | 3 | 2 | 2 | 2 | refs=goose-doc,goose-repo,goose-brew |

## 5. Recommendation

Provenance: `maintainer inference grounded in the comparison table`

Recommended winner: `openhands`

`OpenHands` wins because it satisfies the strict hard-gate rules with the strongest immediate repo-fit evidence (`refs=openhands-doc,openhands-pkg`), while preserving the best frozen shortlist position with primary score `11` and secondary score `5`.

- `aider` ties or trails `openhands` on the scorecard and still carries a blocked follow-up step: Provider-backed coding sessions still need maintainer credentials after local CLI install.
- `goose` ties or trails `openhands` on the scorecard and still carries a blocked follow-up step: Model-backed evaluation still needs maintainer credentials after local CLI setup.

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
  - `python -m pip install openhands-ai`
  - `uv tool install openhands-ai`
  - `openhands --help`
  - `openhands --version`
- evidence gatherable without paid or elevated access:
  - `openhands-doc` (`official_doc`): Introduction - OpenHands Docs
  - `openhands-repo` (`github`): OpenHands/OpenHands
  - `openhands-pkg` (`package_registry`): openhands-ai PyPI package
- expected artifacts to save during evaluation:
  - redacted install log
  - redacted `--help` output capture
  - redacted `--version` output capture
  - notes linking saved artifacts back to `openhands` dossier evidence ids

blocked until later:
- Full provider-backed automation still needs maintainer credentials after local CLI setup.

## 7. Repo-Fit Analysis

Provenance: `committed repo evidence + maintainer inference`

Manifest root expectations:
- keep generated manifests and review outputs aligned under `cli_manifests/openhands` before backend integration work starts
- preserve the canonical comparison packet and approval artifact references while manifest-root surfaces are wired up

Wrapper crate expectations:
- start with the wrapper crate at `crates/openhands` as the first implementation stage
- keep CLI parsing, command execution, and event normalization inside the wrapper seam until behavior is proven

`agent_api` backend expectations:
- add backend adapter work under `crates/agent_api/src/backends/openhands` only after wrapper behavior is reviewable
- map wrapper outputs into existing phase-1 seams without widening the current contracts prematurely

UAA promotion expectations:
- treat UAA promotion review as the final stage after wrapper and backend evidence exists
- do not treat support or capability matrix publication as a substitute for wrapper-first proof

Support/publication expectations:
- preserve `docs_release_track = "crates-io"` and the approved descriptor flags as the publication baseline
- land support-matrix or capability-matrix updates only when the implementation artifacts justify them

Likely seam risks:
- CLI surface drift can invalidate parser assumptions between saved dossier evidence and real execution
- provider-gated or hosted workflows may remain untestable until maintainer access exists, so keep them outside the wrapper-first acceptance path

## 8. Required Artifacts

Provenance: `committed repo evidence + maintainer inference`

Manifest-root artifacts:
- committed manifest snapshots and review artifacts under `cli_manifests/openhands`
- validation output proving manifest-root paths and packet references stay aligned

Wrapper-crate artifacts:
- wrapper crate code and tests under `crates/openhands`
- fixture-backed help/version captures or parser coverage notes for the approved CLI surface

`agent_api` artifacts:
- backend adapter code under `crates/agent_api/src/backends/openhands`
- integration tests or fixtures proving wrapper outputs map cleanly into `agent_api`

UAA promotion-gate artifacts:
- dry-run approval validation via `cargo run -p xtask -- onboard-agent --approval ... --dry-run`
- promotion review evidence showing wrapper and backend outputs satisfy the approved packet

Docs/spec artifacts:
- canonical packet, approval artifact, and any required `docs/specs/**` updates for real behavior changes
- repo guidance updates that point future maintainers at the approved onboarding seam

Evidence/fixture artifacts:
- saved dossier evidence ids and probe output refs linked through `sources.lock.json`
- redacted local evaluation captures, fixtures, and blocker notes required to reproduce acceptance decisions

## 9. Workstreams, Deliverables, Risks, And Gates

Provenance: `maintainer inference grounded in repo constraints`

Required workstreams:
- packet closeout and approval artifact review
- wrapper crate implementation
- `agent_api` backend integration
- UAA promotion review and matrix/publication closeout

Required deliverables:
- approved comparison packet and governance artifact
- wrapper crate code, tests, and manifest outputs
- backend adapter code, tests, and updated repo evidence

Blocking risks:
- provider or account-gated flows may block parity claims after local CLI install succeeds
- release drift between saved dossier evidence and current binaries can invalidate planned parsing assumptions

Acceptance gates:
- packet and approval artifacts remain byte-stable except for allowed promote-time deltas
- wrapper crate proves the approved help/version/non-interactive surfaces with saved evidence
- backend adapter integration does not contradict existing `docs/specs/**` contracts
- at least 3 eligible candidates remain after hard gating and exactly 3 shortlisted candidates are documented

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
- Loser rationale: `aider` ties or trails `openhands` on the scorecard and still carries a blocked follow-up step: Provider-backed coding sessions still need maintainer credentials after local CLI install.

### `goose`

- Snapshot date: `2026-04-28`
- Official links:
  - `https://goose-docs.ai/docs/category/getting-started`
  - `https://github.com/aaif-goose/goose`
- Install / distribution:
  - `brew install block-goose-cli`
  - `curl -fsSL https://github.com/block/goose/releases/download/stable/download_cli.sh | bash`
- Adoption / community:
  - refs `refs=goose-doc,goose-repo,goose-brew`
- Release activity:
  - `goose-doc` `official_doc` captured `2026-04-28T15:31:24Z`
  - `goose-repo` `github` captured `2026-04-28T15:31:24Z`
  - `goose-home` `ancillary` captured `2026-04-28T15:31:24Z`
  - `goose-brew` `package_registry` captured `2026-04-28T19:06:30Z`
- Access prerequisites:
  - Provider credentials are required for meaningful model-backed agent runs.
- Normalized notes:
  - Goose is documentable and installable now, but provider-backed behavior belongs to later proving.
- Loser rationale: `goose` ties or trails `openhands` on the scorecard and still carries a blocked follow-up step: Model-backed evaluation still needs maintainer credentials after local CLI setup.

### Strategic Contenders

- `opencode`: agent_id `opencode` already exists in crates/xtask/data/agent_registry.toml and is already onboarded
- `gemini_cli`: hard_gate.non_interactive_execution.verified_doc_and_package_or_probe: missing required evidence kinds: official_doc

