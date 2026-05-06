<!-- generated-by: xtask onboard-agent; owner: control-plane -->

# Scope brief

This packet covers the control-plane-owned onboarding surfaces for `aider`.

- Registry enrollment in `crates/xtask/data/agent_registry.toml`
- Docs pack in `docs/agents/lifecycle/aider-onboarding`
- Manifest root in `cli_manifests/aider`
- Release/workspace touchpoints in `Cargo.toml` and `docs/crates-io-release.md`
- Approval linkage via `docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml` (`sha256: bc02f5f9b7fe7880446c4cb04b33a9f7c419008755b2229a8243ba878441a854`)

Current proving-run target: complete the runtime-owned wrapper/backend lane, commit manifest evidence, regenerate publication artifacts, and close with `make preflight`.
