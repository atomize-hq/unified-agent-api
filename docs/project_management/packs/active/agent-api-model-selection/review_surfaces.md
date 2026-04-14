# Review Surfaces - Universal model selection (`agent_api.config.model.v1`)

These diagrams orient the pack. They show the expected product/work shape that is intended to land.
They do not, by themselves, satisfy seam-local pre-exec review.

## R1 - High-level workflow

```mermaid
flowchart LR
  Caller["Caller (orchestrator)"] --> Req["AgentWrapperRunRequest.extensions"]
  Req --> Gate["R0 capability gate + normalize_request()"]
  Gate -->|"Ok(None)"| NoOverride["No model override"]
  Gate -->|"Ok(Some(trimmed_id))"| Override["Model override requested"]
  Gate -->|"Err(InvalidRequest / UnsupportedCapability)"| FailPre["Pre-spawn failure (safe)"]

  NoOverride --> Backend["Backend run flow"]
  Override --> Backend
  Backend -->|"Success"| Ok["Completion + events"]
  Backend -->|"Runtime model rejection"| FailRun["Backend failure (safe)"]
```

## R2 - Validation and error surfaces

```mermaid
flowchart TB
  A["R0 gate (unsupported capability)"] -->|"fails before parsing"| U["UnsupportedCapability"]
  B["Pre-spawn validation (string/trim/bounds)"] -->|"invalid agent_api.config.model.v1"| I["InvalidRequest (safe template)"]
  C["Runtime backend rejection"] -->|"safe Backend message"| R["Backend error"]
  R -->|"when stream already open"| E["One terminal Error event (same safe message)"]
```

## R3 - Touch surface map (repo)

```mermaid
flowchart TB
  Req["AgentWrapperRunRequest"] --> Norm["crates/agent_api/src/backend_harness/normalize.rs"]
  Norm --> Cap["Backend capability sets (codex / claude_code)"]
  Norm --> CodexMap["Codex mapping (exec/resume/fork)"]
  Norm --> ClaudeMap["Claude mapping (print/session argv)"]
  CodexMap --> CodexBuilder["crates/codex (builder/argv)"]
  ClaudeMap --> ClaudeReq["crates/claude_code (print request/argv)"]
  Cap --> Matrix["docs/specs/.../capability-matrix.md regeneration"]
  CodexMap --> Tests["crates/agent_api/.../tests"]
  ClaudeMap --> Tests
  Norm --> Tests
```

