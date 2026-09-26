# Codex 0.156.1 closeout rehearsal

**Recorded:** 2026-09-25
**Scope:** supervised closeout rehearsal for the frozen Codex `0.156.1` maintenance packet. This is a durable receipt for the merged packet, not a new disposition authority or a claim that T8 is complete.

## Durable record

- [PR #237](https://github.com/atomize-hq/unified-agent-api/pull/237) merged the relay nested-invocation guard at [28ef97681df2a30ed23a75afdb38bb46484093be](https://github.com/atomize-hq/unified-agent-api/commit/28ef97681df2a30ed23a75afdb38bb46484093be).
- The implementation commit was [aacd785fe6a3920ab016b6ce7986a4c7c5831f18](https://github.com/atomize-hq/unified-agent-api/commit/aacd785fe6a3920ab016b6ce7986a4c7c5831f18); its three recorded CI conclusions succeeded: [36185345670](https://github.com/atomize-hq/unified-agent-api/actions/runs/36185345670) (attempt 2 after an unrelated Linux restart-test failure on attempt 1), [36185346848](https://github.com/atomize-hq/unified-agent-api/actions/runs/36185346848), and [36187018295](https://github.com/atomize-hq/unified-agent-api/actions/runs/36187018295).
- [afcfed2b208897fabec422439923a1f388e67f4f](https://github.com/atomize-hq/unified-agent-api/commit/afcfed2b208897fabec422439923a1f388e67f4f) recorded the tool-generated closeout. Its two final CI conclusions succeeded: [36191614745](https://github.com/atomize-hq/unified-agent-api/actions/runs/36191614745) and [36191614990](https://github.com/atomize-hq/unified-agent-api/actions/runs/36191614990).
- [PR #236](https://github.com/atomize-hq/unified-agent-api/pull/236) merged at 2026-09-25T23:51:37Z as [88ed826173b0c0d2132a1394f9fd643536a066ea](https://github.com/atomize-hq/unified-agent-api/commit/88ed826173b0c0d2132a1394f9fd643536a066ea). The merged [request](https://github.com/atomize-hq/unified-agent-api/blob/88ed826173b0c0d2132a1394f9fd643536a066ea/docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml), [closeout](https://github.com/atomize-hq/unified-agent-api/blob/88ed826173b0c0d2132a1394f9fd643536a066ea/docs/agents/lifecycle/codex-maintenance/governance/maintenance-closeout.json), and [closed handoff](https://github.com/atomize-hq/unified-agent-api/blob/88ed826173b0c0d2132a1394f9fd643536a066ea/docs/agents/lifecycle/codex-maintenance/HANDOFF.md) are the canonical artifacts.

## What was demonstrated

The source review and the frozen packet gates completed successfully. The frozen queue contained 54 work items plus two pre-existing completion-debt identities; the final audit was 0 required, 0 unbaselined, and 2 existing debt. The canonical closeout contains 33 dispositions without reproducing their rows here: 31 live rows carried into closeout and two withdrawn obsolete rows retained in the historical record.

The `execute-agent-maintenance` write run `20260925T192918Z` completed with exit 0 under an external watchdog and made no recursive lifecycle invocation. Regression tests cover the guard's refusal of a nested invocation. The actual closeout used the reachable non-merge implementation SHA above and recorded `2026-09-25T21:25:15Z`; `prepare-agent-closeout --write` and `close-agent-maintenance` both exited 0. The three expected closeout files were produced, repeat close was byte-idempotent, and the frozen request and lifecycle state did not change.

Before closeout, prepare carried the 31 live rows. The operator restored the two already-reviewed obsolete dispositions manually, so the canonical historical record remained complete and the closeout validator accepted it. This is a reproducibility limit of the live-only prepare projection, not a failed validation.

## Limits retained

The watchdog supervised the host process because the relay does not itself impose a bounded runtime; `uaa-0069` remains open. The packet's `latest_validated` pointer remains `0.125.0`, because closeout neither promotes nor retires the stand-down marker. This supervised Codex pass does not establish broader T8 completion, hidden semantic completeness, or automatic retention of obsolete history.
