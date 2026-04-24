# Contributing

Use `docs/cli-agent-onboarding-factory-operator-guide.md` as the procedural hub for the shipped factory workflow. This file is only the contributor entrypoint for repo basics, verification, and hygiene.

## Contribution entrypoints

- Operator procedure hub: `docs/cli-agent-onboarding-factory-operator-guide.md`
- Documentation index: `docs/README.md`
- Normative contract index: `docs/specs/unified-agent-api/README.md`
- Onboarding charter: `docs/project_management/next/cli-agent-onboarding-charter.md`

The operator guide owns the create-mode onboarding flow, maintenance-mode refresh flow, and command ordering. The charter remains normative rather than a duplicate how-to guide.

## Green gate

The repo green gate is:

```sh
cargo run -p xtask -- support-matrix --check
cargo run -p xtask -- capability-matrix --check
cargo run -p xtask -- capability-matrix-audit
make preflight
```

Run targeted commands while iterating as needed, but use the same green gate above for the full repo pass.

## Repo hygiene

- Do not commit generated or scratch artifacts such as `target/`, `wt/`, `_download/`, `_extract/`, repo-root `*.log`, or `cli_manifests/codex/raw_help/`.
- Prefer the workspace helpers in `crates/xtask/` and the `make` targets already wired into the repo.
- If behavior or format changes, update the relevant canonical contract under `docs/specs/**`.
