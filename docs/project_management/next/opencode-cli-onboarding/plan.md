# OpenCode CLI Onboarding - Plan

## Purpose

Turn the existing OpenCode recommendation packet into an execution-ready pre-implementation pack
that respects the onboarding charter's crate-first sequencing:

1. confirm the closed packet and preserve the runtime lock
2. plan the wrapper crate and manifest root
3. plan the `agent_api` backend only after the wrapper contract is explicit
4. review UAA promotion separately from backend support

## Guardrails

- Triads only: code / test / integration. No mixed roles.
- This pack is planning-only. No task in C0-C3 may edit repo code, manifests, or canonical specs
  outside `docs/project_management/next/opencode-cli-onboarding/`.
- Downstream targets may be named precisely, but they stay untouched in this pack:
  - `crates/opencode/`
  - `cli_manifests/opencode/`
  - `crates/agent_api/`
- C0 must preserve the canonical OpenCode runtime surface before C1 starts, reopening it only if contradictory evidence appears.
- C1 must freeze wrapper + manifest expectations before C2 starts.
- C2 must freeze backend scope before C3 starts.
- Backend support and UAA-promoted support must remain distinct.
- The charter plus `docs/specs/**` win if any planning text drifts.

## Branch & Worktree Conventions

- Recommended orchestration branch: `feat/opencode-cli-onboarding`
- Feature prefix: `oco`
- Branch naming:
  - C0: `oco-c0-closeout-code`, `oco-c0-closeout-test`, `oco-c0-closeout-integ`
  - C1: `oco-c1-wrapper-plan-code`, `oco-c1-wrapper-plan-test`, `oco-c1-wrapper-plan-integ`
  - C2: `oco-c2-agent-api-plan-code`, `oco-c2-agent-api-plan-test`, `oco-c2-agent-api-plan-integ`
  - C3: `oco-c3-promotion-review-code`, `oco-c3-promotion-review-test`, `oco-c3-promotion-review-integ`
- Worktrees: `wt/<branch>` (in-repo; ignored by git)

## Triad Overview

- **C0 - Packet closeout confirmation + fixture strategy:** confirm the closed packet remains
  internally consistent, preserve the canonical v1 wrapper surface, and lock the replay-fixture
  strategy that later wrapper planning will inherit.
- **C1 - Wrapper crate + manifest-root planning:** define the first implementation packet for
  `crates/opencode/` and `cli_manifests/opencode/`, including spawn/stream/completion boundaries,
  fixture strategy, artifact inventory, and validation obligations.
- **C2 - `agent_api` backend planning:** define the bounded OpenCode backend adapter plan for
  `crates/agent_api/`, including event mapping, capability advertisement, extension policy,
  redaction, and completion-finality obligations derived from the locked wrapper contract.
- **C3 - UAA promotion review:** decide what stays backend-specific versus what is eligible for UAA
  promotion, and record any follow-on execution pack needed for capability-matrix or contract work.

## Start Checklist (all tasks)

1. `git checkout feat/opencode-cli-onboarding && git pull --ff-only`
2. Read: this plan, `tasks.json`, `session_log.md`, the relevant `C*-spec.md`, and the kickoff
   prompt.
3. Set the task status to `in_progress` in
   `docs/project_management/next/opencode-cli-onboarding/tasks.json` on the orchestration branch.
4. Add a START entry to
   `docs/project_management/next/opencode-cli-onboarding/session_log.md`; commit docs with
   `docs: start <task-id>`.
5. Create the task branch and worktree from `feat/opencode-cli-onboarding`:
   `git worktree add -b <branch> wt/<branch> feat/opencode-cli-onboarding`
6. Do not edit `tasks.json` or `session_log.md` from the worktree.

## End Checklist (code/test)

1. Validate docs-only changes:
   - `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
   - quick markdown review of touched pack docs
2. Commit worktree changes on the task branch.
3. From outside the worktree, ensure the task branch contains the new commit; do not merge into the
   orchestration branch yet.
4. Checkout `feat/opencode-cli-onboarding`; update `tasks.json` status; add an END entry to
   `session_log.md` with scope/results/blockers; commit docs with `docs: finish <task-id>`.
5. Remove the worktree: `git worktree remove wt/<branch>`.

## End Checklist (integration)

1. Merge the code/test task branches into the integration worktree and reconcile them to the phase
   spec.
2. Validate the pack:
   - `jq . docs/project_management/next/opencode-cli-onboarding/tasks.json`
   - quick markdown review of touched pack docs
3. Commit integration changes on the integration branch.
4. Fast-forward merge the integration branch into `feat/opencode-cli-onboarding`; update
   `tasks.json` and `session_log.md`; commit docs with `docs: finish <task-id>`.
5. Remove the worktree.

## Context Budget & Sizing

- Keep each phase narrow enough that one agent can review the charter, packet, relevant specs, and
  touched pack docs without dragging in unrelated crate history.
- If OpenCode onboarding expands beyond the current C0-C3 sequence, add more phases rather than
  widening the existing ones.
