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
cargo run -p xtask -- prepare-agent-closeout \
  --request <path/to/maintenance-request.toml> \
  --commit <sha> \
  --preflight-run <github-run-id | --preflight-from-ci> \
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
warning printed). The exit code is the product; the projection is advisory until `uaa-0025` is
resolved.

---

## 4. Project structure

### New files

```
crates/xtask/src/agent_maintenance/audit_status.rs        # piece 1: derive + emit audit JSON
crates/xtask/src/agent_maintenance/stand_down.rs          # uaa-0048: packet ownership predicate
crates/xtask/src/agent_maintenance/stand_down/tests.rs
crates/xtask/src/agent_maintenance/prepare_closeout.rs    # piece 2: closeout generation
crates/xtask/src/agent_maintenance/prepare_closeout/
    evidence.rs                                           # CI-conclusion resolution, commit-pinned
    findings.rs                                           # derive resolved_findings from written surfaces
crates/xtask/tests/agent_maintenance_uplift_gate.rs
crates/xtask/tests/agent_maintenance_closeout_generation.rs
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

**T5 — Closeout finding derivation.** Map written surfaces to `MaintenanceDriftCategory`
(`registry_manifest_drift`, `support_publication_drift`) with real surface lists; choose
`explicit_none_reason` vs `deferred_findings` from the live drift report.

**T6 — `prepare-agent-closeout` command.** Compose T4 + T5, emit the artifact, and self-verify by
running the real `validate_closeout` before writing.

**T7 — Docs.** Plan doc §18; operator guide gains closeout as an explicit lifecycle step; the
§13 maintainer checklist gains the step it currently omits.

**T8 — Prove it.** Run the chain against all three live packets; confirm `make preflight` green;
both review lanes clean with findings adjudicated.

Order: T1 → T2 → (T3 ∥ T4) → T5 → T6 → T7 → T8.

### Backlog groups that land inside that order

The open items in §8.1 are not a queue to drain after T8. Five of them gate a T item and land with
it. Recorded 2026-09-19, after `uaa-0029`, `uaa-0038`, `uaa-0043`, `uaa-0046` and `uaa-0047` landed
in #216.

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
`uaa-0039`'s enforcement is what pushes on it. Budget that split as a first commit if the checks
land there; do not ritually re-split the two files T3 already corrected.

### T8 sequencing — closeout happens *before* merge

A packet must be closed **on its own branch**, so the merge carries a closed HANDOFF. Codex
demonstrates the failure mode: #153 merged while still open, which replaced main's closed HANDOFF
with the open-run contributor contract. That is a regression to repair, not a pattern to repeat.

Re-derived 2026-09-13 (read-only). The watcher kept opening packets after this spec was written:
#157 (claude_code 2.1.212) and #158 (opencode 1.18.4) are **closed without merging**, superseded by
the open packets below. Each branch request's `target_version` and `version_policy` were read from
the packet branch; re-derive this table again (`git fetch`, `gh pr list`) when T8 starts, because
the watcher will have moved on.

| packet | closed against | why |
|---|---|---|
| claude_code 2.1.236 (#195) | the packet branch | branch request reads `2.1.236` / `upstream_stable_pointer` |
| opencode 1.18.29 (#205) | the packet branch | branch request reads `1.18.29` / `latest_stable_minus_one` |
| codex 0.153.4 (#206) | the packet branch | branch request reads `0.153.4` / `latest_stable_minus_one` |
| codex 0.144.6 (#153, merged) | a branch off `main` | already merged open; needs a catch-up pass |

**Open decisions that gate a closeout.** Re-deriving the table above does not clear these; each
stays until its backlog item records a maintainer decision. None is open: `uaa-0035` and `uaa-0036`
were decided on 2026-09-14 (§8.1).

**Support-audit checks that gate a closeout.** Decided 2026-09-15. Discovery reads help output, so a
surface upstream hides is not an obligation by itself, but it stays one while the wrapper claims it
(maintenance request contract, "Hidden upstream surfaces and wrapper-only rows"). Before a packet
closes, every wrapper-only row in its live coverage report must be sorted into one category —
hidden upstream but still supported, supported only on older upstream versions, obsolete, or a
discovery bug — a surface sorted obsolete must contract publication truth in the same run, and the
live audit must have no unmatched debt rows (contract field invariant 6). Nothing records the
categories yet, and `close-agent-maintenance` checks none of the three; only
`maintenance-audit-status` rejects unmatched debt rows (exit 2). Do not close a packet until
`uaa-0039` lands.

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
on 2026-09-19 added one (`uaa-0048`).

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
| `uaa-0039` | Closeout does not check the support-audit baseline | **T8 prerequisite.** Wrapper-only rows need a recorded category, obsolete surfaces must contract publication, and unmatched debt rows must be empty; closeout checks none of these. Sized 2026-09-20 across the three live packets: codex 35 rows (8 commands, 21 flags, 6 args), claude_code 1 flag, opencode 0 — so this is 36 adjudications and almost all of them are codex. opencode's zero is legitimate, not a gap: its `wrapper_coverage.json` holds one entry whose own note bounds coverage to `run --format json`, against codex's 87 entries. That makes codex the integration case, claude_code the small non-empty case, and opencode the empty case. **Unblocked 2026-09-20 by decisions 1 and 2 (§11).** Its checks are in scope in `validate.rs` (§4), and a wrapper-only row's disposition is a list inside the closeout artifact — `wrapper_coverage.json` is generated from `codex::wrapper_coverage_manifest()` (`main.rs:89`), so a disposition written onto a coverage entry is destroyed by the next regeneration, which is `uaa-0048`'s defect in a second location. Bind the adjudicated report, not the regenerated one: a row sorted obsolete contracts publication and is then absent from the final report, so a validator diffing against that report would reject the entry as extraneous. One list, one authority — never a durable copy beside a per-run copy. Accepted cost: two of the four categories are durable properties of a surface, so codex's 35 rows are re-adjudicated every run; if that becomes painful the fix is to teach the coverage generator to carry those two, never to hand-edit its output. Pointers: T8 sequencing above, the contract's hidden-surface section and invariant 6, and §11 decisions 1 and 2. |
| `uaa-0040` | Supplements cannot keep a hidden flag or positional argument observable | Low, latent. Supplement format v1 carries commands only. Trigger: the first `not_observed` debt row for a flag or argument upstream still ships. |
| `uaa-0041` | Support-surface identity is name-only | Low. Accepted values, arity, and output shape are never compared; opencode `run --format` counts as covered although the wrapper passes only `json`. |
| `uaa-0042` | A command whose wrapper coverage level is `unsupported` is never a support-audit gap | Low, latent: no wrapper coverage declares one today. Trigger: the first such declaration. |
| `uaa-0043` | The gap-list name implies newness the audit never checks | **Resolved in `fb2481c0`.** The request schema now calls the list `unbaselined_gap_surface`, matching the audit's actual baseline test. |
| `uaa-0044` | Release-notes mining and docs cross-check were designed but never built | Low. ADR 0001 §3 signals; codex 0.153.4 hides 11 surfaces from help and 7 appear nowhere in our artifacts. |
| `uaa-0045` | opencode's `RULES.json` was never normalized to the union-model schema | **Resolve before the opencode packet closes.** Its `union` block omits the three identity guards codex and claude_code set, and it has no `globals`, so the union accepts a shard declaring another tool or version and skips the root-flag dedupe. Every missing key is `#[serde(default)]`, so a thin descriptor is silently permissive. Compounds `uaa-0038`. **Corrected 2026-09-20:** the item's "473 required uplifts" figure no longer describes the packet — the live opencode request records `required uplifts this run: none`, 15 preexisting debt rows and 0 discovered upstream surface rows, and the 481 missing-surface rows (394 flags, 60 commands, 27 args) sit in the coverage report, not the uplift queue. The two populations must not be relabelled into each other. That evidence is also not on `staging`: `cli_manifests/opencode/reports/` holds `1.4.11` and `1.14.47` only, and the 1.18.30 reports exist solely on the packet branch, so any re-measurement pins that branch. **Decided 2026-09-20 (§11 decisions 3 and 4).** The three union identity guards are deleted rather than made required, and their checks become unconditional, so step 3's instruction to add them to opencode is withdrawn — normalization removes three keys from the other two descriptors instead. Normalization otherwise covers what a consumer reads: of the eight non-guard `union` keys opencode omits, only `promotion_policy` is read (`manifest_acquisition.rs:100`/`:179`, then `parity-promote.yml:102-110`), so opencode must declare that stance explicitly instead of inheriting `false` by omission; the other seven are recorded as unread. Pointers: Workstream E in the parity generalization plan §5, and §11 decisions 3 and 4. |
| `uaa-0046` | Debt authorization does not constrain matches by target or upstream version | **Resolved in `b52f1242` / `4c292da2`.** Each debt row now carries a required `scope_target_triples`, an `authorized_at_version`, and an `authorization_evidence_ref`, and a row whose scope exceeds the surface's observations is rejected. Wrapper coverage already supports target scope: `scope.target_triples` is a first-class mechanism in `crates/xtask/src/wrapper_coverage_shared.rs`, validated against the agent's expected targets, and claude_code populates it on 21 entries while codex and opencode populate it on none. The debt inventory never adopted it — its parser reads ten fixed keys and silently ignores any other, so a deferral argued for one target authorizes the same surface on a target added later. This is adoption of an existing mechanism, not invention of a new one, but the default must not be adopted with it: an omitted coverage scope means all expected targets, which on an authorization record would grant permission by omission. Debt scope is required instead. Distinct from `uaa-0041`, which is about values, arity and output shape. Pointers: `wrapper_coverage_shared.rs`, and the Surface identity rules in the request contract. |
| `uaa-0047` | Permission-test fixtures restore the directory mode only on the success path | **Resolved in `dea84bf8`.** Both fixtures restore through a `Drop` guard whose body ignores a failed restore, since a panicking destructor during unwinding aborts the process. Each fixture is now established by a direct filesystem probe rather than by the behaviour of the code under test, so a regression cannot present itself as an environmental skip. The file was split to make room. |
| `uaa-0048` | Nightly regeneration force-pushes an open packet branch | **Resolved 2026-09-21.** A maintainer-committed `automation-stand-down.toml` on the base branch is the ownership predicate, checked at all three destructive boundaries; see the stand-down subsection under T8 sequencing for the design and for what was deliberately left out. Original finding: the watcher re-dispatches `agent-maintenance-open-pr` every night with no dedupe against an open packet, so `create-pull-request` resets the packet branch to `staging`, re-applies the packet and force-pushes. Proven 2026-09-19: PRs #211 and #208 had unchanged target versions for four and five days and carried only commits from the previous night. A closeout committed to a packet branch would therefore not survive until merge, regenerating HANDOFF back to the open-run contract. **Widened 2026-09-20.** The same regeneration also invalidates a closeout that was never committed, through the request-hash clock under T4 — so this gates T6's output being usable, not only T8. Protection must mean *automation has lost authority over this packet generation*, never *a valid closeout is present*: the late reading still permits erasing categorization work, a partially authored closeout, or a valid one during an intentional edit. It must also survive a merged packet, which has no open PR and no closeout, so every open-PR-shaped signal misses it — codex 0.155.0 reached that state when #215 merged without promotion. Pointer: T8 sequencing above. **Merged 2026-09-21 as PR #219.** Three follow-ups are split to `uaa-0050`: the acquisition path an agent actually follows, the supersession boundary's stale read, and marker retirement at promotion. |
| `uaa-0049` | A generic engine branches on the codex agent id | Low. `contract_policy.rs` appends one extra `writable_surfaces` entry behind `if entry.agent_id == "codex"`, which the repository's own rule puts in descriptor data. Impact today is one spec file; the cost is the precedent, in the engine that decides what a packet may write. Distinct from the relay-host constants in the same file, which name codex as the local execution host and are intentional (§1 verified state, §4 deliberately untouched). |
| `uaa-0050` | The stand-down guard has no acquisition path, one boundary reads a stale checkout, and a marker never retires | **Resolved 2026-09-21.** Completes `uaa-0048` (PR #219). Three parts. (1) Acquisition: the actor is a coding agent handed the packet PR, so the freeze is an instruction — rendered into the generated `HANDOFF.md` and stated in `AGENTS.md` — not a new remote-write capability for `xtask`, which has never pushed. Four points carry it because an agent's default is wrong on each: freeze before judgment work rather than before the closeout command, publish to base rather than the packet branch, commit only the marker (`git add <path>` then a path-limited `git commit <path>`, since the marker is always a new file), and nothing but retirement releases it. (2) Supersession fetches base and reads through `--from-ref` per candidate; request regeneration uses the same mechanism, because a re-run preserves the original event's `GITHUB_SHA` and step order does not imply freshness. (3) Retirement rides into the promotion PR from `parity-promote`'s pointer-advance step — one commit with the `status` flip, maintainer-merged, exactly one version. Cleanup triggered by a *new version* is rejected: codex ships near-daily and a closeout spans days, so it would delete the marker of the packet being worked on. Accepted residual: nothing refuses to rebuild a promoted version, so a dispatch or 30-day re-run yields a spurious PR — loud, not silent, so the writer-side guard waits until one is observed. Pointers: the stand-down subsection under T8 sequencing, and `uaa-0039` for the admission gate. |

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
   hand editing**, for all three live packets.
3. The generator refuses to emit when CI evidence for the recorded SHA is missing or not green,
   and says why.
4. No existing validation rule was relaxed.
5. All three packets are closed, with HANDOFF regenerated by `close-agent-maintenance` — each
   closed **on its own branch before merge** (codex excepted, as a catch-up against `main`; see
   T8 sequencing). No packet is merged in the open state.
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

Two settled during specification:

- *Does closeout precede promotion?* **Yes** — promotion writes outside the maintenance contract by
  design (`writable_surfaces` excludes the pointers), so it is a separate lane after the run closes.
- *Should the lane run `refresh-agent`?* **No** — `refresh-agent` is the recovery command; packet
  docs are owned by `prepare-agent-maintenance` at open time, and all three packets already
  reconcile `exact`.
