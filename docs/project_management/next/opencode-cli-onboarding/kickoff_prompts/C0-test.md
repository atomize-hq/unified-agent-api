# Kickoff Prompt - C0-test (Packet Closeout + Runtime Lock)

## Scope

Define the runtime-evidence and reproducibility obligations that must accompany the already-locked C0 runtime surface
per `docs/project_management/next/opencode-cli-onboarding/C0-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch downstream code, manifest, or canonical spec paths

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C0-spec.md`, and this prompt.
3. Set `C0-test` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C0-test`.
5. Create the task branch and worktree:
   `git worktree add -b oco-c0-closeout-test wt/oco-c0-closeout-test feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- define the required maintainer smoke evidence for the chosen runtime surface
- identify reproducibility constraints and blocking unknowns
- keep the result planning-only and limited to this pack

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Run the validation above.
2. Commit pack-doc changes on `oco-c0-closeout-test`.
3. Outside the worktree, ensure the task branch contains the new commit; do not merge into the
   orchestration branch yet.
4. Checkout `feat/opencode-cli-onboarding`; update `tasks.json` to `completed`; add an END entry to
   `session_log.md`; commit docs with `docs: finish C0-test`.
5. Remove worktree `wt/oco-c0-closeout-test`.
