---
name: recommend-next-agent
description: Run the repo-owned pre-create recommendation host flow for the next CLI agent, from `recommend-next-agent-research` dry-run/write through `generate` and `promote`.
---

# Recommend Next Agent

Use this skill when a maintainer wants to choose the next CLI agent before the existing `xtask onboard-agent --approval ...` create lane begins.

Normative contract:
- `docs/specs/cli-agent-recommendation-dossier-contract.md`

Operator procedure:
- `docs/cli-agent-onboarding-factory-operator-guide.md`

## Thin Wrapper

1. Optionally read `docs/agents/selection/discovery-hints.json`.
2. Prepare the `pass1` execution packet:

```sh
cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass1 --run-id <pass1_run_id>
```

3. Execute the matching `pass1` packet:

```sh
cargo run -p xtask -- recommend-next-agent-research --write --pass pass1 --run-id <pass1_run_id>
```

4. Review the repo-owned execution evidence under:

`docs/agents/.uaa-temp/recommend-next-agent/research-runs/<pass1_run_id>/`

5. Generate the post-research evaluation run:

```sh
python3 scripts/recommend_next_agent.py generate \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<pass1_run_id> \
  --run-id <pass1_run_id> \
  --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs
```

6. If `generate` returns insufficiency, run exactly one `pass2` retry with a fresh `run_id` and prior insufficiency input:

```sh
cargo run -p xtask -- recommend-next-agent-research --dry-run --pass pass2 \
  --prior-run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<pass1_run_id> \
  --run-id <pass2_run_id>

cargo run -p xtask -- recommend-next-agent-research --write --pass pass2 \
  --prior-run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<pass1_run_id> \
  --run-id <pass2_run_id>

python3 scripts/recommend_next_agent.py generate \
  --research-dir docs/agents/.uaa-temp/recommend-next-agent/research/<pass2_run_id> \
  --run-id <pass2_run_id> \
  --scratch-root docs/agents/.uaa-temp/recommend-next-agent/runs
```

7. Promote exactly one reviewed evaluation run:

```sh
python3 scripts/recommend_next_agent.py promote \
  --run-dir docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id> \
  --repo-run-root docs/agents/selection/runs \
  --approved-agent-id <agent_id> \
  --onboarding-pack-prefix <kebab-case-pack-prefix> \
  [--override-reason "<required when approved agent differs from recommended>"]
```

8. Stop for maintainer approve-or-override, then continue with:

```sh
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --dry-run
cargo run -p xtask -- onboard-agent --approval docs/agents/lifecycle/<onboarding_pack_prefix>/governance/approved-agent.toml --write
```

## Hard Rules

- The repo, not Codex, owns prompt rendering, dry-run packet creation, bounded Codex execution, `freeze-discovery`, validation, and execution evidence.
- Pass support is exactly `pass1` and `pass2`.
- `pass2` requires prior insufficiency input and a fresh `run_id`.
- `--write` is invalid without a preexisting dry-run packet for the same `run_id`.
- The host flow rejects discovery seeds with fewer than 3 candidates before `freeze-discovery`.
- The host flow may canonicalize valid `sources.lock.json` entry hashes before `freeze-discovery`.
- The host flow validates research dossiers against the same repo-owned Python contract that `generate` uses.
- Codex write roots are limited to `docs/agents/.uaa-temp/recommend-next-agent/discovery/<run_id>/` and `docs/agents/.uaa-temp/recommend-next-agent/research/<run_id>/`.
- The execution packet root is `docs/agents/.uaa-temp/recommend-next-agent/research-runs/<run_id>/`.
- The evaluation run root is `docs/agents/.uaa-temp/recommend-next-agent/runs/<run_id>/`.
- Do not hand-author discovery artifacts, `freeze-discovery` outputs, or dossiers outside the repo-owned `recommend-next-agent-research --write` flow.
- `generate` and `promote` CLI shapes stay unchanged.
- `approved-agent.toml` remains the normative create-lane input.
