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
