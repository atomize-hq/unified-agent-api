# Crates.io release guide

This repository publishes five Rust packages for each root `VERSION` bump:

- `unified-agent-api-codex`
- `unified-agent-api-claude-code`
- `unified-agent-api-opencode`
- `unified-agent-api-wrapper-events`
- `unified-agent-api`

Rust library import paths remain `codex`, `claude_code`, `opencode`,
`wrapper_events`, and `agent_api`.

## Bump the release version

Use the xtask helper to update the release version in one pass:

`cargo run -p xtask -- version-bump <new-version>`

This updates:

- the root `VERSION` file
- `Cargo.toml` `[workspace.package].version`
- exact inter-crate publish pins such as `version = "=X.Y.Z"`

The command fails closed if the resulting manifests are not in sync.

## Publish order

Always publish in this order:

1. `unified-agent-api-codex`
2. `unified-agent-api-claude-code`
3. `unified-agent-api-opencode`
4. `unified-agent-api-wrapper-events`
5. `unified-agent-api`

The dependent crates (`wrapper-events` and `agent-api`) require the three leaf
crates to be visible in the crates.io index before `cargo publish --dry-run`
can succeed for the same version.

## First release bootstrap

The first real release for the renamed package set must be done manually from a
maintainer machine with a short-lived crates.io token.

1. Run `make preflight`.
2. Run `python3 scripts/validate_publish_versions.py`.
3. Run `python3 scripts/check_publish_readiness.py`.
4. Authenticate locally with `cargo login`.
5. Publish the three leaf crates:
   - `cargo publish --locked -p unified-agent-api-codex`
   - `cargo publish --locked -p unified-agent-api-claude-code`
   - `cargo publish --locked -p unified-agent-api-opencode`
6. Wait until dependent dry-runs succeed:
   - `cargo publish --dry-run --locked -p unified-agent-api-wrapper-events`
   - `cargo publish --dry-run --locked -p unified-agent-api`
7. Publish the dependent crates:
   - `cargo publish --locked -p unified-agent-api-wrapper-events`
   - `cargo publish --locked -p unified-agent-api`

## Trusted Publishing

After the first release exists on crates.io, configure Trusted Publishing for
all five packages and point them at this repository and
`.github/workflows/publish-crates.yml`.

References:

- `https://crates.io/docs/trusted-publishing`
- `https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/`

The GitHub Actions workflow in this repository uses `rust-lang/crates-io-auth-action@v1`
to exchange GitHub OIDC credentials for a short-lived crates.io token at
publish time.
