# SEAM-4 — Tests

This seam pins the test coverage required to prevent cancellation drift.

## Required tests (v1)

- Harness-level integration test using a fake backend process that blocks until killed:
  - calling `cancel()` causes the fake process to be terminated best-effort
  - completion resolves to `AgentWrapperError::Backend { message: "cancelled" }`
  - no raw backend output leaks into events/errors
- Regression test: drop events receiver without calling cancel:
  - draining continues and completion gating semantics remain correct (no deadlocks)

