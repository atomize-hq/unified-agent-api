---
name: recommend-next-agent
description: Run the pre-create recommendation lane for the next CLI agent, producing frozen scratch research artifacts, post-research runner outputs, a promoted canonical packet, and an approval artifact for maintainer approve-or-override.
---

# Recommend Next Agent

Use this skill when a maintainer wants to choose the next CLI agent before the existing `xtask onboard-agent --approval ...` create lane begins.

Normative contract:
- `docs/specs/cli-agent-recommendation-dossier-contract.md`

## Workflow

1. Prepare or review `docs/agents/selection/candidate-seed.toml`.
2. Freeze a scratch research run before invoking the runner. The research phase owns these artifacts under `~/.gstack/projects/<repo-slug>/recommend-next-agent-research/<run_id>/`:
   - `seed.snapshot.toml`
   - `research-summary.md`
   - `research-metadata.json`
   - `dossiers/<agent_id>.json` for every seeded candidate
3. Only after the research artifacts exist, generate the post-research scratch runner outputs:

```sh
python3 scripts/recommend_next_agent.py generate \
  --seed-file docs/agents/selection/candidate-seed.toml \
  --research-dir ~/.gstack/projects/<repo-slug>/recommend-next-agent-research/<run_id> \
  --run-id <timestamp>-<shortlist_slug> \
  --scratch-root ~/.gstack/projects/<repo-slug>/recommend-next-agent-runs
```

4. Review the scratch artifacts under `~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id>/`.
   - The runner is post-research only and must not replace the frozen research artifacts.
   - `comparison.generated.md` and `approval-draft.generated.toml` are preview artifacts only.
5. Promote one reviewed fresh run into repo-owned review artifacts, the canonical packet, and a create-lane approval artifact:

```sh
python3 scripts/recommend_next_agent.py promote \
  --run-dir ~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id> \
  --repo-run-root docs/agents/selection/runs \
  --approved-agent-id <agent_id> \
  --onboarding-pack-prefix <kebab-case-pack-prefix> \
  [--override-reason "<required when approved agent differs from recommended>"]
```

6. Stop for maintainer approve-or-override.
7. After approval, continue with the existing factory lane:

```sh
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --dry-run
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --write
```

## Artifact Roots

- Scratch runs live only under `~/.gstack/projects/<repo-slug>/recommend-next-agent-runs/<run_id>/` and are never committed.
- Promoted review evidence lives under `docs/agents/selection/runs/<run_id>/`.
- The canonical comparison packet is `docs/agents/selection/cli-agent-selection-packet.md`; it is the maintainer decision surface for approve-or-override.
- The create-lane approval artifact is `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml`; it remains the normative approval artifact consumed by `xtask onboard-agent`.

## Decision Rules

- The research phase must finish before the runner is invoked.
- The runner requires at least 3 eligible candidates and then documents exactly 3 shortlisted candidates.
- Candidates already present in `crates/xtask/data/agent_registry.toml` are rejected as ineligible during `generate` and are never scored or shortlisted.
- The scratch run must contain one dossier per seeded candidate, plus the frozen `research-metadata.json` envelope and `seed.snapshot.toml`.
- The canonical packet follows the no-drift template rule in `docs/templates/agent-selection/cli-agent-selection-packet-template.md`.
- The packet heading/title/section order stays stable; do not rename packet headings.
- Section 5 decision lines must appear exactly, in order, as:
  - `Approve recommended agent`
  - `Override to shortlisted alternative`
  - `Stop and expand research`
- Section 6 must preserve the exact split:
  - `reproducible now`
  - `blocked until later`
- Sections 7-9 are semantically required implementation-handoff sections, not heading-only placeholders.
- Sections 7-9 exact subsection labels must match the plan wording verbatim, including capitalization:
  - Section 7: `Manifest root expectations`; `Wrapper crate expectations`; `agent_api` backend expectations; `UAA promotion expectations`; `Support/publication expectations`; `Likely seam risks`
  - Section 8: `Manifest-root artifacts`; `Wrapper-crate artifacts`; `agent_api` artifacts; `UAA promotion-gate artifacts`; `Docs/spec artifacts`; `Evidence/fixture artifacts`
  - Section 9: `Required workstreams`; `Required deliverables`; `Blocking risks`; `Acceptance gates`
- The dossier `probe_requests` schema stays unchanged; do not add a single-required-probe rule or any minimum required-probe count.
- Research authors should prefer `verified` for `non_interactive_execution` and `observable_cli_surface`.
- Research authors may use `inferred` only for repo-fit claims that the contract explicitly allows to pass that way.
- When public install evidence alone is insufficient for `non_interactive_execution` or `observable_cli_surface`, research should request allowed `help` or `version` probes instead of treating summary prose as gate proof.
- Provider-backed or account-gated blockers belong in `blocked_steps`, not as passable hard-gate support.
- Promotion uses Model B semantics:
  - `comparison.generated.md` remains a committed review byte-copy and the canonical packet stays byte-identical to it.
  - `approved-agent.toml` is rerendered at promote time from the maintainer-approved inputs.
- Replacement of a stale committed review run is never in-place:
  - generate a fresh `run_id`
  - promote that fresh run successfully
  - verify the promoted outputs
  - only then delete the stale committed review directory in the same commit
- If fewer than 3 eligible candidates survive generation, stop before promotion and expand the seed set.
