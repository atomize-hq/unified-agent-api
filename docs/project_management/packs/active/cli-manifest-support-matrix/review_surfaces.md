# Review Surfaces - CLI manifest support matrix

These diagrams orient the pack. They show the actual artifact and data flow expected to land for the feature.
They do not, by themselves, satisfy seam-local pre-exec review.
Active and next seams still require seam-local `review.md` later.

## R1 - Support publication workflow

```mermaid
flowchart LR
  A["Upstream release pins"] --> B["cli_manifests/<agent>/artifacts.lock.json"]
  B --> C["snapshots/<version>/<target>.json and union.json"]
  C --> D["wrapper_coverage.json, versions/<version>.json, current.json, and latest_* pointers"]
  D --> E["xtask support-matrix derives target rows"]
  E --> F["cli_manifests/support_matrix/current.json"]
  E --> G["docs/specs/unified-agent-api/support-matrix.md"]
```

## R2 - Evidence to validation flow

```mermaid
flowchart LR
  C1["Codex manifest root"] --> X["shared root intake + normalization"]
  C2["Claude Code manifest root"] --> X
  X --> Y["derived support row model"]
  Y --> J["JSON renderer"]
  Y --> M["Markdown renderer"]
  Y --> V["consistency validator"]
  J --> O1["published current.json artifact"]
  M --> O2["published support-matrix.md projection"]
  V --> O3["deterministic failure on contradictions"]
```

## R3 - Touch surface map

```mermaid
flowchart TB
  D["docs/specs/unified-agent-api/README.md and support-matrix.md"] --> XT["crates/xtask/src/main.rs and support_matrix.rs"]
  CM["cli_manifests/codex/**"] --> XT
  CC["cli_manifests/claude_code/**"] --> XT
  XT --> SM["cli_manifests/support_matrix/current.json"]
  XT --> TV["crates/xtask/tests/*.rs and validator checks"]
```
