# Seam Map - OpenCode implementation

Primary axis: **crate-first OpenCode landing with deterministic validation and bounded publication
follow-through**. The pack consumes the closed onboarding handoff directly, lands the wrapper crate
plus manifest root first, then lands the `agent_api` backend, and only then addresses the
publication surfaces needed to represent OpenCode as backend support without reopening UAA
promotion.

## Execution horizon (v2.5 policy)

- Active seam: `SEAM-2`
- Next seam: `SEAM-3`
- Future seams: none

Inference note: this split follows the repo-specific execution-horizon guidance in the request. No
stronger repo evidence justified making `agent_api` work active before the wrapper and manifest
foundation exists, and no additional future seam was extracted because generic lifecycle or process
codification is explicitly out of scope.

Only the active seam is eligible for authoritative deep planning by default. The next seam may
later receive seam-local review plus slices, but only after `SEAM-2` publishes its closeout-backed
handoff. No additional future seam remains in this pack after the `SEAM-1` promotion.

## Seams

1. **SEAM-1 - Wrapper crate and manifest foundation**
   - Execution horizon: closed
   - Type: capability
   - Owns: the first concrete OpenCode landing surface across `crates/opencode/`,
     `cli_manifests/opencode/`, the workspace wiring needed to host them, and the deterministic
     validation basis for the manifest root and wrapper.
   - Why it is active: the repo currently has neither `crates/opencode/` nor
     `cli_manifests/opencode/`, so every downstream backend and publication decision depends on
     establishing this implementation foundation first.
   - Expected outputs:
     - concrete wrapper implementation boundary that stays inside the existing OpenCode runtime and
       evidence contracts
     - concrete manifest-root artifact inventory, validator posture, and update rules for
       `cli_manifests/opencode/`
     - deterministic fake-binary, transcript, fixture, and offline-parser validation plan
     - a closeout-backed `THR-05` handoff to backend and publication work

2. **SEAM-2 - `agent_api` OpenCode backend implementation**
   - Execution horizon: active
   - Type: integration
   - Owns: the OpenCode backend inside `crates/agent_api/`, including request mapping, event and
     completion translation, capability advertisement, fail-closed extension handling, redaction,
     bounded payloads, and DR-0012 completion gating coverage.
   - Why it is next: the backend work should consume a landed wrapper plus manifest handoff rather
     than defining wrapper semantics indirectly from `agent_api`.
   - Expected outputs:
     - concrete backend implementation and registration surface for OpenCode
     - explicit capability and backend-specific extension posture for the landed backend
     - deterministic regression coverage for mapping, redaction, bounded payloads, unsupported
       extensions, and completion finality
     - a closeout-backed `THR-06` handoff to publication follow-through

3. **SEAM-3 - Backend support publication and validation follow-through**
   - Execution horizon: next
   - Type: conformance
   - Owns: the bounded publication and validation work needed after code lands so OpenCode can
     appear in manifest/backend support surfaces without implying UAA promotion.
   - Why it is future: the repo has concrete publication work to do, but it depends on actual
     wrapper and backend evidence first. Today the `support-matrix` and `capability-matrix`
     generators still hard-code only Codex and Claude, which makes this a real follow-through seam
     rather than optional cleanup.
   - Expected outputs:
     - OpenCode participation in root-based validation and support publication
     - capability inventory updates that keep backend support, UAA support, and passthrough
       visibility separate
     - an explicit no-promotion posture unless the inherited stale triggers fire

No additional seam was extracted for UAA promotion. That boundary is already governed by the
published onboarding `THR-04` recommendation, and this pack keeps it out of scope unless one of
the inherited stale triggers reopens it.
