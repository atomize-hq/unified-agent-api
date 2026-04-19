# Review Surfaces - OpenCode implementation

These diagrams orient the pack. They show the actual product and repository work shape expected to
land. They do not, by themselves, satisfy seam-local pre-exec review.

Active and next seams still require seam-local `review.md` artifacts later.

## R1 - End-to-end OpenCode run workflow

```mermaid
flowchart LR
  Caller["Caller / orchestrator"] --> Req["AgentWrapperRunRequest"]
  Req --> Backend["OpenCode backend in crates/agent_api"]
  Backend --> Wrapper["crates/opencode wrapper"]
  Wrapper --> Run["opencode run --format json"]
  Run --> Events["typed events + completion handoff"]
  Events --> Caller
  Wrapper --> Evidence["fixtures, fake-binary, transcript evidence"]
```

## R2 - Repo touch surface and validation flow

```mermaid
flowchart TB
  Pack["docs/project_management/next/opencode-implementation/*"] --> Wrapper["crates/opencode/**"]
  Pack --> Manifest["cli_manifests/opencode/**"]
  Wrapper --> Backend["crates/agent_api/**"]
  Manifest --> Validator["xtask codex-validate --root cli_manifests/opencode"]
  Backend --> Support["xtask support-matrix --check"]
  Backend --> CapMatrix["xtask capability-matrix"]
  Wrapper --> Specs["docs/specs/opencode-*.md"]
  Support --> MatrixSpec["docs/specs/unified-agent-api/support-matrix.md"]
```

## R3 - Support-layer publication boundary

```mermaid
flowchart LR
  ManifestRoot["manifest support<br/>cli_manifests/opencode/**"] --> BackendSupport["backend support<br/>opencode wrapper + agent_api backend"]
  BackendSupport --> Visibility["passthrough visibility<br/>backend-only surface stays explicit"]
  BackendSupport -. stale trigger only .-> Uaa["UAA unified support"]
  Matrix["support-matrix rows"] --> ManifestRoot
  Matrix --> BackendSupport
  Matrix --> Uaa
```

## R4 - Deterministic versus live evidence boundary

```mermaid
flowchart LR
  Fixtures["fixtures / fake binary / offline parser"] --> WrapperTests["wrapper + backend tests"]
  WrapperTests --> Gates["default completion gates"]
  LiveSmoke["provider-backed opencode smoke"] -. basis lock or stale-trigger revalidation only .-> Gates
```
