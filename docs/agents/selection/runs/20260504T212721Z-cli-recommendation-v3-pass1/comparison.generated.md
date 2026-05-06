<!-- generated-by: scripts/recommend_next_agent.py generate -->
# Packet - CLI Agent Selection Packet

Status: Generated
Date (UTC): 2026-05-04T21:44:04Z
Owner(s): wrappers team / deterministic runner
Related source docs:
- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/**` for any normative contract this packet cites

Run id: `20260504T212721Z-cli-recommendation-v3-pass1`

## 1. Candidate Summary

Provenance: `<committed repo evidence | dated external snapshot evidence | maintainer inference>`

Shortlisted candidates:
- `goose`
- `openhands`
- `gptme`

Why these 3:
- they are the highest-ranked eligible candidates under the frozen shortlist algorithm

Recommendation in one sentence:
- `Goose` (`goose`) ranks first under the deterministic shortlist contract.

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
| `goose` | 3 | 3 | 3 | 3 | 2 | 2 | refs=goose-install,goose-github,goose-pypi,goose-running-tasks |
| `openhands` | 3 | 3 | 3 | 3 | 2 | 2 | refs=openhands-install,openhands-github,openhands-pypi,openhands-headless |
| `gptme` | 3 | 3 | 3 | 2 | 2 | 2 | refs=gptme-getting-started,gptme-github,gptme-pypi |

## 5. Recommendation

Provenance: `maintainer inference grounded in the comparison table`

Recommended winner: `goose`

`Goose` wins because it satisfies the strict hard-gate rules with the strongest immediate repo-fit evidence (`refs=goose-install,goose-github,goose-pypi,goose-running-tasks`), while preserving the best frozen shortlist position with primary score `12` and secondary score `4`.

- `openhands` ties or trails `goose` on the scorecard and still carries a blocked follow-up step: Confirm the live persisted config filename under `~/.openhands` before encoding wrapper-side file assumptions, because the install page and command reference differ.
- `gptme` loses because `goose` has stronger evidence-backed coverage on `Reproducibility & access friction`.

Approve recommended agent
Override to shortlisted alternative
Stop and expand research

## 6. Recommended Agent Evaluation Recipe

Provenance: `dated external snapshot evidence + maintainer inference`

reproducible now:
- install paths:
  - `curl -fsSL https://github.com/aaif-goose/goose/releases/download/stable/download_cli.sh | bash`
  - `brew install block-goose-cli`
  - `pipx install goose-ai`
- auth / account / billing prerequisites:
  - Configure an LLM provider on first use with `goose configure` or the first-run wizard.
  - If keyring access is unavailable, provider credentials can be supplied through environment variables or config-backed secrets storage.
  - For local/offline operation, start a supported local provider such as Ollama or LM Studio before selecting it in `goose configure`.
- runnable commands:
  - `curl -fsSL https://github.com/aaif-goose/goose/releases/download/stable/download_cli.sh | bash`
  - `brew install block-goose-cli`
  - `pipx install goose-ai`
  - `goose --help`
  - `goose --version`
- evidence gatherable without paid or elevated access:
  - `goose-install` (`official_doc`): Install goose | goose
  - `goose-running-tasks` (`official_doc`): Running Tasks | goose
  - `goose-providers` (`official_doc`): Configure LLM Provider | goose
  - `goose-github` (`github`): GitHub - aaif-goose/goose
  - `goose-pypi` (`package_registry`): goose-ai · PyPI
- expected artifacts to save during evaluation:
  - redacted install log
  - redacted `--help` output capture
  - redacted `--version` output capture
  - notes linking saved artifacts back to `goose` dossier evidence ids

blocked until later:
- any hosted or provider-only workflow remains blocked until a maintainer validates live account, auth, and billing requirements outside the local install path
- any capability that cannot be exercised from the local CLI surface remains blocked until wrapper-first evaluation artifacts are committed

## 7. Repo-Fit Analysis

Provenance: `committed repo evidence + maintainer inference`

Manifest root expectations:
- keep generated manifests and review outputs aligned under `cli_manifests/goose` before backend integration work starts
- preserve the canonical comparison packet and approval artifact references while manifest-root surfaces are wired up

Wrapper crate expectations:
- start with the wrapper crate at `crates/goose` as the first implementation stage
- keep CLI parsing, command execution, and event normalization inside the wrapper seam until behavior is proven

`agent_api` backend expectations:
- add backend adapter work under `crates/agent_api/src/backends/goose` only after wrapper behavior is reviewable
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
- committed manifest snapshots and review artifacts under `cli_manifests/goose`
- validation output proving manifest-root paths and packet references stay aligned

Wrapper-crate artifacts:
- wrapper crate code and tests under `crates/goose`
- fixture-backed help/version captures or parser coverage notes for the approved CLI surface

`agent_api` artifacts:
- backend adapter code under `crates/agent_api/src/backends/goose`
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

### `goose`

- Snapshot date: `2026-05-04`
- Official links:
  - `https://goose-docs.ai/docs/getting-started/installation/`
  - `https://goose-docs.ai/docs/guides/running-tasks/`
  - `https://goose-docs.ai/docs/getting-started/providers/`
  - `https://github.com/aaif-goose/goose`
  - `https://pypi.org/project/goose-ai/`
- Install / distribution:
  - `curl -fsSL https://github.com/aaif-goose/goose/releases/download/stable/download_cli.sh | bash`
  - `brew install block-goose-cli`
  - `pipx install goose-ai`
- Adoption / community:
  - refs `refs=goose-install,goose-github,goose-pypi,goose-running-tasks`
- Release activity:
  - `goose-install` `official_doc` captured `2026-05-04T21:37:24Z`
  - `goose-running-tasks` `official_doc` captured `2026-05-04T21:37:24Z`
  - `goose-providers` `official_doc` captured `2026-05-04T21:37:24Z`
  - `goose-github` `github` captured `2026-05-04T21:37:24Z`
  - `goose-pypi` `package_registry` captured `2026-05-04T21:37:24Z`
- Access prerequisites:
  - Configure an LLM provider on first use with `goose configure` or the first-run wizard.
  - If keyring access is unavailable, provider credentials can be supplied through environment variables or config-backed secrets storage.
  - For local/offline operation, start a supported local provider such as Ollama or LM Studio before selecting it in `goose configure`.
- Normalized notes:
  - The `goose-ai` PyPI package is archived at version `0.9.11` while the active CLI release train on GitHub/docs is much newer; prefer the documented shell installer or Homebrew for onboarding.
  - Provider configuration is still required before useful execution unless install/config is pre-seeded.
  - Local-model operation exists, but some models lack tool-calling and require reduced extension usage.
- Loser rationale: winner

### `openhands`

- Snapshot date: `2026-05-04`
- Official links:
  - `https://openhands.dev/product/cli`
  - `https://docs.openhands.dev/openhands/usage/cli/installation`
  - `https://docs.openhands.dev/openhands/usage/cli/headless`
  - `https://docs.openhands.dev/openhands/usage/cli/command-reference`
  - `https://github.com/OpenHands/OpenHands-CLI`
  - `https://pypi.org/project/openhands/`
- Install / distribution:
  - `uv tool install openhands --python 3.12`
  - `curl -fsSL https://install.openhands.dev/install.sh | sh`
- Adoption / community:
  - refs `refs=openhands-install,openhands-github,openhands-pypi,openhands-headless`
- Release activity:
  - `openhands-product` `ancillary` captured `2026-05-04T21:37:24Z`
  - `openhands-install` `official_doc` captured `2026-05-04T21:37:24Z`
  - `openhands-headless` `official_doc` captured `2026-05-04T21:37:24Z`
  - `openhands-command` `official_doc` captured `2026-05-04T21:37:24Z`
  - `openhands-github` `github` captured `2026-05-04T21:37:24Z`
  - `openhands-pypi` `package_registry` captured `2026-05-04T21:37:24Z`
- Access prerequisites:
  - Configure LLM settings on first run or pre-seed the `~/.openhands` config directory before starting the CLI.
  - If using environment variables, export `LLM_API_KEY` and optionally `LLM_MODEL` or `LLM_BASE_URL`, then pass `--override-with-envs`.
  - Windows users must run the CLI inside WSL (Ubuntu).
- Normalized notes:
  - Headless mode always runs in `always-approve` mode, which is operationally useful but raises the default risk profile.
  - Native Windows is not supported; the official docs require WSL.
  - The retrieved official docs disagree on the exact persisted config filename inside `~/.openhands`.
  - The recommended package install path requires Python `3.12`.
- Loser rationale: `openhands` ties or trails `goose` on the scorecard and still carries a blocked follow-up step: Confirm the live persisted config filename under `~/.openhands` before encoding wrapper-side file assumptions, because the install page and command reference differ.

### `gptme`

- Snapshot date: `2026-05-04`
- Official links:
  - `https://gptme.org/`
  - `https://gptme.org/docs/getting-started.html`
  - `https://gptme.org/docs/config.html`
  - `https://github.com/gptme/gptme`
  - `https://pypi.org/project/gptme/`
- Install / distribution:
  - `pipx install gptme`
  - `uv tool install gptme`
- Adoption / community:
  - refs `refs=gptme-getting-started,gptme-github,gptme-pypi`
- Release activity:
  - `gptme-home` `ancillary` captured `2026-05-04T21:37:24Z`
  - `gptme-getting-started` `official_doc` captured `2026-05-04T21:37:24Z`
  - `gptme-config` `official_doc` captured `2026-05-04T21:37:24Z`
  - `gptme-github` `github` captured `2026-05-04T21:37:24Z`
  - `gptme-pypi` `package_registry` captured `2026-05-04T21:37:24Z`
- Access prerequisites:
  - Set a supported provider API key in the environment or the gptme config directory, or let the first interactive run prompt and save it.
  - For no-API-key operation, run a local model backend such as Ollama and point gptme at it with `OPENAI_BASE_URL` plus a local model id.
  - Windows users need WSL or Docker rather than a direct native install.
- Normalized notes:
  - Native Windows is not directly supported; official guidance points users to WSL or Docker.
  - Some optional tools need extra system dependencies such as Playwright, tmux, or GitHub CLI.
  - Interactive first run will prompt for provider credentials unless a supported local model backend is preconfigured.
- Loser rationale: `gptme` loses because `goose` has stronger evidence-backed coverage on `Reproducibility & access friction`.

