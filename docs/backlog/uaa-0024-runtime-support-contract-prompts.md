# PROMPTS — UAA-0024 Runtime Support Contract

Source contract-spec: [runtime-support-contract.md](../specs/unified-agent-api/runtime-support-contract.md)  
Source plan: [uaa-0024-runtime-support-contract-plan.md](./uaa-0024-runtime-support-contract-plan.md)  
Source tasks: [uaa-0024-runtime-support-contract-tasks.md](./uaa-0024-runtime-support-contract-tasks.md)  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions.
Each prompt is grounded only in the live UAA-0024 contract-spec/plan/tasks stack and current repo truth.

Packet mapping for UAA-0024:

1. **Packet 1** = Tasks 1-5
   - Freeze the validated-only contract
   - Define the embedded metadata model
   - Implement Codex-first `latest_validated` derivation
   - Expose the public `agent_api` runtime-support API
   - Ensure the API works without runtime manifest coupling
2. **Packet 2** = Tasks 6-8
   - Integrate onboarding/publication automation
   - Integrate maintenance automation without widening `requested_control_plane_actions`
   - Close documentation and green-gate drift

## Packet 1 Prompt

```text
/goal Land UAA-0024 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-plan.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/support-matrix.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/cli-agent-onboarding-charter.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/maintenance-request-contract-v1.md

Inspect these live implementation seams before delegating:
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/agent_api/src/lib.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/agent_api/Cargo.toml
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/support_matrix/derive.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/prepare_publication.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/publication_refresh.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/cli_manifests/codex/latest_validated.txt
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/cli_manifests/codex/pointers/latest_validated/x86_64-unknown-linux-musl.txt

Mission:
- Land UAA-0024 Packet 1 only.
- Do not start Packet 2.
- Keep the work bounded to Tasks 1-5 from /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md.

Before editing:
1. Re-read the contract-spec, plan, tasks, support-matrix contract, umbrella `agent_api` contract, onboarding charter, and maintenance request contract.
2. Confirm the packet mapping:
   - Packet 1 = Tasks 1-5 only.
   - Packet 2 = Tasks 6-8 and is out of scope.
3. Re-check the current live code paths listed above.
4. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Freeze and implement the validated-only runtime-support projection.
- Keep the runtime-support payload library-only and embedded/crate-owned.
- Derive Codex-first support data from committed repo truth using `latest_validated`.
- Expose the public `agent_api` API surface for this runtime-support projection.
- Prove there is no consumer-time manifest/JSON/pointer read requirement.

Out of scope:
- onboarding/publication automation integration beyond what Packet 1 minimally needs to compile
- maintenance automation integration
- operator doc reconciliation
- green-gate closeout beyond Packet 1 verification
- widening the surface back to `latest_supported`
- changing the support-matrix contract so `validated == supported`
- any consumer-time repo checkout dependency

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Packet 1 / Tasks 1-5.
- After implementation, run the Packet 1 verification commands:
  - `cargo test -p unified-agent-api --features codex`
  - `cargo test -p xtask --all-targets`
  - `git diff --stat`
  - `git status --short`
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against the contract-spec, plan, tasks, support-matrix contract, umbrella `agent_api` contract, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not invent an empty commit for a review-only step that changed no files.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- the public contract is validated-only for v1 and does not expose `latest_supported`
- the public surface is library-only and does not require runtime reads from `cli_manifests/**`
- the payload is crate-owned/embedded and usable without a repo checkout
- Codex-first derivation is grounded in committed `latest_validated` truth
- the new API does not leak backend-wrapper crate types
- Packet 2 automation/doc work remains untouched except for strictly necessary compile/test alignment

Implementation subagent prompt:
/goal Land UAA-0024 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-plan.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/support-matrix.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/agent_api/src/lib.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/support_matrix/derive.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/cli_manifests/codex/latest_validated.txt, and /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/cli_manifests/codex/pointers/latest_validated/x86_64-unknown-linux-musl.txt first. Work only on Tasks 1-5. Implement the smallest coherent validated-only runtime-support slice that makes the public API real, Codex-first, embedded/crate-owned, and free of consumer-time manifest reads. Do not start Packet 2. Run `cargo test -p unified-agent-api --features codex`, `cargo test -p xtask --all-targets`, `git diff --stat`, and `git status --short`. Final message must state whether Packet 1 is checkpoint-green, what files and symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed UAA-0024 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-plan.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/support-matrix.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/contract.md, and the live diff. Review only Packet 1. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required UAA-0024 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-plan.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/support-matrix.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/agent_api/src/lib.rs, and /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/support_matrix/derive.rs. Fix only the flagged Packet 1 issues without widening scope. Do not start Packet 2. Re-run `cargo test -p unified-agent-api --features codex`, `cargo test -p xtask --all-targets`, `git diff --stat`, and `git status --short`. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State the exact commit(s) created for implementation and any fix rounds.
- Report whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land UAA-0024 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-plan.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/cli-agent-onboarding-charter.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/maintenance-request-contract-v1.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/support-matrix.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/cli-agent-onboarding-factory-operator-guide.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/cli-agent-onboarding-factory-workflow-atlas.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/cli-agent-maintenance-steady-state-plan.md

Inspect these live implementation seams before delegating:
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/onboard_agent.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/prepare_publication.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/publication_refresh.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/prepare.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/refresh.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/drift/publication.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/request/automation.rs

Mission:
- Land UAA-0024 Packet 2 only.
- Packet 1 must already be landed and checkpoint-green on the current tree.
- Keep the work bounded to Tasks 6-8 from /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md.

Before editing:
1. Re-read the contract-spec, plan, tasks, onboarding charter, maintenance request contract, support-matrix contract, operator guide, workflow atlas, and maintenance steady-state plan.
2. Confirm the packet mapping:
   - Packet 1 = Tasks 1-5 and must already be green.
   - Packet 2 = Tasks 6-8 only.
3. Re-check the current live automation/doc seams listed above.
4. Verify Packet 1 is already landed and checkpoint-green on the current tree before delegating.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Integrate runtime-support regeneration/validation into onboarding/publication automation.
- Integrate runtime-support regeneration/validation into maintenance automation.
- Keep automated release-watch maintenance on `requested_control_plane_actions = ["packet_doc_refresh"]`.
- Reconcile the downstream normative/operator docs and close green-gate drift.

Out of scope:
- redesigning Packet 1 API/data model
- widening the contract back to both tiers
- changing the support-matrix contract so `validated == supported`
- widening automated maintenance into a second action queue
- unrelated onboarding or maintenance cleanup outside UAA-0024

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Packet 2 / Tasks 6-8.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p xtask --all-targets`
  - `cargo run -p xtask -- support-matrix --check`
  - `cargo run -p xtask -- capability-matrix --check`
  - `cargo run -p xtask -- capability-matrix-audit`
  - `make preflight`
  - `git diff --stat`
  - `git status --short`
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against the contract-spec, plan, tasks, onboarding charter, maintenance request contract, operator docs, and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not invent an empty commit for a review-only step that changed no files.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- publication/onboarding lanes regenerate or validate the runtime-support artifact in the existing publication machinery
- maintenance lanes regenerate or validate the runtime-support artifact without widening `requested_control_plane_actions`
- the normative/operator docs explicitly say this is a library-only validated-runtime projection
- no touched doc or code path implies consumer runtime manifest reads
- the full repo green story for the touched surfaces is clean

Implementation subagent prompt:
/goal Land UAA-0024 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-plan.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/cli-agent-onboarding-charter.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/maintenance-request-contract-v1.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/cli-agent-onboarding-factory-operator-guide.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/cli-agent-onboarding-factory-workflow-atlas.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/cli-agent-maintenance-steady-state-plan.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/prepare_publication.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/publication_refresh.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/prepare.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/refresh.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/drift/publication.rs, and /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/request/automation.rs first. Work only on Tasks 6-8. Verify Packet 1 is already landed and green before making changes. Implement only the minimum automation/doc reconciliation needed to keep runtime-support regeneration/validation inside the existing publication and packet machinery while preserving `requested_control_plane_actions = [\"packet_doc_refresh\"]`. Do not widen scope beyond Packet 2. Run `cargo test -p xtask --all-targets`, `cargo run -p xtask -- support-matrix --check`, `cargo run -p xtask -- capability-matrix --check`, `cargo run -p xtask -- capability-matrix-audit`, `make preflight`, `git diff --stat`, and `git status --short`. Final message must state whether Packet 2 is checkpoint-green, what files and symbols changed, what verification ran, and whether any reopen condition remains.

Review subagent prompt:
Review the committed UAA-0024 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-plan.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/cli-agent-onboarding-charter.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/maintenance-request-contract-v1.md, the relevant operator docs, and the live diff. Review only Packet 2. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required UAA-0024 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/runtime-support-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-plan.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/backlog/uaa-0024-runtime-support-contract-tasks.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/cli-agent-onboarding-charter.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/docs/specs/maintenance-request-contract-v1.md, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/prepare_publication.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/publication_refresh.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/prepare.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/refresh.rs, and /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/crates/xtask/src/agent_maintenance/drift/publication.rs. Fix only the flagged Packet 2 issues without widening scope. Re-run `cargo test -p xtask --all-targets`, `cargo run -p xtask -- support-matrix --check`, `cargo run -p xtask -- capability-matrix --check`, `cargo run -p xtask -- capability-matrix-audit`, `make preflight`, `git diff --stat`, and `git status --short`. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State the exact commit(s) created for implementation and any fix rounds.
- State whether the full UAA-0024 triplet is now unblocked for implementation closeout.
- If anything is not green, say explicitly what remains and that Packet 2 is not fully closed.
```
