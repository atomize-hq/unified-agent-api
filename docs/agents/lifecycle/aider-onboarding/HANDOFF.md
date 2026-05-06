<!-- generated-by: xtask onboard-agent; owner: control-plane -->

# Handoff

This packet captures the next executable onboarding step for `aider`.

## Release touchpoints

- Path: Cargo.toml will ensure workspace member `crates/aider` is enrolled.
- Path: docs/crates-io-release.md will ensure the generated release block includes `unified-agent-api-aider` on release track `crates-io`.
- Workflow and script files remain unchanged: .github/workflows/publish-crates.yml, scripts/publish_crates.py, scripts/validate_publish_versions.py, scripts/check_publish_readiness.py.

## Approval provenance

- approval ref: `docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml`
- approval artifact sha256: `bc02f5f9b7fe7880446c4cb04b33a9f7c419008755b2229a8243ba878441a854`
- closeout metadata will become authoritative at `docs/agents/lifecycle/aider-onboarding/governance/proving-run-closeout.json`.


## Next executable runtime step

Run `cargo run -p xtask -- scaffold-wrapper-crate --agent aider --write` to create the runtime-owned wrapper crate shell at `crates/aider`. `onboard-agent` does not create the wrapper crate.

## Remaining runtime checklist

- After scaffolding, materialize the bounded runtime packet with `runtime-follow-on --dry-run` for this approval artifact.
- Implement backend/runtime details in `crates/aider` and `crates/agent_api/src/backends/aider`.
- Author wrapper coverage input at `crates/aider` for binding kind `generated_from_wrapper_crate`.
- Populate committed runtime evidence only under `cli_manifests/aider/snapshots/**` and `cli_manifests/aider/supplement/**`.
- Complete `runtime-follow-on --write`; publication refresh and proving-run closeout stay in the next lane.
