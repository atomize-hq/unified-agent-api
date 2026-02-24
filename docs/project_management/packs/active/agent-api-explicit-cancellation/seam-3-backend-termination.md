# SEAM-3 — Backend termination responsibilities (CA-C03)

This seam pins what built-in backends must provide to support explicit cancellation.

## Requirements

- Built-in backends MUST support best-effort termination of the spawned CLI process.
- Termination MUST preserve the redaction/safety posture:
  - cancellation errors must not leak raw backend output.

## Backend notes (informative)

- Codex wrapper uses `kill_on_drop(true)` on spawned commands. One implementation approach for
  best-effort termination is to ensure the child handle is dropped when cancellation is requested.
- Claude Code wrapper similarly kills on drop for spawned commands in its process runner.
