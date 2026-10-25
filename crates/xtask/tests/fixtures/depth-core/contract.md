# Isolated Codex CORE profile v1

This is an isolated fixture-based Codex `run`/CORE profile for upstream0.156.1,
one `codex exec --json` headless selector and Linux x86_64-unknown-linux-gnu
wrapper build. The upstream Linux musl snapshot records the executable SHA256
0b2e9301d6100dddda3b9d5c80ebaeaa3a2f1962388f2f36f6b96a9f08b1f33f;
its help/manifest observations bind grammar/version provenance, not a live model run.
Adapter cases use explicitly synthetic subprocesses, not the upstream executable.
No assertion that synthetic output was emitted by upstream is permitted. A reviewer
must independently establish adequacy for the precise promise below before adoption;
unsupported/version-inapplicable assertions remain unverified even if their tests pass.

The promise is the adapter's noninteractive streaming execution contract: public request
and policy mapping, validation before spawning, observed CLI flags/open model values,
redacted validation/spawn/exit/timeout/cancel/parse errors, exact fixture/build provenance,
synthetic credential/environment/home/cwd sentinels, bounded projected diagnostics,
stdin prompt and never-approval behavior, known/unknown/malformed event normalization,
live events before completion, cancellation/drain/drop and bounded stream behavior.
The mapped capability is agent_api.run. No resume/fork/admin/network/model-output or
sandbox isolation promise is made. Wrapper execution is not an OS sandbox.

cases.json binds each BASE/OUT3/LC2/M obligation to exact substantive assertions and
production inputs. The collector runs the union, keeps exact executable identities and
outcomes, and the host derives a separate receipt/mapping/population for each obligation.
Missing/skipped/empty/unadopted/contradictory evidence never verifies. No blanket NA or
new debt applies. No real registry, production evidence, acceptance entry or pointer changes.
