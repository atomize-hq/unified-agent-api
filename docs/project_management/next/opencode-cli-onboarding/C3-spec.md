# C3 Spec - UAA Promotion Review

Source docs:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/extensions-spec.md`
- `docs/project_management/next/opencode-cli-onboarding/C2-spec.md`

## Decisions (no ambiguity)

- C3 starts only after `C2-integ`.
- C3 is review-only and planning-only. It must not edit canonical specs, capability matrices, or
  runtime code in this pack.
- C3 owns one question: what OpenCode support remains backend-specific, and what is justified for
  UAA promotion after C1/C2 have made the actual backend scope concrete?
- C3 must distinguish:
  - backend support
  - backend-specific extension coverage (`backend.opencode.*`)
  - candidate `agent_api.*` promotions
- Any follow-on work that changes canonical specs, capability matrices, or implementation code must
  be deferred to a separate execution pack.

## Task Breakdown (no ambiguity)

- `C3-code`:
  - draft the promotion review using the concrete wrapper/backend planning artifacts from C1/C2
- `C3-test`:
  - define the evidence and validation gates required before any capability is promoted or kept
    backend-specific
- `C3-integ`:
  - reconcile the review and evidence obligations into the final pre-implementation recommendation

## Scope

- promotion candidacy of OpenCode capabilities at the UAA layer
- intentional non-promotion of backend-specific or unstable surfaces
- required follow-on audits, matrix updates, or execution packs

## Acceptance Criteria

- the pack ends with an explicit backend-support versus UAA-promotion recommendation
- backend-specific fallback paths remain explicit
- any required follow-on pack for canonical spec or matrix changes is named directly
- the result remains confined to this planning directory

## Out of Scope

- editing `docs/specs/unified-agent-api/**`
- regenerating or editing capability matrices
- modifying crate code or manifest artifacts
- reopening C0-C2 scope except via explicit blocker escalation
