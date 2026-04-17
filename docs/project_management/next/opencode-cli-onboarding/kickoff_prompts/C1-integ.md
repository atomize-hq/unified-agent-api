# Kickoff Prompt - C1-integ (`crates/opencode/` + `cli_manifests/opencode/` Planning)

## Scope

Merge the wrapper/manifest scope and validation plans into one execution-ready C1 packet per
`docs/project_management/next/opencode-cli-onboarding/C1-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch downstream wrapper or manifest files

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C1-spec.md`, and this prompt.
3. Set `C1-integ` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C1-integ`.
5. Create the integration branch and worktree:
   `git worktree add -b oco-c1-wrapper-plan-integ wt/oco-c1-wrapper-plan-integ feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- merge `oco-c1-wrapper-plan-code` and `oco-c1-wrapper-plan-test`
- reconcile the pack to `C1-spec.md`
- hand C2 one bounded wrapper contract and validation plan

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Merge the upstream C1 branches into the integration worktree and reconcile them to the spec.
2. Run the validation above.
3. Commit integration changes on `oco-c1-wrapper-plan-integ`.
4. Fast-forward merge `oco-c1-wrapper-plan-integ` into `feat/opencode-cli-onboarding`; update
   `tasks.json` to `completed`; add an END entry to `session_log.md`; commit docs with
   `docs: finish C1-integ`.
5. Remove worktree `wt/oco-c1-wrapper-plan-integ`.
