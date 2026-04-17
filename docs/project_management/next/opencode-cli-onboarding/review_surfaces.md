# Review Surfaces - OpenCode CLI onboarding

These diagrams orient the pack. They show the expected product/work shape that is intended to
land. They do not, by themselves, satisfy seam-local pre-exec review.

Active and next seams still require seam-local `review.md` artifacts later.

## R1 - High-level onboarding workflow

```mermaid
flowchart LR
  Host["Caller / orchestrator"] --> Req["AgentWrapperRunRequest"]
  Req --> Backend["OpenCode backend in crates/agent_api"]
  Backend --> Wrapper["crates/opencode wrapper"]
  Wrapper --> Run["opencode run --format json"]
  Run --> Stream["typed events + completion handoff"]
  Stream --> Host
  Wrapper --> Manifest["cli_manifests/opencode evidence root"]
  Backend --> Review["backend-specific support or UAA promotion review"]
```

## R2 - Contract and dependency flow

```mermaid
flowchart TB
  Packet["source packet + charter"] --> S1["SEAM-1 runtime/evidence contract"]
  S1 --> S2["SEAM-2 wrapper + manifest contract"]
  S2 --> S3["SEAM-3 agent_api backend mapping"]
  S3 --> S4["SEAM-4 promotion review"]
  S2 --> Specs["docs/specs/opencode-* contracts"]
  S3 --> Uaa["docs/specs/unified-agent-api/**"]
  S4 --> FollowOn["follow-on pack only if promotion/spec work is justified"]
```

## R3 - Touch surface map (repo)

```mermaid
flowchart TB
  Plan["docs/project_management/next/opencode-cli-onboarding/*"] --> Wrapper["crates/opencode/**"]
  Plan --> Manifest["cli_manifests/opencode/**"]
  Wrapper --> Backend["crates/agent_api/**"]
  Manifest --> Backend
  Backend --> Matrix["docs/specs/unified-agent-api/capability-matrix.md"]
  Backend --> Ext["docs/specs/unified-agent-api/extensions-spec.md"]
  Wrapper --> Contract["future docs/specs/opencode-wrapper-run-contract.md"]
  Manifest --> ManifestContract["future docs/specs/opencode-cli-manifest-contract.md"]
```
