# Wrapper-only implementation

Read docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze/input-contract.json. Implement and test the approved wrapper in `crates/depth_fixture`. Preserve the frozen support_depth identities and all acquired files, including generated coverage. No backend or agent_api test is due.

- Allowed source paths: crates/depth_fixture/**
- Update `docs/agents/.uaa-temp/runtime-follow-on/runs/fr2-freeze/handoff.json` so it passes validation with:
  agent_id = depth_fixture; manifest_root = cli_manifests/depth_fixture; runtime_lane_complete = true; publication_refresh_required = true; blockers = [].
- required_commands must match input-contract.json exactly.
- Deliver substantive wrapper Rust source and wrapper tests; controller diagnostics are not implementation evidence.

Depth candidate boundary: handoff.json is the only run file you may write. The input contract, prompt, lifecycle state, depth record, approval and depth-acquisition statement are immutable. The controller writes all streams, execution evidence, validation, status, written paths and summary after observation. Do not write snapshots.
