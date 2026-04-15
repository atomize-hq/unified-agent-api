# Seam Map - CLI manifest support matrix

Primary axis: **integration-first publication pipeline**. The feature turns existing manifest evidence into deterministic support truth without changing runtime `agent_api` behavior.

## Execution horizon (inferred)

- Active seam: `SEAM-1`
- Next seam: `SEAM-2`
- Future seams: `SEAM-3`, `SEAM-4`, `SEAM-5`

Why this horizon:

- `SEAM-1` must pin semantics and authority before any generator code can safely freeze output shapes.
- `SEAM-2` must extract the neutral shared normalization seam before publication and validation work can reuse it.
- `SEAM-3` through `SEAM-5` stay future until the semantic model and shared normalization boundary are fixed.

## Seams

1. **SEAM-1 - Support semantics and publication contract**
   - Execution horizon: active
   - Type: integration
   - Owns: target-scoped support semantics, naming cleanup, canonical publication locations, and the neutral `xtask support-matrix` entrypoint contract.
   - Verification path: docs/spec alignment plus a stable command contract in `crates/xtask/src/main.rs`.

2. **SEAM-2 - Shared wrapper normalization and agent-root intake**
   - Execution horizon: next
   - Type: integration
   - Owns: the reusable normalization seam extracted from existing wrapper-coverage code plus neutral loading of manifest/version/pointer/report inputs from each agent root.
   - Verification path: shared-module unit coverage and thin-adapter review against current Codex and Claude inputs.

3. **SEAM-3 - Support-matrix derivation and publication**
   - Execution horizon: future
   - Type: capability
   - Owns: single-pass row derivation, deterministic JSON rendering, and Markdown projection from the same model.
   - Verification path: golden outputs and contradiction handling against checked-in fixture roots.

4. **SEAM-4 - Consistency validation and repo-gate enforcement**
   - Execution horizon: future
   - Type: conformance
   - Owns: generator-level contradiction checks, pointer/status consistency rules, Markdown staleness detection, and repo-gate integration decisions.
   - Verification path: validator tests plus deterministic failure behavior for mismatched manifest inputs.

5. **SEAM-5 - Neutral fixture and regression conformance**
   - Execution horizon: future
   - Type: conformance
   - Owns: Codex, Claude, and synthetic third-agent-shaped fixture coverage so the neutral seam stays neutral over time.
   - Verification path: fixture suites and golden/regression coverage over row ordering, renderer output, and future-agent-shaped inputs.
