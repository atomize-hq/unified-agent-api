# Native context guards and finite observations

This implements the first context slice of the approved
[NC-P1/NC-P2 closure plan](support-depth-native-context-coverage.md). The
[v2 contract](../../specs/support-depth-native-evidence-v2.md#finite-native-context-observations)
is normative. It installs no live profile, classification or adequacy mapping and makes no native
model call. The later case-coverage and owning-path work remains separate.

The packet producer runs the existing collector with internal `--context-guard`. This adds local
eligibility checks before the native version probe and before/after each case. It preserves the
selected existing Codex home and normal authentication resolver. It neither clears the native
environment nor copies, inventories or hashes credentials. An explicit selected home is also
forwarded to the guarded version probe. Ordinary native diagnostics without this flag and ordinary
SDK consumers keep their existing behavior.

Local checks bind the selected executable to the pinned acquisition before probing and validate canonical distinct home/project roles, supported inherited environment
channels, bounded known nonsecret config and absence of foreign loader inputs. Unknown system,
OS-home/XDG/project inputs refuse with finite codes. Recognized boolean feature settings must remain
explicitly false; native effective values may differ and remain to be measured. Known pinned bundled
system-skill files are checked against the existing offline acquisition's exact names and digests;
extra files, symlinks, incomplete inventories and changed bytes refuse. This checks permitted asset
presence, not skill execution or complete loader behavior. Relevant nonsecret environment values and authentication-channel presence are bound in memory;
changed values or removals refuse. Credential values and reviewed non-effect channels are excluded,
so auth refresh does not invalidate the nonsecret snapshot. Context/config/role identity drift refuses; these checks are not an OS sandbox
or an atomic guarantee against concurrent changes.

The wrapper emits private finite tracing events after final command overlays and after successful
spawn on the actual streaming exec/resume and app-server paths. The standalone collector owns one
exclusive global subscriber, so observations from Tokio tasks are included. A private per-process
owner token prevents late or foreign responses from being attributed to another case. No public observer API
or raw command, argv, environment, config/layer, protocol or error trace is added. Existing native
config/requirements responses are projected before serialization. Actual thread/start, resume and
fork responses now supply partial model/provider, approval (including all granular flags), reviewer,
sandbox-mode, cwd/root and instruction-source predicates. Actual turn/start responses separately
record acceptance. Missing fields remain unknown, and neither response establishes full effective
turn or loader closure. Unknown values become finite
unknowns; they are never copied or hashed into evidence.

The fixed collector now requests 28 cases, shared with producer registration and journal recovery:
continuity for the five selectors, owned cancellation for each selector, and no-match controls for
resume/fork last and id in each existing regime. Empty-project controls own fresh disposable paths
and fresh UUIDs, never copy sessions, and require the exact native session-not-found result. Generic
failure is not success. The attempt bound is 900 seconds; no external bypass is enabled.

Bounded project metadata comparison records changes without opening project file contents. The
original project/config/environment baseline remains checked during empty-project cases. Canonical
roles and existing directory identities are checked before descent; an ancestor alias or replaced
directory refuses before traversing a foreign tree. Failed eligibility skips the follow-up sample. This is
partial effect observation, not atomic confinement, content equality or complete mutation proof.

The managed initial/resume `config/read` now also projects the native layer stack. Only when the
private diagnostic owner is active, the wrapper requests `includeLayers`, projects each layer to
class, enabled, empty and role-match booleans, records which classes supplied effective values,
and removes `layers` before the reply reaches the SDK caller. At pinned upstream
`b412ff32c417f855c2b2d1581b77058eed87c84b`, `config_manager_service.rs` computes `config` and
`origins` identically with or without layers, and the protocol omits the member when it is not
requested; ordinary callers without that owner still send `includeLayers:false`. Admission now
requires a present config to carry an eligible stack: session flags, project, user and system
only, in native precedence, one user and one system layer, each at its assigned path role.
Enterprise-managed, MDM, legacy managed, unknown or misplaced layers refuse. This measures the
managed route's pre-turn input stack. It does not cover exec or fork, which do not read config,
the thread/turn overrides applied afterwards, or loader effects, so `layer_inputs` stays in
`missing` on every route.

Each completed case retains its ordinary outcome journal first, then its finite context journal.
The parent reads only strict finite fields and preserves earlier outcomes if a later context file
is missing, truncated, interrupted or malformed. Missing/duplicate diagnostics or mismatching child
roles cannot grant context credit. Nothing automatically retries a run or posts an attestation.

Native admission now validates typed per-case observations against private source-owned
expectations instead of approving one entire machine-specific context digest. All six required
context facts must have exact reviewed source/corroboration bindings. Named source-derived defaults
also require the exact pinned revision/binary, established input absence/precedence and a passing
native corroborating case. Intent, a producer's `complete` flag or a matching source hash alone is
insufficient. Synthetic complete fixtures test admission; they establish no real native facts.

The current collector deliberately supplies no complete fact bindings or default derivations.
Its journals name the remaining facts: layer inputs, native authentication resolver, transitive
runtime, effective turn, loader effects and mutation scope. Managed initial/resume's config reads
omit layers and precede turn overrides; exec/fork do not acquire those reads. These partial facts
stay partial. `complete` remains false until the source-reviewed native cases actually establish
the missing facts; the profile stays absent. The next slice supplies that finite case/mapping work,
then existing maintenance consumers and the isolated chain can use independently admitted results.

Verification uses source-owned fake native children and sentinel credentials only. Required CI has
no real login, credential fixture, model/provider call, additional job, daemon or scheduler.

The expanded production policy matrix exercises both last/id selectors. Invalid policies/sandboxes
and unsupported fork overrides are checked across unattended, interactive and external *request*
regimes with a child-spawn marker; those rejection tests enable no native external execution.
The acquisition crosscheck explicitly retains 94 identities, 207 operation edges and 53 due tuples,
including unresolved claim-only `--full-auto`. Passing these tests does not approve the proposed
classifications or splice their synthetic results into a native attempt.

Remaining implementation work is explicit: exec/fork layer correspondence, native resolver and runtime/turn/loader
correspondence, bounded native allow/deny mutation assertions, external execution in an approved
isolated environment, per-operation timeout/drop/exit observations, independently bound hermetic
harness collection and substantive mappings for all due tuples. All six context facts remain
unresolved until their complete assertions exist. This increment adds observation machinery and
case controls; it does not finish context/case/mapping closure or install a live profile.
