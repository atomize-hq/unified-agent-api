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
# Piece 1's input. Read-only.
cargo run -p xtask -- maintenance-audit-status \
  --request <path/to/maintenance-request.toml> \
  --emit-json _ci_tmp/audit-status.json

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
`EXIT_NOT_ELIGIBLE` in `manifest_acquisition.rs:74`.

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

**T2 — Wire the gate into `parity-acquire`.** Run after the union job, before the commit step.
Branch on exit code: 0 → mark closeout-ready; 3 → render the relay invocation into the PR body and
mark not-closeout-ready; other → fail. Contract test for the wiring.

**T3 — Relay-packet rendering.** On the uplift branch, render the relay invocation (prompt path,
dry-run→write `--run-id` handshake) into the packet PR body from the existing renderer, so the
maintainer pastes one command. Reuses `docs.rs` rendering; no new prompt source of truth.

**T4 — Closeout evidence resolution.** Commit-pinned CI conclusion lookup, fail-closed. This is the
highest-risk unit; it decides whether a governance artifact can be trusted.

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

### T8 sequencing — closeout happens *before* merge

A packet must be closed **on its own branch**, so the merge carries a closed HANDOFF. Codex
demonstrates the failure mode: #153 merged while still open, which replaced main's closed HANDOFF
with the open-run contributor contract. That is a regression to repair, not a pattern to repeat.

| packet | closed against | why |
|---|---|---|
| claude_code 2.1.212 (#157) | the packet branch | its request is already valid there (`2.1.212` / `upstream_stable_pointer`) |
| opencode 1.18.4 (#158) | the packet branch | same |
| codex 0.144.6 | a branch off `main` | already merged open; needs a catch-up pass |

**There is no merge dependency on this work.** The invalid `2.1.140` claude_code request exists only
on `main`; the #157 branch carries the corrected one, so closing claude_code never touches the bad
copy. Merging #157/#158 *before* closing them would actively make things worse.

### 8.1 Known debt carried out of T1

Four items were adjudicated as real but deferred. Each is a `todo` in `docs/backlog.json` with its
own context, file list and deliverables; plan doc §18.3 carries the same table. **Two of them must
be resolved as part of T2, not after it** — they are marked below.

| id | item | disposition |
| --- | --- | --- |
| `uaa-0023` | Typed errors for support-audit evidence faults | Independent follow-up. Round 5 classifies some faults by matching error message text, because the packet scoped `support_audit.rs` out of the write set. A message drift yields exit 1, never a false clean, so this is fragility rather than a correctness hole. |
| `uaa-0024` | Union-vs-per-target coverage coherence contract | Independent follow-up. Needs the contract defined before it can be implemented, and it belongs at report-generation time rather than in the gate. |
| `uaa-0025` | Projection cannot detect same-request staleness | **Resolve during T2.** `request_sha256` cannot detect the case it was added for. The right answer depends on whether T2 treats the exit code as authoritative and the JSON as advisory — decide it there. |
| `uaa-0026` | Exit 3 lost when `--emit-json` cannot be written | **Resolve during T2.** The computed outcome is discarded by a `?` on the write path. T2's exit-code routing has to state a precedence either way, so pin it there. |

### 8.2 What T1 changed about T2

The gate now has three outcomes to route, not two: clean (0), uplifts required (3), and
insufficient evidence (2) — where 2 now includes *the matrix ran but did not finish*. Because
`parity-acquire` continues on a partial matrix by design (`fail-fast: false`, only
`REQUIRED_TARGET` enforced, other missing legs are warnings), a flaked macOS or Windows leg now
lands on exit 2 where it previously passed silently.

T2's original wiring sketch — "other → fail" — is therefore no longer adequate on its own.

**Decided 2026-07-25:** on an exit 2 caused specifically by incompleteness, re-dispatch only the
missing targets once; if the acquisition is still incomplete after that retry, fail the run. Every
other exit 2 fails immediately. This keeps a flaked runner from blocking the lane — which is why
`fail-fast: false` is there — without ever letting a genuinely partial acquisition through. T2 must
therefore distinguish incompleteness from other validation failures, not just read the exit code.

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
