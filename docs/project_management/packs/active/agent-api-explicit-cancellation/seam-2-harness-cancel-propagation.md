# SEAM-2 — Harness cancellation propagation (CA-C02)

This seam pins how the backend harness observes cancellation without regressing drain-on-drop.

## Requirements

- The harness must continue draining backend event streams even if the consumer drops the universal
  receiver (BH-C04 posture).
- Explicit cancellation must be orthogonal:
  - it must not depend on receiver drop, and
  - it must not depend on consumer-side timeout wrappers.

## Driver model (pinned)

The harness run driver is split into:

- Pump/drainer task: forwards events while receiver is alive; always drains to stream end.
- Completion sender task: awaits backend completion; sends completion outcome independently.

Explicit cancellation introduces a third shared signal:

- A cancellation signal is observed by both tasks.
- On cancellation:
  - the pump/drainer stops forwarding but MUST still drain as required,
  - the backend process is requested to terminate (best-effort), and
  - the completion sender resolves completion to the pinned cancellation error if the backend does
    not complete first.
