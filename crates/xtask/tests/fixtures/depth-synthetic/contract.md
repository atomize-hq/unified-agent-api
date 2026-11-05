# Isolated synthetic wrapper profile v1 (`synthetic-wrapper-v1`)

This is a machinery proof on the synthetic `depth_fixture` subject. It qualifies no real agent,
and its results are never reused for Codex or any other agent.

The subject is upstream `depth_fixture` 0.2.0 on target `x86_64-unknown-linux-gnu`: one `depth_fixture run --json`
headless selector and one wrapper-only `run` operation with template CORE. No capability is mapped,
so M is not due and no `agent_api` source is in scope. The fixture CLI and its wrapper both live in
`crates/depth_fixture`. The cases run the fixture binary built from reviewed source with
`DEPTH_FIXTURE_UPSTREAM_VERSION=0.2.0` on that same tuple, so evidence is never cross-target.
In the isolated repository the digest of the acquired 0.2.0 release asset is provenance.

The promise is the wrapper's streaming execution contract as its crate documentation states it:
- the public request and its argv, with the prompt after `--`;
- validation before spawn;
- wrapper-accepted model values;
- distinct validation, spawn, exit, timeout, cancel, malformed-output and missing-terminal errors;
- the enrolled version;
- a cleared environment with only explicit variables, the caller's working directory, null and
  non-terminal stdin, and no writes to home or the working directory;
- no credential passed implicitly, and no parent-environment sentinel in any output;
- bounded line and diagnostic projection;
- known, unknown and malformed event normalization, with session attribution;
- events before completion, a terminal event plus a zero exit for success, bounded buffering with
  backpressure, a deadline on every poll, and kill-and-reap on timeout, cancel and drop.

The wrapper is not a sandbox, and no network or OS-isolation promise is made.

`cases.json` binds each BASE, OUT3 and LC2 obligation to exact substantive assertions and production
inputs. The collector runs the union of cases and keeps the exact executable identities and
outcomes. The host derives a separate receipt, mapping and population for each obligation.
Missing, skipped, empty, unadopted or contradictory evidence never verifies. No blanket NA and no
new debt apply. No real registry, production evidence, acceptance entry or pointer changes.
