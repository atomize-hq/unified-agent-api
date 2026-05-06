---
seam_id: SEAM-2
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: threaded-seams/seam-2-agent-api-opencode-backend/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - seam-1-closeout.md
    - ../../opencode-cli-onboarding/governance/seam-3-closeout.md
  required_threads:
    - THR-04
    - THR-05
    - THR-06
  stale_triggers:
    - wrapper event or completion semantics drift
    - capability advertisement or extension registry drift
    - redaction or bounded-payload posture drift
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 `agent_api` OpenCode backend

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-2-agent-api-opencode-backend/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - `1adb8f1` `SEAM-2: complete slice-00-backend-contract-and-registration-baselines`
  - `f9c9982` `SEAM-2: complete slice-1-request-event-and-completion-mapping`
  - `4adefdf` `SEAM-2: complete slice-2-capability-advertisement-and-extension-ownership`
  - `ed424c5` `SEAM-2: complete slice-3-validation-and-redaction-boundary`
  - `cargo check -p unified-agent-api --features opencode`
  - `cargo test -p unified-agent-api --features opencode`
- **Contracts published or changed**:
  - `C-03` published through `docs/specs/opencode-agent-api-backend-contract.md` plus landed
    `crates/agent_api/src/backends/opencode/**` implementation and backend tests
- **Threads published / advanced**:
  - `THR-06` now publishes the landed OpenCode backend request/event/completion mapping,
    conservative capability posture, fail-closed extension boundary, and deterministic validation
    evidence for `SEAM-3`
- **Review-surface delta**:
  - `crates/agent_api/**` now exposes a feature-gated OpenCode backend that consumes the landed
    wrapper crate instead of re-implementing wrapper transport or parser behavior
  - public OpenCode backend events remain bounded and redacted: text maps to `TextOutput`,
    lifecycle maps to `Status`, parse failures surface as safe `Error` events, and completion data
    stays `None`
  - capability advertisement is intentionally conservative and now matches the landed backend
    contract and generated capability inventory:
    <!-- xtask-governance-check:opencode-capabilities:start -->
    `agent_api.run`, `agent_api.events`, `agent_api.events.live`,
    `agent_api.config.model.v1`, `agent_api.session.resume.v1`, `agent_api.session.fork.v1`
    <!-- xtask-governance-check:opencode-capabilities:end -->
    are the claimed OpenCode v1 capability ids under the current runtime evidence
  - deterministic fake-binary validation, timeout redaction, and missing-binary redaction are now
    the default backend proof path; live-provider smoke is still basis-lock evidence only
- **Planned-vs-landed delta**:
  - S2 landed the contract-defined capability set for model selection and session reuse/fork while
    keeping unrelated universal surfaces, including add-dirs, cancel, handle, and final-text
    claims, outside the OpenCode v1 advertisement boundary
  - validation and redaction hardening landed as backend test coverage and harness redaction
    behavior rather than as new canonical spec mutations
- **Downstream stale triggers raised**:
  - wrapper event or completion semantics drift
  - capability advertisement or extension registry drift
  - redaction or bounded-payload posture drift
- **Remediation disposition**:
  - none
- **Promotion blockers**:
  - none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
