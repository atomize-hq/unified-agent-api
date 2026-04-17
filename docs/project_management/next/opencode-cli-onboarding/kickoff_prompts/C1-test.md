# Kickoff Prompt - C1-test (`crates/opencode/` + `cli_manifests/opencode/` Planning)

## Scope

Define the validation strategy for the future wrapper crate and manifest root per
`docs/project_management/next/opencode-cli-onboarding/C1-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch `crates/opencode/` or `cli_manifests/opencode/` yet

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C1-spec.md`, and this prompt.
3. Set `C1-test` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C1-test`.
5. Create the task branch and worktree:
   `git worktree add -b oco-c1-wrapper-plan-test wt/oco-c1-wrapper-plan-test feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- define fixture, fake-binary, parity-evidence, and maintainer-smoke obligations
- make the automated versus manual validation split explicit
- keep the result planning-only and limited to this pack

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Run the validation above.
2. Commit pack-doc changes on `oco-c1-wrapper-plan-test`.
3. Outside the worktree, ensure the task branch contains the new commit; do not merge into the
   orchestration branch yet.
4. Checkout `feat/opencode-cli-onboarding`; update `tasks.json` to `completed`; add an END entry to
   `session_log.md`; commit docs with `docs: finish C1-test`.
5. Remove worktree `wt/oco-c1-wrapper-plan-test`.
