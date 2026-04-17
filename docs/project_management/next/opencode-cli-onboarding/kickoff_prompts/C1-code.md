# Kickoff Prompt - C1-code (`crates/opencode/` + `cli_manifests/opencode/` Planning)

## Scope

Define the implementation plan for `crates/opencode/` and `cli_manifests/opencode/` per
`docs/project_management/next/opencode-cli-onboarding/C1-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch `crates/opencode/` or `cli_manifests/opencode/` yet

Expected downstream targets to plan, not edit:
- `crates/opencode/`
- `cli_manifests/opencode/`

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C1-spec.md`, and this prompt.
3. Set `C1-code` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C1-code`.
5. Create the task branch and worktree:
   `git worktree add -b oco-c1-wrapper-plan-code wt/oco-c1-wrapper-plan-code feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- define the wrapper crate contract in terms of spawn, stream, completion, parsing, and redaction
- define the manifest-root artifact inventory and ownership rules
- keep `crates/agent_api/` out of scope except for explicit inputs handed to C2

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Run the validation above.
2. Commit pack-doc changes on `oco-c1-wrapper-plan-code`.
3. Outside the worktree, ensure the task branch contains the new commit; do not merge into the
   orchestration branch yet.
4. Checkout `feat/opencode-cli-onboarding`; update `tasks.json` to `completed`; add an END entry to
   `session_log.md`; commit docs with `docs: finish C1-code`.
5. Remove worktree `wt/oco-c1-wrapper-plan-code`.
