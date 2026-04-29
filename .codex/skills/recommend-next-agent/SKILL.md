---
name: recommend-next-agent
description: Run the pre-create recommendation lane for the next CLI agent, producing frozen scratch research artifacts, post-research runner outputs, a promoted canonical packet, and an approval artifact for maintainer approve-or-override.
---

# Recommend Next Agent

Use this skill when a maintainer wants to choose the next CLI agent before the existing `xtask onboard-agent --approval ...` create lane begins.

Normative contract:
- `docs/specs/cli-agent-recommendation-dossier-contract.md`

## Workflow

1. Read optional discovery guidance from `docs/agents/selection/discovery-hints.json`.
2. Run discovery pass 1 and write exactly these scratch artifacts under `docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/`:
   - `candidate-seed.generated.toml`
   - `discovery-summary.md`
   - `sources.lock.json`
3. Use only the fixed pass 1 query family:
   - `best AI coding CLI`
   - `AI agent CLI tools`
   - `developer agent command line`
4. Apply discovery-hint precedence exactly:
   - hard discovery rejections win first
   - `exclude_candidates` beats `include_candidates`
   - valid `include_candidates` may bypass soft preferences only
   - hints do not affect evaluation scoring
5. Review or lightly edit `candidate-seed.generated.toml`, then freeze discovery into the research root:

```sh
python3 scripts/recommend_next_agent.py freeze-discovery \
  --discovery-dir docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id> \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>
```

6. Complete research against `docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/`. The research phase owns these artifacts:
   - `seed.snapshot.toml`
   - `discovery-input/candidate-seed.generated.toml`
   - `discovery-input/discovery-summary.md`
   - `discovery-input/sources.lock.json`
   - `research-summary.md`
   - `research-metadata.json`
   - `dossiers/<agent_id>.json` for every candidate in `seed.snapshot.toml`
7. Only after the research artifacts exist, generate the post-research scratch runner outputs:

```sh
python3 scripts/recommend_next_agent.py generate \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<run_id> \
  --run-id <run_id> \
  --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs
```

8. If `generate` returns `insufficient_eligible_candidates`, widen discovery exactly once:
   - use a fresh `run_id`
   - use only the fixed pass 2 query family:
     - `alternatives to <top surviving candidate>`
     - `top coding agent CLI open source`
     - `CLI coding assistant blog`
   - if pass 1 has zero surviving candidates after hard rejection, omit the candidate-relative query and use only the two generic widening queries
   - exclude every pass 1 candidate already seen, whether accepted or rejected
   - emit at most 3 new candidates
   - do not mutate or overwrite pass 1 artifacts
   - repeat freeze, research, and `generate` once
9. If pass 2 still returns `insufficient_eligible_candidates`, stop and report structured insufficiency. Do not perform ad hoc seed surgery.
10. Review the scratch artifacts under `docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id>/`.
   - The runner is post-research only and must not replace the frozen research artifacts.
   - `comparison.generated.md` and `approval-draft.generated.toml` are preview artifacts only.
   - review `discovery/**` in the run alongside the dossiers and shortlist outputs.
11. Promote one reviewed fresh run into repo-owned review artifacts, the canonical packet, and a create-lane approval artifact:

```sh
python3 scripts/recommend_next_agent.py promote \
  --run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id> \
  --repo-run-root docs/agents/selection/runs \
  --approved-agent-id <agent_id> \
  --onboarding-pack-prefix <kebab-case-pack-prefix> \
  [--override-reason "<required when approved agent differs from recommended>"]
```

12. Stop for maintainer approve-or-override.
13. After approval, continue with the existing factory lane:

```sh
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --dry-run
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --write
```

## Artifact Roots

- Scratch discovery lives only under `docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/` and is never committed.
- Distinct passes own distinct discovery directories. Widening never overwrites pass 1 artifacts.
- `docs/agents/selection/candidate-seed.toml` remains a fallback curated pool and example; it is not the v2 reviewed runtime input.
- Scratch runs live only under `docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id>/` and are never committed.
- `docs/agents/.uaa-temp/**` is operator-owned scratch space.
- `docs/agents/*/.staging/**` remains internal promote-time staging owned by the scripts.
- Promoted review evidence lives under `docs/agents/selection/runs/<run_id>/`.
- Promoted v2 review evidence must include `discovery/candidate-seed.generated.toml`, `discovery/discovery-summary.md`, and `discovery/sources.lock.json`.
- The canonical comparison packet is `docs/agents/selection/cli-agent-selection-packet.md`; it is the maintainer decision surface for approve-or-override.
- The create-lane approval artifact is `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml`; it remains the normative approval artifact consumed by `xtask onboard-agent`.

## Decision Rules

- The research phase must finish before the runner is invoked.
- `freeze-discovery` is the only step allowed to create `research/<run_id>/seed.snapshot.toml`.
- `generate` reads only `research/<run_id>/seed.snapshot.toml` and no longer accepts `--seed-file`.
- Discovery pass 1 emits at most 5 candidates after hard rejection. Discovery pass 2 emits at most 3 new candidates.
- The runner requires at least 3 eligible candidates and then documents exactly 3 shortlisted candidates.
- Candidates already present in `crates/xtask/data/agent_registry.toml` are rejected as ineligible during `generate` and are never scored or shortlisted.
- The scratch run must contain one dossier per seeded candidate, plus the frozen `research-metadata.json` envelope and `seed.snapshot.toml`.
- Scratch v2 runs must declare `run-status.json.workflow_version = "discovery_enabled_v2"`. Promote branches on that discriminator rather than inferring from missing files.
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
- If pass 1 yields zero surviving candidates after hard rejection, pass 2 omits the candidate-relative widening query and uses only the two generic widening queries.
- If pass 1 returns fewer than 3 eligible candidates after evaluation, widen exactly once. If pass 2 still returns fewer than 3 eligible candidates, stop with explicit insufficiency.
- Promotion uses Model B semantics:
  - `comparison.generated.md` remains a committed review byte-copy and the canonical packet stays byte-identical to it.
  - `approved-agent.toml` is rerendered at promote time from the maintainer-approved inputs.
- No packet-template change is required for v2 because discovery changes provenance and reviewed-input flow, not the stable packet decision surface.
- Replacement of a stale committed review run is never in-place:
  - generate a fresh `run_id`
  - promote that fresh run successfully
  - verify the promoted outputs
  - only then delete the stale committed review directory in the same commit
