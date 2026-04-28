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
- The canonical comparison packet is `docs/agents/selection/cli-agent-selection-packet.md`.
- The create-lane approval artifact is `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml`.

## Decision Rules

- The research phase must finish before the runner is invoked.
- The runner compares exactly 3 eligible candidates.
- Candidates already present in `crates/xtask/data/agent_registry.toml` are rejected as ineligible during `generate` and are never scored or shortlisted.
- The scratch run must contain one dossier per seeded candidate, plus the frozen `research-metadata.json` envelope and `seed.snapshot.toml`.
- The canonical packet follows the no-drift template rule in `docs/templates/agent-selection/cli-agent-selection-packet-template.md`.
- Promotion uses Model B semantics:
  - `comparison.generated.md` remains a committed review byte-copy and the canonical packet stays byte-identical to it.
  - `approved-agent.toml` is rerendered at promote time from the maintainer-approved inputs.
- Replacement of a stale committed review run is never in-place:
  - generate a fresh `run_id`
  - promote that fresh run successfully
  - verify the promoted outputs
  - only then delete the stale committed review directory in the same commit
- If fewer than 3 eligible candidates survive generation, stop before promotion and expand the seed set.
