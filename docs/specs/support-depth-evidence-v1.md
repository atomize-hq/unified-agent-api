# Support-depth execution evidence, revision 1

Status: Normative. This format implements the approved slice D evidence decisions D-P1/D-P2.
The [support-depth contract](support-depth-contract.md) governs verification, reuse and authority;
Annex B's depth-record format remains revision 1. This document changes no enrollment or path enablement.

## Boundary

An E result references a receipt by the existing repository-relative path and SHA-256 of its
bytes. Store it under the agent's existing `reports/<version>/` evidence files, outside the
`coverage.*.json` namespace. There is no second authoritative evidence store. References inside
the receipt also name repository-relative regular proof files and their content identities.
Generated executables are identified by digest, not required to be committed as binaries.

**MA4 delivery containment** (approved by the maintainer on 2026-11-05). At MA4 delivery only,
every E reference MUST lie in its own record's `<manifest_root>/reports/<version>/evidence/`, and
every payload a receipt references MUST lie in that receipt's evidence destination. Any other
reference refuses the whole delivery, whether or not the referenced file changed, in production
and under an installed route. This narrows only what MA4 delivers. A file reference elsewhere
keeps the form Annex B allows (any repository-relative path), and evaluation and every other
consumer are unchanged. The [synthetic subject design](../agents/lifecycle/support-depth-synthetic-subject-design.md)
states how delivery binds the evidence path class.

The D1 typed reader decides **form and internal mapping identity only**. It does not establish
producer trust, actual execution, independent adequacy, currentness, complete dependency scope,
reuse, qualification or route admission. Parsing a self-authored receipt never verifies an
obligation. Empty/mismatched test selections, non-passing cases and incomplete provenance can be
recorded truthfully; the later evaluator MUST NOT turn them into verification. No command calls
this typed reader for admission in D1. Production depth refusal remains in force.

Workspace evidence binding MUST require a regular stage-0 index entry, regular working file and
directory ancestors without symlinks, and working bytes equal to the indexed blob. Missing paths
have no binding. Invalid paths, present untracked files, unmerged entries, missing indexed files,
non-regular modes, symlinks and working/index disagreement MUST refuse the binding operation.
Recheck the bound inputs before returning. This is a coherent observation, not serialization or
an authorization receipt; immutable integration readers still use that exact tree's blobs.
Object replacement MUST be disabled when resolving an indexed blob; a replacement reference
does not change the object's indexed content identity.

## Encoding and limits

The input is UTF-8 JSON, at most 1,048,576 bytes. Whitespace/member order are insignificant; every
object takes exactly its listed members, no unknown or duplicate members. No member is optional
or null. Each list contains at most 4,096 entries and each string is nonblank, contains no NUL
and has at most 4,096 UTF-8 bytes. A file reference is exactly `{ "path", "sha256" }`;
paths contain no empty, `.`, `..` or `.git` segment, backslash or leading/trailing slash.
Digests are 64 lowercase hex characters. Git commit/tree identities are 40 lowercase hex
characters in this repository. Positive numeric host ids/attempts are JSON u64 values greater
than zero. `schema_version` is the integer `1`. File-reference lists have no repeated path;
string lists have no repeated value. Requested/executed case identities and assertion-case
identities do not repeat within their respective lists. Array order has meaning for identities.

### Receipt

| Member | Exact value/shape |
| --- | --- |
| `schema_version` | `1` |
| `scope` | Scope below |
| `producer` | GitHub Actions attribution below |
| `build` | Build below |
| `provenance` | List of `{ "kind", "origin", "file" }`; kind is `binary`, `source` or `fixture`; origin is a nonblank provenance/source-contract description; file is a file reference. Entries have unique file paths. |
| `cases` | `{ "requested", "executed", "raw_output" }`; requested is a list of case identities, executed is a list of case results, raw_output is a file reference. |
| `adequacy` | `{ "mapping", "endorsement" }` below |

`scope` is exactly `{ "agent_id", "version", "event", "enrollment_origin", "obligation",
"policy_identity", "obligations_identity", "rules_revision", "trust_revision", "review_revision" }`.
The version is canonical stable `MAJOR.MINOR.PATCH`. Event and obligation use Annex B's typed
Event and obligation-reference shapes. Event strings, operation/target and optional capability
are nonblank; exactly one of obligation `id` or `capability` is stated. Origin is `production`
or `isolated`, an attribution claim to be independently checked, never permission. P/O are SHA-256
identities; the three revision strings identify the governing definitions/trust/review rules.
Their equality/current authority is a later verdict, not a schema inference.

`producer` is exactly `{ "kind", "repository", "workflow_path", "workflow_commit", "collector",
"run_id", "run_attempt", "job", "artifact_name" }`. Kind is `github_actions`; repository is
`owner/name` using nonempty ASCII alphanumeric/`-_.` components other than `.`/`..`;
workflow_path is a safe path under `.github/workflows/` ending in `.yml` or `.yaml`;
workflow_commit is a Git commit identity; collector is a file reference. Run id and attempt are
positive integers. Job and artifact name identify the producing job and its run/attempt-scoped
artifact. The receipt does not include the uploading artifact's own digest or an id assigned
after upload: that would require a self-reference. The host adapter resolves those independently.

`build` is exactly `{ "source_commit", "source_tree", "host_target", "toolchain", "features",
"environment", "dependency_graph", "covered_inputs", "absent_inputs" }`. Commit/tree are Git
identities. Toolchain, sanitized relevant environment and full resolved dependency graph are
file references; covered_inputs is a list of file references and absent_inputs is a list of safe
paths, with no path stated as both present and absent. Features is a string list. These are
claims about the actual built inputs; naming the PR head is insufficient. The later evaluator
MUST establish the affected crate's full resolved build graph, immutable external package
identities and all referenced implementation/build/config/test/fixture/adapter/protocol/requirement
inputs. Missing dependency provenance is unverified, never an empty dependency closure. The
typed reader does not interpret the referenced graph/environment file's contents.

A case identity is exactly `{ "name", "executable_sha256" }`, binding a name to the actual test
executable's digest. A case result is exactly `{ "case", "outcome" }`; case is that identity,
outcome is `passed`, `failed`, `ignored`, `skipped` or `aborted`. Requested and executed populations
are recorded separately; the evaluator MUST reject missing/extra/filtered/zero/unfinished cases
and every other selection/outcome failure required by the governing contract.

### Adequacy and its identity

The mapping is exactly `{ "obligation", "reviewer_identity", "executor_identity", "assertions" }`.
Obligation uses the same form as scope. Assertions is a list of exactly `{ "case", "assertion",
"production_inputs", "counterfactual" }`; case is a case identity, assertion is a file reference,
production_inputs is a file-reference list, and counterfactual explains why removing the production
validation/parser/cleanup under test fails that assertion. Reference the smallest practical
assertion/production files; names and counts are not an adequacy argument. Invocations/identities
are nonblank attribution strings; a reader cannot authenticate their independence from spelling.

The endorsement is exactly `{ "kind", "repository", "pull_request", "id", "author",
"source_commit", "body_sha256", "mapping_sha256" }`. Kind is `pr_review` or `issue_comment`;
repository follows the producer repository syntax. Pull request/id are positive host ids,
source_commit is the reviewed source revision, and author identifies the authenticated host author.
The digests bind the host body's exact bytes and this mapping. Mapping SHA-256 is over its complete
typed JSON serialized with Annex B.7's canonical JSON algorithm, omitting no members. No mapping
member is named `identity`. The reader MUST reject a mismatched mapping digest. The endorsement
itself is excluded from that digest so there is no hash cycle. Its body MUST adopt the exact
mapping digest; checking that body, author authority and independent review is later admission.

## Approved initial trust boundary

Only recognized existing repository GitHub Actions producers may qualify execution evidence in
the initial revision-1 profile. The approved NE-P1/NE-P2 [native revision-2 mechanism](support-depth-native-evidence-v2.md)
adds a separately governed isolated maintainer-attested producer; its live profile remains absent.
Ordinary local relay/gate and unregistered native output remains diagnostic. The profile MUST identify the
approved collector/workflow implementation revision and existing maintainer change authority,
rooted in reviewed integration code. Observing a code hash or authenticated artifact does not
approve a candidate collector. D1 installs no approved producer allowlist or host adapter.

The host adapter MUST independently resolve repository/workflow/job/run/attempt/artifact
correspondence and actual built inputs, not accept a caller's `trusted` flag. It MUST define
discovery keys including source Event/P/O, tuple and covered input identities, fully paginate
the relevant run population and enumerate every attempt independently of E's selected references.
Incomplete discovery cannot verify evidence. Contradictory qualifying attempts stay unverified;
there is no latest-green or automatic contradiction override. Existing T4 preflight association
retains its historical highest-attempt policy and cannot supply this depth verdict unchanged.

An existing authorized maintainer MUST adopt the content-bound independent adequacy mapping in
an authenticated review/comment on the packet PR. Multiple mappings may be endorsed together.
Different reviewer/executor agent invocations can provide independence without different GitHub
accounts; the maintainer attests that independence and substantive mapping. The executor merely
naming a reviewer, an AI lane's local file or a generic PR approval is insufficient. No standing
role, extra mandatory review count or CI check is introduced; unchanged admissible mappings are
reused. This is existing repository review trust, not protection against a dishonest maintainer.

Producer/review host references and needed proof material MUST remain resolvable or the later
verdict is unverified. Keep content-bound proof inputs with existing reports for subsequent
promotion. Secrets, unbounded raw output and real credentials MUST NOT enter these files; use
sanitized environment descriptions and synthetic credential sentinels. Exact-version/target
reuse, production-enrollment origin and complete current dependencies remain governed by the
support-depth contract; no compatibility rule, debt waiver or acceptance authority is added.

## Revision-1 dependency and environment payloads

The referenced `build.dependency_graph` is a named JSON object with exactly
`{ "schema_version": 1, "roots", "packages", "inputs", "scopes" }`.
`roots` is a nonempty unique list of resolved package ids for the affected crates. Each package is
`{ "id", "origin", "dependencies" }`; dependencies are unique resolved ids. Every referenced id
must exist, and every package must be reachable from a root. Cycles are permitted (the resolved
build graph can include separate build/dev units). Origin is one of:

- `{ "kind": "local", "directory" }`: a repository-relative package directory, including its
  Cargo.toml and every regular file below it in `inputs` and `scopes`.
- `{ "kind": "registry", "source", "checksum" }`: an immutable resolved package id, registry
  source identity and SHA-256 package checksum, corroborated by the host's resolved graph.
- `{ "kind": "git", "source", "commit" }`: an immutable resolved package id, repository source
  identity and full lowercase Git commit, corroborated by the host's resolved graph.

`inputs` is a nonempty unique-path list of file references. It includes Cargo.toml, Cargo.lock,
all local package files, build/config inputs, and every explicitly referenced implementation,
adapter, protocol, assertion, fixture, upstream and requirement input. `scopes` is a unique list
of repository-relative directories whose complete regular-file inventories are included. Local
package directories MUST be scopes. Relevant shared input directories MUST also be scopes when
adding a file can affect the obligation. Absent relevant optional configuration inputs are bound
by `build.absent_inputs`. At minimum `.cargo/config`, `.cargo/config.toml`, `rust-toolchain`,
`rust-toolchain.toml` and each local package's `build.rs` MUST be covered or explicitly absent.
The build's covered-input set is exactly the graph's input set.

The graph is a claim, not proof of completeness. A private independently resolving producer
adapter MUST corroborate the full graph against actual resolved build provenance, including
external packages and inputs outside package directories; unsupported build scripts, environment,
external paths or incomplete dependency discovery cannot verify. Empty/missing provenance never
means an empty closure. Readers compare all covered bytes, Git executable modes and scoped inventories with both the
actual built commit/tree and the current coherent indexed workspace. Missing or changed inputs,
new files in a relevant scope, executable-mode changes (including working/index disagreement on
platforms with executable permissions), or an absent input becoming present invalidate the result. Changes
outside that dependency set do not. No current profile permits cross-version or cross-target reuse.

The referenced `build.environment` is exactly `{ "schema_version": 1, "target", "features",
"variables" }`. Features are unique strings. Variables is a unique-name list of
`{ "name", "value" }` containing the complete relevant sanitized environment. No secret is stored;
if relevant values cannot safely be represented and corroborated, evidence remains unverified.
The producer adapter must corroborate this payload and the toolchain report against the actual
build, and evaluation compares it with the independently selected current build environment.
Variable values may be empty. These payloads use the receipt's named-object, unknown-member, duplicate, null, safe-path, digest,
string/list and 1 MiB bounds. Structurally valid incomplete claims remain readable but unverified.

## Read-only verdict evaluation

The evaluator distinguishes malformed form (an error), `unverified` (missing, stale, mismatched or
untrusted proof), `failed` (a qualifying counterexample) and `verified` (all predicates established).
It never trusts the E result label. An id obligation must be selected by O; an M/capability
obligation must have the exact capability-to-operation mapping in P and the operation in O. A complete independently discovered population includes all
applicable runs and attempts, including missing or nonterminal ones. Each referenced receipt must
match its authenticated artifact bytes, original Event/P/O/enrollment, actual source commit/tree,
resolved graph, toolchain, environment, provenance, requested/executed cases and raw output. A PR
head association alone is insufficient. Missing population facts or an unsupported producer yields
unverified; a favorable selected receipt cannot hide another qualifying run. Contradictory passing
and failing qualifying attempts yield unverified pending investigation.

The [request-freeze checkpoint rule](#authenticated-request-freeze-checkpoints-and-implementation-readiness)
defines the authenticated receipt exception for the packet-bound synthetic maintenance and
onboarding producer. Reporting checkpoints remain applicable population members.

An authenticated authorized maintainer adoption must bind the exact mapping digest, reviewed
source and reviewer/executor invocations, and attest independence and the substantive counterfactual
for each assertion. The evaluator requires nonempty assertion/production bindings, complete case
coverage and current files. Nonblank prose or a matching digest alone does not establish adequacy.
Original and current scopes, obligation definitions and applicable trust/review revisions must be
independently established. A changed P/O needs the original P/O reuse binding in current E; Event
changes require that binding even when P/O content is identical. Production use requires original
production enrollment. Neither isolated nor legacy proof can acquire production origin by reuse.

D1b installs the pure predicate with private host fixtures. Its public workspace reader has no
recognized live producer until the following CI adapter slice; it reports unverified rather than
accepting caller-supplied host assertions. It supplies no writer admission or production enrollment.
Publisher consumption follows with that adapter, keeping existing production enrollment refusal.

For depth-enrolled ON4 gates, the following bounded evaluation is specified, not installed
(OR-P13, [routing design, "Maintainer decisions" and "Route composition"](../agents/lifecycle/support-depth-onboarding-routing-design.md#maintainer-decisions)).
Before delivery, the depth-reading parts of the fixed charter checklist item 7 command inventory
MUST run through the commands' shared checking functions in the proposal's scratch view, inside
a borrowed fresh evaluation scope bound to the exact proposed record and actual inputs. Route
presence alone MUST NOT make derivation trust isolated evidence. Every registry reader used by
these checks, including both preparation and revision loading for `runtime-advertising-model
--check`, MUST use authenticated routed reading, read-only. Default generation, neutral writes,
NE8 write refusal and production verdicts are unchanged.

After delivery, the unchanged subprocess commands MUST independently resolve their own installed
route over a clean committed candidate whose HEAD the host already holds, in CI and at IN1 for
the actual integration candidate before the protected ref advances (OR-P13, same design section).
A borrowed scope does not cross a process boundary, and a scratch tree cannot resolve authority.
Every leaf of the fixed inventory, including the recursive `make preflight` test leg, MUST have
its proposal-phase execution and required independent committed-candidate execution accounted
for. Registry-reading tests MUST retain their assertion purpose: production-parser controls use
hermetic undeclared fixture registries; candidate-registry checks use authenticated routed reads
of the actual candidate. Format, lint and build remain subprocess checks before delivery; other
local checks use hermetic subprocesses or authenticated shared checking functions as appropriate.
Each evaluation binds its actual inputs. A prior local pass supplies no authority for changed
candidate bytes; an admission-relevant change MUST fail independent evaluation, while a harmless
change may pass fresh checks. No serialized verdict, environment token, reusable gate receipt or
candidate-hosting step is introduced. Local preparation, independent checks, current IN1 admission
and the terminal integration gate retain the separate meanings stated by the charter.

## D2 isolated CI producer

The initial live producer is deliberately limited to the isolated channel-validation source
contract under `crates/xtask/tests/fixtures/depth-ci/`. Its canonical version is `0.0.0`, target
`x86_64-unknown-linux-gnu`, agent `codex` and operation `controlled_channel`, obligation `V1`.
It exercises `wrapper_events::ValidatedChannelString::new`; it is not a Codex CLI grammar,
production enrollment or verification of other obligations. The existing `rust-tests` job runs
the normal full workspace tests and the dedicated collector/cases; no new CI job or required
status check is introduced. The producer's child build uses pinned Rust 1.89.0; project source
continues to meet the workspace MSRV. Unknown harness/configuration/native linkage refuses proof.

The collector emits one immutable `witness.json` in an Actions artifact named
`depth-channel-v1-<run_id>-<run_attempt>`. It contains schema version 1, run/attempt, workflow
implementation commit, actual `build`, `graph`, `environment`, `provenance`, `cases`, and the
referenced `toolchain_report` and `raw_output` bytes as strings. The complete JSON is at most
1 MiB. Payload objects use the revision-1 forms above; witness report/output strings are bounded
payload bytes, not receipt attribution strings. The artifact contains exactly that one regular
file. ZIP traversal, duplicate/extra/encrypted entries, symlinks, oversized archives or payloads,
missing host digest and digest mismatch cannot qualify. No executable is committed or uploaded.

This witness is execution output, separate from a candidate receipt. The private adapter derives
scope/definition from the actual built frozen fixture and coherent current indexed fixture,
corroborates each receipt field against authenticated witness data and source, resolves every
applicable run and attempt independently of E, and establishes exact approved job/upload-step/
artifact correspondence. An artifact name alone does not bind it to a job. The approved upload
and binding steps expose the host-assigned artifact id/digest; their successful authenticated
job/attempt logs and artifact metadata must agree. Missing/nonterminal applicable attempts and
incomplete or saturated pagination remain unverified.

Producer approval is rooted in canonical repository protected staging obtained independently
from GitHub, not a local remote, a receipt or a candidate digest. The complete behavior-affecting
collector/helper/dependency closure and workflow bytes must match that approved source. The
first candidate run cannot qualify before those implementation bytes have been integrated.
Supported registry sources must match locked crate archive checksums, contents and inventories;
resolved Cargo metadata and actual compiled package populations must agree. Build configuration
and relevant environment must be closed and captured. No missing input is an empty dependency set.

Upload execution proof first. After compilation, independently review the exact content-bound
mapping (including executable identity), then obtain existing maintainer adoption on the packet
PR. General D2 authorization is not a fabricated host endorsement. Unchanged admissible mappings
can be reused across executions after their reviewed source inputs and applicable definitions/
trust/review are checked; a different unrelated source commit alone does not require another
review. Receipt proof material remains under existing report paths; diagnostic materialization
occurs only in a disposable checkout and grants no canonical write authority.

Official publication consumes opaque evaluated results bound to the record state. Neither a
public `Verdict` object nor a hash callback can manufacture that result set. Official publication
requires production-origin evidence; this isolated profile cannot qualify a production row.
Existing acceptance-entry and qualification rules still apply after verification. Non-enrolled
publication remains unchanged; raw E success labels or acceptance entries cannot replace proof.

### Initial host-resolution and adoption details

The adapter's repository is fixed to `atomize-hq/unified-agent-api`, workflow
`.github/workflows/ci.yml`, job `rust-tests`. Its complete local producer closure is the workspace
`crates/` inventory, Cargo.toml, Cargo.lock, that workflow and `scripts/depth_ci_archive.py`.
These cover xtask's compiled workspace dependencies and the controlled subject. Current indexed
bytes/modes and actual built/workflow implementation must match the independently obtained
protected-staging source. This initial profile conservatively refuses changes to that closure;
it does not infer approval from candidate hashes. Locked registry archives, extracted contents
and complete inventories are checked before build-time code executes and again after compilation.
The reviewed `build-profile.json` fixes the selected package/feature/build-script populations.
Only the six explicitly listed registry build scripts are supported; native linkage refuses.

The generation's Event request commit resolves exactly one canonical staging packet PR through
GitHub's commit-to-PR association. Execution discovery exhausts the workflow's runs since the
Event's date, without filtering on a selected receipt or success. A canonical-repository head
with the same frozen Event/P/O is applicable before artifact inspection, across pull_request,
merge_group, workflow_dispatch and push. Every attempt from 1 through the authenticated attempt
count is included. Unknown applicable events, missing/nonterminal attempts, absent/expired
required execution witnesses, changing totals/populations, duplicates or the
1,000-row/100-attempt bounds refuse verification. Foreign-repository heads are outside this producer profile. The approved collector
records checkout HEAD; a PR checkout must be the host head or a two-parent test merge whose second
parent is that head. Workflow implementation and checkout commit agree. The adapter confirms
source/tree, reviewed implementation, graph/configuration and current scoped inputs separately.

Successful collection/upload/binding steps in the exact attempt's `Rust tests (workspace)` job
must expose exactly one timestamped `DEPTH_ARTIFACT_BINDING=` JSON log record within the binding
step. Its `id`, `name`, and `digest` agree with immutable API artifact metadata and the downloaded
ZIP's SHA-256 and size. A failure elsewhere in that job does not erase a faithfully collected
counterexample. All execution-required attempts must agree on environment and content-addressed
toolchain report; a favorable run never selects those current inputs by itself.

The maintainer review/comment body contains exactly one fenced `depth-adequacy-v1` JSON object:

```json
{
  "schema_version": 1,
  "source_commit": "<full reviewed source commit>",
  "mappings": [{
    "mapping_sha256": "<canonical adequacy mapping SHA-256>",
    "reviewer_identity": "<actual independent review invocation>",
    "executor_identity": "github-actions:rust-tests/isolated-channel-v1",
    "independence_attested": true,
    "counterfactual_adopted": true
  }]
}
```

The executor identity names this stable CI producer; the receipt separately binds exact run and
attempt ids. A review must be submitted (approved or commented), not pending/dismissed, and its
commit id must match the body's reviewed source. A comment is obtained from that exact packet PR.
Its actual GitHub author currently has write, maintain or admin authority on the canonical repo.
The complete body digest, exact mapping digest, actual source, author and host id form the
endorsement. Reused mappings retain their original reviewed source and require unchanged
assertion/production bytes/modes, source contract and original P/O; unrelated commits alone do not
require a new endorsement. The trust/review revisions are `github-actions-channel-v1` and
`maintainer-mapping-v1`. This is maintainer adoption of an independent review, not an executor's
self-assertion or proof against a dishonest authorized maintainer.

`cargo run -p xtask -- depth-ci-prove` resolves this proof read-only. It creates receipt/payload
material only in a disposable checkout, invokes the same evaluator and publisher JSON/Markdown
renderer, and reports isolated results. The selected receipt and its referenced payloads remain
coherent staged evidence; other attempts are corroborated directly from their authenticated
immutable host payloads, so a packet cannot omit an unfavorable attempt. No canonical report,
acceptance entry, registry enrollment, workflow policy or production pointer is written. Before
implementation integration or exact maintainer adoption, this diagnostic cannot report verified.

Producer approval includes the presence/absence of root `.cargo/config`, `.cargo/config.toml`,
`rust-toolchain` and `rust-toolchain.toml`; the initial profile requires all four absent. Before
artifact processing for an execution-required attempt the adapter approves the independently
authenticated run/attempt head's producer/workflow closure as well as the separately checked actual checkout. A witness-provided
synthetic merge cannot approve an unreviewed executed workflow. Artifact decoding executes only
the helper read from the immutable independently approved staging anchor, never executable bytes
from a discovered candidate head.

## Isolated Codex CORE evidence bridge

The approved E1-P1/P2 evidence increment installs a second compiled isolated profile alongside
channel V1. It pins Codex0.156.1, one headless `run`/CORE selector, the `agent_api.run` mapping and
Linux `x86_64-unknown-linux-gnu` wrapper execution. The frozen source contract, P/O, case mapping
and resolved build profile live under `crates/xtask/tests/fixtures/depth-core/`. Its upstream Linux
musl snapshot and executable digest are provenance, distinct from the GNU wrapper build target.
Adapter subprocesses are synthetic; passing fixtures do not assert a live upstream model run.
The precise version applicability and substantive adequacy still require independent review and
exact authorized maintainer adoption; unsupported claims remain unverified.

`depth-core-collect` reuses the existing collector, closed build environment, pinned Rust and
locked registry verification. The fixed workspace all-features build is recorded explicitly;
only Codex run is claimed. It executes the union of the reviewed cases, binding the executable,
complete build/input/mode/configuration closure and exact requested/executed outcomes. CORE binds
all committed `docs/specs/**` requirements, including the canonical run protocol and capability
contracts, as one fixed complete scope. That scope must match built, independently reviewed and
current inputs; changed bytes, modes or requirement inventory invalidate the evidence. Channel V1
keeps its existing input boundary. Its witness
is uploaded as `depth-codex-core-v1-<run_id>-<attempt>` by the existing `rust-tests` job, with separate
successful collection/upload/binding steps and exact authenticated artifact correspondence. No
new job, required status check or service is added. Unavailable collection stays unavailable.

The existing canonical-repository/protected-staging trust anchor, exhaustive applicable run and
attempt discovery, immutable artifact checks, independent mapping adoption and currentness rules
apply unchanged. A profile is selected only from the two compiled definitions, never a receipt,
remote URL, environment override or user-supplied trust flag. CORE's trust revision is
`github-actions-codex-core-v1`; its executor is
`github-actions:rust-tests/isolated-codex-core-v1`. `maintainer-mapping-v1` remains the adoption
revision and the same exact `depth-adequacy-v1` body format applies. Each of the eleven id
obligations and mapped M obligation has its own content-bound mapping. Partial adoption cannot
qualify the full profile. Implementation integration and independent adoption remain prerequisites
for any live verified result; the candidate cannot endorse itself.

One aggregate witness yields a separately authenticated receipt and complete population for every
due obligation. Each receipt's cases are exactly that obligation's mapped subset; its raw log is
the complete union execution. Failing unrelated cases cannot hide a counterexample for their own
obligation, and a favorable E-selected receipt cannot omit another applicable attempt. Receipt
paths under `reports/0.156.1/evidence/core-v1/<run>-<attempt>/<obligation>/` distinguish R/V/G/E/A/X,
OUT3, LC2 and M. Malformed form is an error; missing, skipped, unadopted, stale, unavailable or
contradictory evidence remains unverified, and qualifying counterexamples fail.

The read-only `evaluate_isolated_record` bridge authenticates the fixed CORE profile and evaluates
every due obligation. Its opaque result set binds the exact record state; changed Event/P/O/E or
entries invalidate that set. A separate private seal includes the complete typed record, including
schema version and entry order; the acceptance digest still deliberately excludes entries.
E labels, public Verdict objects, caller callbacks and receipt JSON
cannot construct verified results. `depth-core-prove` materializes host receipts/payloads only in
a disposable checkout, calls that evaluation path and the existing publisher, and prints per-
obligation decisions. These are isolated reporting facts, never acceptance or write authority.

Official production consumers still require production-origin proof; `production_results` remains
unverified. This increment neither enables public maintenance writers nor changes registry,
production enrollment, trust to a disposable host, acceptance entries or pointers. The next
increment must independently bind the supported isolated public paths and IN1 before a complete
maintenance-chain proof can be claimed. Unit host fixtures establish predicate composition;
they are not live authenticated evidence or a maintainer endorsement.

## Isolated maintenance binding foundation

The maintainer approved IR-P1, IR-P2 and IR-P3 in the
[isolated route design](../agents/lifecycle/support-depth-isolated-route-design.md) on 2026-10-25.
Implementation is staged. The initial binding foundation MUST leave the live isolated profile
uninstalled until the exact repository identity, original installation baseline and credential
references have their separate authorization and the profile is installed through reviewed source.
An absent profile MUST refuse before host reads; a CLI flag, environment value, remote, registry
declaration, receipt or serialized diagnostic MUST NOT install or authorize one.

Binding uses one private, non-deserializable context. It independently checks the fixed repository
id, owner id, expected name, protected staging tip, approved source closure, installation baseline
and actual workspace/Git ownership. Each profile names exactly one compiled subject. The Codex
subject binds Codex0.156.1 maintenance `run`/CORE and `agent_api.run` on the acquired
`x86_64-unknown-linux-musl` tuple; the synthetic subject is defined in
[Synthetic wrapper machinery-proof profile](#synthetic-wrapper-machinery-proof-profile). The GNU Rust
wrapper build host is distinct evidence metadata. Registry declarations remain necessary and every registered retained
root is mediated; unsupported or extra selections MUST refuse. Neither this context nor its
read-only diagnostic is an acceptance verdict or proof of OS isolation.
The isolated registry is separately checked governing data, with coherent bytes, exact scope and
freshness retained. The approved isolated declaration must already exist on the isolated protected
baseline; resolving a context does not install or edit it. Source approval MUST NOT require enabling
the canonical production registry to admit the isolated declaration, nor treat arbitrary
governing-data edits as approved code.

The synthetic onboarding selection below specifies the only pre-declaration exceptions to the
protected-baseline declaration and registry-equality requirements: the exact ON1 bootstrap delta
and the maintainer's first declaration transition (OR-P1, OR-P14,
[routing design, "Route composition"](../agents/lifecycle/support-depth-onboarding-routing-design.md#route-composition)).
They are not installed. All other scope, source approval, freshness and ownership checks remain.

Planning, a derived invocation-local proposal view and apply MUST retain the caller's binding and
check freshness against independently observed host/source/tip and coherent local inputs. A copied
scratch tree MUST NOT resolve its own authority. Reuse existing expected-before delivery checks;
do not convert a prior check or serialized context into a reusable write permission. Ordinary
registry loading and unsupported public writer refusals remain until their own reviewed wiring.

IR-P3 applies only to the future packet-bound isolated producer: authenticated terminal producer-job
evidence may be evaluated while its enclosing workflow runs, with all applicable attempts,
unfavorable witnesses and missing/unfinished-producer refusals retained. Consumers must follow the
producer without producer self-admission. The complete CI result policy and terminal gate still
govern MERGE. Existing channel/CORE fixture profiles retain whole-workflow completion; this
foundation does not implement packet evidence, change their behavior or enable production results.

## Authenticated request-freeze checkpoints and implementation readiness

For the packet-bound synthetic maintenance and onboarding producer, the following shared rule
specifies the reporting-only exception for the genuine MA2 request freeze before MA4 and genuine
ON2 freeze before implementation. The mechanisms below govern its implementation. Production/live
activation MUST remain closed until separately authorized phase C proves trusted-emitter isolation
and exact host transport in the real workflow layout. The
[freeze/readiness design](../agents/lifecycle/support-depth-freeze-readiness-design.md#four-mechanism-decisions)
records delivery and qualification obligations. This docs amendment specifies pending behavior.
An evaluator MUST refuse the exception whenever required authority cannot be established.

1. Applicability and complete independent population discovery MUST precede classification.
   Every matching original Event/P/O run and attempt remains represented, across PRs and all
   supported events, including reporting checkpoints, unsuccessful attempts, retries and missing
   or nonterminal attempts. Each row MUST retain authenticated identity, observable outcome,
   classification and its authority or reason. An independently generation-bound checkpoint
   MUST NOT disappear solely because an auxiliary producer fixture is absent. Unavailable
   required binding facts, ambiguous classification or incomplete/changing population prevents
   verification. Every attempt from 1 through the authenticated attempt count and existing
   pagination, duplicate, unsupported-event and discovery-bound refusals still apply.
2. An applicable attempt MAY omit a support-execution receipt only if independently authenticated
   as the exact authorized reporting-only request-freeze checkpoint in its permitted context.
   Source differences, absent artifacts, lifecycle labels, timestamps, PR identity and selected
   results supply no checkpoint authority. A checkpoint establishes zero successful obligations,
   substantive execution, adoption or currentness. Verification still requires independently
   qualifying execution and adoption for every due obligation. Checkpoints alone leave support
   unverified.
3. Protected IN1 MUST authorize the exact freeze transition, including MA2's authenticated combined
   acquisition + freeze transition, against its authenticated predecessor through independently
   approved policy. The combined reporting-context, admission and protected-history proof MUST
   bind authenticated repository identity, original
   Event/P/O/enrollment, predecessor and its admission, exact admitted head/tree, actual checkout relationship, complete source/context
   closure governing reporting versus execution, approved IN1 implementation/workflow provenance,
   host admission run/check and attempt with its decision, and protected lifecycle history.
   An independently approved pure transition planner shared with the real MA2 and ON2 freeze
   writers MUST derive the exact freeze delta from the authenticated predecessor plus any
   independently validated acquisition delta, original request/approval, authoritative
   generation/writer inputs, retained entries/work and approved policy identity. For genuine MA2,
   acquisition and freeze MAY share the existing single transport commit. The acquisition delta
   MUST be authenticated through existing acquisition authority, using the exact resolved
   acquisition output paths, acquired-input references, lockfile checksums and contents, applicable
   OF-P4 maintainer acquisition statement, and existing acquisition validators. OF-P4 supplies
   onboarding provenance only from the named protected integration ref. Each changed acquisition
   output MUST have independently authenticated exact bytes, presence/absence, mode and object
   kind. A path label, candidate digest or semantic validation alone supplies no such authority.
   Any output lacking byte-for-byte acquisition authentication MUST remain an explicit FR2
   prerequisite and MUST refuse checkpoint admission until resolved. Version metadata and the
   complete lockfile lie outside O's acquired-input references. Only the authenticated
   acquisition outputs MAY enter that delta. Registry, RULES, SCHEMA and other policy inputs retain
   their existing prerequisite authority. The freeze delta MUST equal the shared planner output
   over those authenticated inputs. The complete expected tree MUST equal the authenticated
   predecessor + validated acquisition delta + planned freeze delta. IN1 MUST require complete
   candidate-tree equality, including exact bytes, modes, object kinds, creates, replacements,
   deletions and preservation everywhere else.
   The plan MUST cover every actual writer effect, executable packet content and reporting/runtime
   projections. Variable inputs MUST have explicit independently authenticated sources. IN1 MUST
   NOT execute candidate writers or packets, trust candidate expected-output files or policy
   selection, or substitute local workspace bytes for the authenticated predecessor. Hidden
   implementation, dependency, build, workflow and unrelated changes fail reporting eligibility.
   Candidate-authored admission files and similarly named successful checks cannot establish
   authority.
   ON2's existing boundary has acquisition and its OF-P4 statement admitted on the integration
   branch before the dry-run freeze. Its checkpoint therefore has an empty acquisition delta and
   MUST preserve those predecessor inputs. Candidate-only acquisition or statement bytes MUST
   refuse. These rules preserve the existing MA2 transport commit boundary.
   [Acquisition authority inspection and FR2 prerequisites](../agents/lifecycle/support-depth-freeze-readiness-design.md#acquisition-authority-at-the-ma2-commit-boundary).
4. The admitted checkpoint and its permitted checkout/context relationships MUST be immutable
   within the generation. Later admission or success MUST NOT replace that authority or
   retrospectively exempt another candidate. Checkpoint eligibility does not seal the attempt
   population. A later retry of the exact historical checkpoint MAY retain reporting status only
   under that same authenticated context, without reopening the freeze window or clearing
   execution evidence. A test merge, merge group, dispatch, push or retry MUST authenticate its
   actual source, workflow and relevant inputs against the permitted relationship. Nominal head
   association alone grants no exception.
5. Readiness is the independently authenticated protected IN1 admission of the first
   execution-required checkpoint, linked to the previously admitted freeze checkpoint and the
   same original Event/P/O/enrollment with the bindings in rule 3. It is irreversible within the
   generation and supplies zero execution, adoption or support credit. Applicable attempts'
   lifecycle progression MUST be checked against authenticated IN1 admission history through
   the shared progression predicate, including admitted higher-stage ancestors with no producer
   run. History discovery MUST be independent of producer discovery, verify commit/tree/blob
   identities against authenticated anchors and exhaust relevant parent relationships from the
   generation anchor to candidate/protected roots. Predecessor references MUST cross-check that
   history. Relevant host populations and protected roots MUST be rechecked before acceptance.
   A descendant rollback or a new freeze-labelled head after readiness, including another
   PR, MUST NOT reopen reporting eligibility. Missing, conflicting, expired or revoked authority
   leaves dependent conclusions unverified and cannot restore a freeze window.
6. Every matching candidate without positively authenticated reporting authorization remains
   execution-required. Initial implementation and later execution/publication/proving attempts
   require ordinary execution authentication, even if unsuccessful, on another PR, before
   readiness, or never admitted. A later successful readiness admission MUST NOT exempt those
   attempts. Collection MUST neither invoke nor wait for its own packet admission. Structural
   freeze authorization MUST NOT require the execution receipt waived by rule 2, and readiness
   authentication MUST NOT become a prerequisite to the verdict authorizing that same admission.
7. Reporting classification waives only the support-execution receipt. Repository, run/attempt,
   original generation, head/tree, actual checkout, workflow/context, authority and observable
   outcome MUST still be authenticated. Unknown, nonterminal, cancelled, aborted or failed
   designated reporting/producer outcomes and failed collection prevent verification. A terminal
   successful designated reporting/producer outcome is required at the existing `rust-tests`
   scope. Whole-workflow completion remains governed separately before MERGE. If a designated
   checkpoint invokes substantive support execution or supplies execution evidence, the exception
   is invalid. The evaluator MUST retain and authenticate both passing and failing evidence and
   apply the existing counterexample/conflict rules. Unexpected passing checkpoint output cannot
   establish adopted support.
8. Historical checkpoint source authentication establishes only that checkpoint's identity,
   permitted delta and context. Every execution-required attempt MUST meet the existing complete
   protected-staging source approval, actual built/workflow closure, indexed bytes/modes,
   graph/configuration/compiled populations, toolchain/environment, provenance and current scoped
   input predicates. Source approval MUST be evaluated against independently obtained protected
   staging at evaluation time. Earlier execution at identical source S MAY qualify after S reaches
   that staging with all other predicates satisfied. An execution at S1 followed by current S2
   still fails exact source equality. A different historical execution closure cannot qualify
   through readiness.
   Independent adoption, due obligations, separate maintenance/onboarding generations and
   receipts, enrollment origin and currentness rules still apply. Collection MUST reuse the
   existing command and `rust-tests` job under the source-installed finite profile selection.
   CP-P5 consumer ordering, the terminal workflow/MERGE gate and the finite synthetic wrapper
   scope continue to govern.
9. Protected IN1 MUST emit a canonical versioned admission record after the required terminal
   producer dependency, owning/progression/freeze checks and per-root freshness loops, and before
   successful admission completion. The record MUST contain its schema/kind, repository and
   original Event/P/O/enrollment/generation identity, candidate and parent commit/tree,
   actual checkout commit/tree, evaluated source/context and head relationship, old/proposed
   lifecycle state, checkpoint kind, decision and bounded reasons, predecessor admission identities,
   approved IN1 source/workflow
   and complete relevant policy/dependency closure, observed protected roots/tips, accepted origin,
   registry anchor and other authoritative inputs, consumed context/evidence identities where
   applicable, and host workflow/run/attempt/job/check/event claims. Admissions without depth
   producers MUST be representable. The first record MUST bind an authenticated bootstrap or
   enrollment anchor. IN1 MUST retain reached refusal decisions. Failed publication, crashes or
   cancellation cannot establish successful authenticated admission. Admission records the
   decision over observed roots before integration. Subsequent protected lineage MUST be
   independently authenticated. Bounded reconstruction supplies a cross-check of the retained
   decision and MUST NOT replace it with a later re-evaluation.
10. The existing collector's trusted reporting path MUST emit a canonical versioned reporting-only
    context record before its own IN1 admission. It MUST contain schema/kind, repository and
    original Event/P/O/enrollment, host run/attempt/job/event/head/workflow and relevant
    dispatch/input identities, actual candidate-data checkout commit/tree and permitted head
    relationship, separately identified approved collector/policy source and relevant build/workflow
    closure, complete relevant source/context/configuration and required indexed/worktree
    observations, dispatcher mode, observation that the enrolled substantive path was not invoked,
    reporting outcome and candidate/predecessor references or digests. The claim is bounded to
    the approved dispatcher's isolated observation of its enrolled path. Context publication and
    terminal producer observation MUST precede independent IN1 checks/admission publication.
    The evaluator MUST join context, admission and protected history. The producer MUST NOT
    depend on its own future admission or require that admission's digest in its payload.
11. Each control record MUST have deterministic canonical bytes and a format-specific
    domain-separated digest, be published through the existing proof transport, and have a
    fixed-format versioned commitment binding kind, payload digest and returned artifact identity
    in the exact emitting job's host-retained log. The evaluator MUST independently authenticate
    approved policy/workflow provenance and the exact emitting job/run/attempt/event, using
    attempt-specific job enumeration and that job's log, then verify artifact identity and payload
    bytes. Same-name checks, aggregate-log matches and same-run artifact membership alone grant
    no authority. Missing, duplicate, ambiguous or conflicting commitments, wrong-job or
    cross-attempt replay and substituted bytes MUST prevent verification. Retries MUST have
    separately authenticated observations. Decision identities MUST be immutable. Conflicting
    decisions over one substantive `inputs_identity` MUST prevent verification. The key MUST bind
    candidate/parent/checkout objects, repository, policy, enrollment anchor, roots/tips/registry,
    evaluated blobs, normalized observed authority, checkpoint classification and generation.
    Predecessors MUST contribute a sorted set of commit/transition-digest pairs. Equivalent
    predecessor retries MUST yield the same key. Emitting session claims and predecessor record
    identities MUST be excluded. Every reached decision MUST participate, including quarantined
    decisions. Only complete records contribute frontier, readiness or progression authority.
    Observation-incomplete and history-deficient records MUST be quarantined and MUST permit
    healthy retries and other candidates to reconstruct. Reconstruction MUST fail only for genuine
    integrity failures, including object mismatches, history gaps, bound overflow, duplicate
    identities and contradictory reached decisions over one key. Different root snapshots and
    independently observed host authority MUST remain distinguishable. Revocation MUST affect present eligibility while
    preserving historical readiness and execution accountability.
12. Trusted emitters MUST be isolated from candidate-controlled processes that could alter their
    runtime, inputs, observations or publication. Protected IN1 MUST use a clean separate
    VM-backed hosted job executing independently approved policy/dependencies with candidate
    material as bounded data. The existing `rust-tests` reporting branch MUST be selected and run before any
    candidate-controlled code in that job, with exclusively trusted execution through publication
    and no concurrent candidate processes. Candidate code, builds/tests/helpers, local actions,
    packets, build scripts and candidate executable caches MUST NOT execute in trusted emitter
    runtimes. A second process/directory, later step, signature or approved workflow provenance
    alone cannot establish isolation. Required tests MUST be preserved through a reviewed layout.
    Phase C MUST prove that layout and transport, with a reviewed workflow-layout amendment if
    needed before activation. Collection continues to use the existing command/job and finite
    profile. CP-P5 read-only host credentials MUST NOT become signing authority.
13. Independently enumerated host streams MUST enforce 1,000-run and 100-attempt limits.
    Admission-history resolution MUST enforce a 1,000-unique-commit bound, and each canonical
    decision/context control record MUST be at most 64 KiB. Existing source/object/artifact bounds
    remain required. Exhaustion MUST yield `unverified` without truncating required discovery.
    Retention MUST be explicitly configured. Unavailable, expired or deleted required payloads,
    commitments, history or provenance MUST yield `unverified`. Verification requires continued
    availability of all required evidence.
14. Maintenance collection and discovery MUST use the same canonical binding resolver over the
    authenticated standing request and canonical versioned depth report at exact immutable host
    heads. It MUST validate supported structures/versions, agreement on original request/approval,
    frozen P/O, enrollment/agent and all existing binding fields, and re-derive values required by
    existing policy before enumerating matching attempts. Auxiliary fixture presence, absence or
    contents MUST NOT decide generation membership. Coherent records identifying another
    generation with no conflicting authenticated association receive out-of-generation treatment.
    A target binding in either record with the other missing, malformed, stale or contradictory,
    ambiguous report entries, or removed/replaced required records at a head independently
    associated through admission/enrollment history MUST yield `unverified`. Heads without an
    independent target-generation association retain ordinary unrelated-head treatment. Records
    supply authenticated input evidence. Reporting authorization requires the combined proof.

### IN1 admission control record v1

The isolated FR1 route uses an independently versioned `in1_admission` JSON object. Canonical
bytes are compact UTF-8 JSON with lexicographically ordered object members, decimal integers,
ordered arrays and no trailing newline. Parsing MUST reproduce those exact bytes. Unknown
members, duplicate members, alternate encodings, unsupported versions and payloads exceeding
65,536 bytes MUST refuse. The identity is SHA-256 over the literal UTF-8 domain
`support-depth-in1-admission-v1` followed by one NUL byte and the canonical payload. Generation
identity uses the separate `support-depth-in1-generation-v1` NUL-terminated domain over canonical
generation bytes.

The record contains the following fields.

- `schema_version` is `1` and `kind` is `in1_admission`.
- `repository` records the host name, numeric repository id and owner id.
- `generation` contains the original Event, complete selection/enrollment, P identity and
  optional O identity. `generation_sha256` binds its canonical identity. Both are null for
  record-free bootstrap/preparation.
- `candidate`, `parent`, `checkout`, `enrollment_anchor` and `protected_tip` each contain full
  commit and tree identities. FR1 emits `head_relationship` as `exact_head`.
- `old_state` and `new_state` contain lifecycle path, authenticated blob identity and stage when
  present. The blob preserves the complete evaluated lifecycle state.
- `checkpoint`, `decision` and `reasons` record the evaluated transition. FR1 emits `bootstrap`
  for the first anchored decision and `unclassified` thereafter. `request_freeze` and
  `execution_required` are reserved for subsequent owning predicates. Decisions are `admitted`
  or `refused`. Reasons are the bounded enums `owning_predicates_satisfied`,
  `owning_predicate_refused` and `observation_incomplete`, with at most sixteen entries.
  After initial input capture succeeds, an unavailable producer or isolated-host read before a
  substantive decision yields a refused decision with the sole reason `observation_incomplete`. This state is sticky through subsequent
  successful reads, including source object acquisition and retained history reads during
  predicate evaluation. Only these incomplete-only refusals are excluded from both sides of
  identical-input conflict comparisons.
  Once the owning predicates reach admission or refusal, the emitter MUST preserve that decision,
  its corresponding owning reason and the normalized authority facts consumed up to that boundary.
  Closing captured-context and binding rechecks MUST still run. Any failed closing recheck adds
  `observation_incomplete` after the owning reason, without replacing the decision or changing
  its substantive authority commitment. The only combined forms are admitted with
  `[owning_predicates_satisfied, observation_incomplete]` and refused with
  `[owning_predicate_refused, observation_incomplete]`. These reached decisions MUST participate
  in identical-input conflict comparisons. Any record containing `observation_incomplete` MUST
  NOT authorize readiness, predecessor frontiers, generation-history completion or progression,
  and its emitting job MUST conclude failure. A closing failure prevents invocation success.
  Reached contradictory decisions over identical normalized authority MUST refuse.
- `predecessors` is a sorted unique list of retained successful decision identities at every
  applicable parent frontier. Before invoking the owning policy predicates, IN1 MUST resolve
  that frontier from authenticated history and pin both the predecessor identities and checkpoint
  classification. An empty authenticated frontier selects `bootstrap`, and an existing frontier
  selects `unclassified`. Retention-time reconstruction MUST preserve those pinned values on
  both admission and refusal. A failed reconstruction adds `observation_incomplete` to the reached
  decision, retains its original evaluated-input identity and prevents invocation success.
  History MUST check parent-frontier coverage and Bootstrap classification. Any record with
  deficient, unavailable or superseded predecessor references or inconsistent classification MUST
  be quarantined in reconstructed history, for both reached admissions and refusals. Quarantined
  records MUST contribute no authority to frontier completeness, readiness, generation-history
  completion or progression. Their presence MUST permit reconstruction and healthy complete
  retries. Every reached decision retains its substantive-key conflict participation. The sole-reason
  incomplete-only refusal is exempt because no decision was reached. Late retries remain
  independently discoverable, and earlier retained predecessor references remain immutable.
- `policy` names independently supplied implementation/workflow provenance, workflow id, job key,
  app id and the approved closure. The isolated FR1 route resolves `workflow_commit` from the
  source-installed authenticated protected installation commit, independently of candidate JSON.
  `closure` records Git tree/blob ids for `crates`, `Cargo.toml`, `Cargo.lock`, `.github`,
  `.github/workflows/ci.yml`, `scripts`, `scripts/depth_ci_archive.py` and installed profile scopes.
  The evaluator rehashes bounded Git objects and requires each evaluated source object to equal
  the independently protected anchor's object, as producer closure approval does. Root Cargo and
  toolchain overrides remain unsupported. `implementation` is SHA-256 of canonical closure JSON
  prefixed by `support-depth-in1-git-policy-v1` and a NUL byte; it summarizes those Git identities,
  never uncommitted files or a compile-time directory inventory. Candidate-provided hashes cannot
  approve policy. Phase C must bind the deployed evaluator to that approved source and runtime.
  `canonical_tip`, `accepted_origin`, `registry_blob` and `inputs` bind observed authority and
  evaluated input blob identities.
- `observed_authority` is a lowercase SHA-256 commitment to normalized authoritative host
  facts consumed during this IN1 evaluation. Approved predicate code derives these facts from
  authenticated producer and isolated-host reads. Candidate files cannot supply facts.
  The normalized fact set contains the following data.

  - Applicable Event/P/O producer run/attempt identities, head commit, workflow path/id,
    repository and event, sorted canonically. Ignored out-of-scope runs contribute no content
    facts. The closing population comparison contributes an explicit discovery-stability boolean.
    The canonical profile includes the required enclosing terminal status. IR-P3
    uses producer job terminal state while the enclosing workflow may progress independently.
  - The unique packet PR identity and governing base, matching witness artifact identities,
    names, exact run/head bindings, expiry, size and digest, and downloaded witness-byte digests.
    Producer job identities, run/attempt, status/conclusion and the exact required collection,
    upload and binding step statuses/conclusions are included. Only the binding step's time
    interval is consumed. Log facts contain matching artifact-binding id/name/digest tuples.
  - Every accepted authorized adoption's endorsement kind, repository, PR/id, author login/id,
    accepted permission, source commit, mapping digest and digest of the exact referenced body
    bytes. Exhausted authorized-adoption populations explicitly encode an empty array for absence.
    Author login and current permission MUST be authenticated before any fallible adoption-body
    parsing or referenced-source/mapping processing. Unauthorized comments, including malformed
    adoption JSON, and ignored records contribute no content facts and MUST NOT truncate capture
    or alter discovery completeness. Ordinary authorized comments contribute no adoption facts.
  - Isolated and canonical repository ids/owners/names, protected staging identities and protection,
    baseline/candidate ancestry status and merge base, candidate host presence, governing registry
    path/type/blob, and canonical approved-scope path/blob/mode/type inventory with tree truncation
    and completeness. Source rows outside installed scopes contribute no content facts.
  - Each discovered producer, PR, artifact, job, comment and review stream records a start fact
    (`discovery`, `false`) and, only after bounded exhaustive pagination, a completion fact
    (`discovery`, `true`). Relevant population facts include empty arrays for observed absence.
    Comment/review stream completion is independent of ignored record counts and page boundaries.
    Before a substantive decision, transport-unavailable reads contribute only the fixed
    `["unavailable"]` availability fact and set `observation_incomplete`. After that boundary,
    failed closing reads set the separate incompleteness reason without salting the pinned fact
    set. Error text contributes no facts.

  Each fact hashes canonical `[endpoint, kind, SHA256(canonical(normalized_value))]` under the
  NUL-terminated `support-depth-in1-host-read-v1` domain. Population facts use the stream endpoint
  before pagination parameters, retaining its semantic query, and arrays sorted by canonical
  JSON bytes, preserving multiplicity. Unrelated workflow metadata, ignored log/comment text and
  population ordering preserve identity. Fact digests use lowercase hexadecimal. The sorted
  unique set of fact digests is encoded canonically and hashed under
  `support-depth-in1-observed-authority-v1` followed by NUL. The empty set represents an evaluation
  consuming no host facts. Facts are pinned when the owning predicates reach their substantive
  decision; closing observations track completeness separately. If no substantive decision is
  reached, capture ends before retention-time admission-history reconstruction and publication. Repeated reads
  and independent read order preserve identity. Existing host response, pagination,
  attempt, population and canonical encoding bounds apply, and the record retains its 65,536-byte
  bound. This required fixed-size digest participates in evaluated-input identity. Reached
  contradictions over the same repository inputs and normalized authority refuse, even when a
  reached decision also records failed closing observations. An incomplete-only observation
  retains a refusal that permits a later complete retry to recover.
- `producer_context` is nullable and `execution_evidence` lists consumed evidence identities.
  FR3 supplies the reporting-context association. An admission without a producer is supported.
- `claims` binds workflow run, attempt, emitting job, check and event to independent host facts.
  The invocation event MUST be obtained independently before policy evaluation and matched against
  the emitting session event before publication. The entire `claims` object is excluded from
  evaluated-input identity.

`inputs_identity` uses the NUL-terminated `support-depth-in1-inputs-v1` domain over canonical
record JSON after removing `claims`, `decision` and `reasons` and replacing `predecessors` with
its sorted unique array of `[commit, transition_digest]` pairs from independently authenticated
referenced records. A transition digest uses the NUL-terminated
`support-depth-in1-transition-v1` domain over canonical `[old_state, new_state]`, including null
states. Unresolved audit references mark history deficiency and contribute no authenticated pair.
Checkpoint classification and every other substantive field remain in the key. Complete and
incomplete reached decisions with an identical key MUST conflict when their decisions differ.

All other identity-bearing record fields MUST also be captured before the owning policy predicates
run. This includes authenticated root/policy snapshots, content-addressed candidate/parent/checkout
objects, lifecycle states, generation and its digest, authoritative input blob maps and execution
references. Explicit absence is permitted only after a successful authenticated source lookup.
Malformed present state/depth bytes and failed source reads MUST abort input capture. Such an
invocation cannot reach a substantive owning decision or publish a record with substituted nulls,
empty maps or default classifications. Its independently started attempt remains incomplete and
unverified. `producer_context` remains the explicit FR1 reserved absence. Normalized host facts
continue accumulating during evaluation and are pinned at the reached decision boundary.

After input capture, closing or emission failures MUST preserve every pinned identity field and
the reached decision. A retained reached-incomplete decision participates in conflict detection
against a contradictory complete retry over those same inputs, while supplying no successful
historical authority. Later parent-frontier changes can quarantine either decision in reconstructed
history while preserving its pinned inputs and substantive-key conflict participation.

The fixed commitment line has exactly this form, with a lowercase 64-character payload digest
and a positive decimal returned artifact id.

```text
IN1_ADMISSION_V1 kind=in1_admission sha256=<digest> artifact=<id>
```

It MUST appear exactly once in the approved job's retained log within its binding step interval.
Attempt-specific job enumeration MUST authenticate the workflow job key, app, check and attempt.
The downloaded proof artifact MUST contain exactly one regular `admission.json` entry. Its host
archive digest/size, canonical payload digest, provenance and association claims MUST agree.
The transport MUST independently stream-decompress the complete entry with a 65,536-byte expanded
limit, require completed decompression with no excess compressed data, and verify the actual
expanded length and CRC against the ZIP declarations. Local and central declarations MUST agree
when present. A declared size or CRC matching only a prefix MUST refuse.
When flag bit 3 is set, the required data descriptor, with or without its signature and including
ZIP64 sizes, MUST match those declarations and actual payload values. Missing, truncated or
inconsistent descriptors MUST refuse.
Required publication precedes successful completion. Upload/log failure leaves an incomplete
attempt. The fake host models these facts independently of producer artifacts.

Independent discovery permits at most 1,000 runs and 100 attempts per run in each host stream,
1,000 unique commits per resolution and 64 KiB per control record. Every relevant parent is
traversed to the authenticated enrollment anchor, with commit/tree/blob content hashes checked.
Complete populations and protected roots are checked again after resolution and at consumption.
Generation-history completeness MUST cover both protected lineage and every candidate ancestor
through all parents back to the authenticated anchor. Within that generation, an ancestor whose
lifecycle stage differs from any same-generation parent's stage, or first introduces the generation,
MUST have an authenticated retained `admitted` decision for that commit and generation. The
enrollment anchor at or below `enrolled` remains the bootstrap exception. Absence of that decision
is missing authority and MUST yield `unverified`, including for an unmerged ancestor whose run
disappeared from an otherwise complete listing. Ordinary ancestors preserving the generation and stage require no
admission. A pending candidate's own transition requires no future admission in this history check.
Overflow, gaps, missing predecessors, conflicts over identical evaluated inputs or unavailable
required publication yield unverified authority. Revocation removes present eligibility and
preserves historical progression. These are credential-free modeled-host predicates. Deployed
GitHub policy provenance, emitter isolation, retention and transport qualification remain phase C.

## Synthetic wrapper machinery-proof profile

The maintainer approved MP-P1 in the
[machinery-proof rescope](../agents/lifecycle/support-depth-machinery-proof-rescope.md) and SA-P3 in
the [synthetic subject design](../agents/lifecycle/support-depth-synthetic-subject-design.md) on
2026-11-02. This amendment adds one more compiled isolated profile, `synthetic-wrapper-v1`. It
proves the maintenance machinery on a synthetic subject. It is a machinery proof only: its
results qualify no real agent, are never reused for Codex or any other agent, and are labeled
machinery proof wherever they are reported.

**Subject.** The agent is `depth_fixture`, a non-runtime registry entry under the
[registry contract](agent-registry-contract.md#runtime-and-non-runtime-entries). Its one maintenance
enrollment is upstream version `0.2.0` (the baseline release is `0.1.0`) on target
`x86_64-unknown-linux-gnu`. That is both the release asset's tuple and the wrapper build host, so no
cross-target evidence arises and no IR-P2-style mapping is needed. It
has one headless mode, selector `run-json` with command path `depth_fixture run` and the presence
predicate `--json`, and one operation, `run`. That operation selects template `CORE` with
`shared_mapping = wrapper_only` and `forwarding = false`. It permits no not-applicable outcome and
maps no capability. The due obligations are BASE (R1, V1, G1, E1, A1, X1-X4), OUT3 and LC2. A
wrapper-only operation creates no adapter obligation, so M is not due and no `agent_api` source is
in scope.

**Evidence.** Each due obligation maps to substantive cases in `crates/depth_fixture`. These are the
wrapper tests against the fixture CLI's scripted scenarios. The cases run the fixture binary built
from reviewed source at the enrolled version. The digest of the acquired release asset is provenance,
as the upstream snapshot is for CORE. The frozen contract, P/O, case mapping and resolved build
profile live in one fixture directory owned by this profile. The profile's names are distinct from
CORE's:
- trust revision `github-actions-synthetic-wrapper-v1`;
- executor `github-actions:rust-tests/isolated-synthetic-wrapper-v1`;
- artifact `depth-synthetic-wrapper-v1-<run_id>-<attempt>`;
- receipts under
  `cli_manifests/depth_fixture/reports/0.2.0/evidence/synthetic-wrapper-v1/<run>-<attempt>/<obligation>/`.

`maintainer-mapping-v1` stays the adoption revision, and the same `depth-adequacy-v1` body format
applies. Each of the eleven due obligations has its own content-bound mapping that needs
independent adoption. Partial adoption cannot qualify the profile.

**Producer.** IR-P3 producer terminality applies: authenticated evidence may be evaluated once the
existing `rust-tests` producer job is terminal and its collection, upload and binding steps and
immutable artifacts are authenticated. Every applicable attempt counts, and the complete pre-MERGE
workflow gate still governs. Collection reuses the existing `rust-tests` job. No new job, required
status check or service is added.

**Native context.** Under MP-P2, NE-P1/P2 and NC-P1/P2 are Codex qualification rules and are not part
of this profile. The fixture CLI reads no instruction file, configuration, cache or credential. X2
is shown by cases that pass a cleared environment, an explicit working directory and null stdin.

**Selection.** A subject is chosen only by a compiled profile, never by a registry declaration,
receipt, remote, environment value or flag. A Codex declaration does not satisfy the synthetic
subject, nor the reverse. Extra declarations, versions, targets, modes, mappings, retained roots or
not-applicable permissions refuse. The live isolated profile stays uninstalled until roadmap C
authorizes the disposable repository, its baseline and credential references. The canonical
registry never lists `depth_fixture`. `production_results` stays unverified.

**Staging.** Each step is a reviewed PR into `staging`:
1. Compiled subject selection in the isolated binding.
2. The profile definition, its frozen fixtures under `crates/xtask/tests/fixtures/depth-synthetic/`,
   the case mapping to the single `depth_ci_synthetic` test target, and the local
   `depth-synthetic-collect` command. The collector builds the fixture with
   `DEPTH_FIXTURE_UPSTREAM_VERSION` set to the enrolled version and records that variable. The
   whole-workflow verdict path refuses this profile.
3. The collection, upload and binding steps in the existing `rust-tests` job, named
   `<Collect|Upload|Bind> isolated synthetic depth <witness|witness|artifact>`, and the profile's
   agent id in its evidence scope.
4. A read-only IR-P3 verdict for this profile. It reuses the CORE evaluation over CI evidence. It
   reads only the repository named by an evidence host, which only a resolved IR-P1 context for
   the synthetic subject can mint; with the live profile uninstalled, there is none. The host
   keeps the authenticated repository and owner ids, rechecked before and after discovery, so a
   renamed and replaced repository is refused. The repository's protected staging tip must equal
   the binding's. PR, run, producer and endorsement
   repositories are that repository. An attempt is admitted once its `rust-tests` producer job and
   its collection, upload and binding steps are terminal and successful, even while the workflow
   runs. A cancelled or unconcluded producer job stays unverified; a job that failed after its
   evidence steps succeeded still counts. The population recheck compares run identities (id, attempt, head, path, repository,
   event), not unrelated progress. Channel and CORE keep the canonical repository and whole-workflow
   completion. It ships positive and negative evaluation tests in the shape of the CORE bridge. The
   NE/NC native producer (`depth_evidence/verdict/native*`) is Codex qualification under MP-P2 and
   is not used.

## Synthetic onboarding proof selection

The maintainer approved OF-P1 to OF-P9 in the
[onboarding freeze decisions](../agents/lifecycle/support-depth-onboarding-freeze-decisions.md) on
2026-11-05. This bounded amendment is that document's slice 0. It adds one onboarding selection
for the synthetic subject, in the isolated workspace only, so that later slices can prove the
onboarding machinery there. It is a proof workspace, not onboarding path enablement, which still
needs the support-depth contract's "Path enablement" items 1, 3, 4 and 5 and per-agent
authorization. Like the maintenance profile above, it qualifies no real agent.

**Subject and selection.** The subject is the same compiled `depth_fixture` subject, with the same
version `0.2.0`, target `x86_64-unknown-linux-gnu`, headless mode, `run` operation, `CORE`
template, `wrapper_only` mapping, no forwarding and no not-applicable permission. Only the
lifecycle path differs. The isolated registry's `enabled_paths` is exactly `["onboarding"]`, and
its one enrollment states `lifecycle_path = "onboarding"`. The Event is the onboarding Event
(`approval_commit` and `approval_recorded_at`), taken from the approval artifact whose
`[support_depth]` table states `0.2.0`. The approval may state either maintenance mode. The fixture
templates state `explicitly_deferred`, with no `maintenance.release_watch` in the registry entry
(OF-P3 A).

**One path per workspace.** An isolated workspace holds either the maintenance selection or the
onboarding selection, never both. `enabled_paths` names exactly one path, and the one enrollment
is on that path. Every retained depth record carries the Event of that path. Two paths, an
enrollment on a path that is not enabled, and a retained record of the other path all refuse. The
maintenance selection above is unchanged.

**Compiled scope.** Only the synthetic subject admits the onboarding path. The Codex compiled
subject stays maintenance-only. No declaration, approval, receipt, remote, environment value or
flag selects onboarding scope, and every other agent, version, target or path refuses. Production
installs no isolated profile, so no route exists. The production registry keeps refusing
`support_depth` tables, and the canonical registry never lists `depth_fixture`.

**Grants nothing.** The selection admits a workspace and nothing more:
- the `synthetic-wrapper-v1` producer, receipts and verdict above stay bound to the maintenance
  Event, and an onboarding generation's evidence stays unverified until the separately bound
  onboarding producer variant specified below is implemented (OR-P5);
- maintenance routes refuse an onboarding enrollment, as they do today;
- ON1 to ON5 keep refusing depth until roadmap E.6 wires them, in the isolated workspace only;
- the create lane gets no packet freeze, pointer, `validated` or `supported` route.

**Bootstrap and first declaration (specified, not installed).** OR-P1 and OR-P14 of the
[routing design, "Maintainer decisions" and "Route composition"](../agents/lifecycle/support-depth-onboarding-routing-design.md#maintainer-decisions)
extend only this source-installed synthetic route with these bounded states:

Before declaration only, the one-enrollment conditions above give way to the record-free states
below. After declaration those conditions apply in full; bootstrap grants no resolved enrollment
or depth acceptance authority.

1. Before enrollment, the subject entry/root may be absent, or the entry may be the exact
   unenrolled scaffold. The expected descriptor comes from compiled synthetic templates and the
   exact approval is bound; neither declares enrollment. No retained record or acceptance is
   admitted. ON1 MUST validate its proposed registry with the ordinary parser and MUST write no
   `support_depth` table. Bootstrap IN1 admits only the exact absent-to-scaffold registry delta,
   independently compared with the protected registry; captured registry bytes in a scratch view
   MUST still remain identical.
2. Between integrated ON1 and the first declaration, record-free preparation checkpoints may
   land reviewed Annex-B governing adoption, actual direct acquisition, union/coverage/report
   generation and ordinary NE3 `snapshotted` metadata. Each MUST preserve the exact scaffold
   registry entry and approval identity, add no depth table, record, entry or acceptance effect,
   and receive IN1 admission followed by post-tip observation. Metadata MUST describe actual
   acquisition and precede declaration; ON1 invents no metadata and NE3 gains no authority after
   declaration.
3. The scaffold-to-declared commit is a separate maintainer-owned reporting transition. IN1 MUST
   admit exactly the onboarding `support_depth` table for the scaffolded entry and approval
   version, reviewed CORE classification rows, the integration-branch acquisition statement
   (OF-P4 B), and any coherent reporting publication changed by the declaration, with no record,
   entry or acceptance effect. Only after admission and protected-tip integration may the route
   capture that declared registry; protected-registry equality then holds again. Advancing the
   tip first is not admission. Only ON1 and this transition have registry-delta exceptions. ON2
   MUST still refuse unresolved enrollment and can freeze only after the charter's prerequisites.

**Finite approval adapter (specified, not installed).** The private projection shared by routed
ON1–ON5 and IN1 MUST validate the complete compiled synthetic descriptor and approval, rejecting
unknown or misplaced fields and any fake backend (OR-P2,
[routing design, "Maintainer decisions"](../agents/lifecycle/support-depth-onboarding-routing-design.md#maintainer-decisions)).
Fixed subject/version/target, expected descriptor, producer case mapping and offline-executor
selection belong in the compiled profile/private adapter. Shared orchestration and admission
operate on validated path/Event/subject inputs; neither `depth_fixture` nor Codex identity waives
ownership, adequacy, currentness, effect admission, integration or delivery checks. Production
approval acceptance is unchanged; no second subject is added.

**Onboarding producer binding (specified, not installed).** OR-P5 of the
[routing design, "Maintainer decisions" and "Inspection A — producer population and CI terminality"](../agents/lifecycle/support-depth-onboarding-routing-design.md#inspection-a--producer-population-and-ci-terminality)
adds a finite onboarding variant of `synthetic-wrapper-v1`, reusing the fake CLI, substantive
cases, target, eleven due obligations, revision-1 receipt schema, adoption format and IR-P3
producer terminality policy. The compiled fixture selection, collector/archive/verifier and
fixture identities MUST change together. Its built fixture and witness MUST bind the actual
approval Event and frozen onboarding P/O. Authentication MUST resolve the approval-linked
generation PR and independently discover the complete applicable attempt population, then
evaluate only matching receipts. Maintenance and onboarding Event/P/O populations remain
separate: maintenance receipts MUST NOT be relabeled or reused for this positive; onboarding
needs fresh evidence and independent adoption of every due mapping.

Initial implementation and later publication/proving runs on the same Event/P/O, including all
retries, remain applicable even on another PR or without a witness (OR-P5, same design section).
The exact pre-implementation freeze is subject to the shared
[request-freeze checkpoint rule](#authenticated-request-freeze-checkpoints-and-implementation-readiness);
ON2 implementation remains execution-required, including unsuccessful candidates before readiness.
Missing, unfinished or aborted producers, failed evidence steps, revoked adoption, changed source
or dependencies and incomplete/changing populations cannot qualify. Failed attempts remain
observable. Terminal authenticated `rust-tests` producer evidence may qualify while the enclosing
workflow runs; whole-workflow completion and the terminal gate still precede MERGE. Collection
uses the existing `rust-tests` job, with no new command, job or live profile. The producer MUST
neither invoke nor wait for its own packet admission; CP-P5's consumer ordering remains unchanged.
This can qualify only the synthetic wrapper promise, never a runtime adapter or real agent.

**Templates.** `crates/xtask/tests/fixtures/depth-synthetic/onboarding/` holds the isolated
registry-entry variant with deferred maintenance and the approval template. It also holds a
template acquisition statement at `governance/depth-acquisition/<version>.toml` (OF-P4 B), which
lists each acquired input that O binds (OF-P6). The statement's fields are not normative until the
onboarding charter defines them (slice 1), and nothing reads it until slice 2.
