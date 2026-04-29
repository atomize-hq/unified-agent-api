<!-- generated-by: xtask onboard-agent; owner: control-plane -->

# Threading

1. Apply the control-plane onboarding packet with `onboard-agent --write`.
2. Run `cargo run -p xtask -- scaffold-wrapper-crate --agent aider --write` to create the runtime-owned wrapper crate shell at `crates/aider`; `onboard-agent` does not create the wrapper crate.
3. Implement backend/runtime details in `crates/aider` and `crates/agent_api/src/backends/aider`.
4. Populate manifest evidence under `cli_manifests/aider` from committed runtime outputs, regenerate support/capability publication artifacts, and close the proving run with `make preflight`.
