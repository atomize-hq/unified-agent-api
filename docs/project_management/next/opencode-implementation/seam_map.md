# Seam Map - OpenCode implementation

Primary axis: **crate-first OpenCode landing with deterministic validation and bounded publication
follow-through**. The pack consumes the closed onboarding handoff directly, lands the wrapper crate
plus manifest root first, then lands the `agent_api` backend, and only then addresses the
publication surfaces needed to represent OpenCode as backend support without reopening UAA
promotion.

## Execution horizon (v2.5 policy)

- Active seam: none
- Next seam: none
- Future seams: none

Inference note: the forward execution window is now empty. `SEAM-1` through `SEAM-3` have landed,
and no additional future seam was extracted because generic lifecycle or process codification is
explicitly out of scope.

Only the active seam is eligible for authoritative deep planning by default. No additional queued
or future seam remains in this pack after `SEAM-3` closeout.

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
   - Execution horizon: closed
   - Type: integration
   - Owns: the OpenCode backend inside `crates/agent_api/`, including request mapping, event and
     completion translation, capability advertisement, fail-closed extension handling, redaction,
     bounded payloads, and DR-0012 completion gating coverage.
   - Why it was active: the backend work needed to consume the landed wrapper plus manifest
     handoff rather than defining wrapper semantics indirectly from `agent_api`.
   - Expected outputs:
     - concrete backend implementation and registration surface for OpenCode
     - explicit capability and backend-specific extension posture for the landed backend
     - deterministic regression coverage for mapping, redaction, bounded payloads, unsupported
       extensions, and completion finality
     - a closeout-backed `THR-06` handoff to publication follow-through

3. **SEAM-3 - Backend support publication and validation follow-through**
   - Execution horizon: closed
   - Type: conformance
   - Owns: the bounded publication and validation work needed after code lands so OpenCode can
     appear in manifest/backend support surfaces without implying UAA promotion.
   - Why it was active: `SEAM-2` published `THR-06`, no queued seam remained behind it in this
     pack, and the remaining forward work was the bounded publication follow-through this seam
     owned.
   - Expected outputs:
     - OpenCode participation in root-based validation and support publication
     - capability inventory updates that keep backend support, UAA support, and passthrough
       visibility separate
     - an explicit no-promotion posture unless the inherited stale triggers fire

No additional seam was extracted for UAA promotion. That boundary is already governed by the
published onboarding `THR-04` recommendation, and this pack keeps it out of scope unless one of
the inherited stale triggers reopens it.
