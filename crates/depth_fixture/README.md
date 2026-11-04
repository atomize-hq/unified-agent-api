# depth_fixture

The synthetic subject of the isolated support-depth machinery proof
([SA-P1](../../docs/agents/lifecycle/support-depth-synthetic-subject-design.md)). It is not
published and the canonical agent registry never lists it.

- `src/bin/depth_fixture.rs` is the fake upstream CLI. Its whole API is in this file, so a proof
  run on it "certifies no nonexistent API". The upstream version it reports comes from
  `DEPTH_FIXTURE_UPSTREAM_VERSION` at build time (default `0.1.0`).
- `src/lib.rs` is the wrapper: one streaming `run` operation, wrapper-only, with no `agent_api`
  backend and no capability claim.

Passing tests here prove the wrapper against this fixture only. They qualify no real agent.
