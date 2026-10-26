# Isolated packet intake readiness design

Status: PI-P1, PI-P2 and PI-P3 approved by the maintainer on 2026-10-25.
The first bounded read-only intake is implemented; complete PI-P3 declarations and positive proof
remain separate work. No live profile or production enrollment is installed.
The follow-up [complete pilot policy proposal](support-depth-complete-pilot-policy.md) presents
exact initial/resume/managed-fork declarations, remaining dispositions and pending policy choices.
Baseline: staging `43a42bbcbc51ce2d21e5144a091d2bde5bac760f` (PR #298), 2026-10-25.
This is sequence2 of the approved [isolated route design](support-depth-isolated-route-design.md).
The [depth contract](../../specs/support-depth-contract.md),
[registry contract](../../specs/agent-registry-contract.md#support-depth) and
[evidence contract](../../specs/support-depth-evidence-v1.md) remain normative. The bounded implementation adds paired optional per-unit policy to the existing
registry/P schema, preserving old omission bytes. It changes no committed enrollment, snapshot,
writer admission, workflow, host profile or production enrollment.

## Outcome and source facts

Prepare truthful read-only intake for the actual Codex0.156.1 headless `codex exec --json`
selection on acquired `x86_64-unknown-linux-musl`; GNU remains the wrapper build/execution host.
A plan is neither a readiness verdict nor permission to write. Reuse the private context and
existing GitInputs/Source/Proposal ownership and freshness checks. The live profile stays None.

The [exact classification appendix](support-depth-packet-intake-classifications.md) reconciles all
54 selected identities and all14 acquired input files. Original per-target flags normalized out of
the union remain included. There are30 explicit approved assignments and24 unresolved identities;
none are execution evidence. The population includes the wrapper-only `--full-auto` claim and the
TUI-root `--no-alt-screen` observation. Name-level exclusions and unsupported/passthrough levels
remove nothing from depth accounting.

The approved design's baseline could not resolve this packet:

- `support_depth.rs::resolve` refuses every predicate before acquisition interpretation.
- `acquisition::derive` already validates schema, complete shards/union, wrapper claims and reports,
  but marks every selected unit classification_required; it has no reviewed per-unit assignments.
- `OperationDeclaration`/P has operation-wide templates and BASE promise strings, but no structured
  coverage of individual units or binding to the acquisition bytes used for their classification.
  Clearing O's unresolved list or treating those strings as a classification language would bypass
  classification rules3/4/6. The current P representation is insufficient for this proposal.
- `isolated_maintenance/scope.rs` admits only the proposed one-run/CORE scope, and no live profile
  is installed. A future ADMIN or multi-flow declaration requires a reviewed fixed-scope source
  change; schema support must not silently broaden this authority.

The first implementation now derives the exact acquired presence predicate and semantic basis from
one validated GitInputs view, freezes optional classification rows in P, adds their templates to
P/O and retains all missing rows as unresolved. FreezePlan exposes the basis hash and facts for
registry authoring. Classification-bearing policy is conservatively limited to one exact mode and
target. Newly supported predicate/per-unit intake refuses mapped operation selections, including
P-only, until complete flow policy is separately reviewed and implemented. The real-data test is
an unmapped initial-surface diagnostic, not a declaration that relabels the actual adapter run.
Legacy predicate-free/field-absent mapped fixtures retain their existing unqualified semantics. Currentness uses the same resolver/derive path
for preparation, refreeze, closeout and retained promotion. The real-data regression proves54
selected identities,30 approved rows,24 unresolved rows and retention of56 historical uplift rows.
No E or acceptance is synthesized. The fixed isolated scope explicitly refuses the new policy
fields until separately reviewed complete declarations authorize that scope.

The real packet also exposed an unrelated-command alias blocker in the old intake: `app-server
generate-ts` uses `-p` for `--prettier` while root uses it for `--profile`; plugin commands use `-m`
for `--marketplace` while root uses it for `--model`. Local duplicate/name/alias validation still
covers every command. Inherited-root alias coherence now applies to exact selected command paths;
those unselected nested commands neither invalidate `codex exec` nor join its selected population.
Selected JSON alias/arity conflicts, including normalized-union conflicts, still refuse.

Two facts change the positive-proof plan. The public wrapper streaming path always supplies a
last-message file, even when callers omit one. PI-P1 recommends explicit ADMIN mutation obligations
for that output effect and worktree creation. Separately, the production
[adapter harness](../../../crates/agent_api/src/backends/codex/harness.rs) dispatches initial exec,
exec resume, and app-server fork. Under the existing complete-mapping rule, a proposed future
`agent_api.run -> [run]` classification cannot qualify that capability from the initial path alone.
The currently committed production registry contains no depth enrollment; this identifies a gap
in the future proposal, not an installed false qualification.

## Decisions and recommendations

| Id | Recommendation | Consequence |
| --- | --- | --- |
| PI-P1 — exact assignments | Approve the appendix's30 explicit assignments, including CORE+ADMIN for output-file/Git mutation and denied/no-effect CORE treatment for exec remote intent; keep24 U-* identities unresolved | No blanket CORE inheritance, REMOTE claim from accepting an address, headless EXCLUDED shortcut, or acceptance from forwarding. The operation remains not ready while a selected unit is unresolved. Independent adequacy still proves each promised boundary |
| PI-P2 — minimal P representation | Add only `classification_basis` and `classifications` to the existing operation entry, with the finite row format below, frozen in selection/P | Maintainer-owned registry remains the sole policy inventory. Changes to classified semantic facts invalidate the reviewed assignments and require an explicit P update/refreeze. Missing rows remain unresolved; there is no runtime approval file, free-text classifier or executor decision |
| PI-P3 — truthful mapped scope | Retire the assumption that the first complete mapped proof has only one CORE operation and twelve obligations. Plan the actual initial/resume/fork flow set, add the effect-driven templates and corresponding evidence, and review its exact declarations before a positive packet freeze | Initial needs the PI-P1 mutation promise; resume needs SESSION; the adapter-owned app-server fork needs managed-protocol/session review. Do not relabel a mapped operation wrapper-only, omit supported flows, weaken the complete-mapping rule or reuse the old twelve-case adoption. Intake plumbing can proceed read-only; incomplete policy must refuse a real freeze, not appear as valid P-only policy |

Approved PI-P3 authorizes a direction only, not unnamed additional enrollment rows or evidence
mappings. The follow-up must present the exact complete declarations and additional selected-unit
classifications before enabling that scope. No new template, NA, override, debt authorization or
capability advertisement is recommended. The previous CORE-only diagnostic fixtures keep their
existing finite scope, trust binding, targets and whole-workflow-completion behavior.

## Finite classification rows inside P

The implemented optional operation members are:

- `classification_basis`: one SHA-256 of the finite semantic fact projection described below,
  hashed with the existing B.7/part_identity canonicalization. O still references all14 complete
  raw inputs; this digest does not replace or weaken those file references.
- `classifications`: ordered declaration rows `{surface, templates, rationale}`. `surface` is the
  existing SurfaceIdentity triple, `templates` explicitly assigns one or more existing non-EXCLUDED
  templates, and `rationale` states the reviewed effects/default/value boundary. No expressions,
  callbacks, arbitrary conditions, approval booleans or second classifier service.

Each row belongs to its owning operation and the version/targets/modes that operation serves.
Duplicate, foreign or unowned surfaces, unsupported templates and empty rationale refuse. A row
covers the unit's whole acquired value/default regime; a finite alias normalization is not a
license to approve arbitrary values. If that whole regime is not reviewed, omit the row and retain
classification_required. Do not encode partial value approval in the rationale string. Future
value-specific selection needs a separately reviewed representation, not a substring convention.

The basis is not a second snapshot framework. Derive it from the one validated intake view as
`{agent, version, target, binary, features?, known_omissions?, rules_sha256, schema_sha256, units}`. `binary` contains
selected-shard semantic_version, target_triple and sha256. `features` retains the complete selected
shard feature regime. `known_omissions` retains its complete payload and absent/present distinction;
unknown/unprojectable payloads and numeric semantic payloads unsupported by B.7 refuse. RULES and schema use their complete content digests. `units` is the sorted
complete54-identity selected population, with each original selected-shard and normalized union
row payload, provenance kind and applicable target sets, plus the owning wrapper claim payload
and its scope/level where present. Include observed arity, aliases, usage/notes and every other
supported row field, even where it is not yet semantically understood; never silently drop a new
field. Keep root and command-local identities distinct. Reuse existing schema validation and
canonical sets; no generic semantic-diff engine or network lookup.

Only acquisition bookkeeping outside these row/binary/feature facts is excluded: snapshot
collected_at, report generated_at, producer wrapper version labels, and union input file digests
(O binds those raw references; selected-target semantic facts are independently bound above). Reports are fully validated
recomputations, not independent classification facts. Any unknown schema/row field or unavailable
projection refuses. A timestamp-only regeneration keeps this P basis stable, while changed raw
refs still change O and invalidate E unless explicit reuse is admissible. A changed binary,
feature regime, rule/schema, selected unit/alias/arity/default/claim payload or applicability changes
the basis and needs maintainer review/refreeze; facts not captured in acquisition are not certified
by this projection. Bound configuration/default semantics remain required by BASE and any missing
classification. Additions cannot disappear merely because a previously approved row set lacked them.

The alternative is storing14 exact raw references directly in P. That is easy to implement but
would require policy refreeze on report timestamps alone; the single existing canonical-hash
projection avoids that repeated ceremony without a new policy store.

P freezes the declaration exactly, including these rows, their order and input identities. O keeps
its existing surface population; its operation obligations are BASE plus the union of its declared
parent templates and its explicit per-unit templates, with M supplied by complete capability
mappings. All resolved templates appear in P's existing template table. A row cannot replace or
remove parent obligations, M, retained selection or authorized debt. This is additive classification,
not the currently unsupported override mechanism. CORE+ADMIN adds OUT1/LC1/FX1 to OUT3/LC2.

For each selected surface, a matching row may resolve classification only when its full input
basis matches. An absent row stays classification_required. A stale semantic basis refuses rather than
silently reusing assignments; present malformed policy/acquisition is never downgraded to absence.
Acquisition fields do not describe every hidden alias/value/default: these uncertainties remain
explicitly unresolved. Source-informed maintainer assignment is not independent adequacy or A1
proof that a tag's source produced the recorded binary. Neither is inferred from coverage level.

Implemented compatibility: both members are optional and absent on existing declarations; preserve
omission with optional fields rather than serializing new empty defaults into old records. Explicit
lists, where present, are copied in declaration order under B.7. New readers must preserve existing
v1 P/O identities and bytes where these fields are absent. Old strict readers fail closed on new
fields. Amend the registry and record-selection contracts in the implementation PR; no schema
revision is changed by this design. This representation implements existing classification rules;
if implementation changes those rules or cannot preserve old canonical identities, bring that
specific governing/format revision decision back instead of silently migrating records or evidence.
Old records do not gain classification authority because the new fields are absent.

## Intake and freeze flow

1. Capture the actual caller once. Resolve the installed isolated authority before any supported
   writer can use its scoped registry. Keep ordinary loading and other route refusals unchanged.
   Missing live profile refuses before host calls. Pure diagnostics carry no authority.
2. Read the declaration and retained roots with the existing scoped loader. Resolve structural
   selection, RULES target authority and nonshrinking retained coverage. Missing/incomplete mapped
   policy refuses. A future fixed-scope change must explicitly authorize PI-P3's approved selection.
3. Extract one private read-only acquisition view from the existing validation half of `derive`:
   complete schema/shards/union/reports/claims, original per-target units, file references and alias
   facts. No second snapshot framework; use the same GitInputs observation for policy and O.
4. With complete acquisition, resolve the exact command/mode/predicate from those facts. Interpret
   `{flag="--json"}` only as a present, zero-arity canonical flag. Validate arity/alias consistency
   on the selected target and normalized union; unknown or conflicting alias/value/default regimes
   refuse. No `--json=false`, guessed short alias, root/child precedence guess or first-match rule.
   This is declaration/acquisition interpretation, not a general argv parser. Full invocation
   grammar and boundary evidence remain G1/R1 obligations.
5. Normalize only acquired long/short aliases before selector matching, preserving original name
   identities in O. The source-only `--experimental-json` alias is not an acquired fact; refuse it
   until a reviewed acquisition extension binds it. Do not insert a hardcoded alias inventory or
   fetch upstream source at runtime. Distinct selectors with unresolved overlapping requirements
   refuse; actual resume/fork/review command paths are never matched by a parent-prefix shortcut.
6. Match the classification semantic basis and exact per-unit rows, derive the full O obligation
   union, unresolved list and work using that same view. Verify the coherent inputs after deriving.
   Currentness must recompute the same P/O through prepare, closeout and promotion, including
   retained closeout after working-request replacement. Any semantic input/mode/inventory change
   changes the relevant identity and requires refreeze/new E or admissible explicit reuse.
7. Preserve Proposal expected-before writes and original caller checks through scratch and apply.
   A read-only plan is not a serialized write ticket. Public acceptance stays refused until the
   later producer, route and IN1 increments supply their independently owning predicates.

Only support the exact single presence predicate for this first intake implementation; other
predicate grammars/exclusions remain refused. Keep the no-acquisition P-only path truthful: when
all version acquisition is actually absent and structural policy is valid, freeze P only, no O/E,
no readiness or acceptance. Retain existing refusal when targeted carried work cannot be durably
preserved in a P-only replacement. Partial/malformed/unreadable acquisition refuses. Unknown
classifications in a complete acquisition yield a bound O with unresolved units, not a false ready
state. Missing complete mapped policy is invalid and cannot use P-only as a fallback.

## Carried work and real proof

The old Codex0.156.1 request retains56 uplift rows; same-version Events carry them forward, and
prior O work is also retained. The earlier ordinary audit/plan showed a plausible zero-current-work
ordinary revalidation with exact old-request/closeout correspondence. It applied nothing. A protected
ordinary bootstrap delivery and closure, followed by the actual depth P/O, remain required proof.
Do not reset the baseline, erase/relabel work or treat the old ordinary closeout as depth evidence.
If actual work remains nonempty, return its exact semantic corroborator requirement; intake approval
does not authorize weakening that guard. New classifications are also not semantic uplift evidence.

## Small implementation sequence and acceptance

The first implemented sequence adds the optional P representation and one coherent read-only intake
with exact --json/recorded alias handling and invalidation, keeping all unresolved rows visible.
Do not claim that this alone produces a valid complete mapped packet. Review the PI-P3 declarations,
remaining classifications and actual carried-work path before installing positive packet policy.
Then connect guarded preparation/refreeze using that complete policy. Packet producer/IR-P3,
public routes/IN1 and actual isolated chain proof follow separately. No background jobs, new CI
framework, extra required check, host provisioning or production rollout is part of this design.

Meaningful implementation cases must include:

- Exact54-unit reconciliation, wrapper-only and unsupported/passthrough units, normalized-out original
  shard flags, every retained root and target authority; other targets cannot borrow musl evidence.
- Canonical presence and recorded aliases; wrong arity, hidden/unbound aliases, overlap and partial
  acquisition refuse. No backend success synthesized from matching a mode name.
- One valid classified fixture plus missing/foreign/duplicate rows, changed/default/input facts and timestamp-only regeneration,
  stale basis, explicit P refreeze and changed O/E invalidation; nothing silently clears unresolved.
- ADMIN mutation due on the output path actually supplied by streaming, including its default temp
  path, and worktree selection; denied remote behavior has no connection/effect or credential leak.
- Complete adapter mapping controls: initial-only cannot qualify run; resume/fork obligations stay
  due. No wrapper-only relabel or generic arbitrary-argument qualification.
- True absence P-only versus malformed/partial acquisition and nonempty carried work; repeat/new
  Events, retained O work and ordinary callers preserve their current semantics.
- Old v1 canonical identities/receipt fixtures remain unchanged when optional fields are absent;
  fixed-profile scope widening cannot be induced by an input row or runtime flag.

Every implementation uses the existing12 local gate groups and both GPT-6.1 Sol High review lanes
at standard/default speed. For this documentation-only design, reconcile source/data identities,
check links and hygiene, and obtain both frozen-source reviews. No Rust behavior changes or broad
Rust gate reruns are claimed here. Policy approval and subsequent evidence adoption remain separate.
