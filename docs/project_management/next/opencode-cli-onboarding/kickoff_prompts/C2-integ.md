# Kickoff Prompt - C2-integ (`crates/agent_api/` Backend Planning)

## Scope

Merge the OpenCode backend scope and validation plan into one execution-ready C2 packet per
`docs/project_management/next/opencode-cli-onboarding/C2-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch `crates/agent_api/` or canonical specs under `docs/specs/`

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C2-spec.md`, and this prompt.
3. Set `C2-integ` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C2-integ`.
5. Create the integration branch and worktree:
   `git worktree add -b oco-c2-agent-api-plan-integ wt/oco-c2-agent-api-plan-integ feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- merge `oco-c2-agent-api-plan-code` and `oco-c2-agent-api-plan-test`
- reconcile the pack to `C2-spec.md`
- hand C3 an explicit promotion-review input set

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Merge the upstream C2 branches into the integration worktree and reconcile them to the spec.
2. Run the validation above.
3. Commit integration changes on `oco-c2-agent-api-plan-integ`.
4. Fast-forward merge `oco-c2-agent-api-plan-integ` into `feat/opencode-cli-onboarding`; update
   `tasks.json` to `completed`; add an END entry to `session_log.md`; commit docs with
   `docs: finish C2-integ`.
5. Remove worktree `wt/oco-c2-agent-api-plan-integ`.
