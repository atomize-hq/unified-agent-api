# SPEC — Closing the acquisition ↔ maintenance-lifecycle gap

**Status:** Draft, awaiting maintainer approval
**Date:** 2026-07-25
**Base:** `origin/main` (== `origin/staging`)
**Supersedes nothing.** Extends `docs/agents/lifecycle/parity-generalization-plan.md` (§16–§17).

---

## 1. Objective

The parity acquisition and promotion lanes landed as one agent-agnostic subsystem, but they
were built alongside the existing `agent-maintenance` lifecycle rather than into it. Acquisition
now performs the *artifact* half of the relay's job while nothing closes the maintenance run,
and nothing checks whether the upstream release introduced surface the wrapper does not support.

This spec closes that gap so the whole thing operates as **one system with exactly two
human-in-the-loop points** — approve closeout, trigger promotion — plus ordinary PR merges.

### Target users

The repository maintainer (`@spenquatch`) operating the pipeline, and any future contributor
reading the lifecycle docs to understand how a packet reaches `latest_validated`.

### Verified current state

Established by direct inspection at `origin/main`, not assumed:

| fact | evidence |
|---|---|
| Relay step 4 (version-scoped artifacts) is what `parity-acquire` implements | `contract_policy.rs:229`; relay prompt "Required workflow" |
| Relay steps 1–3 (classify surface, land uplifts) have no automated counterpart | same |
| All three audits currently reconcile `exact`, `required_uplifts_this_run` empty | `refresh-agent --dry-run` for codex / opencode |
| Live drift is clean for all three agents | `check-agent-drift --agent {codex,claude_code,opencode}` |
| No packet is closed; merging one regresses main's HANDOFF to the open-run contract | `git show origin/main:…/HANDOFF.md` header |
| codex closeout is stale and the tool knows | recorded `27f3c079…` vs live `68f6e69e…` |
| main's claude_code request is invalid (`version_policy`); the #157 *branch* carries the corrected one, so this never blocks the work | `refresh-agent` rejection; branch request reads `2.1.212` / `upstream_stable_pointer` |
| Promotion is outside the maintenance contract **by design** | `writable_surfaces` excludes `latest_validated.txt` and `pointers/` |
| Codex-as-relay-host is intentional, not leakage | HANDOFF models `maintained agent packet` ≠ `local execution host` |

**Corrected 2026-09-24 (`uaa-0058`).** The row stating that all three audits reconcile `exact`
stopped holding on 2026-09-19, when `b52f1242` made authorization require a report
(`fully_authorized = report.is_some() && …`). The pre-acquisition placeholder therefore turns every
debt row into an uplift; the live packet requests now differ from their acquired reports and strict
request loaders refuse them.

The last row matters: an earlier reading of this as a defect was **wrong** and is not in scope.

---

## 2. What we are building

### Piece 1 — Post-acquisition uplift gate

After `parity-acquire` writes multi-target artifacts, re-derive the support-surface audit against
those *fresh* artifacts and branch on `required_uplifts_this_run`:

- **empty** → the packet is relay-free and closeout-ready.
- **non-empty** → the packet needs the Codex relay. Acquisition still succeeds and still commits
  (the artifacts are valid and worth keeping), but the job renders the relay invocation into the
  PR body and marks the packet not-closeout-ready.

The gate judges a request re-frozen against the fresh artifacts, not the request first written when
the packet opens. That first request is a placeholder because its target reports do not yet exist,
and no strict loader accepts it after acquisition changes the live audit. Inside `parity-acquire`,
after the union/report/validate step has produced the target artifacts and before the maintenance
audit gate, acquisition therefore runs `prepare-agent-maintenance --from-request`. The acquisition
commit then carries the regenerated request, packet docs, and artifacts together. Only that
completed generation lets the uplift-branch relay invocation start (implemented by PR B,
`uaa-0058`).

This converts today's *accidental* safety — the relay is skippable only because these particular
releases added no surface — into a decided property.

### Piece 2 — Closeout generator

Derive `maintenance-closeout.json` from run evidence so `close-agent-maintenance` becomes a
one-command maintainer approval instead of a hand-authoring task.

This does **not** weaken the guarantee. `close-agent-maintenance` independently re-validates every
finding against a live `check-agent-drift` run (`closeout/validate.rs:207-302`): resolved findings
must be *absent* from live drift, `explicit_none_reason` is legal only when live drift is clean,
and `deferred_findings` must match live drift as an exact bijection. The generator cannot fabricate
drift truth; it can only save the maintainer from transcribing it.

`preflight_passed` is the one field nothing cross-checks. It is derived from the packet PR's CI
conclusion **pinned to the exact commit the closeout records**, and the generator **fails closed**:
if it cannot prove a green run for that SHA it refuses to emit, never defaulting and never emitting
`false`. The maintainer's approval remains the judgement that the run is done — which is a decision,
not a fact.

---

## 3. Commands

New and changed surfaces. Every new command follows the existing `xtask` conventions
(clap `Args` struct, `run()` returning a typed error with `exit_code()`).

```bash
# NEW — complete an existing packet generation inside parity-acquire, after the
# union/report/validate step has produced the target artifacts and before the maintenance audit
# gate. The acquisition commit then carries the regenerated request, packet docs, and artifacts
# together. Reads only the request's recorded agent_id, opened_from, request_recorded_at,
# request_commit and every detected_release field accepted by prepare; it does not reconcile the
# audit it replaces. request_commit remains the commit of the event that opened the generation:
# the opening workflow's github.sha. It is not guaranteed to equal the checked-out base. A
# post-acquisition --from-request re-freeze preserves it, so it identifies neither the base nor the
# tree from which the completed audit was derived; the acquisition commit records that tree.
# Regenerates the request and packet docs against the current tree exactly as prepare does,
# while preserving request_recorded_at and request_commit. An unchanged rerun writes identical bytes.
cargo run -p xtask -- prepare-agent-maintenance \
  --from-request <path/to/maintenance-request.toml> \
  [--write]

# NEW — emit the support-surface audit as machine-readable JSON, derived from live artifacts.
# Piece 1's input. Read-only apart from the --emit-json projection.
# --expect-target-version (T2c, typed in T2d) exits 5 unless the request's
# detected_release.target_version equals it, compared before the validated request load and any
# evidence work. Omit it outside acquisition runs.
cargo run -p xtask -- maintenance-audit-status \
  --request <path/to/maintenance-request.toml> \
  [--expect-target-version <version>] \
  --emit-json _ci_tmp/audit/status.json

# NEW — generate a closeout artifact from run evidence. Fails closed.
# Corrected 2026-09-24: this block previously declared `--preflight-run <github-run-id |
# --preflight-from-ci>`. That surface is withdrawn; see the note below the fence.
cargo run -p xtask -- prepare-agent-closeout \
  --request <path/to/maintenance-request.toml> \
  --commit <sha> \
  --recorded-at <rfc3339-utc> \
  --write

# NEW — ask whether automation still has authority over a packet generation. Read-only, and
# deliberately an ownership predicate rather than a validity check. Exit 3 means a maintainer
# froze it and is a green outcome; every other non-zero exit is a guard that could not answer,
# which also means do-not-mutate but fails the job. `--from-ref` reads the marker directory from a
# git ref instead of the working tree, so a boundary that is about to mutate the remote can read a
# freshly fetched base rather than a checkout taken minutes earlier. See the stand-down subsection
# under §8's T8 sequencing.
cargo run -p xtask -- maintenance-stand-down-check \
  --agent <agent-id> \
  --target-version <version> \
  [--from-ref <git-ref>]

# EXISTING — unchanged contract; now consumes a generated artifact.
cargo run -p xtask -- close-agent-maintenance \
  --request <path/to/maintenance-request.toml> \
  --closeout <path/to/maintenance-closeout.json>
```

**`prepare-agent-closeout` takes no run id (corrected 2026-09-24).** The withdrawn
`--preflight-run` moved the fail-closed guarantee out of the tool and into the operator. T4 refuses
to rank or filter candidate runs precisely so that re-running CI until it is green cannot change the
recorded conclusion; a caller-supplied run id restores exactly that steering, because the operator
chooses which run to hand over and nothing downstream can tell a chosen run from the only run. The
conclusion is therefore resolved from the commit alone, under the selection policy stated in T4
below. `--preflight-from-ci` is withdrawn with it and was never implementable: the run that prepares
a closeout cannot supply its own conclusion, because T4 refuses any run that is not `completed`, and
`close-agent-maintenance` runs in neither CI nor `make preflight`.

**`--recorded-at` is an argument, not a clock (stated 2026-09-24).** It mirrors
`prepare-agent-maintenance`'s `--request-recorded-at`, and it satisfies §5's determinism rule the way
that command does: re-supplying the committed value reproduces the artifact byte for byte, while a
caller that passes the real time records the truth. `SOURCE_DATE_EPOCH` must **not** be consulted for
this field — `parity-acquire.yml` derives it from the upstream publish time, so honouring it would
assert that the maintainer closed the packet on the day upstream shipped the release.

**`wrapper_only_dispositions` is not an input and is not derived (stated 2026-09-24).** Each
disposition is a judgement about why the wrapper claims a surface upstream's help does not show, and
the four categories are indistinguishable from repository state alone. `prepare-agent-closeout`
therefore refuses when a live wrapper-only row has no disposition, naming each unadjudicated row, and
that refusal is the work queue. A separate durable disposition file is **not** the answer — see
`uaa-0039`, "one list, one authority, never a durable copy beside a per-run copy". The direction of
travel that item records is that the two durable categories migrate into the wrapper-coverage
generator, which is an ordinary PR merge and so adds no human-in-the-loop point.

Exit codes follow the established convention: `2` for validation failure (the artifact or the
evidence is wrong), `1` for internal error. `maintenance-audit-status` additionally uses `3` for
"uplifts required", so a workflow can branch on the gate without parsing stdout — mirroring
`EXIT_NOT_ELIGIBLE` in `manifest_acquisition.rs:74` — and, since T2a (`72191bb3`), `4` for an
incomplete acquisition (`snapshots/<version>/union.json` has `complete: false`), and, since T2d
(`a3c8ce53`), `5` for a target-version mismatch.

| exit | constant | meaning | error? |
| --- | --- | --- | --- |
| 0 | — | clean: no uplifts, reconciliation not drifted | no |
| 3 | `EXIT_UPLIFTS_REQUIRED` | uplifts required and the debt baseline still matches; contributor relay work needed | no — a result |
| 4 | `EXIT_INCOMPLETE_ACQUISITION` | union incomplete; names `missing_targets` | yes |
| 5 | `EXIT_TARGET_VERSION_MISMATCH` | the request's `detected_release.target_version` differs from `--expect-target-version`; checked before any evidence work, so it wins over malformed evidence or an invalid request field — which also means exit 5 says nothing about whether the rest of the request is valid | yes — blocking in CI only when the run commits |
| 2 | `EXIT_VALIDATION` | evidence missing, bound to another version, or malformed; invalid request; drift with no uplifts; and, even when uplifts remain, a debt row that matches no live gap or frozen debt rows or a debt count that differ from the live audit | yes |
| 1 | `EXIT_INTERNAL` | internal fault | yes |

A computed 0 or 3 survives a failed `--emit-json` write (the stale projection is removed and a
warning printed). The exit code is the product; the projection is advisory. **Corrected
2026-09-23:** this sentence read "advisory until `uaa-0025` is resolved", which `56f13a39`
resolved. Resolving it did not make the projection authoritative — it made the advisory status
enforceable, by allocating a fresh projection directory per invocation and exporting its path only
for exit 0 and exit 3, so a later step cannot read a file it was never told about.

---

## 4. Project structure

### New files

```
crates/xtask/src/agent_maintenance/audit_status.rs        # piece 1: derive + emit audit JSON
crates/xtask/src/agent_maintenance/stand_down.rs          # uaa-0048: packet ownership predicate
crates/xtask/src/agent_maintenance/stand_down/tests.rs
crates/xtask/src/agent_maintenance/prepare_closeout.rs    # piece 2: closeout generation (T6)
crates/xtask/src/agent_maintenance/closeout/evidence.rs   # T4: CI-conclusion resolution, commit-pinned
crates/xtask/src/agent_maintenance/closeout/findings.rs   # T5: derive resolved_findings from written surfaces
crates/xtask/tests/agent_maintenance_uplift_gate.rs
crates/xtask/tests/agent_maintenance_closeout/evidence.rs
crates/xtask/tests/agent_maintenance_closeout/findings.rs
```

### Changed files

```
crates/xtask/src/main.rs                       # register both commands
.github/workflows/parity-acquire.yml           # uplift gate after the union job; relay rendering
.github/workflows/agent-maintenance-open-pr.yml# surface closeout-readiness on the packet PR
docs/agents/lifecycle/parity-generalization-plan.md  # §18: lifecycle integration
docs/cli-agent-onboarding-factory-operator-guide.md  # closeout as an explicit lifecycle step
docs/specs/agent-registry-contract.md          # if the gate becomes registry-visible truth
```

### Deliberately untouched

- `crates/xtask/src/agent_maintenance/closeout/validate.rs` — the validation contract is correct
  as written and is what makes generation safe. Generation must satisfy it, not relax it.
  **Decided 2026-09-20: the freeze is directional, not locational, and `uaa-0039` is the
  authorized exception.** §7's *Ask first* states the real rule — "relaxing any existing
  validation rule to make generation easier" — and §2 Piece 2 states what it protects: the
  validator re-derives drift truth independently, so the generator cannot fabricate it. Adding a
  check does not touch that property; removing or weakening one does, wherever it lives. Read this
  bullet as a direction rather than a boundary: **no change may weaken a validation rule or make
  generation easier, and a prerequisite change that adds a check is in scope provided it lands
  before the generator that must satisfy it.** The ordering is what carries the guarantee. A
  validator hardened before its generator exists is written against the contract; one hardened
  afterwards is written against whatever that generator happens to emit, and will pass because it
  was shaped to. `uaa-0039` lands before `prepare-agent-closeout` exists, so its checks are in
  scope here, and the validator it leaves behind is then the fixed authority T6 must satisfy
  unmodified. `maintenance-request-contract-v1.md:327` already scheduled it this way.
- `contract_policy.rs` relay-host constants — intentional design.
- `execute-agent-maintenance` — the relay itself stays as-is; we only decide *when* it is needed.

---

## 5. Code style

Follow `AGENTS.md` and the surrounding code, specifically:

- **700-line cap per Rust file** (`make loc-check`). `prepare_closeout.rs` is pre-split into
  submodules for this reason; do not let a module drift toward the cap.
- Errors are typed enums with `Display` + `exit_code()`, never `anyhow` in these paths.
- Comments explain **why**, not what. The existing lifecycle code is dense with rationale comments
  at non-obvious decisions; match that density rather than narrating mechanics.
- Workflow shell: `set -euo pipefail`; every `while read … done < <(…)` strips CR
  (`tr -d '\r'`) — enforced by `c4_spec_process_substitution_loops_strip_cr_for_the_windows_runners`.
- No agent-specific branching in reusable workflows or generic engines. Agent-specific behavior
  belongs in committed descriptor data.
- Deterministic output: honour `SOURCE_DATE_EPOCH`; artifacts must be byte-reproducible from
  committed inputs.

---

## 6. Testing strategy

| layer | what it covers |
|---|---|
| Unit — `audit_status` | empty vs non-empty `required_uplifts_this_run`; exit code 3 distinct from 1; `Exact` vs `Satisfied` reconciliation both handled |
| Unit — `prepare_closeout` | generated artifact passes `validate_closeout` unmodified; `resolved_findings` non-empty (a hard rule at `validate.rs:162`); `explicit_none_reason` chosen iff live drift clean; bijection honoured when drift is non-empty |
| Unit — evidence | **fails closed** when CI for the SHA is missing, red, or ambiguous; never emits `preflight_passed: false`; never defaults |
| Contract — workflow wiring | `parity-acquire` invokes the gate after the union job and before commit; relay rendering appears only on the non-empty branch |
| Regression | a closeout whose `request_sha256` is stale is rejected (the failure mode observed on codex 0.144.6) |
| End-to-end | dry-run the full chain against the three live packets; all three currently reconcile `exact`, so all three must come out closeout-ready |

**Corrected 2026-09-24 (`uaa-0058`, `uaa-0066`).** The end-to-end row's premise no longer holds:
the post-acquisition reports produce non-empty uplift queues, so the three packets do not all come
out closeout-ready.

`make preflight` is the authoritative gate; nothing is done until it is green.

**Explicit non-goal:** do not add a test that asserts the *current* audit is `exact`. That is a
property of today's upstream versions, not of the system, and would fail on the first release that
adds surface — which is precisely the case the gate exists to handle.

---

## 7. Boundaries

### Always

- Re-derive the audit from **live artifacts**, never from the frozen request block.
- Keep `close-agent-maintenance`'s validation the sole authority on drift truth.
- Fail closed on missing or ambiguous evidence.
- Commit-pin every derived claim to the SHA the closeout records.
- Run the parallel review loop (Codex + Opus) on every material candidate, then adjudicate.

### Ask first

- Anything that changes what `requires_manual_closeout = true` obligates the human to.
- Relaxing any existing validation rule to make generation easier.
- Changes to the registry contract or `RULES.json` schema.
- Any additional human-in-the-loop point beyond the two named here.

### Never

- Auto-close a maintenance run. `requires_manual_closeout = true` is the contract.
- Promote (`dry_run: false`, or advancing `latest_validated`) as part of this work.
- Push, merge, close, or alter remote PRs without an explicit in-session request.
- Hand-author agent version selections — the watcher owns detection and target selection.
- Hand-edit generated packet docs (anything carrying a `generated-by:` header).
- Write outside a packet's declared `writable_surfaces`.

---

## 8. Task breakdown

**T1 — `maintenance-audit-status` command. DONE (`b55cef4f`).** Derives the audit from live
artifacts, emits JSON, exits 3 when uplifts are required. Five implementation rounds, four review
rounds; 29 command tests, suite 463/0, real-repo exits codex 0 / opencode 2 / claude_code 2.

The gate refuses on three grounds, not one: missing evidence, evidence bound to another version,
and — added in round 5 — an acquisition that never finished. `union.json`'s `complete` flag is the
authority for the last. See §8.1 for the debt this carried out, and plan doc §18 for why the
completeness check exists.

**T2 — Wire the gate into `parity-acquire`. DONE (`916c9e9b`).** Original
sketch: run after the union job, before the commit step; branch on exit code: 0 → mark
closeout-ready; 3 → render the relay invocation into the PR body and mark not-closeout-ready;
other → fail. Contract test for the wiring.

Landed in three commits. T2a (`72191bb3`): exit 4 for an incomplete acquisition; the computed
outcome survives a projection write failure (`uaa-0026`); three-case projection cleanup; the
`evidence.rs` split. T2b (`e87a9a1d`): the `Maintenance audit gate` step in the `union` job,
numeric exit routing, snapshot capture retry-once, contract tests. T2c (`24a95b52`):
`--expect-target-version`, so the gate checks the version this run acquired; the gate records its
verdict instead of failing, the artifact bundle upload runs `always()`, and a terminal step fails
the job on a blocking verdict after commit and upload. Exit 3 renders only a placeholder summary
line; the real relay invocation is T3.

Two more rounds closed it. The T2c review (both lanes) proved the contract tests bound spelling, not
behaviour, and that a version mismatch was masked by the request loader and indistinguishable from
other validation failures. T2d (`a3c8ce53`) added a behavioural harness that runs the extracted gate
and terminal steps, a typed exit 5 checked before the validated load, terminal-code validation, a
random output delimiter, and the maintainer-decided `union` condition that keeps non-required legs.
The T2d re-review (both lanes) found the harness still injected the step env it should check, plus
an inline-table hole in the pre-check; T2e (`916c9e9b`) closed both. Suite 488/0; real-repo exits
codex 0 / opencode 2 / claude_code 2, codex `--expect-target-version 0.145.0` 5. Debt carried out is
in §8.1; plan doc §19 has the full account.

**Support-audit classification (2026-09-15, before T3).** The first post-merge nightly showed exit 3
hiding drift: seven opencode debt rows the wrapper already covered were reported as removed upstream
surface. Four commits fixed it. `dce4b1fb` retired the seven rows. `ed31d54a` replaced
`removed_upstream_surface` with `unmatched_debt_surface`, whose rows carry an `observation`
(`covered_by_wrapper`, `excluded_by_rules`, or `not_observed`), because help output cannot prove a
removal. `170acc68` made the gate exit 2 on unmatched debt rows or a changed debt baseline even when
uplifts remain. `30898655` renamed the uplift reason to `unbaselined_gap`. The contract gained the
hidden-surface policy that T8 enforces (T8 sequencing below). Backlog items `uaa-0039`…`uaa-0044`
carry what the work exposed (§8.1).

**T3 — Relay-packet rendering. DONE (`c3607d6b`).** On the uplift branch, render the relay
invocation (prompt path, dry-run→write `--run-id` handshake) into a managed comment on the packet
PR, from the existing renderer, so the maintainer pastes one command. Reuses `docs.rs` rendering;
no new prompt source of truth. Exit 2 writes no projection, so a debt-baseline failure names its
unmatched rows only in the gate's error line; T3 must carry that line to the PR (`uaa-0031`).

Landed across five commits. `129ea8f5` split the two test binaries that were within five lines of
the §5 cap. `56f13a39` made the projection invocation-scoped (`uaa-0025`). `fba98f95` gave the
verdict a channel that survives the job it reports on (`uaa-0031`). `b6498b37` stopped the commit
step rebasing an audited acquisition onto a moved branch (`uaa-0034`). `c3607d6b` added the
publisher job and the relay rendering. Acceptance criterion 1 is met in wiring and under test; it
has not yet been observed on a runner, which T8 does.

**Corrected 2026-09-24 from runner evidence.** The relay invocations published on packet PRs #223
(opencode), #227 (codex), and #230 (claude_code) exit 2 at request load. Each request was frozen
before acquisition and does not match the acquired reports (`uaa-0058`). The repair is the
post-acquisition re-freeze described in Piece 1. The generated relay prompt also omitted the debt
re-authorization step required at every new version (`uaa-0061`).

**Decided 2026-09-19: a comment, not the PR body.** The body is set from `body-path` on
`create-pull-request`, pointed at the generated `governance/pr-summary.md`, and the watcher
re-dispatches `agent-maintenance-open-pr` nightly with no dedupe against an already-open packet. The
body is therefore rewritten from that file every night, so a verdict written into it survives at
most one day. The body also declares the frozen request as its source of truth, while the verdict is
live post-acquisition evidence. `uaa-0048` carries the wider problem this exposed.

**T4 — Closeout evidence resolution.** Commit-pinned CI conclusion lookup, fail-closed. This is the
highest-risk unit; it decides whether a governance artifact can be trusted.

**Which commit carries the evidence (established 2026-09-20).** `ci.yml` triggers on
`pull_request`, not `pull_request_target`, and the run's `head_sha` is the packet-branch head. Check
runs attach there and nowhere else. Measured on #215: head `f51e0b62` carries 17 check runs, its
merge commit `404ff19d` carries 0, and run `35500177443` reports `headSha=f51e0b62`. A closeout that
records a merge commit therefore finds no evidence at all, and because T4 fails closed, it refuses
permanently rather than intermittently. T4 resolves against the packet head; the commit-selection
rule is load-bearing from the first commit, not a refinement.

**The request hash is a clock (established 2026-09-20).** `request_recorded_at` is a field inside
`maintenance-request.toml`; the nightly run passes a fresh `date -u` value for it; `request.rs:399`
digests the whole file; `closeout/validate.rs:145` rejects a mismatch. A generated closeout
therefore stops validating at the next nightly run whether or not it was ever committed — §6's
regression row records this as a hand-authoring hazard, and with the nightly cadence it is a
deadline. Do not resolve it by excluding the timestamp from the digest: that hides one mutation and
leaves force-push erasure and every other request rewrite intact. `uaa-0048` carries the fix, and
its invariant is that a request generation's bytes are stable while that generation is active.
On the acquisition lane with no target report at open, the request first becomes strictly loadable
when its post-acquisition freeze is committed. Before that, strict loaders refuse the placeholder,
so no closeout can exist. On the docs-only lane, or when the target reports already existed on base
at open, the first freeze is already strictly loadable. Loadable is not protected. A generation is
protected against regeneration and force-push only once a stand-down marker names it (`uaa-0048`).
Declare that marker after the acquisition commit (`uaa-0063`).

**T5 — Closeout finding derivation.** Map written surfaces to `MaintenanceDriftCategory`
(`registry_manifest_drift`, `support_publication_drift`) with real surface lists; choose
`explicit_none_reason` vs `deferred_findings` from the live drift report.

**T6 — `prepare-agent-closeout` command.** Compose T4 + T5, emit the artifact, and self-verify
through the validator's real input path.

**Self-verification writes first and validates second (corrected 2026-09-24).** The earlier wording,
"self-verify by running the real `validate_closeout` before writing", is not satisfiable.
`load_linked_closeout` is the only public validating entry and it is path-addressed:
`maintenance_pack_prefix_from_closeout_path` requires exactly six path components ending in the
literal `maintenance-closeout.json`, so a candidate written under a temporary name or into a
temporary directory is refused on its path before its bytes are read. T6 therefore writes the
canonical path, validates it through `load_linked_closeout`, and on refusal restores the previous
bytes — or, where no closeout existed, names the invalid file it has left behind. Widening the
validator to accept a candidate path is not an alternative: §4 makes it the fixed authority T6 must
satisfy unmodified, and the guarantee here is that the bytes validated are the bytes on disk.

T6 also owes the branch-aware `reject_unbound` message described under T8 sequencing, and must call
the commit binding explicitly rather than assume the validator covers it.

**T6 does not author `wrapper_only_dispositions`.** It carries forward the dispositions already
recorded for surfaces that are still live, and refuses when a live wrapper-only row has none, naming
each unadjudicated row. See the note under §3. This means a packet whose rows have never been
adjudicated cannot be closed by T6 alone — which is the mechanism reporting a missing human
judgement, not a gap in T6.

**T7 — Docs. DONE (2026-09-25).** The [operator guide](../../cli-agent-onboarding-factory-operator-guide.md#maintenance-mode) now records the freeze, relay, closeout, and separate-promotion procedure; the [plan's current lifecycle checklist](../../agents/lifecycle/parity-generalization-plan.md#current-lifecycle-checklist-2026-09-25) records the maintainer sequence; and the [Codex 0.156.1 proof receipt](../../agents/lifecycle/codex-maintenance/governance/proof/2026-09-25-closeout-rehearsal.md) records the supervised rehearsal and its limits.

**T8 — Prove it. INCOMPLETE.** Run the chain against all three live packets; confirm `make preflight` green;
both review lanes clean with findings adjudicated. The Codex rehearsal is evidence for the lifecycle, not completion of the remaining packet proofs.

Order: T1 → T2 → (T3 ∥ T4) → T5 → T6 → T7 → T8.

### Backlog groups that land inside that order

The open items in §8.1 are not a queue to drain after T8. Five of them gate a T item and land with
it. Recorded 2026-09-19, after `uaa-0029`, `uaa-0038`, `uaa-0043`, `uaa-0046` and `uaa-0047` landed
in #216.

**Re-derived 2026-09-23. Both groups below are now done, and no open item is a prereq for T4–T8.**
The 2026-09-19 table was stale by construction: it was recorded before `uaa-0050` through
`uaa-0055` were filed and before `uaa-0056` was deliberately not created. Every open item's trigger
was re-checked against the tree rather than against its own note, and the result is one gated item,
21 independent, and nothing retired:

| class | items | note |
|---|---|---|
| **Prereq for a T** | *none* | The `uaa-0048` → `uaa-0045` → `uaa-0039` chain was the entire prerequisite set and is discharged. |
| **Gated after T8** | `uaa-0019` | A *promotion* gate, not a closeout gate. It fires when a claude_code version is promoted, which T8 does not do. It will fire: `latest_validated` is 2.1.29 against a 2.1.267 packet, and the item records subtype drift observed as early as 2.1.63. |
| **Independent** | `uaa-0023`, `uaa-0024`, `uaa-0027`, `uaa-0028`, `uaa-0033`, `uaa-0037`, `uaa-0040`, `uaa-0041`, `uaa-0042`, `uaa-0044`, `uaa-0049`, `uaa-0053`, `uaa-0054`, `uaa-0055`, plus the seven `agent_api.*` items outside this spec (`uaa-0002`, `uaa-0003`, `uaa-0008`, `uaa-0009`, `uaa-0010`, `uaa-0014`, `uaa-0021`) | None touches the T4–T8 path. The four whose adjacency was suspected are settled by construction rather than by judgement — see below. |
| **No longer necessary** | *none* | Three retirement classes were tested and none retired an item: a trigger reading "before the first packet closeout" (now enforced by `uaa-0039`), an argument from `upstream_available_on` on a wrapper-only row (retired by `uaa-0051`), and any premise that opencode's global-flag model is enabled (reverted by `uaa-0052`). |

Four verdicts rest on a structural fact rather than on severity, and are recorded here because the
fact is what keeps them true:

- `uaa-0024` (union-vs-per-target coherence) cannot gate anything here, because the audit reads
  **exactly one** report: `select_report_path` returns a single path, preferring `coverage.any.json`.
  Per-target reports are never cross-read, and the item's own kickoff puts its fix in the
  report-generation lane.
- `uaa-0033` (artifact bundle mismatch) is not adjacent to T4 despite both naming CI. T4 resolves
  check-run conclusions pinned to a commit; the item concerns an uploaded artifact. `MaintenanceCloseout`
  has no artifact or bundle field at all.
- `uaa-0028` (`--emit-json` deletes what it names) had T3 as its trigger, and T3 did not fire it.
  `prepare-agent-closeout` takes `--write`, not `--emit-json`, so T6 does not extend the surface either.
- `uaa-0049` (codex agent-id branch) does not gate T6's output. The closeout's three write paths all
  sit under `{maintenance_root}/**`, which `writable_surfaces` already grants every agent.

One dependency outside this spec was found and is recorded on both items: **`uaa-0042` is a
prerequisite for `uaa-0053`**, not for any T. No `wrapper_coverage.json` declares a command
`unsupported` today, and deciding opencode's target surface across its 60 unwrapped commands is the
most likely first such declaration.

| group | items | lands |
|---|---|---|
| ~~Verdict visibility~~ **Done** | `uaa-0031`, `uaa-0025` — `uaa-0028` stayed open, because the chosen surface did not make the projection path durable. `uaa-0034` was pulled in and resolved with them | **Landed with T3.** `uaa-0031` decides the surface T3 renders to, and its deliverable reads "implemented with T3". T3 is the projection's first consumer, so `uaa-0025` cannot follow it. |
| Closeout prerequisites | `uaa-0048`, then `uaa-0045`, then `uaa-0039` | **Immediately after the verdict-visibility group, before T4–T8 reach a live packet.** None blocks T3, and none may wait for T8. `uaa-0045`'s trigger fired when `uaa-0038` was resolved in `a474cdbc`: the two are halves of one defense — the workflow clears a stale shard, the merger checks that a shard is what its filename claims — and only the first half exists. `uaa-0048` blocks T8 outright: a closeout committed to a packet branch does not survive the next nightly regeneration. |

**Three merge units, not one (decided 2026-09-20).** The group was first recorded as a single
landing. It is three, because the three items have independent acceptance conditions and pairing
them lets the slowest hold the rest. `uaa-0045` is a ready evidence-integrity repair; `uaa-0039` is
36 adjudications plus any publication contractions they oblige, whose human half may outlast its
code half. `uaa-0048` goes first because it is the only one with a deadline — see the request-hash
note under T4 below.

**Corrected 2026-09-20.** This paragraph named `c4_spec_ci_wiring.rs` (699 code lines) and
`agent_maintenance_audit_status.rs` (695) as within five lines of the §5 cap and told the
implementer to budget their splits as the first commits of each group. T3 split both in `129ea8f5`,
so that instruction now aims at files with room. Measured at `a4606b6b`:

| file | code lines | headroom |
|---|---|---|
| `crates/xtask/src/agent_maintenance/support_audit.rs` | 599 | 101 |
| `crates/xtask/tests/agent_maintenance_audit_status.rs` | 548 | 152 |
| `crates/xtask/src/agent_maintenance/closeout/validate.rs` | 486 | 214 |
| `crates/xtask/tests/c4_spec_ci_wiring.rs` | 473 | 227 |
| `crates/xtask/src/manifest_union.rs` | 331 | 369 |

The advice still holds; only its target moved. `support_audit.rs` is the file near the cap, and
`uaa-0039`'s enforcement was expected to push on it. Budget that split as a first commit if checks
land there; do not ritually re-split the two files T3 already corrected. **Outcome 2026-09-22:** no
split was needed. `uaa-0039` added one derivation to `support_audit.rs` and put its checks in a new
`closeout/support_audit_truth.rs`, which is where they belong — they are closeout rules, not audit
derivation.

### T8 sequencing — closeout happens *before* merge

A packet must be closed **on its own branch**, so the merge carries a closed HANDOFF. Codex
demonstrates the failure mode: #153 merged while still open, which replaced main's closed HANDOFF
with the open-run contributor contract. That is a regression to repair, not a pattern to repeat.

Re-derived 2026-09-13 (read-only). The watcher kept opening packets after this spec was written:
#157 (claude_code 2.1.212) and #158 (opencode 1.18.4) are **closed without merging**, superseded by
the open packets below. Each branch request's `target_version` and `version_policy` were read from
the packet branch; re-derive this table again (`git fetch`, `gh pr list`) when T8 starts, because
the watcher will have moved on.

**Re-derived 2026-09-23.** All three rows moved, exactly as the paragraph above predicted. The
2026-09-13 table read #195 / 2.1.236, #205 / 1.18.29 and #206 / 0.153.4; every one of those packets
has been superseded. Current:

| packet | branch | closed against | why |
|---|---|---|---|
| claude_code 2.1.267 (#211) | `automation/claude_code-maintenance-2.1.267` | the packet branch | branch request reads `2.1.267` / `upstream_stable_pointer` |
| codex 0.155.0 (#220) | `automation/codex-maintenance-0.155.0` | the packet branch | branch request reads `0.155.0` / `latest_stable_minus_one` |
| opencode 1.18.31 (#223) | `automation/opencode-maintenance-1.18.31` | the packet branch | branch request reads `1.18.31` / `latest_stable_minus_one` |
| codex 0.144.6 (#153, merged) | — | a branch off `main` | already merged open; needs a catch-up pass |

**The packet branch is not a preference — `staging` cannot close two of the three (established
2026-09-23).** A closeout re-derives its wrapper-only baseline from the coverage report for the
target version, and that report is committed by the acquisition run to the packet branch. On
`staging` only codex's is present:

| agent | reports on `staging` | live packet | request on `staging` |
|---|---|---|---|
| codex | `0.155.0` present | 0.155.0 (#220) | `0.155.0` |
| claude_code | `2.1.29` only | 2.1.267 (#211) | `2.1.140` |
| opencode | `1.14.47`, `1.4.11` | 1.18.31 (#223) | `1.14.49` |

Run from a `staging` checkout, `wrapper_only_baseline` finds no version directory, returns `None`,
and `uaa-0039`'s validator takes its `reject_unbound` path — whose message reads "has no coverage
report for `<version>`". The refusal is correct and the diagnosis is wrong: the report exists, on
the branch the maintainer is not on. T6 should make that message branch-aware, because the two
conditions are indistinguishable from an absent directory and only one of them is a real gap.

**Open decisions that gate a closeout.** Re-deriving the table above does not clear these; each
stays until its backlog item records a maintainer decision. None is open: `uaa-0035` and `uaa-0036`
were decided on 2026-09-14 (§8.1).

**Support-audit checks that gate a closeout.** Decided 2026-09-15. Discovery reads help output, so a
surface upstream hides is not an obligation by itself, but it stays one while the wrapper claims it
(maintenance request contract, "Hidden upstream surfaces and wrapper-only rows"). Before a packet
closes, every wrapper-only row in its live coverage report must be sorted into one category —
hidden upstream but still supported, supported only on older upstream versions, obsolete, or a
discovery bug — a surface sorted obsolete must contract publication truth in the same run, and the
live audit must have no unmatched debt rows (contract field invariant 6). **Landed 2026-09-22
(`uaa-0039`).** The record is `wrapper_only_dispositions[]` in the closeout artifact and
`close-agent-maintenance` enforces all three, deriving each from the repository rather than reading
the artifact's claims — which matters most for invariant 6, because a request that records the same
unmatched rows as the live audit reconciles `exact` and would otherwise close. The closeout also
binds the report it adjudicated in `wrapper_only_baseline_ref`, since a row correctly sorted
obsolete is gone from the regenerated report by the time closeout runs. The contract's
hidden-surface section carries the category table and the evidence each one owes.

**Declaring a stand-down (landed 2026-09-21, `uaa-0048`).** A maintainer freezes a packet
generation by committing `<maintenance root>/governance/automation-stand-down/<version>.toml`,
naming `agent_id`, `target_version` (which must equal the file name), a `reason`, and at least one
of `request_recorded_at` / `request_sha256`. `maintenance-stand-down-check` reads the directory;
`agent-maintenance-open-pr` asks before every step that can destroy a generation. Five choices are
load-bearing:

- **It is an ownership predicate, not a validity check.** Protection means *automation has lost
  authority over this packet generation*, never *a valid closeout is present*. The second reading
  still permits erasing half-finished categorization work, a partially authored closeout, or a
  valid one during an intentional edit. A marker whose recorded generation no longer matches the
  request on the branch is reported and still stands automation down, because a request that moved
  under a maintainer is exactly when they most need automation to keep away.
- **Markers are a sibling of the request, not a field in it.** A stand-down expressed inside
  `maintenance-request.toml` would change the request's bytes, and `request.rs:399` digests the
  whole file — so declaring the protection would invalidate the closeout it protects.
- **One file per frozen generation, named for it.** Two packet PRs for one agent can be open at
  once — supersession exists because of it — so two freezes have to coexist. A single per-agent
  file would make freezing the newer packet silently revoke the older one's protection, with no
  signal beyond a one-line edit. Naming the file for the generation also makes the match key
  checkable: a name outside strict `MAJOR.MINOR.PATCH` is rejected rather than quietly never
  matching, so `v0.155.0.toml` cannot read as protection that is not there.
- **A directory that cannot be read in full stands automation down.** Every entry is validated,
  not only the one that might match. A malformed, misnamed or wrong-agent entry means the freeze
  set is unknown, and an unknown freeze set is not an authorization.
- **Markers live on the base branch, not the packet branch.** The packet branch is inside the
  `add-paths` tree that `create-pull-request` resets and force-pushes, so a marker carried there
  could be destroyed by the operation it exists to block. Base is never written by the nightly run.
  This is also what makes a merged-but-unpromoted packet protectable: codex 0.155.0 has no open PR,
  no branch and no closeout, but base carries its packet, so base carries its markers.

A freeze survives a legitimate version bump. `prepare-agent-maintenance` and `refresh-agent` write
named files through `workspace_mutation` and never sweep the directory, so regenerating the packet
for a newer version leaves existing markers in place; a freeze is lifted by deleting its file,
never by the watcher moving on.

Enforcement is at three boundaries, re-asked each time rather than inherited: request regeneration,
branch and packet-body replacement, and supersession. Supersession is the one that asks about a
packet other than the run's own — a newer version opens a different branch and therefore a
different concurrency group, so it can close an older packet mid-closeout — so it asks per
candidate.

**Corrected 2026-09-24.** The decision is to enforce stand-down at four boundaries. PR B
(`uaa-0058`) implements the fourth at the post-acquisition re-freeze: it fetches base fresh and asks
through `--from-ref`. A declared stand-down (exit 3) skips only the re-freeze and the job continues
as before; any other non-zero exit means the guard could not answer and fails the step.

**The boundary that mutates the remote reads base fresh.** The job checks out base once and
`create-pull-request` resets the packet branch to *current* base, so the exposure between reading
the predicate and force-pushing is the job's whole duration — minutes — not the gap between two
steps. Re-reading the job-start checkout would be theatre: it cannot produce a different answer
than the first check did, for any reason a maintainer can cause. The pre-replacement check
therefore fetches base and reads the committed markers through `--from-ref`, where any git failure
is an internal error rather than an absent marker.

**A guard that cannot answer stands down, and says so in red.** A false authorization is the only
outcome this may never produce, so a compile failure, an unreachable base, a missing registry and a
malformed marker all route to *do not mutate*. But `::error` annotates without failing a step, and
a guard that quietly stood down every night would stop every maintenance packet while the job
stayed green — an outage that surfaces weeks later as "packets stopped appearing". The declared
stand-down (exit 3) keeps the job green; a guard that could not answer fails it.

**Deliberately not built, with reasons.** An expected-head check before branch replacement was
considered and dropped: `peter-evans/create-pull-request@v8` owns its own push, so the check cannot
be `--force-with-lease` on our side and has to be a separate step, which leaves the same race it
was meant to close. With the predicate read from a freshly fetched base immediately before that
step, what remains is the seconds between the fetch and the action's own push. It supplements the
ownership predicate rather than replacing it, and the predicate does the work. A queue-level dedupe
in `agent-maintenance-release-watch` was also dropped: it would save a wasted dispatch, but it is
not sufficient enforcement on its own (an already-queued invocation passes it, and
`workflow_dispatch` and re-runs bypass the queue entirely), so it would be a second copy of the
rule that must stay in step with the one that enforces. A stood-down run is cheap and its notice is
useful evidence, at the cost of one cached `cargo build -p xtask` per frozen agent per night. And
`close-agent-maintenance` was **not** changed to require a marker: that would change what
`requires_manual_closeout = true` obligates the human to, which §7 puts on the *Ask first* list.

**Deploy note.** Merging the guard does not protect runs already in flight. A dispatch that starts
after the merge picks up the guarded workflow, but one already executing does not. Before beginning
the first closeout, confirm no `agent-maintenance-open-pr` run is in progress for that agent.

**Completing the stand-down (`uaa-0050`, resolved 2026-09-21).** `uaa-0048` landed the predicate and
its three guards. It left three things unsettled, decided after a two-round ChatGPT Pro approach
review reconciled against the tree, and resolved the same day.

*Acquisition is an instruction, not a new capability.* The actor who performs a closeout is a
coding agent handed the packet PR, not a maintainer at a keyboard. That agent already holds git and
already pushes to the packet branch, so declaring a freeze needs no remote-write capability added
to `xtask` — which has never pushed, and whose every `git` invocation is `rev-parse`, `log`,
`ls-tree` or `show`. The declaration is delivered as an instruction instead: through the generated
`HANDOFF.md`, which calls itself the canonical contributor execution contract and already renders
the target version and the exact commands for its other steps, and as a standing rule in
`AGENTS.md` beside the existing conditional obligation to run `manifest-validate` after touching
`cli_manifests/**`. Those two obligations differ in one respect worth stating where the rule is
written: skipping `manifest-validate` fails CI, while skipping the freeze fails nothing until a
cron job destroys the work. The admission gate under `uaa-0039` is what converts the second kind
into the first.

Four points carry the instruction, because an agent's default behaviour is wrong on each:

- **Freeze before substantive judgment work, not before the closeout command.** The request-hash
  clock invalidates a closeout that was never committed, so local work is exposed from the first
  adjudication, not from the first push.
- **The marker goes to the base branch, never the packet branch.** A control file for this packet,
  sitting in this packet's own directory, is the most natural thing in the world to commit to the
  branch the force-push replaces.
- **Publish only the marker.** `git add <path>` followed by a path-limited `git commit <path>` is
  satisfiable by construction, where "keep the diff to one file" is a property to be maintained.
  The `git add` is required because the marker is always a new file, and the path on `commit` is
  what keeps a dirty packet tree out of the commit. An isolated worktree is recovery for the
  already-dirty case and belongs in the admission gate's refusal output, not in the documented
  path.
- **Nothing releases the freeze but retirement.** Closing, pushing, approving and merging all leave
  it in force. A closed `HANDOFF.md` should say so, since it no longer carries the acquisition
  step.

*Supersession read a stale checkout.* As shipped in `uaa-0048`, the fresh-fetch read was used at one
boundary only. Supersession called the predicate against the job-start checkout while running at
the *end* of the job, so it carried the same job-length window the replacement boundary was fixed
for — and it is the boundary that acts on a packet other than the run's own. All three boundaries
now fetch base and read through `--from-ref`, supersession per candidate. Request regeneration
needed the same mechanism for a different reason: that boundary is first after checkout, but a
re-run preserves the original event's `GITHUB_SHA`, so a fresh checkout is not a current one and
step order does not imply freshness.

**Corrected 2026-09-24.** The decision makes the acquisition re-freeze the fourth boundary. PR B
(`uaa-0058`) implements it with the same fresh-fetch and `--from-ref` rule: exit 3 skips that
re-freeze without failing the job, and any guard failure fails the step.

*A marker retires with its own promotion, never with a newer version.* Cleanup belongs in
`parity-promote`'s pointer-advance step, where removing the promoted version's marker rides into
the promotion PR alongside the `status: reported → validated` flip. That gives four properties at
once: one commit, so a failed cleanup leaves an extra marker and a failed promotion never leaves a
missing one; a maintainer merge, so automation proposes the unfreeze rather than performing it;
exactly one version named, so a second frozen generation is untouched; and correct timing, since
the record only reads `validated` once that PR lands, not when the workflow runs. Tying cleanup to
a *new version* was considered and rejected: upstream cadence says nothing about whether the
previous packet completed — codex ships near-daily and a closeout spans days — so a newer version's
first run would delete the marker of the packet being worked on, which is the supersession hazard
this guard exists to prevent, reintroduced as housekeeping. Letting the nightly delete markers is
also the restrained party holding the eraser: today it only reads, and base is never written by it.

Accepted with that change: once a marker is removed at promotion, nothing refuses to rebuild the
version. Neither `agent-maintenance-open-pr` nor `prepare-agent-maintenance` compares a target
version against the pointer — `prepare.rs` reads `latest_validated.txt` only to confirm the file
exists, and `current_validated` arrives as a caller-supplied argument. A `workflow_dispatch` or a
re-run inside GitHub's 30-day window can therefore regenerate an already-promoted packet. The
result is a spurious PR that regresses nothing unless it is merged, which is loud rather than
silent, so a writer-side guard refusing a `status: validated` version is deferred until one is
observed.

**There is no merge dependency on this work.** The invalid `2.1.140` claude_code request exists only
on `main`; the open claude_code packet branch carries the corrected policy, so closing claude_code
never touches the bad copy. Merging an open packet *before* closing it would actively make things
worse.

### 8.1 Known debt carried out of T1 and T2

Each item is in `docs/backlog.json` with its own context, file list and deliverables; plan doc
§18.3 and §18.5 carry the same items. T1 carried out four (`uaa-0023`…`uaa-0026`). T2 review
adjudication, and a code reading while writing the 2026-09-13 handoff, added six more
(`uaa-0027`…`uaa-0032`). The T2c review round (2026-09-13, both lanes) confirmed the handoff-reading
items (`uaa-0029`…`uaa-0032`; `uaa-0031` by reading only) and added two more (`uaa-0033`,
`uaa-0034`). Two more arrived with the codex feature pass and the pre-merge nightly simulation
(`uaa-0037`, `uaa-0038`), and were missing from this table until 2026-09-19. The 2026-09-15
support-audit classification added six (`uaa-0039`…`uaa-0044`). A 2026-09-16 ChatGPT Pro review of a
bounded repository bundle added three (`uaa-0045`…`uaa-0047`). Deciding T3's visibility surface
on 2026-09-19 added one (`uaa-0048`). Measuring `uaa-0045` on 2026-09-21 added one (`uaa-0051`), and reviewing it
as merged on 2026-09-22 added four more (`uaa-0052`…`uaa-0055`), one of which is a regression
that item introduced.

| id | item | disposition |
| --- | --- | --- |
| `uaa-0023` | Typed errors for support-audit evidence faults | Independent follow-up. Round 5 classifies some faults by matching error message text, because the packet scoped `support_audit.rs` out of the write set. A message drift yields exit 1, never a false clean, so this is fragility rather than a correctness hole. |
| `uaa-0024` | Union-vs-per-target coverage coherence contract | Independent follow-up. Needs the contract defined before it can be implemented, and it belongs at report-generation time rather than in the gate. |
| `uaa-0025` | Projection can survive a failed run as a stale result | **Resolved in `56f13a39`.** Fixing the classification would improve cleanup without establishing freshness, so the rule is that the exit code is the only authority, enforced on both sides. CI allocates a fresh projection directory per invocation under the runner's scratch space and exports its path only for exit 0 and exit 3, so a later step cannot read a file it was never told about. `live_derivation_attempted` is now `live_evidence_read`, a failed post-derivation cleanup warns instead of discarding its error, and the `--emit-json` doc no longer claims the file is "either current or absent" |
| `uaa-0026` | Exit 3 lost when `--emit-json` cannot be written | **Resolved in `72191bb3`.** The computed outcome takes precedence; the stale projection is removed and a warning printed. |
| `uaa-0027` | Snapshot retry can mix two attempts in raw_help | Low. The retry never clears attempt 1's `raw_help/<version>/<target>/`. raw_help is never committed. |
| `uaa-0028` | `--emit-json` cleanup deletes whatever path it names | Low, suspected. No ownership guard. Its trigger was T3 making the projection path durable; T3 did not. The projection stays in a per-invocation directory and the verdict that leaves the job is serialized separately from it, so nothing durable is handed to `--emit-json`. The defect is unchanged and the trigger now reads: a stable or shared projection location, an artifact-supplied output path, or cleanup broadened to more failure paths |
| `uaa-0029` | `parity-acquire` exports no `workflow_call` outputs | **Resolved in `3da66250`.** `on.workflow_call.outputs` now exports `closeout_ready`, `uplifts_required` and `audit_exit_code`, all from the one gate evaluation. Read `uaa-0031` before consuming them: a blocking verdict never arrives through this channel. |
| `uaa-0030` | One failed snapshot leg skips `union`, gate, commit and upload | Medium. The "continues on a partial matrix" premise in §8.2 was wrong and is corrected there. **Decided 2026-09-13: preserve completed legs** — `union` runs unless cancelled or `plan` failed; the gate routes exit 4; commit and upload run; the job still fails. **Refined 2026-09-13:** a failed *required* target still hard-fails with no union (`manifest-union` cannot build one); only non-required leg failures preserve work. **Resolved in `a3c8ce53`.** |
| `uaa-0031` | A blocking verdict is not visible on the packet PR | **Resolved in `fba98f95` / `c3607d6b`.** A blocking verdict fails `union`, and GitHub empties a failed job's outputs, so the verdict leaves the job as an artifact instead — written and uploaded before the terminal failing step, both on `always()`. A new caller job reads it and maintains one marked comment on the packet PR, holding `pull-requests: write` and nothing else: no repository write, no PAT, no checkout of the packet branch. An audit that never ran is recorded as unobserved rather than clean, and a blocking verdict is kept distinct from a failed delivery. A managed comment from a higher run id wins, so a slow earlier run cannot overwrite a current verdict with a superseded one |
| `uaa-0032` | `--expect-target-version` mismatch fails out-of-packet runs | Medium. Dry runs and promote-prerequisite re-runs now end red. **Decided 2026-09-13: fail only when committing** — a `commit: false` mismatch emits a notice and stays green; no other exit 2 may be downgraded. Mechanism: the target version is compared before the validated request load and a mismatch gets its own exit code 5. A promote-prerequisite re-run (`commit: true`) for a version the ref's request does not name is **accepted as red-but-committed**. **Resolved in `a3c8ce53` / `916c9e9b`.** |
| `uaa-0033` | Artifact bundle does not match what the run committed | Low. The `always()` upload can succeed with only stale checkout files, and omits the support-matrix files the commit stages. |
| `uaa-0034` | Commit step can push a rebased tree the gate never judged | **Resolved in `b6498b37`.** The commit step no longer rebases onto a moved branch and pushes anyway; it refuses. The retry's premise did not hold either — `agent-maintenance-open-pr` serializes on the packet branch name and the acquisition runs inside that run, so the nightly regeneration lands between acquisitions rather than during one, and a rejected push means something out of band. The bundle and the verdict are uploaded before this point, so the refusal costs a re-run rather than the work, and the verdict reports it as reached-but-not-delivered |
| `uaa-0035` | Is opencode's TUI root command excluded from parity? | **Decided 2026-09-14: exclude it.** opencode `RULES.json` excludes the root command and its 20 root-position flags (`interactive`), and report generation now checks an excluded command's flags and arguments against their own exclusions instead of dropping them, so a future root flag reaches the work queue. **Resolved in `3f7ad4c7`.** |
| `uaa-0036` | claude_code debt rows name `claude`, report-derived surfaces name `claude_code` | **Decided 2026-09-14: `command_path` is rooted at the agent id.** The two claude_code debt rows and the contract examples now read `claude_code`, and a test binds every debt row to its agent id. **Resolved in `fa739c7d`.** |
| `uaa-0037` | codex-snapshot discards raw help capture errors when no feature is enabled | Low, latent. The default crawl runs as `let _ = discover_commands(…)`, so a capture failure is dropped and the snapshot still exits 0 while the leg goes red on the raw-help upload. Trigger: a codex release whose `features list` is empty or fails, or any change to raw help capture or the snapshot job's upload order. |
| `uaa-0038` | union counts a stale committed per-target snapshot as present | **Resolved in `a474cdbc`.** The materialize step now removes each planned target's destination file before looking for this run's artifact, so a target whose leg failed is reported missing instead of inheriting its committed snapshot. The removal is unconditional and precedes the lookup; inside the copy branch it would have skipped the missing-target case it exists to catch. |
| `uaa-0039` | Closeout does not check the support-audit baseline | **Resolved 2026-09-22.** `close-agent-maintenance` records every wrapper-only row in `wrapper_only_dispositions[]`, binds the adjudicated report in `wrapper_only_baseline_ref`, and refuses an uncategorised row, an obsolete row still present in the live report, or a non-empty live `unmatched_debt_surface`. All three are re-derived from the repository. The admission gate landed with it: the command refuses unless a stand-down marker for the target version is visible on `origin/staging`, reusing `uaa-0050`'s `stand_down::run` and treating any predicate error as a refusal. Thirteen tests, all hand-authored artifacts through the manual validator. Original statement: **T8 prerequisite.** Wrapper-only rows need a recorded category, obsolete surfaces must contract publication, and unmatched debt rows must be empty; closeout checks none of these. Sized 2026-09-20 across the three live packets: codex 35 rows (8 commands, 21 flags, 6 args), claude_code 1 flag, opencode 0 — so this is 36 adjudications and almost all of them are codex. opencode's zero is legitimate, not a gap: its `wrapper_coverage.json` holds one entry whose own note bounds coverage to `run --format json`, against codex's 87 entries. That makes codex the integration case, claude_code the small non-empty case, and opencode the empty case. **Unblocked 2026-09-20 by decisions 1 and 2 (§11).** Its checks are in scope in `validate.rs` (§4), and a wrapper-only row's disposition is a list inside the closeout artifact — `wrapper_coverage.json` is generated from `codex::wrapper_coverage_manifest()` (`main.rs:89`), so a disposition written onto a coverage entry is destroyed by the next regeneration, which is `uaa-0048`'s defect in a second location. Bind the adjudicated report, not the regenerated one: a row sorted obsolete contracts publication and is then absent from the final report, so a validator diffing against that report would reject the entry as extraneous. One list, one authority — never a durable copy beside a per-run copy. Accepted cost: two of the four categories are durable properties of a surface, so codex's 35 rows are re-adjudicated every run; if that becomes painful the fix is to teach the coverage generator to carry those two, never to hand-edit its output. Pointers: T8 sequencing above, the contract's hidden-surface section and invariant 6, and §11 decisions 1 and 2.  **Pointer corrected 2026-09-22 (`uaa-0052`).** The 2026-09-21 pointer said opencode was no longer the empty case, because `uaa-0045` had enabled its global-flag model. `uaa-0052` reverted that, so opencode is the empty case again: 0 wrapper-only rows against 481 missing surfaces. One wrapper-only row across all three agents is a reporting defect rather than a surface — codex `['debug','prompt-input'] --image`. **`uaa-0051` resolved it on 2026-09-22**, so no wrapper-only row across the three agents is now a known reporting defect; the regenerated codex reports hold 32 rows in `coverage.any.json` and 34 in `coverage.all.json`, against claude_code's 0 and opencode's 0. The withdrawn pointer also argued from `upstream_available_on`; do not reuse that reasoning, because on a wrapper-only row the field is a fallback carrying no upstream flag data (`uaa-0051`). **The adjudication baseline is not an open question — state it, do not choose it (corrected 2026-09-22).** It is `coverage.any.json`: `support_audit.rs`'s `select_report_path` prefers it over `coverage.all.json`, `manifest_validate/versions.rs` *requires* it to exist for any packet at `reported`, `validated` or `supported`, and `manifest-report` always writes it while writing `coverage.all.json` only when `union.complete`. **What does need stating is a boundary:** the four categories have no honest slot for a *target-partial* surface — one upstream ships on some targets while the wrapper forwards on all. codex `sandbox --allow-unix-socket` and `--log-denials` are the live case (darwin-only upstream, `passthrough` everywhere), which is the whole difference between `any`'s 32 rows and `all`'s 34. Such a surface is not hidden, not version-scoped, not obsolete and not a discovery bug. Under the `any` baseline it never reaches adjudication, so no fifth category is needed — and no fifth category is wanted, because the surface is not a wrapper defect at all. **Settled 2026-09-22 by cross-review.** Three findings, each verified against the tree. First, the wrapper claim is accurate: those two entries describe the generic `NonTuiCommandRequest` forwarding path, which does forward on every target, while the typed path is already platform-gated (`crates/codex/src/commands/sandbox.rs:85-92` tests `matches!(platform, SandboxPlatform::Macos)`). `passthrough` asserts generic forwarding, not upstream recognition, so an unscoped claim is true as written. Second, declaring `scope` on those entries to make the rows converge is forbidden: `docs/specs/codex-wrapper-coverage-generator-contract.md` §"Scope rules (normative)" states the generator MUST NOT emit any `scope` field for v1, and the same contract forbids the generator from consulting upstream snapshots — which is the only place the scope value could have come from. claude_code populates `scope` on 21 entries (18 `win32-x64`, 3 `linux-x64,darwin-arm64`) because no equivalent generator contract binds it; that precedent does not transfer to codex. Third, a structural validator rejecting a wrapper claim whose target set exceeds upstream `available_on` was considered and declined: `RULES.json` `report.wrapper_only_policy.mode` is `report`, deliberately, and `uaa-0044` records codex 0.153.4 hiding 11 surfaces from help, so such a check would fail on every legitimate hidden-surface claim. The wrapper-only row *is* the mechanism. **Correcting an error this row carried from 2026-09-22 to 2026-09-22:** an earlier revision claimed these two rows "already mark linux and windows `partial` today." They do not. `classify_backend_support` (`support_matrix/derive.rs:612`) returns `Unsupported` on a non-empty `missing_*` list *before* it reaches `report_has_partial_caveat`, and codex 0.155.0 carries 7 missing commands, 40 missing flags and 3 missing args on every scope, so `cli_manifests/support_matrix/current.json` records `backend_support: "unsupported"` on all four targets. Wrapper-only rows contribute nothing to that classification while any surface is missing. Portability is still the support matrix's question and existence is this taxonomy's — but do not argue the split from a `partial` that is not there. |
| `uaa-0040` | Supplements cannot keep a hidden flag or positional argument observable | Low, latent. Supplement format v1 carries commands only. Trigger: the first `not_observed` debt row for a flag or argument upstream still ships. |
| `uaa-0041` | Support-surface identity is name-only | Low. Accepted values, arity, and output shape are never compared; opencode `run --format` counts as covered although the wrapper passes only `json`. |
| `uaa-0042` | A command whose wrapper coverage level is `unsupported` is never a support-audit gap | Low, latent: no wrapper coverage declares one today. Trigger: the first such declaration. |
| `uaa-0043` | The gap-list name implies newness the audit never checks | **Resolved in `fb2481c0`.** The request schema now calls the list `unbaselined_gap_surface`, matching the audit's actual baseline test. |
| `uaa-0044` | Release-notes mining and docs cross-check were designed but never built | Low. ADR 0001 §3 signals; codex 0.153.4 hides 11 surfaces from help and 7 appear nowhere in our artifacts. |
| `uaa-0045` | opencode's `RULES.json` was never normalized to the union-model schema | **Resolved 2026-09-21.** The three guards are gone and the tool, version and shard-target checks run unconditionally for every agent; opencode declares `globals` and `union.promotion_policy`; `manifest-validate` now requires `globals` and rejects a `dedupe_key` the merger would refuse. Measured against the pinned packet branch (`automation/opencode-maintenance-1.18.30` @ `962329a1`): enabling the global-flag model drops opencode's coverage-report missing surfaces from 481 to 136, because 345 of its 394 `missing_flags` were one global flag repeated per subcommand — 305 of those are `--help`, `--log-level`, `--print-logs`, `--pure` and `--version` on all 61 subcommands. Committed historical artifacts were not regenerated; the next acquisition run for a version produces the new shape. Two scope calls are recorded below the table. Original statement: **Resolve before the opencode packet closes.** Its `union` block omits the three identity guards codex and claude_code set, and it has no `globals`, so the union accepts a shard declaring another tool or version and skips the root-flag dedupe. Every missing key is `#[serde(default)]`, so a thin descriptor is silently permissive. Compounds `uaa-0038`. **Corrected 2026-09-20:** the item's "473 required uplifts" figure no longer describes the packet — the live opencode request records `required uplifts this run: none`, 15 preexisting debt rows and 0 discovered upstream surface rows, and the 481 missing-surface rows (394 flags, 60 commands, 27 args) sit in the coverage report, not the uplift queue. The two populations must not be relabelled into each other. That evidence is also not on `staging`: `cli_manifests/opencode/reports/` holds `1.4.11` and `1.14.47` only, and the 1.18.30 reports exist solely on the packet branch, so any re-measurement pins that branch. **Decided 2026-09-20 (§11 decisions 3 and 4).** The three union identity guards are deleted rather than made required, and their checks become unconditional, so step 3's instruction to add them to opencode is withdrawn — normalization removes three keys from the other two descriptors instead. Normalization otherwise covers what a consumer reads: of the eight non-guard `union` keys opencode omits, only `promotion_policy` is read (`manifest_acquisition.rs:100`/`:179`, then `parity-promote.yml:102-110`), so opencode must declare that stance explicitly instead of inheriting `false` by omission; the other seven are recorded as unread. Pointers: Workstream E in the parity generalization plan §5, and §11 decisions 3 and 4. **Partially reverted 2026-09-22 by `uaa-0052`.** Enabling opencode's `globals.effective_flags_model` was wrong: that model relocates a global flag's coverage delta to the root path, and opencode parity-excludes the root command and all 20 of its root flags, so the deduped subcommand copies were reported under no delta list at all. Rebuilding the pinned packet both ways: `missing_flags` 394 → 49 while `excluded_flags` stayed at 20, and all 345 removed rows were subcommand copies of those 20 excluded keys — whose own notes read "where a subcommand accepts the same flag, that copy stays in parity", the policy the dedupe inverts. The model stays declared with `enabled: false`. Everything else this item landed stands. Read the 481 → 136 figure above as a suppression, not a coverage measurement. |
| `uaa-0046` | Debt authorization does not constrain matches by target or upstream version | **Resolved in `b52f1242` / `4c292da2`.** Each debt row now carries a required `scope_target_triples`, an `authorized_at_version`, and an `authorization_evidence_ref`, and a row whose scope exceeds the surface's observations is rejected. Wrapper coverage already supports target scope: `scope.target_triples` is a first-class mechanism in `crates/xtask/src/wrapper_coverage_shared.rs`, validated against the agent's expected targets, and claude_code populates it on 21 entries while codex and opencode populate it on none. The debt inventory never adopted it — its parser reads ten fixed keys and silently ignores any other, so a deferral argued for one target authorizes the same surface on a target added later. This is adoption of an existing mechanism, not invention of a new one, but the default must not be adopted with it: an omitted coverage scope means all expected targets, which on an authorization record would grant permission by omission. Debt scope is required instead. Distinct from `uaa-0041`, which is about values, arity and output shape. Pointers: `wrapper_coverage_shared.rs`, and the Surface identity rules in the request contract. |
| `uaa-0047` | Permission-test fixtures restore the directory mode only on the success path | **Resolved in `dea84bf8`.** Both fixtures restore through a `Drop` guard whose body ignores a failed restore, since a panicking destructor during unwinding aborts the process. Each fixture is now established by a direct filesystem probe rather than by the behaviour of the code under test, so a regression cannot present itself as an environmental skip. The file was split to make room. |
| `uaa-0048` | Nightly regeneration force-pushes an open packet branch | **Resolved 2026-09-21.** A maintainer-committed `automation-stand-down.toml` on the base branch is the ownership predicate, checked at all three destructive boundaries; see the stand-down subsection under T8 sequencing for the design and for what was deliberately left out. Original finding: the watcher re-dispatches `agent-maintenance-open-pr` every night with no dedupe against an open packet, so `create-pull-request` resets the packet branch to `staging`, re-applies the packet and force-pushes. Proven 2026-09-19: PRs #211 and #208 had unchanged target versions for four and five days and carried only commits from the previous night. A closeout committed to a packet branch would therefore not survive until merge, regenerating HANDOFF back to the open-run contract. **Widened 2026-09-20.** The same regeneration also invalidates a closeout that was never committed, through the request-hash clock under T4 — so this gates T6's output being usable, not only T8. Protection must mean *automation has lost authority over this packet generation*, never *a valid closeout is present*: the late reading still permits erasing categorization work, a partially authored closeout, or a valid one during an intentional edit. It must also survive a merged packet, which has no open PR and no closeout, so every open-PR-shaped signal misses it — codex 0.155.0 reached that state when #215 merged without promotion. Pointer: T8 sequencing above. **Merged 2026-09-21 as PR #219.** Three follow-ups are split to `uaa-0050`: the acquisition path an agent actually follows, the supersession boundary's stale read, and marker retirement at promotion. |
| `uaa-0049` | A generic engine branches on the codex agent id | Low. `contract_policy.rs` appends one extra `writable_surfaces` entry behind `if entry.agent_id == "codex"`, which the repository's own rule puts in descriptor data. Impact today is one spec file; the cost is the precedent, in the engine that decides what a packet may write. Distinct from the relay-host constants in the same file, which name codex as the local execution host and are intentional (§1 verified state, §4 deliberately untouched). |
| `uaa-0050` | The stand-down guard has no acquisition path, one boundary reads a stale checkout, and a marker never retires | **Resolved 2026-09-21.** Completes `uaa-0048` (PR #219). Three parts. (1) Acquisition: the actor is a coding agent handed the packet PR, so the freeze is an instruction — rendered into the generated `HANDOFF.md` and stated in `AGENTS.md` — not a new remote-write capability for `xtask`, which has never pushed. Four points carry it because an agent's default is wrong on each: freeze before judgment work rather than before the closeout command, publish to base rather than the packet branch, commit only the marker (`git add <path>` then a path-limited `git commit <path>`, since the marker is always a new file), and nothing but retirement releases it. (2) Supersession fetches base and reads through `--from-ref` per candidate; request regeneration uses the same mechanism, because a re-run preserves the original event's `GITHUB_SHA` and step order does not imply freshness. (3) Retirement rides into the promotion PR from `parity-promote`'s pointer-advance step — one commit with the `status` flip, maintainer-merged, exactly one version. Cleanup triggered by a *new version* is rejected: codex ships near-daily and a closeout spans days, so it would delete the marker of the packet being worked on. Accepted residual: nothing refuses to rebuild a promoted version, so a dispatch or 30-day re-run yields a spurious PR — loud, not silent, so the writer-side guard waits until one is observed. Pointers: the stand-down subsection under T8 sequencing, and `uaa-0039` for the admission gate. |
| `uaa-0051` | The effective-flags model was enforced on the union side of the coverage comparison only | **Resolved 2026-09-22.** `normalize_union_commands` deletes a subcommand's copy of a root flag, so a wrapper entry claiming that flag at the subcommand found no counterpart and the row was emitted as `wrapper_only` — a surface the wrapper claims and upstream does not have. Upstream has it, at root, and the model's own definition says the subcommand inherits it. `upstream_flag_availability` now credits the root flag when the agent's model is enabled and a root key matches, and reports *that flag's* availability, so one genuinely absent on some target still fails the filter. One instance repository-wide: codex `['debug','prompt-input'] --image`. Three decisions taken and recorded on the backlog item: the corrected row **disappears** rather than carrying a new field, because under the model the union does show the surface and a field would also be a report-schema change; missing-surface detection needs **no mirror**, because it iterates the post-normalization union where a global appears once, at root; and the descriptor's `reporting` clause is **neither unimplemented nor false** — it is enforced by the union normalization on one side and this credit on the other, and all three descriptors now say so. codex 0.155.0's six reports were regenerated (one row removed from each, nothing added, support-matrix derivation unchanged), which is in scope where `uaa-0045` declined to regenerate opencode's promoted history: 0.155.0 is the reported packet `uaa-0039` will adjudicate. |
| `uaa-0052` | opencode's effective-flags model cancels its root parity exclusions, so 345 surfaces are reported nowhere | **Resolved 2026-09-22.** A regression `uaa-0045` introduced in PR #222. `normalize_union_commands` deletes every subcommand copy of a root flag; the model's `reporting` clause puts the surviving delta at the root path; opencode parity-excludes the root command and all 20 of its root flags. The three compose to silence: the surface appears under no delta list, neither `missing_*` nor `excluded_*`. Proven by rebuilding the pinned packet both ways — `missing_flags` 394 → 49, `excluded_flags` 20 either way, and every one of the 345 removed rows a subcommand copy of one of those 20 keys. The exclusions' own notes say "where a subcommand accepts the same flag, that copy stays in parity", so the switch inverted a policy written 20 times in the same file. Fix: `enabled: false`, block kept so the stance is stated rather than absent, plus a `manifest-validate` guard refusing the model when the root command is parity-excluded. The root cause is mechanism choice, not the model — `parity_exclusions` is the auditable way to suppress a surface and its contract is that an excluded unit *stays visible* under `excluded_*` with a category and a reason. Using it here costs 345 hand entries today, which is `uaa-0055`. |
| `uaa-0053` | opencode's wrapper covers one command, so 481 upstream surfaces are unwrapped | **Deliberately off this spec's critical path (maintainer, 2026-09-22).** `wrapper_coverage.json` holds one entry — `run`, six flags, one argument, generated 2026-04-18 — against codex's 87 and claude_code's 30. Support-matrix rows read `backend_support: unsupported` on all 18, and `latest_validated` is 1.4.11 while upstream ships 1.18.31. So opencode is wired into the measuring half of the lifecycle and never built out on the wrapping half, and the 481 is the accurate size of that gap rather than reporting noise. The maintainer wants it finished; it must not pull T4–T8 with it, and T4–T8 must not wait for it. `uaa-0039` keeps treating opencode as the empty case for wrapper-only rows, which stays true while the wrapper claims one command. |
| `uaa-0054` | A root-flag parity exclusion under an enabled effective-flags model silently deletes subcommand copies | Low, latent. The narrower half of `uaa-0052`, filed rather than guarded. codex declares `--no-alt-screen` as a root-path exclusion with the model enabled; it costs nothing at 0.155.0 because no subcommand repeats that flag, and a descriptor-level guard strict enough to catch it would reject that harmless declaration. Deciding the case needs the union, which `manifest-validate` does not read. Trigger: the first subcommand repeating a parity-excluded root flag, or a second agent adopting a root-flag exclusion under an enabled model. |
| `uaa-0055` | `parity_exclusions` cannot cover a path pattern, so suppressing a repeated global costs one entry per subcommand | Design, **needs a maintainer decision** — §7 puts `RULES.json` schema changes on *Ask first*. An exclusion unit names one exact `(path, key)` pair, so excluding `--help` across opencode's 61 subcommands is 61 entries and the five repeated globals are 305. That verbosity is why `uaa-0045` reached for the structural switch instead. A pattern-covering form keeps the contract opencode's own descriptor states — excluded units stay visible under `excluded_*` and never appear under `missing_*` — at one entry with one category and one reason. Not on the critical path: `uaa-0039` adjudicates concrete rows, which either form preserves. |

### 8.2 What T1 changed about T2

The gate now has three outcomes to route, not two: clean (0), uplifts required (3), and
insufficient evidence — where insufficient evidence includes *the matrix ran but did not finish*.
T2a gave that case its own exit code, **4**, so CI can route it by number; every other evidence
or validation failure stays exit 2.

T2's original wiring sketch — "other → fail" — is therefore no longer adequate on its own.

**Decided 2026-07-25:** retry missing legs once; if the acquisition is still incomplete after that
retry, fail the run. The maintainer then clarified the mechanism: a step-level retry-once of the
snapshot capture inside each matrix leg, because Actions cannot re-run individual matrix legs.
T2b implemented it that way. Every other failure fails without a retry.

**Correction (2026-09-13).** This section and plan doc §18.1 / §18.4 said `parity-acquire`
continues on a partial matrix by design. The job graph does not. `fail-fast: false` keeps the other
legs running, but `union` has `needs: [plan, snapshot]` and no status-function `if:`, so a leg that
fails both capture attempts skips `union` — and with it the gate, the commit and the artifact
bundle. `union.json` is written with `complete: false` only when a leg succeeds but its snapshot
artifact never arrives. Because the matrix and `union.expected_targets` come from the same list,
exit 4 is close to unreachable in CI; it still matters for local runs against committed history.
**Decided 2026-09-13:** one failed leg must not discard the other legs' work. `union` will run
unless the run was cancelled or `plan` failed, so the gate routes exit 4 and the job fails after
commit and upload (`uaa-0030`). A failed *required* target is the exception: no union can be built
without it, so that run still hard-fails with nothing committed.

---

## 9. Acceptance criteria

1. An acquisition run whose release adds no surface emits a closeout-ready marker; one that adds
   surface emits a relay invocation and does not mark closeout-ready.
2. `prepare-agent-closeout` produces an artifact that `close-agent-maintenance` accepts with **no
   hand editing**, for all three live packets. Qualified 2026-09-24: "no hand editing" is a property
   of the *artifact*, not a claim that no judgement was required to reach it. Every live wrapper-only
   row must already carry a disposition, because those are adjudications and §3 records that T6
   neither derives nor invents them. Measured that day, claude_code and opencode have zero such rows
   and satisfy this criterion outright; codex has 32 and satisfies it once they are adjudicated. T6
   refusing on an unadjudicated row is this criterion being enforced, not missed.
   **Corrected 2026-09-24.** The preceding claude_code zero was measured from trunk's 2.1.29
   report, not the live 2.1.273 packet. At 2.1.273 claude_code has four wrapper-only rows: three
   commands and one flag. opencode has zero at 1.18.31. codex has 32 at 0.156.0: eight commands,
   18 flags and six arguments. claude_code therefore satisfies this criterion, like codex, once
   its rows are adjudicated.
3. The generator refuses to emit when CI evidence for the recorded SHA is missing or not green,
   and says why.
4. No existing validation rule was relaxed.
5. All three packets are closed, with HANDOFF regenerated by `close-agent-maintenance` — each
   closed **on its own branch before merge** (codex excepted, as a catch-up against `main`; see
   T8 sequencing). No packet is merged in the open state. **Not satisfiable for opencode as
   written; see `uaa-0066`.**
6. The lifecycle has exactly two human-in-the-loop points plus PR merges, and the docs say so.
7. `make preflight` green; new code has tests; both review lanes clean.

---

## 10. Risks

- **CI-evidence availability.** GitHub retains run metadata finitely. Closeouts are written once and
  committed, so regeneration is rare, but the generator must degrade to a clear refusal rather than
  a wrong answer.
- **Audit re-derivation cost.** Re-deriving after acquisition adds runtime to an already long
  matrix job. Measure before optimizing.
- **`Satisfied` provenance.** The roadmap records a known defended-in-depth weakness: reconciliation
  trusts coverage-report *content*, so an emptied report is not distinguished from genuine
  reconciliation. This spec does not fix that and must not appear to. Out of scope, noted so the
  gate is not mistaken for a stronger guarantee than it is.
- **Scope creep into the relay.** T3 renders an invocation; it does not orchestrate the relay.

---

## 11. Open questions

**None open.** Three were opened 2026-09-20 and decided the same day; verifying them raised a
fourth, decided with them. Each sits on §7's *Ask first* list, so each is recorded here with its
evidence rather than settled by an implementer in passing. The closeout-prerequisite group is
unblocked.

**Corrected 2026-09-24 (`uaa-0066`).** One §7 *Ask first* question is now open for the maintainer:
acceptance criterion 5 requires closing opencode, but 473 of opencode 1.18.31's 481 live uplifts are
newly discovered surface that field invariant 3 forbids deferring, while `uaa-0053` says T4–T8 must
not wait for wrapper expansion. The maintainer must choose one of three paths before T8 starts on
opencode: amend criterion 5 to close codex and claude_code now and opencode after `uaa-0053`; pull
`uaa-0055` forward so deliberate non-support becomes parity exclusions; or baseline the unwrapped
surface as debt before the freeze. Under the current derivation, the 481 report gaps are the uplift
queue: every gap is an uplift unless a debt row authorizes it at the target version. That is why the
queue is 481: 473 new surfaces plus 8 debt rows not yet re-authorized. This supersedes the `uaa-0045`
caution, which was written when the queue was empty. The four decisions below remain settled.

1. *Is `uaa-0039` the authorized exception to §4's `validate.rs` freeze?* **Yes — and §4's wording
   was the defect.** The freeze is directional, not locational: adding a check is in scope,
   weakening one is not, wherever it lives. The rule and its ordering rationale now live in the
   `validate.rs` bullet in §4. The validator `uaa-0039` leaves behind is the fixed authority T6
   must satisfy unmodified.
2. *Where does a wrapper-only row's disposition live?* **A list inside the closeout artifact.** The
   alternative was not available: `cli_manifests/<agent>/wrapper_coverage.json` is generated from
   `codex::wrapper_coverage_manifest()` (`main.rs:89`), so a disposition hand-written onto a
   coverage entry is destroyed by the next regeneration — and `generated_at` on codex's file moved
   on 2026-09-17, so regeneration is routine rather than rare. Pushing the disposition up into the
   Rust source avoids that but puts a governance adjudication behind a code change and the
   wrapper's release cycle. The closeout artifact is the opposite: written once per run,
   maintainer-approved, already digest-bound to its request, and its shape takes the addition
   without a new mechanism — `resolved_findings` and `deferred_findings` are already
   `Vec<MaintenanceFinding>` beside scalars (`types.rs:70-78`). Two constraints follow. Bind the
   **adjudicated** report rather than recomputing against the final one, because a row sorted
   obsolete contracts publication and is then gone from the regenerated report. And keep one
   authority: one list, in the artifact, never a durable copy beside a per-run copy. T6 emits this
   field, so its shape is settled before T6 starts. Accepted cost: two of the four categories
   (hidden upstream but still supported, supported only on older upstream versions) are durable
   properties of a surface, so codex's 35 rows are re-adjudicated every run. If that proves
   painful, the fix is to teach the coverage generator to carry those two — never to hand-edit its
   output.
3. *Are the union identity guards required, and may one be explicitly `false`?* **Neither — all
   three are deleted and their checks become unconditional.** The struct already answers it:
   `tool_name`, the value compared against, is mandatory, while the decision to compare it is
   `#[serde(default)] bool` and defaults off (`manifest_union.rs:97-113`). That is backwards, and
   it is exactly how opencode became permissive by omission. No agent wants them off — codex and
   claude_code set all three true and opencode's `false` is nobody's choice — and there is no
   second acquisition model to configure for: `manifest_union.rs:245` hardcodes `mode: "union"`
   and `union.mode` is never deserialized. The generalization plan names `union.tool_name` and
   `union.raw_help_layout` as the only two agent-specific behaviours that belong in manifest data;
   these three are universal safety checks and belong with the shard-target check `uaa-0045` step
   2 already makes unconditional for every agent. Deleting them collapses presence and permitted
   value into nothing, needs no migration beyond removing three keys from two descriptors, and
   cannot regress the way a required field with a legal `false` can. One implementation note: made
   unconditional, `require_semantic_version`'s check is unreachable, because a `None` semantic
   version already fails the same-version comparison (`manifest_union.rs:175-191`) — expect two
   checks, not three, and do not add a test for a case that cannot occur.
   **Implemented 2026-09-21.** Two checks, plus the shard-target check, all unconditional
   (`manifest_union.rs`). The three keys are gone from codex's and claude_code's descriptors. Every
   shard committed anywhere in the repository, and all three shards on the pinned opencode packet
   branch, already satisfy both identity checks, so no packet was invalidated by making them
   mandatory.
4. *Does `uaa-0045`'s "normalize" cover the `union` keys opencode omits beyond the guards?* **Only
   the one a consumer reads.** opencode's `union` block holds 5 keys against codex's and
   claude_code's 16; beyond the three guards it omits `availability`, `canonical_keys`,
   `conflict_handling`, `determinism`, `mode`, `partial_union_policy`, `platform_granularity` and
   `promotion_policy`. Seven of those eight are read by nothing. `promotion_policy` is the
   exception and is in scope: `manifest_acquisition.rs:100` deserializes it, `:179` resolves
   `allow_promote_when_incomplete` with `.unwrap_or(false)`, and `parity-promote.yml:102-110`
   reads the resolved value to decide whether an incomplete union may promote. Measured
   2026-09-20: opencode's acquisition plan for `1.18.30` resolves `false`. Absent policy is absent
   permission by design, so this is fail-closed rather than a hole — but it is still a stance held
   by omission, which is the defect decision 3 removes, so opencode must declare it rather than
   inherit it. Record the other seven as unread instead of adding them; `mode` in particular can
   never be honoured. One of them is already cited as an authority it does not carry: the
   generalization plan attributes the engine's refusal to build a union without the required
   target to `partial_union_policy.when_required_target_missing`, but that refusal is hardcoded at
   `manifest_union.rs:196-201` and the key is never read. Correct the citation; do not adopt the
   key to make it true.
   **Implemented 2026-09-21, and extended to the four top-level keys.** opencode declares
   `union.promotion_policy` with `allow_promote_when_incomplete: false`, which is the value it
   already resolved, so nothing changes except that the stance is now stated. The same rule then
   had to be applied one level up, because the item's step 3 predates this decision and named four
   top-level keys — `comparison`, `features`, `globals`, `supplements` — from before anyone asked
   which of them a consumer reads. Enumerated across all ten structs that deserialize a `RULES.json`
   root: only `globals` is read (`manifest_union.rs:70`). `comparison` and `features` are read by
   nothing, and `features` would additionally describe a feature-enable CLI surface opencode does
   not have. `supplements` is read by nothing either — `parity-acquire.yml:413` finds the file by
   filesystem convention at `<root>/supplement/commands.json`, which opencode already has and
   already uses. So opencode gained `globals` alone, and the other three are recorded as unread.
   Adding them would have been the step the item's own 1b forbids: copying another agent's
   operational values to satisfy structural similarity.
   **Corrected 2026-09-22 (`uaa-0052`).** Declaring `globals` was right; *enabling* the model inside
   it was not, and that part is reverted. The decision rule this paragraph states — declare what a
   consumer reads — answers which keys belong in a descriptor, and says nothing about which values
   are safe. `globals.effective_flags_model` is read, so opencode must declare it; the value must
   still be the one that is true for opencode, and with its root command parity-excluded that value
   is `false`. A declared key is a place to state a stance, never an instruction to turn it on.

Two settled during specification:

- *Does closeout precede promotion?* **Yes** — promotion writes outside the maintenance contract by
  design (`writable_surfaces` excludes the pointers), so it is a separate lane after the run closes.
- *Should the lane run `refresh-agent`?* **No** — `refresh-agent` is the recovery command; packet
  docs are owned by `prepare-agent-maintenance` at open time, and all three packets already
  reconcile `exact`.

  **Corrected 2026-09-24 (`uaa-0058`, `uaa-0062`).** The premise that all three packets reconcile
  `exact` no longer holds. The decision is for the acquisition lane to complete the generation with
  `prepare-agent-maintenance --from-request` after the target artifacts exist; PR B (`uaa-0058`)
  implements it. The lane does not use `refresh-agent`, which strict-loads the stale request and
  cannot perform that transition. `refresh-agent` remains the documented recovery command pending
  `uaa-0062`.
