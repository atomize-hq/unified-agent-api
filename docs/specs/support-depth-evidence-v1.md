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
the initial profile. Local relay/gate output remains diagnostic. The profile MUST identify the
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
witnesses, changing totals/populations, duplicates or the 1,000-row/100-attempt bounds refuse
verification. Foreign-repository heads are outside this producer profile. The approved collector
records checkout HEAD; a PR checkout must be the host head or a two-parent test merge whose second
parent is that head. Workflow implementation and checkout commit agree. The adapter confirms
source/tree, reviewed implementation, graph/configuration and current scoped inputs separately.

Successful collection/upload/binding steps in the exact attempt's `Rust tests (workspace)` job
must expose exactly one timestamped `DEPTH_ARTIFACT_BINDING=` JSON log record within the binding
step. Its `id`, `name`, and `digest` agree with immutable API artifact metadata and the downloaded
ZIP's SHA-256 and size. A failure elsewhere in that job does not erase a faithfully collected
counterexample. All applicable attempts must agree on environment and content-addressed toolchain
report; a favorable run never selects those current inputs by itself.

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
artifact processing the adapter approves the independently authenticated run/attempt head's
producer/workflow closure as well as the separately checked actual checkout. A witness-provided
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
and actual workspace/Git ownership. It binds only Codex0.156.1 maintenance `run`/CORE and
`agent_api.run` on the acquired `x86_64-unknown-linux-musl` tuple. The GNU Rust wrapper build host
is distinct evidence metadata. Registry declarations remain necessary and every registered retained
root is mediated; unsupported or extra selections MUST refuse. Neither this context nor its
read-only diagnostic is an acceptance verdict or proof of OS isolation.
The isolated registry is separately checked governing data, with coherent bytes, exact scope and
freshness retained. The approved isolated declaration must already exist on the isolated protected
baseline; resolving a context does not install or edit it. Source approval MUST NOT require enabling
the canonical production registry to admit the isolated declaration, nor treat arbitrary
governing-data edits as approved code.

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
