# Kickoff Prompt - C3-integ (UAA Promotion Review)

## Scope

Merge the promotion review and evidence obligations into the final pre-implementation OpenCode
onboarding recommendation per `docs/project_management/next/opencode-cli-onboarding/C3-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch canonical specs, capability matrices, or downstream code

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C3-spec.md`, and this prompt.
3. Set `C3-integ` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C3-integ`.
5. Create the integration branch and worktree:
   `git worktree add -b oco-c3-promotion-review-integ wt/oco-c3-promotion-review-integ feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- merge `oco-c3-promotion-review-code` and `oco-c3-promotion-review-test`
- reconcile the pack to `C3-spec.md`
- finish with one explicit backend-support versus UAA-promotion recommendation

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Merge the upstream C3 branches into the integration worktree and reconcile them to the spec.
2. Run the validation above.
3. Commit integration changes on `oco-c3-promotion-review-integ`.
4. Fast-forward merge `oco-c3-promotion-review-integ` into `feat/opencode-cli-onboarding`; update
   `tasks.json` to `completed`; add an END entry to `session_log.md`; commit docs with
   `docs: finish C3-integ`.
5. Remove worktree `wt/oco-c3-promotion-review-integ`.
