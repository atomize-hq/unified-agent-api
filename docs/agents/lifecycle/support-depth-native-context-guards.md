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
presence, not skill execution or complete loader behavior. Auth refresh does not invalidate the
nonsecret snapshot. Context/config/role identity drift refuses; these checks are not an OS sandbox
or an atomic guarantee against concurrent changes.

The wrapper emits private finite tracing events after final command overlays and after successful
spawn on the actual streaming exec/resume and app-server paths. The standalone collector owns one
exclusive global subscriber, so observations from Tokio tasks are included. A private per-process
owner token prevents late or foreign responses from being attributed to another case. No public observer API
or raw command, argv, environment, config/layer, protocol or error trace is added. Existing native
config/requirements responses are projected before serialization. Unknown values become finite
unknowns; they are never copied or hashed into evidence.

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
