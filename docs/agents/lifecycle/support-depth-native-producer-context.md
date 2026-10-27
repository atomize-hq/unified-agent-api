# Native producer: context guards and real adapter child observations

Baseline: staging `80c3555ebdf70e0218fe5c33008d98a294e013e9` (PR #302).
This is the first native-producer increment: a fixed offline context and observations
of actual production adapter spawns. The session-fixture extension is described below. It creates no packet P/O/E, classification
assignment, writer profile, qualification or production enrollment.

## What runs

`scripts/observe_codex_adapter_children.py` builds the source-owned
`xtask` example `codex_native_context` with Cargo's locked dependency graph. The driver
uses `CodexBackend::run` and the existing typed pilot route decoder. It runs all five
selectors with unattended, interactive and external variants: 15 separate fresh private
contexts. Initial/resume use exec or app-server as the real adapter selects; fork always
uses app-server. Unknown selectors/variants refuse. No fake CLI or native-binary shim
stands in for Codex, and no observer hook is added to ordinary SDK behavior.

Linux syscall tracing records successful and failed execve calls, complete argv/env,
cwd changes and clone/fork ownership. A native child must be the direct child of the
built driver; an unrelated or nested process cannot establish that observation. The
trace parser rejects missing spawned-process files, malformed or truncated known syscall
records, ambiguous ownership and oversized inputs. Unknown/detached syscall records are
retained with file, PID, line and original text; they set `incomplete_trace` and remove
complete child-boundary credit for that case. Trace hashes and parsed records are retained; `--raw-traces` optionally preserves the complete bounded
raw inputs in a private directory for inspection. Tracing changes timing and is not
timing/performance or successful-execution evidence.

## Context guards

Before launch, the producer requires a clean committed source tree, the pinned musl
binary, the complete pinned protocol/config report and the reviewed PR #302 baseline
snapshot. It binds source commit/tree and binary, driver, tracer, launcher, GNU runtime
library and collector/helper hashes; drift before packaging refuses the audit. Local
build correspondence is recorded, not asserted as A1 or a trusted build attestation.

Each case starts with only exact source-owned config and minimal private Git metadata;
unexpected files, directories, symlinks or loader/config input refuse. Bubblewrap creates
an empty root and private pid/user/network namespaces. It mounts the fixed Codex binary,
locally built driver, tracer and their measured read-only runtime libraries. Host home,
project/ancestor settings and `/etc/codex` are absent. The producer environment is cleared;
there are no credentials or external network. CODEX_BINARY/RUST_LOG are deliberately
absent in the driver environment, leaving the real wrapper to generate them.

After the actual spawn, the producer checks the complete observed executable, environment,
cwd and exact finite invocation, including native safety flags, resume selection and private
last-message output grammar. The recorded parent environment must match the cleared context;
the native child must contain the wrapper's canonical CODEX_BINARY/CODEX_HOME/RUST_LOG values.
An extra option, foreign environment key/home/log/binary, output escape or route mismatch
refuses. This postcondition detects SDK drift **after private execution**; it is not a
pre-spawn security boundary for ordinary callers or maintenance-writer admission.

The ordinary boundary audit has a three-second owned timeout; its outer namespace has a 20-second
deadline and is killed/reaped on overrun. Startup/session/output mutations are inventoried
with file digests, modes, directory types and symlink targets. Observer trace files are
recorded separately and excluded from native mutation attribution. Changed controlled
config outside the exact approved native addition below is retained as an explicit context
refusal alongside the observed spawn. The post-run config bytes are recorded for review;
a passing config postcondition is not a complete context or execution verdict.

The existing approved context decisions remain in force: preserve native feature values
(including six reported true despite false config), and inventory pinned native startup
assets/state while prohibiting external user/host skills, hooks and plugins. The earlier
config-only baseline supplies reviewed acquisition facts, not the effective internal
configuration of each new adapter run. tool_registry and other unexposed defaults remain
unproved. No assignment is inferred from a source hash or plausible configuration.

## Execution and proof limits

These are actual offline adapter invocations. Initial or resumed paths may attempt native
thread/turn work, but the namespace has no credentials or external network. Successful
model execution, approval execution, session continuation/fork and live cost are not claimed.
Native errors, empty-session failures and timeouts are preserved as outcomes. None becomes a
verified M/BASE/template result; `execution_verified` and `context_complete` remain false.

Native descendants can attempt OS-discovery programs such as getconf/lsb_release. The
minimal namespace does not supply them. Their failed exec attempts, native PATH changes
and errors remain in the audit, distinct from the SDK's directly observed child. They
are neither hidden nor approved for a later positive execution context. Successful
native command/sandbox behavior, provider/auth/default/loader semantics, independent
adequacy/adoption and the full allowed/denied regimes remain required.

The private Git fixture also exposes a native config mutation: Codex can append
`[projects."/pilot/work"]` with `trust_level = "trusted"`. The maintainer explicitly
approved only that exact native addition in this task. The byte-exact postcondition accepts
unchanged config or the original bytes plus that exact stanza. Other project paths, trust
values, extra keys, base changes or reformatting remain `controlled_config_drift`. This
does not authorize live calls, skill execution, host/user project trust or a generic config write.

Individual 207 classification edges remain proposals. The existing 53 operation/obligation/
target template baseline is unchanged, and is not 53 test cases. Packet binding is explicitly
absent. The read-only evaluator and every public writer keep their existing refusals; no
runtime evaluator consumes this audit as E or uses it to clear unknowns.

## Reproduction

Use a clean checkout and trusted local Rust, bubblewrap, strace and GNU runtime tools:

```sh
python3 scripts/observe_codex_adapter_children.py \
  --binary /absolute/path/to/pinned/codex \
  --strace /absolute/path/to/strace \
  --output /private/output/adapter-child-context.json \
  --raw-traces /private/output/raw-traces
```

Source and launch-tool identities are checked again before writing the output. Output/raw
trace paths should be outside tracked source (or in ignored private scratch). Filesystem
state ids, private output names, process ids and outcomes may vary; this is an audit capture,
not deterministic execution evidence. No live-model credentials should be supplied.

The [recorded native audit](../../../cli_manifests/codex/reports/0.156.1/pilot-adapter-children.json)
is refreshed from a clean source revision; its source/tree and measured helper hashes identify
that observation. The [fixture readiness extension](support-depth-native-execution-fixture.md)
adds explicit per-process exit coverage, session-state bindings and cancellation observations.
The original PR #303 capture remains in Git at `9966b2f2`; it had nine complete and six
incomplete cases, all retained honestly. The new collector does not reinterpret those results.
Later documentation/test/data commits do not turn an audit into evidence for another packet.

The [depth contract](../../specs/support-depth-contract.md#obligation-templates) and
[evidence contract](../../specs/support-depth-evidence-v1.md) still control classification,
currentness, adequacy and adoption. This producer increment grants none of those authorities.

Next: finish remaining positive-context prerequisites, then capture actual native turn and
interaction cases through production adapters, with separate live-call authorization,
packet binding, independent adequacy and authenticated adoption. Install only reviewed
individual rows/profile before the public isolated maintenance-chain proof. Production
enrollment follows proof/adoption. No background job, new service or required CI job is added.
