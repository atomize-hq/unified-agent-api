# Session Decision Dossier — Support-depth policy

- Project: unified-agent-api
- Date: 2026-09-28
- Authoring session: 01a0e3d2-6df1-79d2-ab66-e134a84ccb93
- Working repository: `/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api`
- Branch / HEAD: `docs/support-depth-policy-proposal` / `4cc2204e7488247338598d91e378e8da26324a70`
- Status: proposal locally committed; exact routing adjustment reviewed CLEAN; normative adoption and implementation pending.
- Primary topic: family obligations, evidence, and staged maintenance/onboarding/shared-API workflows.

## Executive Summary

The existing automation proves workflow execution and name coverage, but those facts do not establish the desired semantic depth of wrapper support. Codex and Claude Code maintenance rehearsals completed under the current contract; do not reopen them. A reviewed proposal adds independent obligations, bound evidence, explicit legacy treatment and prospective acceptance gates.

The latest committed adjustment separates optional additive Unified Agent API integration from individual-agent maintenance. Maintenance retains required wrapper work and existing shared promises. A later cross-agent workflow discovers and selects additions over landed wrappers. The adjustment received Pro advisory signoff and a CLEAN bounded subagent review.

**Recommended next task:** prepare a bounded family-and-obligation research/decision packet, then normative contracts. Latest rollout recommendation: design the common policy upfront; implement/prove maintenance first; extend/prove onboarding; then implement the additive shared-API workflow. This sequence was discussed, not yet patched into section 7 or independently reviewed as a new delivery plan. The user requested context transfer immediately afterward.

## Objective / Problem Statement

### Objective
Establish what finished support must mean and what evidence proves it, before expanding OpenCode or propagating large amounts of inadequate forwarding.

### In Scope
Representative family research, explicit policy decisions, evidence and identity design, staged activation, and bounded contract/implementation planning across existing lifecycle machinery.

### Out of Scope
Immediate runtime changes, broad Codex/Claude retrofits, unrestricted OpenCode expansion, automatic promotion, changing the public runtime-support payload, or treating this draft as adopted normative authority.

### Constraints
Canonical authority is `docs/specs/**`; drafts and ADRs cannot override it. Keep direct TUI invocation excluded by mode. Preserve manual closeout, stand-down protection and separate promotion. Required non-TUI wrapper work cannot be deferred by an executor merely because it is large. “Thoughts?” requests are discussion-only. No further remote landing was performed or implicitly authorized by this handoff.

## Current State

- [Proposal](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-proposal.md) SHA256: `69bf774e3b17c4500b2ef3c5cfd0eb04556e736a86ef4462b93258030459b2f3`.
- Baseline staging snapshot: `f61534be004305a32e36f9631f218f8094572b59`.
- Local commit `8a093b1b09ac2de34430d50e1a88a1dd2f571400`: records previously reviewed full proposal.
- Local commit `4cc2204e7488247338598d91e378e8da26324a70`: routing adjustment only, 20 insertions/5 deletions, sections 1/5/7.
- No push, PR or staging merge for these commits. Do not call them remotely landed.
- [Backlog](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/backlog.json) remains modified and uncommitted: adds `uaa-0072` plus timestamp, with previous items structurally unchanged. SHA256 `c5f776b1a18472096d6099e6c5e1786f1573c7abbcabaee7a9a55f29c40939b6`. Preserve it. The proposal references this local item; publishing the proposal will require resolving that dependency deliberately.
- Dossier/handoff are stored under the primary checkout for discovery, but the authoritative working candidate is in the managed worktree above. Do not edit the primary checkout by accident.

## Key Decisions

### 1. Independent obligations, preserving old coverage labels
- Decision: assess request representation, validation, accepted values/grammar, output, errors, lifecycle, applicability and shared mapping independently. Keep `explicit`, `passthrough`, and `intentionally_unsupported` meanings.
- Why: name coverage and method existence do not prove semantics. Linear support tiers hide different promises.
- Alternative rejected: relabel all passthrough as deficient or all explicit methods as complete.
- Evidence: proposal sections 2–3; coverage-generator/scenario contracts; manifest report filtering.
- Consequence: families define requirement templates, not achieved status; cross-cutting security effects receive enforceable obligation IDs.

### 2. Family defaults need representative research before implementation
- Proposed defaults: core run/session; machine-readable inventory/query; state-changing administration; diagnostics/compatibility; persistent helpers.
- Approved bounded forwarding can be a finished diagnostic interface. Unknown effects or persistent/mutating modes cannot inherit that permission automatically. Unknown output is unverified, not N/A.
- Evidence: proposal section 3 and Pro consultation. These remain proposed defaults rather than a complete approved operation inventory.
- Consequence: research representative operations across agents and resolve overrides, required proof and activation scope. Do not require exhaustive historical classification before beginning a bounded pilot.

### 3. Separate normative rules, acquired obligations and evolving evidence
- Decision: maintain four bindings: opening-event metadata; immutable policy/delegation plus initial authorization baseline; acquisition-derived obligations; evolving implementation/grant/test state.
- Why: ordinary discovery under existing rules changes concrete obligations, not policy identity. Unchanged policy cannot conceal changed evidence or grants.
- Alternative rejected: one digest that conflates all states or self-attested booleans as proof.
- Evidence: proposal section 4; three-item review artifacts.
- Consequence: second freeze binds acquisition; changed relevant inputs invalidate dependent evidence; exact serialization/receipt schemas still need contracts.

### 4. Preserve narrowly delegated debt operations
- Decision recorded at user direction: executor may renew existing debt within frozen delegation and retire fully satisfied/proven-removed rows; new debt or expanded authority requires maintainer approval/refreeze. Partial satisfaction or missing help is not retirement proof.
- Why: preserve established bounded executor responsibility without allowing it to rewrite authority.
- Alternative considered: remove all debt writes from executor; rejected in favor of constrained delegation.
- Evidence: proposal section 4, Pro three-item CLEAN review, `uaa-0072`.
- Consequence: transition history and frozen authorization baseline need independently checked bindings; due new non-TUI gaps cannot become blanket debt.

### 5. Prospective migration and truthful publication
- Decision proposed: content-bound enumerable legacy baseline can remain visibly unassessed across unchanged releases; changed/new behavior and selected retrofits become due. Keep support matrix facts separate from capability advertising.
- Why: avoid broad retrofits while preventing legacy treatment from swallowing new obligations.
- Evidence: proposal sections 5–6; support-matrix and runtime-support contracts.
- Consequence: retain version-only `latest_validated` public runtime records. No semantic-depth payload extension. Activated generations cannot bypass due-obligation gates through direct promotion.

### 6. Separate additive shared integration from maintenance
- Decision accepted for narrow proposal inclusion: maintenance handles required wrappers and existing shared promises; optional additions use a later cross-agent workflow.
- Why: allow useful agent-specific wrappers and behavior-based comparison without forcing shared abstractions into each release packet.
- Alternatives: all additions inside maintenance; or removing shared responsibility from maintenance entirely. Neither chosen.
- Evidence: [exact adjustment](/Users/spensermcconnell/.codex/attachments/support-depth-policy-proposal-20260927/workflow-separation/proposal.diff), Pro advisory KEEP, two independent causal reviewers CLEAN.
- Consequences: older unmapped operations remain discoverable; stable identities avoid duplicate tasks; selected input bindings are revalidated. Candidates are neither verified support nor debt. Moving a due task or deleting advertising cannot erase its obligation. Single-agent selection does not waive the existing two-backend rule for non-allowlisted universal capability publication. Required onboarding integration remains intact.

### 7. Staged rollout recommendation, not yet a recorded implementation plan
- User proposed maintenance first, then onboarding, then the new shared workflow. Assistant recommended that order with common policy designed upfront.
- Rationale: reuse a proven evaluator while bounding each production rollout.
- Qualification: maintenance proof includes acquisition through closeout and all relevant publication/promotion gates; no activation before its final enforcement exists. Onboarding gets its own end-to-end proof.
- Consequence: section 7 still describes generic plumbing/pilot/product work; refine it only through an explicitly scoped subsequent planning decision. Do not claim this sequence already has causal review or normative adoption.

## Research Findings

- `crates/xtask/src/agent_maintenance/support_audit.rs` clones gaps into missing wrapper and backend lists. Derive these separately from authoritative obligations; do not simply empty the backend list.
- Coverage generation permits explicit through a field OR method; passthrough flags/arguments can disappear from missing lists. Evaluate semantic depth alongside name coverage, including covered declarations.
- `docs/specs/unified-agent-api/support-matrix.md` allows wrapper/backend support before unified support; capability and support matrices retain separate meanings.
- `crates/xtask/src/capability_publication.rs` checks approval truth and universal capability promotion. Adapter behavior requires actual evidence in addition to matrix checks.
- OpenCode retains canonical JSON run; helper expansion needs a separate wrapper scope amendment. External 1.18.31 snapshot counted 481 total = 473 new + 8 existing unrenewed debt identities (also 60 commands + 394 flags + 27 arguments). These are accounting units, not API methods. Refresh external packet before execution; it was not in the original bundled baseline.

## Evidence / Source Map

### Repository files
All paths below are relative to the working repository above.

- `docs/agents/lifecycle/support-depth-policy-proposal.md`: full decision package, boundaries, sources and tests.
- `docs/backlog.json`: uaa-0072 plus coordination with uaa-0041 semantics, uaa-0042 unsupported-command accounting, uaa-0053 OpenCode expansion and uaa-0066 lifecycle acceptance.
- `docs/specs/maintenance-request-contract-v1.md`: frozen requests, debt, scope.
- `docs/specs/cli-agent-onboarding-charter.md`: onboarding and universal promotion.
- `docs/specs/unified-agent-api/support-matrix.md`, `runtime-support-contract.md`, `capabilities-schema-spec.md`: distinct published claims.
- `crates/xtask/src/agent_maintenance/`: preparation, audit, execution, reconciliation, closeout and generated docs.
- `crates/xtask/src/runtime_follow_on/`, `onboard_agent/`: onboarding/runtime consumers.

### Consultation and raw artifacts
- [Same Pro session](https://chatgpt.com/c/6ab9a464-4684-83ea-8391-ea88e04e8849). User explicitly requested continuity. Reuse this session through the in-app browser when another consultation is authorized; keep Pro selected. Advice is not project authority.
- `/Users/spensermcconnell/.codex/attachments/support-depth-policy-proposal-20260927`: initial independent review, remediation and CLEAN closure.
- `/Users/spensermcconnell/.codex/attachments/support-depth-policy-proposal-20260927/pro-followup`: full proposal Pro review; no P1/P2, three non-blocking observations.
- `/Users/spensermcconnell/.codex/attachments/support-depth-policy-proposal-20260927/three-item-followup`: accepted clarifications, backlog diff and Pro CLEAN review.
- `/Users/spensermcconnell/.codex/attachments/support-depth-policy-proposal-20260927/workflow-separation`: bounded approach prompt/response, exact before/after/diff, verification and local landing metadata.
- `/Users/spensermcconnell/.codex/attachments/support-depth-policy-proposal-20260927/workflow-separation/causal-review`: frozen boundary, raw authority/handoff reviews, parent adjudication and final result. Reviewed only `8a093b1b..4cc2204e` with fresh gpt-5.6-sol/xhigh subagents. No P1/P2. One P3: future candidate-group split/merge test; recorded, not patched. CLEAN terminal.
- Latest proof: 60 local links resolve, diff whitespace and make hygiene pass, candidate/backlog hashes stable. No runtime tests claimed for documentation changes.

## Open Questions / Unresolved Risks

- Approve concrete family assignments, operation/mode boundaries and obligation exceptions using representative evidence.
- Define canonical policy/obligation/evidence schemas, dependency digests, receipt producer checks and adequate assertion mappings.
- Select bounded maintenance pilot, version/targets, activation inventory and success/failure cases.
- Select a genuine onboarding rehearsal. OpenCode maintenance expansion alone does not prove onboarding a new agent.
- Formalize staged activation so unenrolled paths cannot bypass activated gates and old schemas cannot masquerade as new proof.
- Decide later how to publish the local proposal/backlog together; no remote reconciliation has been done.
- Avoid coupling unrelated source churn to all legacy reassessments; retain transitive dependencies sufficient for claimed proof.

## Recommended Next Steps

1. **Bounded family-and-obligation research/decision packet.** Produce a table of representative operations across agents: upstream facts, family/mode, promised interface, required obligations, acceptable evidence, classification/override owner, existing mapping responsibility and activation treatment. Record unknowns and exact citations. Success means actionable decisions and bounded scope, not exhaustive wrapper implementation.
2. **Review decisions and prepare normative contracts.** Translate approved choices into schema, receipt, freeze, legacy and gate contracts; reconcile the proposed maintenance-first sequence. No implementation before those controlling decisions are settled.
3. **Maintenance implementation and proof.** Shared evaluator/evidence infrastructure plus full affected maintenance acceptance path. Prove both real bounded completion and negative stale/missing-evidence/regression paths before scoped activation.
4. **Onboarding implementation and proof.** Reuse shared machinery; independently prove approved minimum integration through publication/proving closeout.
5. **Additive shared workflow.** Build candidate discovery/selection and bounded execution after the preceding paths are proven and landed. Preserve existing capability publication rules.

## Resume Instructions

1. Read this dossier and the proposal in the managed worktree; inspect the causal-review result only for the reviewed boundary.
2. Reconfirm branch, HEAD, status and hashes before edits. Preserve uncommitted backlog work. Read current AGENTS instructions. Do not treat primary-checkout HEAD as candidate state.
3. Do not rerun completed Claude/Codex closeout, relitigate the CLEAN routing patch, or implement uaa-0072 just because it exists.
4. Start by freezing the research/decision packet's representative examples, deliverables and non-goals. Confirm staged activation implications in existing contracts. Keep this next effort read-only research/planning until the user authorizes specific changes.
5. If reviewing later deltas, honor bounded causal review: one discovery round, consolidated validated P1/P2 fixes, fresh closure restricted to consequences, at most two supplemental cycles, CLEAN terminal. User explicitly selected gpt-5.6-sol/xhigh for this workstream's causal reviewers.
