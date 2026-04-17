# Kickoff Prompt - C0-code (Packet Closeout + Runtime Lock)

## Scope

Preserve the closed OpenCode recommendation packet as a concrete v1 runtime-lock baseline per
`docs/project_management/next/opencode-cli-onboarding/C0-spec.md`.

Planning-only rule:
- edit only `docs/project_management/next/opencode-cli-onboarding/`
- do not touch `crates/opencode/`, `cli_manifests/opencode/`, `crates/agent_api/`, or
  `docs/specs/**`

Expected downstream targets to plan, not edit:
- `crates/opencode/`
- `cli_manifests/opencode/`

## Start Checklist

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, `C0-spec.md`, and this prompt.
3. Set `C0-code` to `in_progress` in `tasks.json` on the orchestration branch.
4. Add a START entry to `session_log.md`; commit docs with `docs: start C0-code`.
5. Create the task branch and worktree:
   `git worktree add -b oco-c0-closeout-code wt/oco-c0-closeout-code feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## Requirements

- preserve one canonical OpenCode runtime surface for v1
- name the deferred surfaces and why they are deferred
- capture install/auth/provider prerequisites and any blocker conditions
- keep the result planning-only and confined to this pack

Validation:
- `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
- quick markdown review of touched pack docs

## End Checklist

1. Run the validation above.
2. Commit pack-doc changes on `oco-c0-closeout-code`.
3. Outside the worktree, ensure the task branch contains the new commit; do not merge into the
   orchestration branch yet.
4. Checkout `feat/opencode-cli-onboarding`; update `tasks.json` to `completed`; add an END entry to
   `session_log.md`; commit docs with `docs: finish C0-code`.
5. Remove worktree `wt/oco-c0-closeout-code`.
