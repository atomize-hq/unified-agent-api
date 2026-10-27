# Route-aware Codex pilot readiness

Baseline: staging `8636a8ff0b8f089c2f5625f92735c4043042f225` (PR #300).
This read-only implementation follows CP-1/CP-2 in the
[complete pilot policy](support-depth-complete-pilot-policy.md). It installs no isolated profile,
registry enrollment, writer admission, evidence adoption or production qualification.

## Current routes

The typed request selects initial, resume-last/id or fork-last/id. Initial/resume use exec for
normal unattended and external-sandbox requests; `non_interactive=false` uses app-server.
Fork always uses app-server. The five existing selector names remain unchanged; `*-json` names
describe the historical selection, not an assertion that every policy variant runs CLI exec.
No RPC method becomes a CLI surface. Malformed selectors, dual resume/fork intent, unknown
safety values and incompatible external sandbox/approval intent refuse the diagnostic decoder.
Other prompt/model/add-dir/capability validation remains owned by the production adapter.

Every route now prepares the canonical command environment, including explicit/ambient/default
binary resolution, home creation, generated CODEX_BINARY/RUST_LOG and final normalized overlays.
Managed initial/resume additionally read native configuration and requirements before thread
start/resume, preserve effective writable roots, then start a turn. Managed fork uses list for
last selection, forks the chosen source, then starts a turn. Native interactions, granular policy,
immutable file-change context, persistence, deadlines, cancellation and cleanup belong to those
managed boundaries. Fixtures do not establish successful native model/approval execution.

The resulting recommendation is to add MANAGED to initial and resume as well as fork. The two
additional unique obligations are OUT4 and LC4 per affected operation: initial17, resume18,
fork18 including BASE and M, totaling53 proposed due tuples. Registry/P remains the only place
for actual approved operation/template and individual surface assignments. The finite diagnostic
requires complete effect coverage; it does not install any such assignment. Approval of the
earlier49-tuple plan must not be interpreted as approval of newly acquired unit classifications.

## Read-only API and binding

`support_depth::codex_pilot::readiness(root, request)` reads the existing coherent stage-0 view.
No candidate declaration returns `None`; it does not synthesize or write a declaration. A present
candidate must cover the exact three operations, five typed selector/path/mode triples, one musl
target and complete `agent_api.run` mapping. These fixed modes use empty `predicates`: the private
decoder supplies their meaning, while acquisition checks actual parent exec JSON presence and
resume-last flag arity. Generic mode interpretation and all public writers retain their refusals.
`decode_route` is a finite diagnostic of selector/safety context, not a second full run validator.

The existing acquisition validator and P/O derivation are reused. Resume owns its exact child,
exec ancestor and app-server populations plus effective globals. Initial owns exec and app-server
plus globals. These include every acquired CLI unit of the actual unattended/managed variants;
original shard identities, normalized-out units and claims remain included. Fork owns app-server
plus globals. The registry's ownership check admits these supplemental paths only under the
complete fixed pilot declaration; generic contexts retain their exact-path ownership rule. One basis per operation
binds all served modes, CLI facts, protocol/config content, controlled config bytes and current
implementation source, including nested acquisition/registry/record interpreter modules. Retained work is carried by the existing planner. No new policy store,
generic predicate grammar, executor, workflow or required CI check is introduced.

All14 existing acquisition references remain in O. Additional references bind the complete
`reports/0.156.1/pilot-protocol-facts.json` and source inputs, including file executable modes.
The report is acquired from the pinned musl binary's offline experimental schema generator and
the exact upstream configuration schema. It includes selected client requests, server requests,
notifications (including request resolution), their transitive definitions, complete response
schemas and JSON-RPC envelopes. Shutdown is actual stdio EOF plus owned cleanup, not an invented
shutdown RPC. Numeric JSON-schema values stay in the complete raw content digest, outside B.7's
semantic JSON projection. Missing, partial, different or untracked protocol/config input refuses;
the report must match both the source-pinned digest and selected snapshot binary.

Reproduce acquisition with:

```sh
python3 scripts/acquire_codex_pilot_protocol.py \
  --binary /absolute/path/to/pinned/codex \
  --config-schema /absolute/path/to/pinned/config.schema.json \
  --output cli_manifests/codex/reports/0.156.1/pilot-protocol-facts.json
```

Both supplied files are digest checked. The generator uses a private home and cleared environment,
has a finite timeout and performs no model request. Its output is deterministic. Source URLs,
original generated-file hashes and limits are retained in the report. These are schema/source
correspondence facts, not A1 build attestation or evidence of executed obligations.

The expected controlled config disables every schema-admitted boolean feature explicitly.
Object-only feature defaults remain named as unbound facts; neither schema validation nor the
expected file proves effective defaults. Actual private filesystem roots, loader absence, exact
child environment/overlays, effective runtime config/requirements and credentials remain to be
observed by the later packet producer. Missing or unknown observations always remain in
`remaining`; the API has no qualified/ready success flag and synthesizes no E or acceptance.
Ordinary callers keep their normal SDK behavior.

The initial30 approved/24 unresolved/56 carried inventory remains historical and unchanged.
New per-operation populations require fresh reviewed whole-regime rows. This slice preserves
unresolved rows, reports missing runtime context, producer/adequacy and isolated-chain proof,
and cannot clear those items by supplying a plausible rationale or candidate basis.

The [runtime-context observation and current classification inventory](support-depth-codex-runtime-context.md)
now records offline native config/feature/startup facts and exact proposals. It does not establish
the complete runtime context, install classifications or clear the diagnostic's remaining items.
Packet-specific execution evidence and the public isolated maintenance chain follow. Production
enrollment follows the reviewed isolated proof.
