# Packet-bound native producer

This implements the foreground producer portion of
[the approved native evidence design](support-depth-native-evidence-design.md).
The [v2 contract](../../specs/support-depth-native-evidence-v2.md#foreground-producer-implementation)
governs exact behavior. It is intentionally dormant until a reviewed isolated profile is installed.
Ordinary [native diagnostics](support-depth-native-results.md) remain usable with the caller's login.

After the remaining context, case/mapping and host prerequisites are reviewed and installed, one
explicit invocation has this form (paths are placeholders, not credential fixtures):

```sh
cargo run --locked -p xtask -- depth-native-collect \
  --live --register --binary /absolute/path/to/pinned/codex \
  --working-dir /absolute/path/to/disposable-project --attempt run-01
```

The command reserves its attempt under
`cli_manifests/codex/reports/0.156.1/evidence/native-v1/`. Create the report parent first. Never
reuse an attempt leaf or blindly repeat a registration that timed out: inspect the packet PR first.
The plan is retained before one start registration; native version probes/calls follow only after
the registration is independently confirmed. The native home may be forwarded with `--codex-home`;
no login is read/copied, and no API-key fixture is required. Native model/provider configuration
remains caller-owned. The build has a separate cleared environment.

Failures retain the whole requested case population and every valid completed outcome. Unknown
context stays unknown. A registered interrupted attempt remains visible and unverified; do not
delete it, rerun until green or substitute an earlier successful smoke.

Commit/deliver retained proof through the existing packet process, then prepare the completion:

```sh
cargo run --locked -p xtask -- depth-native-prepare-finish \
  --attempt run-01 --evidence-commit <full-immutable-proof-commit>
```

This only prints a template with human attestations false. Review the exact execution/completeness
statement and independent adequacy adoption before authorizing any posting. Neither command posts
a finish or automatically edits E, closes a packet, promotes a version or activates depth.

Required CI uses fake hosts and finite fake native children. It requires no login or model calls.
The next integration work must close effective context and reviewed case coverage, connect native
verdicts to the existing record-sealed consumer path, then prove the isolated maintenance chain.
