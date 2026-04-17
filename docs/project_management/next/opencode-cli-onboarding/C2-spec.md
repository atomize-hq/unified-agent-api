# C2 Spec - `crates/agent_api/` Backend Planning Bounded By The Wrapper Contract

Source docs:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/capabilities-schema-spec.md`
- `docs/specs/unified-agent-api/extensions-spec.md`
- `docs/project_management/next/opencode-cli-onboarding/C1-spec.md`

## Decisions (no ambiguity)

- C2 starts only after `C1-integ`.
- C2 is planning-only. It must not edit `crates/agent_api/` or canonical specs under `docs/specs/`.
- Planned backend identity: `opencode`.
- C2 must be bounded by the locked wrapper contract from C1. It may not invent new wrapper
  semantics without feeding that change back to C1.
- C2 must define:
  - run request mapping from the universal facade into the wrapper
  - event bucket mapping into the universal envelope
  - completion, redaction, and bounded-payload obligations
  - capability ids and backend-specific extension keys
  - fixture-backed test strategy with no real-provider requirement in default gating

## Task Breakdown (no ambiguity)

- `C2-code`:
  - define the OpenCode backend adapter plan for `crates/agent_api/`
- `C2-test`:
  - define the future adapter test matrix, fixture sources, and safety assertions
- `C2-integ`:
  - reconcile backend scope and validation scope into one execution-ready `agent_api` planning
    packet

## Scope

- mapping the wrapper contract into `AgentWrapperRunRequest`, `AgentWrapperEvent`, and
  `AgentWrapperCompletion`
- capability advertisement and backend-specific extension strategy
- redaction and completion-finality obligations from the charter
- fixture-only default validation strategy for backend planning

## Acceptance Criteria

- the pack states the planned `agent_api` backend identity and bounded event/run/completion mapping
- the pack explicitly lists capabilities that are available immediately versus those that remain
  backend-specific or deferred
- the pack defines test obligations for redaction, completion finality, capability gating, and
  extension handling
- C3 receives explicit promotion-review inputs without reopening wrapper scope

## Out of Scope

- editing `crates/agent_api/`
- editing `docs/specs/unified-agent-api/**`
- promoting capabilities into the UAA surface
- changing wrapper scope without reopening C1
