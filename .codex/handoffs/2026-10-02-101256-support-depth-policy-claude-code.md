# Handoff: Support-depth policy decision package for Claude Code

## Session Metadata

- Created: 2026-10-02 10:12:56 America/Indianapolis.
- Candidate worktree: /Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api.
- Candidate branch and HEAD: docs/support-depth-policy-proposal at 4cc2204e7488247338598d91e378e8da26324a70 (verified October 2).
- Primary checkout: /Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api, branch feat/t6-prepare-agent-closeout at 95b06667296a9c9ceee5f6c6c76eb05da54d8769. It is not the support-depth candidate.
- Session duration: multi-session work from September 28 through this handoff.
- Continues from: [September 28 handoff](2026-09-28-180807-support-depth-policy-research-next.md).
- Historical rationale: [decision dossier](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.codex/dossiers/support-depth-policy-decisions-2026-09-28.md). Current drafts supersede its migration/legacy recommendations.

## Current State Summary

Two full Draft documents now form a bounded research and recommended-decision package: the revised [proposal](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-proposal.md) and new [research companion](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-research-decisions.md). They incorporate the maintainer's greenfield and minimum-necessary-machinery constraints, representative operations, D1-D13 decisions, maintenance-first/onboarding-second proof, and a later additive shared-API workflow. Both are local, unstaged and uncommitted. Pro's final consultation found one P2 contradiction in onboarding timing; a one-line correction was applied to the companion on September 29 and independently reviewed CLEAN in that narrow delta. Pro has not re-reviewed the corrected bytes. No normative adoption, pilot enrollment, runtime implementation, closeout, publication, push or PR is claimed.

## Codebase Understanding

### Architecture Overview

This Rust workspace has CLI wrappers under crates/codex/ and crates/claude_code/, a shared API in crates/agent_api/, and lifecycle/registry/publication automation in crates/xtask/. docs/specs/** is normative; these docs/agents/lifecycle/** documents are Draft recommendations. Name coverage, semantic support depth, shared adapter integration and published capability are distinct claims. The proposed policy uses existing lifecycle approvals, packet inputs, validation, tests, reports and closeout wherever they can carry needed authority and evidence. Four logical bindings separate opening event, policy/delegation authority, acquired obligations, and evolving execution/grant/evidence state; four new files are not required.

### Critical Files

| File | Purpose |
| --- | --- |
| [Proposal](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-proposal.md) | Draft architecture, greenfield adoption, path-specific activation and delivery order. Current SHA-256: 690dac5dd45ad65c5fd4ad716a759a761bb6006fa01a501b9709a2cce638e8e2. |
| [Research companion](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-research-decisions.md) | Eighteen representative rows, obligation/evidence templates, D1-D13 register and P0-P5 plan. Current SHA-256: b35b0e7944b21e4f08bb4610f2bab02c879006c169a96a8322806e9beba21ebd. Corrected timing: line 351. |
| [Backlog](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/backlog.json) | Pre-existing uncommitted uaa-0072 addition; preserve byte-for-byte. SHA-256: c5f776b1a18472096d6099e6c5e1786f1573c7abbcabaee7a9a55f29c40939b6. |
| [Maintenance request contract](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/specs/maintenance-request-contract-v1.md) | Existing request/debt authority for later normative work. |
| [Onboarding charter](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/specs/cli-agent-onboarding-charter.md) | Existing approval, minimum integration and capability rules. |

### Key Patterns Discovered

- Families are obligation templates, not support tiers. Mixed modes and cross-cutting effects need explicit treatment.
- Required wrapper work, existing shared repairs and onboarding minimum integration remain due. Optional new shared-API additions belong in a later cross-agent workflow; discovery alone does not advertise support.
- Greenfield enrollment is prospective and bounded. Explicitly unenrolled scope makes no new depth claim but does not waive independent duties. No legacy inventory, grandfathering, semantic carry-forward ledger or migration engine is recommended.
- Maintenance may activate before complete onboarding proof only if every alternate writer affecting the maintenance acceptance outputs enforces the gate or refuses those writes. Onboarding requires its own independent proof.
- Enrollment can begin with implementation/evidence work outstanding once the path can enforce its rules. Real-agent qualification is due before production acceptance, not before enrollment.

## Work Completed

### Tasks Finished

- Completed bounded family/obligation research and two full candidate Draft documents.
- Incorporated the maintainer's greenfield clarification and minimum-necessary-machinery constraint.
- Sent frozen candidate to a fresh [ChatGPT Pro consultation](https://chatgpt.com/c/6abbfe88-ba84-83e9-96b4-d17ccec51e37). Pro returned ADJUST, one P2: companion line 351 made real-agent qualification a prerequisite to P4-O enrollment, contrary to proposal line 227. Pro found no P1 or other P2 in the bounded packet.
- Corrected only companion line 351: prove independent new-entry machinery before P4-O; qualify the selected real agent on its exact approved scope and upstream evidence before production acceptance. A separate read-only narrow reviewer returned CLEAN: exactly one removed/one added line, no causal P1/P2 or meaningful P3.
- On September 29, verified the correction introduced no whitespace error and left proposal/backlog hashes unchanged. On October 2, rebound live branch/HEAD/status and all three hashes. No runtime tests are claimed.

### Files Modified

| File | Current state and reason |
| --- | --- |
| [support-depth-policy-proposal.md](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-proposal.md) | Tracked, modified, unstaged. Full greenfield/minimal-machinery Draft; untouched by the final one-line correction. |
| [support-depth-policy-research-decisions.md](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-research-decisions.md) | Untracked. Full research companion plus reviewed correction at line 351. Ordinary git diff omits this file. |
| docs/backlog.json | Tracked, modified, unstaged before this latest work. Not edited by the final clarification or handoff; preserve it. |
| This handoff | Stored under the primary checkout's .codex/handoffs/ for Claude Code discovery, not a candidate policy document. |

### Decisions Made

The maintainer accepted greenfield clean adoption and minimum necessary evidence/binding/enrollment machinery as design constraints. D1-D13 remain recommendations pending maintainer disposition, not adopted normative rules. Previously settled inputs include separate name coverage and semantic obligations; constrained delegated debt renewal and proved retirement; separate logical authority/acquisition/execution bindings; OpenCode's canonical run --format json transport; required maintenance/existing shared repairs versus optional additive integration. The initial maintenance pilot and real new-agent onboarding selection remain open. Goose is a research candidate, not selected or qualified.

## Pending Work

### Immediate Next Steps

1. In Claude Code, open this handoff, then change directory to the candidate worktree above. Read both complete Draft documents and the controlling specs relevant to the next bounded task. Recheck branch, HEAD, status and hashes first; do not edit the primary checkout by accident.
2. Determine the maintainer's requested continuation: D1-D13 disposition and exact pilot/agent choices; a fresh Pro check of the corrected candidate; local document landing; or an authorized normative-contract packet. This handoff does not choose among them or authorize runtime implementation. If no narrower task is provided, prepare a concise read-only decision/landing readiness report.
3. Before any landing, reconcile the uncommitted backlog dependency deliberately, recheck both complete document bytes and corrected line, and use a review scope matching the bytes actually committed. Keep local commit, remote landing, normative adoption, enrollment and qualified production support distinct.

### Blockers/Open Questions

- D1-D13 need maintainer disposition for normative contracts. Exact maintenance version/targets, recurring selection, pilot evidence and genuinely new onboarding agent remain pending.
- Pro has not issued CLEAN on corrected bytes. Local narrow reviewer CLEAN covers one line only; it is not runtime proof.
- Proposal references the uncommitted backlog item. Resolve that publication/landing relationship before relying on it in a commit or PR.
- Later P1/P2 work must inspect actual final writers and conflict control, source trust/evidence, and the normative capability allowlist discrepancy.

### Deferred Items

Normative docs/specs/** adoption, runtime/schema/workflow implementation, OpenCode helper expansion, pilots, cross-agent additive automation, historical wrapper retrofit, promotion and remote publication remain outside this completed Draft-document work.

## Context for Resuming Agent

### Important Context

The September 28 handoff/dossier described a legacy baseline and migration treatment; the maintainer later explicitly rejected migration-only work because this policy is greenfield. The current drafts supersede that older advice. They retain reproducible bindings, authorized debt transitions and scoped enrollment through existing machinery where sufficient. Do not reintroduce a second enrollment registry, duplicate evidence ledger or approval ceremony without a concrete gap.

Pro's sole P2 was a timing contradiction, not a request to redesign onboarding. Current companion line 351 separates the prerequisites: synthetic isolated proof can establish lifecycle machinery before P4-O; only the selected real agent's own exact-scope upstream evidence can establish production qualification. Known support work may remain at enrollment, while publication/acceptance gates block premature claims. The narrow CLEAN review is not Pro's fresh verdict.

The managed candidate worktree is an ordinary filesystem path Claude Code can open. It is presently the checkout with the current two-document candidate; primary checkout branch/HEAD differ. The earlier proposal baseline is locally committed at 4cc2204e; revised two-document package is only working-tree content.

### Assumptions Made

- The user wants a transferable handoff for Claude Code's next bounded step; no specific commit, push, PR, adoption, runtime change or new Pro consultation was requested in this turn.
- Existing canonical specs control until separately amended by authorized work.

### Potential Gotchas

- Git diff alone omits the untracked companion. Include its full content/hash in any review or commit inventory.
- Old handoff says section 7 sequencing was not written; it now appears in proposal section 7. Old migration language is superseded.
- September 29 Pro packet used pre-correction companion SHA-256 0bde90c31c71799ade4a0d21d5fb5cd77492acd8bb5d83c569e266d65c9827ef; current companion is b35b0e7944b21e4f08bb4610f2bab02c879006c169a96a8322806e9beba21ebd.
- If future work reaches an open maintenance closeout, read current automation-stand-down instructions first; nightly regeneration can destroy packet work. This handoff does not authorize that closeout.

## Environment State

### Tools/Services Used

Local macOS/zsh, git, Codex in-app-browser ChatGPT Pro, and a read-only narrow reviewer. [Full Pro reply](/Users/spensermcconnell/.codex/attachments/support-depth-final-pro-signoff-20260929-1f24de07/pro-response-rendered.txt) and [consultation record](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.codex/guidance/2026-09-29-support-depth-final-pro-signoff.md) preserve the final Pro packet. [Earlier full-candidate independent review](/Users/spensermcconnell/.codex/attachments/support-depth-minimal-machinery-20260929-730dbddb/independent-review/result.json) covers the pre-correction frozen bytes. [Pre-correction companion](/Users/spensermcconnell/.codex/attachments/support-depth-final-pro-signoff-20260929-1f24de07/inputs/docs/agents/lifecycle/support-depth-policy-research-decisions.md) enables exact delta comparison.

### Active Processes

No task-owned watcher, test, server or consultation is known to be running. Pro consultation and narrow review completed.

### Environment Variables

No task-specific environment variable or secret value is required to resume.

## Related Resources

- [Candidate proposal](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-proposal.md) and [research companion](/Users/spensermcconnell/.codex/worktrees/support-depth-proposal/unified-agent-api/docs/agents/lifecycle/support-depth-policy-research-decisions.md).
- [Pro conversation](https://chatgpt.com/c/6abbfe88-ba84-83e9-96b4-d17ccec51e37) and [saved reply](/Users/spensermcconnell/.codex/attachments/support-depth-final-pro-signoff-20260929-1f24de07/pro-response-rendered.txt).
- [Earlier decision dossier](/Users/spensermcconnell/__Active_Code/atomize-hq/unified-agent-api/.codex/dossiers/support-depth-policy-decisions-2026-09-28.md), with legacy treatment superseded by current drafts.

## Ready-to-use Claude Code prompt

Read this handoff and both complete support-depth Draft documents in the managed candidate worktree. Rebind live branch, HEAD, status and file hashes; preserve the existing dirty backlog. Treat the September 28 dossier as historical where it conflicts with the newer greenfield drafts. The final Pro response found one onboarding timing P2; the companion has a one-line local correction at line 351, independently reviewed CLEAN for that delta but not rechecked by Pro. First report current decision/landing readiness and D1-D13 choices requiring maintainer disposition. Proceed with a specific edit, contract packet, commit or publication only within the scope the maintainer gives you. Do not call this Draft adopted policy, qualified runtime support or remote landing.
