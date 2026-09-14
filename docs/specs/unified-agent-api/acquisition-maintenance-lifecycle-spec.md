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
| 3 | `EXIT_UPLIFTS_REQUIRED` | uplifts required; contributor relay work needed | no — a result |
| 4 | `EXIT_INCOMPLETE_ACQUISITION` | union incomplete; names `missing_targets` | yes |
| 5 | `EXIT_TARGET_VERSION_MISMATCH` | the request's `detected_release.target_version` differs from `--expect-target-version`; checked before any evidence work, so it wins over malformed evidence or an invalid request field — which also means exit 5 says nothing about whether the rest of the request is valid | yes — blocking in CI only when the run commits |
| 2 | `EXIT_VALIDATION` | evidence missing, bound to another version, or malformed; invalid request; drift with no uplifts | yes |
| 1 | `EXIT_INTERNAL` | internal fault | yes |

A computed 0 or 3 survives a failed `--emit-json` write (the stale projection is removed and a
warning printed). The exit code is the product; the projection is advisory until `uaa-0025` is
resolved.

---

## 4. Project structure

### New files

```
crates/xtask/src/agent_maintenance/audit_status.rs        # piece 1: derive + emit audit JSON
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

**T3 — Relay-packet rendering.** On the uplift branch, render the relay invocation (prompt path,
dry-run→write `--run-id` handshake) into the packet PR body from the existing renderer, so the
maintainer pastes one command. Reuses `docs.rs` rendering; no new prompt source of truth.

**T4 — Closeout evidence resolution.** Commit-pinned CI conclusion lookup, fail-closed. This is the
highest-risk unit; it decides whether a governance artifact can be trusted.

**T5 — Closeout finding derivation.** Map written surfaces to `MaintenanceDriftCategory`
(`registry_manifest_drift`, `support_publication_drift`) with real surface lists; choose
`explicit_none_reason` vs `deferred_findings` from the live drift report.
Open decision in its path: `uaa-0035` (whether opencode's TUI root command, surface `commands` /
`opencode` / `opencode`, is excluded from parity). T5 must surface that row as an unresolved
obligation, never pick a disposition for it.

**T6 — `prepare-agent-closeout` command.** Compose T4 + T5, emit the artifact, and self-verify by
running the real `validate_closeout` before writing.

**T7 — Docs.** Plan doc §18; operator guide gains closeout as an explicit lifecycle step; the
§13 maintainer checklist gains the step it currently omits.

**T8 — Prove it.** Run the chain against all three live packets; confirm `make preflight` green;
both review lanes clean with findings adjudicated.

Order: T1 → T2 → (T3 ∥ T4) → T5 → T6 → T7 → T8.

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
stays until its backlog item records a maintainer decision.

- `uaa-0035`: before closing **any** opencode packet whose audit lists the root command (`commands`
  / `opencode` / `opencode`), or starting its uplift work, the maintainer decides whether that TUI
  entry point is excluded from parity. Closeout validation is not known to force this, so check it
  by hand.
- `uaa-0036`: before closing **any** claude_code packet, adding a claude_code debt row, or starting
  claude_code uplift work, the maintainer decides whether `command_path` is rooted at the agent id
  (`claude_code`, what report-derived surfaces use) or the binary name (`claude`, what the debt
  inventory and contract examples use). Until then claude_code's install debt is misreported as new
  uplifts.

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
`uaa-0034`).

| id | item | disposition |
| --- | --- | --- |
| `uaa-0023` | Typed errors for support-audit evidence faults | Independent follow-up. Round 5 classifies some faults by matching error message text, because the packet scoped `support_audit.rs` out of the write set. A message drift yields exit 1, never a false clean, so this is fragility rather than a correctness hole. |
| `uaa-0024` | Union-vs-per-target coverage coherence contract | Independent follow-up. Needs the contract defined before it can be implemented, and it belongs at report-generation time rather than in the gate. |
| `uaa-0025` | Projection can survive a failed run as a stale result | **Reopened; resolve with or before T3.** T2a's cleanup fix was disproved by a probe: the request load derives the audit internally, so later request-validation failures are classified preflight and leave a stale projection behind exit 2. Also folds in the misleading `live_derivation_attempted` name and the silent post-derivation cleanup failure. Latent until T3, the first consumer. |
| `uaa-0026` | Exit 3 lost when `--emit-json` cannot be written | **Resolved in `72191bb3`.** The computed outcome takes precedence; the stale projection is removed and a warning printed. |
| `uaa-0027` | Snapshot retry can mix two attempts in raw_help | Low. The retry never clears attempt 1's `raw_help/<version>/<target>/`. raw_help is never committed. |
| `uaa-0028` | `--emit-json` cleanup deletes whatever path it names | Low, suspected. No ownership guard. Matters once T3 makes the projection path durable. |
| `uaa-0029` | `parity-acquire` exports no `workflow_call` outputs | Medium, latent. The caller cannot see `closeout_ready` / `uplifts_required`. **T3 prerequisite.** |
| `uaa-0030` | One failed snapshot leg skips `union`, gate, commit and upload | Medium. The "continues on a partial matrix" premise in §8.2 was wrong and is corrected there. **Decided 2026-09-13: preserve completed legs** — `union` runs unless cancelled or `plan` failed; the gate routes exit 4; commit and upload run; the job still fails. **Refined 2026-09-13:** a failed *required* target still hard-fails with no union (`manifest-union` cannot build one); only non-required leg failures preserve work. **Resolved in `a3c8ce53`.** |
| `uaa-0031` | A blocking verdict is not visible on the packet PR | Medium. **Confirmed on a runner 2026-09-14:** the watcher dispatches `agent-maintenance-open-pr` on `staging`, so the acquire check runs attach to the `staging` head commit (`a36a115d`), and packet PRs #195, #205 and #206 carry only `CI` checks. T3 must post the verdict to the PR itself (body, comment, or a status on the packet head SHA). Also carries the exit-3 `required_uplifts` detail, which `_ci_tmp` cleanup deletes today — T3 renders it. |
| `uaa-0032` | `--expect-target-version` mismatch fails out-of-packet runs | Medium. Dry runs and promote-prerequisite re-runs now end red. **Decided 2026-09-13: fail only when committing** — a `commit: false` mismatch emits a notice and stays green; no other exit 2 may be downgraded. Mechanism: the target version is compared before the validated request load and a mismatch gets its own exit code 5. A promote-prerequisite re-run (`commit: true`) for a version the ref's request does not name is **accepted as red-but-committed**. **Resolved in `a3c8ce53` / `916c9e9b`.** |
| `uaa-0033` | Artifact bundle does not match what the run committed | Low. The `always()` upload can succeed with only stale checkout files, and omits the support-matrix files the commit stages. |
| `uaa-0034` | Commit step can push a rebased tree the gate never judged | Low, suspected, pre-dates T2c. |
| `uaa-0035` | Is opencode's TUI root command excluded from parity? | **Open maintainer decision, raised 2026-09-14** by the pre-merge gate simulation. The root command is named (`commands` / `opencode` / `opencode`) since `507cf300` and is a required uplift until decided. Gates any opencode closeout or uplift work; pointers sit in T5, T8, and the non-TUI debt inventory. An exclusion would also drop the command's 20 root flags and its `project` argument from the report. |
| `uaa-0036` | claude_code debt rows name `claude`, report-derived surfaces name `claude_code` | **Open maintainer decision, raised 2026-09-14** (Opus lane; pre-existing). The two claude_code install debt rows never match, so they show as new uplifts. Gates any claude_code closeout, debt-row edit, or uplift work; pointer in T8. |

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

None blocking. Two settled during specification:

- *Does closeout precede promotion?* **Yes** — promotion writes outside the maintenance contract by
  design (`writable_surfaces` excludes the pointers), so it is a separate lane after the run closes.
- *Should the lane run `refresh-agent`?* **No** — `refresh-agent` is the recovery command; packet
  docs are owned by `prepare-agent-maintenance` at open time, and all three packets already
  reconcile `exact`.
