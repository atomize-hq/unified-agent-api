# Isolated governing-data templates for the synthetic subject

These files are the governing data for `depth_fixture`, the synthetic support-depth subject
(SA-P1). The subject's code lives in this repository, but its governing data is installed only
in the disposable isolated repository's protected baseline. Nothing here is installed in the
canonical repository, and the canonical registry never lists the agent.

| Template | Installed at |
| --- | --- |
| `registry-entry.toml` | appended to `crates/xtask/data/agent_registry.toml` |
| `manifest-root/*` (including `VALIDATOR_SPEC.md`, which adopts Annex B) | `cli_manifests/depth_fixture/` |

Before installing, replace `{{ISOLATED_OWNER}}` and `{{ISOLATED_REPO}}` with the isolated
repository, which is also the fixture CLI's upstream. That repository must be separately
authorized (roadmap C).

The other manifest-root files come from the real commands, as `parity-acquire.yml` runs them:
- `depth-fixture-snapshot` writes the baseline snapshot;
- `manifest-union`, `depth-fixture-wrapper-coverage`, `manifest-report` and
  `manifest-version-metadata` write the union, coverage, reports and version metadata;
- `manifest-promote` writes the pointers.

`crates/xtask/tests/depth_synthetic_governing.rs` runs that sequence over these templates with
the real fixture CLI. It also checks that the declared `support_depth` table equals the
selection in the reviewed `../record.json`.

## Upstream releases

Release validation runs `crates/depth_fixture/tests/acquired_binary.rs` with
`DEPTH_FIXTURE_BINARY` set to the downloaded executable, so the wrapper is exercised against the
acquired release, not the locally built CLI.

Release-watch for `github_releases` supports only `latest_stable_minus_one`. Watch therefore
targets the enrolled `0.2.0` only after a newer stable release exists. The isolated repository
publishes `v0.1.0` (the baseline), then `v0.2.0` and `v0.3.0`. Each release carries
`depth_fixture-x86_64-unknown-linux-gnu.tar.gz`, built with `DEPTH_FIXTURE_UPSTREAM_VERSION` set to
that version.

The onboarding proof uses its own registry-entry variant, approval and acquisition statement
templates, in `../onboarding/`.

The registry entry uses the `unpublished` release track, so release docs and release-doc drift
checks never list the fixture crate.

The production registry still refuses `support_depth` tables (roadmap B.8). Until that lands,
commands that load the production registry need the entry without its `support_depth` table.
