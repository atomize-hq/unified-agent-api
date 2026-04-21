<!-- generated-by: xtask onboard-agent; owner: control-plane -->

# Handoff

This packet previews the next executable control-plane artifacts for `gemini_cli`.

## Release touchpoints

- Path: Cargo.toml will ensure workspace member `crates/gemini_cli` is enrolled.
- Path: docs/crates-io-release.md will ensure the generated release block includes `unified-agent-api-gemini-cli` on release track `crates-io`.
- Workflow and script files remain unchanged: .github/workflows/publish-crates.yml, scripts/publish_crates.py, scripts/validate_publish_versions.py, scripts/check_publish_readiness.py.

## Manual Runtime Follow-Up

- Create the wrapper crate at `crates/gemini_cli` and keep any file edits runtime-owned.
- Implement backend behavior under `crates/agent_api/src/backends/gemini_cli` and ensure backend-owned capability extensions match the preview.
- Author wrapper coverage input at `crates/gemini_cli` for binding kind `generated_from_wrapper_crate`.
- Populate `cli_manifests/gemini_cli/current.json`, pointers, versions, and reports from committed runtime evidence once the agent exists.
- Re-run `xtask onboard-agent --dry-run` after runtime-owned work changes the proposed artifact set.
