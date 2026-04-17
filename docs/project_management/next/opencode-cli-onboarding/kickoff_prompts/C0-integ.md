# Kickoff Prompt - C0-integ (Packet Closeout + Runtime Lock)

## Scope

Merge the preserved C0 runtime-lock baseline and validation obligations into one execution-ready C0 packet
per `docs/project_management/next/opencode-cli-onboarding/C0-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch downstream code, manifest, or canonical spec paths

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C0-spec.md`, and this prompt.
3. Set `C0-integ` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C0-integ`.
5. Create the integration branch and worktree:
   `git worktree add -b oco-c0-closeout-integ wt/oco-c0-closeout-integ feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- merge `oco-c0-closeout-code` and `oco-c0-closeout-test`
- reconcile the pack to `C0-spec.md`
- hand C1 one locked runtime surface plus explicit inputs and blockers

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Merge the upstream C0 branches into the integration worktree and reconcile them to the spec.
2. Run the validation above.
3. Commit integration changes on `oco-c0-closeout-integ`.
4. Fast-forward merge `oco-c0-closeout-integ` into `feat/opencode-cli-onboarding`; update
   `tasks.json` to `completed`; add an END entry to `session_log.md`; commit docs with
   `docs: finish C0-integ`.
5. Remove worktree `wt/oco-c0-closeout-integ`.
