# Kickoff Prompt - C3-code (UAA Promotion Review)

## Scope

Draft the OpenCode UAA promotion review per
`docs/project_management/next/opencode-cli-onboarding/C3-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch canonical specs, capability matrices, or downstream code

Expected downstream targets to review, not edit:
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/extensions-spec.md`

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C3-spec.md`, and this prompt.
3. Set `C3-code` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C3-code`.
5. Create the task branch and worktree:
   `git worktree add -b oco-c3-promotion-review-code wt/oco-c3-promotion-review-code feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- decide what is a backend-specific OpenCode capability versus a UAA promotion candidate
- base the review on C1/C2 concrete scope, not the original packet alone
- keep the review planning-only and confined to this pack

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Run the validation above.
2. Commit pack-doc changes on `oco-c3-promotion-review-code`.
3. Outside the worktree, ensure the task branch contains the new commit; do not merge into the
   orchestration branch yet.
4. Checkout `feat/opencode-cli-onboarding`; update `tasks.json` to `completed`; add an END entry to
   `session_log.md`; commit docs with `docs: finish C3-code`.
5. Remove worktree `wt/oco-c3-promotion-review-code`.
