# Kickoff Prompt - C3-test (UAA Promotion Review)

## Scope

Define the evidence and follow-on validation gates required before any OpenCode capability is
promoted or intentionally left backend-specific per
`docs/project_management/next/opencode-cli-onboarding/C3-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch canonical specs, capability matrices, or downstream code

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C3-spec.md`, and this prompt.
3. Set `C3-test` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C3-test`.
5. Create the task branch and worktree:
   `git worktree add -b oco-c3-promotion-review-test wt/oco-c3-promotion-review-test feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- define the review evidence needed for promotion or intentional non-promotion
- call out follow-on tests, audits, or execution packs needed after this planning pack
- keep the result planning-only and limited to this directory

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Run the validation above.
2. Commit pack-doc changes on `oco-c3-promotion-review-test`.
3. Outside the worktree, ensure the task branch contains the new commit; do not merge into the
   orchestration branch yet.
4. Checkout `feat/opencode-cli-onboarding`; update `tasks.json` to `completed`; add an END entry to
   `session_log.md`; commit docs with `docs: finish C3-test`.
5. Remove worktree `wt/oco-c3-promotion-review-test`.
