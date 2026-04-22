# Crates.io release guide

Rust library import paths remain the corresponding library crate names declared
by each published package.

All published crates use SPDX metadata `MIT OR Apache-2.0` and package both
`LICENSE-MIT` and `LICENSE-APACHE`.

## Bump the release version

Use the xtask helper to update the release version in one pass:

`cargo run -p xtask -- version-bump <new-version>`

This updates:

- the root `VERSION` file
- `Cargo.toml` `[workspace.package].version`
- exact inter-crate publish pins such as `version = "=X.Y.Z"`

The command fails closed if the resulting manifests are not in sync.

<!-- generated-by: xtask onboard-agent; section: crates-io-release -->
## Published crates

This repository publishes 6 Rust packages for each root `VERSION` bump:

- `unified-agent-api-codex`
- `unified-agent-api-claude-code`
- `unified-agent-api-opencode`
- `unified-agent-api-gemini-cli`
- `unified-agent-api-wrapper-events`
- `unified-agent-api`

## Publish order

Always publish in this order:

1. `unified-agent-api-codex`
2. `unified-agent-api-claude-code`
3. `unified-agent-api-opencode`
4. `unified-agent-api-gemini-cli`
5. `unified-agent-api-wrapper-events`
6. `unified-agent-api`
<!-- /generated-by: xtask onboard-agent; section: crates-io-release -->

The dependent crates (`wrapper-events` and `agent-api`) require the leaf crates
to be visible in the crates.io index before `cargo publish --dry-run` can
succeed for the same version.

## Automated bootstrap

Trusted Publishing still cannot create a brand-new crate on crates.io. The
first publish for a crate must use a normal crates.io API token, after which
Trusted Publishing can handle future releases.

This repository automates that boundary inside
`.github/workflows/publish-crates.yml`:

1. The workflow computes the crates.io-publishable workspace graph from `cargo metadata`.
   Crates restricted to alternate registries (for example `publish = ["internal"]`)
   are excluded from this release plan.
2. For each crate at the target `VERSION`, it checks crates.io:
   - if `crate/version` already exists, the crate is skipped
   - if the crate exists but the version is new, the crate is published with
     Trusted Publishing / OIDC
   - if the crate does not exist at all, the crate is published with the
     protected bootstrap token
3. After each publish, the workflow waits until downstream crates pass
   `cargo publish --dry-run` or the just-published crate version becomes visible
   in the registry.

### Required release secret

The GitHub `release` environment must define:

- `CRATES_IO_TOKEN`: normal crates.io API token used only when the workflow
  encounters a crate that does not yet exist on crates.io

The publish job passes that secret as `BOOTSTRAP_CARGO_REGISTRY_TOKEN`. Existing
crates still publish through `rust-lang/crates-io-auth-action@v1` and the
short-lived OIDC token path.

### Validation and plan preview

Release validation now includes:

1. `make preflight`
2. `python3 scripts/validate_publish_versions.py`
3. `python3 scripts/check_publish_readiness.py`
4. `python3 -m unittest discover -s scripts -p 'test_*.py'`
5. `python3 scripts/publish_crates.py --plan --root . --release-version <VERSION>`

The `--plan` output shows the computed dependency order and one of:

- `skip`
- `publish-with-oidc`
- `publish-with-bootstrap-token`

## Trusted Publishing

Trusted Publishing remains the default publish path for existing crates. Each
published crate should be configured on crates.io to trust this repository and
`.github/workflows/publish-crates.yml`.

When a newly added crate is bootstrapped by the workflow, configure Trusted
Publishing for that crate in crates.io afterward so the next release uses OIDC
automatically.

## Manual fallback

If the release workflow is unavailable, or if a maintainer needs to recover from
an environment secret or ownership issue, the same release can still be
published manually from a maintainer machine:

1. Run `make preflight`.
2. Run `python3 scripts/validate_publish_versions.py`.
3. Run `python3 scripts/check_publish_readiness.py` to verify SPDX license
   metadata and packaged license files.
4. Authenticate locally with `cargo login`.
5. Preview the computed order with:
   - `python3 scripts/publish_crates.py --plan --root . --release-version <VERSION>`
6. Publish crates in that order:
   - for a brand-new crate, use the normal crates.io token path
   - for an existing crate, either use `cargo publish` with a normal token or
     rerun the GitHub workflow once the secret / ownership issue is fixed

Rotate `CRATES_IO_TOKEN` after a bootstrap publish or on the normal secret
rotation schedule.

## Reruns and log expectations

The publish executor is idempotent across partial runs:

- `Skipping <crate>@<version>; already published.` means the exact version
  already exists on crates.io.
- `Publishing <crate>@<version> via publish-with-oidc.` means the crate already
  exists and is using Trusted Publishing.
- `Publishing <crate>@<version> via publish-with-bootstrap-token.` means the
  crate did not exist on crates.io and is being bootstrapped with the protected
  release secret.

References:

- `https://crates.io/docs/trusted-publishing`
- `https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/`

The GitHub Actions workflow in this repository uses `rust-lang/crates-io-auth-action@v1`
to exchange GitHub OIDC credentials for a short-lived crates.io token at
publish time.
