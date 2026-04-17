# Kickoff Prompt - C2-test (`crates/agent_api/` Backend Planning)

## Scope

Define the future adapter validation strategy for OpenCode inside `crates/agent_api/` per
`docs/project_management/next/opencode-cli-onboarding/C2-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch `crates/agent_api/` or canonical specs under `docs/specs/`

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C2-spec.md`, and this prompt.
3. Set `C2-test` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C2-test`.
5. Create the task branch and worktree:
   `git worktree add -b oco-c2-agent-api-plan-test wt/oco-c2-agent-api-plan-test feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- define fixture-backed test obligations for event mapping, redaction, capability gating, and
  completion finality
- keep live provider requirements out of default validation
- keep the result planning-only and limited to this pack

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Run the validation above.
2. Commit pack-doc changes on `oco-c2-agent-api-plan-test`.
3. Outside the worktree, ensure the task branch contains the new commit; do not merge into the
   orchestration branch yet.
4. Checkout `feat/opencode-cli-onboarding`; update `tasks.json` to `completed`; add an END entry to
   `session_log.md`; commit docs with `docs: finish C2-test`.
5. Remove worktree `wt/oco-c2-agent-api-plan-test`.
