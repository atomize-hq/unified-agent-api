# Native support-depth evidence, revision 2

Status: Normative for the approved NE-P1/NE-P2 admission increment.
The [depth contract](support-depth-contract.md) and revision-1
[evidence predicates](support-depth-evidence-v1.md) govern adequacy, currentness and reuse.
The [approved design](../agents/lifecycle/support-depth-native-evidence-design.md) explains the
trust tradeoff. GitHub authenticates maintainer statements; it does not attest local execution.
This mechanism trusts honest authorized maintainer execution/completeness attestations for the
finite isolated Codex profile. The live profile remains uninstalled in this increment.

## Common encoding and authority

Use revision-1 UTF-8, named-object, unique-field, string/path/digest/Git identity and 1 MiB/4096-entry
limits. No unknown fields, positional objects or null members. Numeric host ids are positive u64.
Source, repository, context and case approval are fixed by private source-owned profile data;
public constructors, receipt fields, local remotes and environment values cannot select trust.
No native call, GitHub write, credential read or live profile installation is added here.

A native receipt has exactly revision-1's top-level members, with `schema_version: 2` and this
`producer` in place of the revision-1 producer. Reuse Scope, Build, Provenance, Cases and Adequacy
without changing their identities. A depth record remains revision 1. Revision-1 Actions parsing,
producer resolution and target checks remain unchanged.

`producer` is exactly `{kind, repository, pull_request, registration, completion, collector}`.
Kind is `maintainer_native`. Repository is `{name,id,owner_id}`: name uses revision-1 `owner/name`
syntax; ids are positive host identities. Registration and completion are each exactly
`{id,body_sha256}`. Collector is a file reference. All values are claims until independent host
resolution. The record origin must be isolated to admit native evidence.

## Attempt records

Each host comment contains at most one exact fenced block of its respective type, using newline
syntax as in `depth-adequacy-v1`; each JSON block is bounded to 65,536 bytes. Other Markdown and
the separate existing adequacy block may coexist. Repeated/unterminated/malformed relevant blocks
from authoritative actors refuse. Body SHA-256 binds the complete UTF-8 host body, not only JSON.

A `depth-native-start-v1` block is exactly:

- `schema_version`: 1; `profile`: `isolated-codex-native-v1`.
- `repository`: repository identity above; `pull_request`: positive packet PR.
- `packet_head`: full Git commit before native execution.
- `scope`: revision-1 Scope, naming one due obligation used as the aggregate discovery anchor.
  Event/P/O, origin, agent/version and all revision strings bind the whole generation; the anchor
  obligation must be selected by O. Other due obligation receipts reuse that same attempt.
- `plan`: file reference to the immutable pre-run plan.

The packet PR, full generation and current record are resolved independently of selected E.
A recognized authoritative start matching this generation is applicable before its plan, source,
context, outcome or completion is inspected. The anchor is not a favorable obligation filter.

A `depth-native-finish-v1` block is exactly:

- `schema_version`: 1; `registration`: exact start comment reference.
- `evidence_commit`: immutable proof commit; `bundle`: file reference below.
- `executor_identity`: bounded actual invocation attribution string.
- `population_through_id`: positive comment id observed before posting this finish.
- `registrations`: unique positive start ids in this generation through that snapshot, including
  this start. Start id <= snapshot id < finish id is required.
- `execution_attested`, `outcomes_attested`, `registration_order_attested`,
  `completeness_attested`: booleans; all must be true to qualify.

A finish attests its own prefix; it does not list future starts. Current evaluation separately
includes all subsequently discovered registrations and previously bound ids. Every applicable
start needs exactly one authoritative finish. Missing, malformed, duplicate, unavailable or
nonterminal evidence remains unverified. Preserve qualifying failures and contradictions; do not
retry/select the latest green or reset the population. A terminal failed case is still evidence.

Permission is checked before an unreferenced comment can enter the authoritative population.
Definitively unauthorized unreferenced copies are chatter. A selected/previously bound host
reference with revoked/missing authority refuses; an indeterminate lookup never means absence.
Every E-bound receipt retains its exact start/finish body digests even when unselected; conflicting
commitments for one host id refuse. Every authoritative finish must resolve an exact retained
authoritative start before its generation can be deemed irrelevant. A surviving finish whose
start was deleted is evidence of unavailable history, not ignorable chatter.
Comment ids, host authors and actual canonical repository write/maintain/admin permission are
independently resolved. Permission responses must identify the same user, not just a role string.
Full pagination, unique ids, stable closing reads and finite bounds are mandatory. Deleted never-
bound history and bypass execution cannot be reconstructed: completeness also relies on the
maintainer's NE-P1 honesty attestation, not an append-only-log claim.

## Retained plan and bundle

`plan` is exactly `{schema_version, scope, packet_head, build, native, context, collector,
requested}`. Schema version is 1; scope/head match registration. Build and requested case identities
use revision-1 forms. Native is exactly `{version,target,executable_sha256,acquisition}`: canonical version, acquired
native target, actual executable SHA-256 and a file reference to the pinned acquisition facts.
The executable itself is not committed or uploaded. Each aggregate bundle is bounded to 16 MiB
across its indexed payloads. Context and collector are file references. Plan.context binds pre-execution nonsecret context intent,
not an observation of effective native behavior.
Plan and bundle identity bytes use compact UTF-8 JSON with object keys sorted bytewise, no
insignificant whitespace, and array order preserved; numeric revision/host ids use integer encoding.
The frozen source-owned case population must match requested, with no missing or extra case.
The entire plan exists before native execution; registration commits its exact digest.

`bundle` is a bounded canonical index exactly `{schema_version, plan, observed_context, claims, files}`:

- schema_version is 1; plan matches the registration reference.
- observed_context is a separate file reference to the post-registration measured, finite nonsecret
  context projection. Its path differs from plan.context; both must match their separately approved
  profile expectations. Unknown effective context cannot be cleared by an intent file or attestation.
- claims is a nonempty unique list of file references, one per due obligation covered by this
  attempt; its exact population is fixed by the approved profile/record.
- files is a unique-path list of every retained referenced payload. Every listed claim, plan,
  graph, environment, toolchain, context and raw observation payload must be included. The index
  excludes itself, derived receipts and later endorsements, avoiding hash cycles.

Each claim is exactly `{scope, build, provenance, cases, mapping}` using common revision-1 forms.
Build matches the plan. Scope is derived from the same frozen generation and a due obligation;
requested cases are exactly that obligation's approved mapped subset, executed outcomes exactly
project that subset from the aggregate execution. Mapping must have independently reviewed
assertion/production bindings and exact authenticated adoption under `depth-adequacy-v1`.
Every plan-requested case belongs to a claim; none disappears into a favorable subset.
This index and all referenced regular files live under the agent's existing reports tree.

The adapter reads immutable blobs/modes/inventories from the independently authenticated proof
commit and compares complete source-owned expectations, current coherent working/index bytes and
scope inventories. A caller's local file or supplied hash is insufficient. The packet head must
contain the frozen generation; proof commit must descend from that head. The canonical protected
source and fixed isolated repository identity/baseline are checked before and after resolution.
The governing registry is checked separately against protected isolated staging, including when it
is a build input under `crates/xtask/data/`; canonical production source need not enroll depth.
All other approved producer inputs remain rooted in canonical protected source.
Actual build/source/toolchain/graph/environment, collector/executable, target pair and context
must match independently approved profile expectations; an attestation cannot clear an unknown.
For native v2, Build.host_target and build Environment.target bind the GNU wrapper build; scope's
obligation target and native binary provenance bind the exact musl acquisition file reference. Require the exact approved pair.

Receipts are derived only after start/finish and adequacy host references exist. Their common
fields must exactly match the authenticated claim, with its mapping and independently resolved
endorsement; producer references bind the actual start/finish and collector. A finish may contain
the existing adequacy fence; execution attestation alone does not imply adequacy adoption.

## Admission and staged implementation

The private native adapter corroborates all applicable attempts before passing facts to shared
currentness/adequacy/outcome predicates. Form errors are errors; unavailable/unsupported/untrusted
proof is unverified; qualifying counterexamples fail; contradictory qualifying pass/fail evidence
is unverified. Only internal evaluated results may be sealed to the complete depth record.
No public Verdict, parsed receipt or successful diagnostic grants write authority.

Public evaluation recognizes revision 2 but returns unverified without an installed native profile,
before any host access. The first increment includes strict forms, complete population resolution,
immutable bundle checks and the private admission adapter, tested with synthetic host records.
The live collector, actual context closure, complete native adequacy, finite public-path selection
expansion and host/profile installation follow separately. Production and unsupported paths retain
refusal. No credential fixture, real model execution or extra required CI job is introduced.
