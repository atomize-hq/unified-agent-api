# Documentation Index

## Operator procedure

- Canonical factory workflow: `docs/cli-agent-onboarding-factory-operator-guide.md`

Use that guide for create-mode onboarding, maintenance-mode refresh, artifact ownership boundaries, and command sequencing. This index stays as a docs map, not a competing procedure manual.

## Normative docs

- Unified Agent API contract index: `docs/specs/unified-agent-api/README.md`
- Normative contracts: `docs/specs/`
- Architecture decisions: `docs/adr/`
- Onboarding charter: `docs/specs/cli-agent-onboarding-charter.md`

## Repo entry docs

- Repo overview: `README.md`
- Contributor entrypoint: `CONTRIBUTING.md`

## Green gate

The repo green gate is:

```sh
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

The operator guide is the procedural source of truth for how that gate fits into the factory lifecycle.
