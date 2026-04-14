---
seam_id: SEAM-4
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-4-claude-code-mapping/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - seam-2-closeout.md
    - seam-3-closeout.md
  required_threads:
    - THR-05
  stale_triggers:
    - Claude argv ordering contract changes
    - new universal keys touch fallback-model semantics
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-4 Claude Code backend mapping

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-4-claude-code-mapping/slice-4-seam-exit-gate.md`
- **Landed evidence**:
  - commit `09f7a69` (plumb normalized model id into Claude print request)
  - commit `7ac3af1` (pin argv ordering + fallback exclusion for print/session flows)
  - commit `982e014` (safe runtime rejection + event/completion parity)
  - tests:
    - `cargo test -p agent_api --features claude_code "model_mapping::"`
    - `cargo test -p agent_api --features claude_code claude_runtime_model_rejection_is_safely_redacted_and_parity_is_preserved`
- **Contracts published or changed**: no contract text changes; implementation now satisfies `C-07` per `docs/specs/claude-code-session-mapping-contract.md`.
- **Threads published / advanced**: `THR-05` marked published in `../threading.md`.
- **Review-surface delta**: added Claude mapping tests for model argv ordering + runtime rejection parity.
- **Planned-vs-landed delta**: none.
- **Downstream stale triggers raised**: none.
- **Remediation disposition**: none opened.
- **Promotion blockers**: none.
- **Promotion readiness**: ready.

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
