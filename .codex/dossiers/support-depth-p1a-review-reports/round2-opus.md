# round2-opus: reviewer report

Lane: Opus 5.5 xhigh, adversarial. Candidate: `f495a750`. Scope: remediation follow-up.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

I found three P2 findings and four P3 notes in f495a750, so this is **not CLEAN**. Each P2 comes from text that f495a750 added. I made no edits. The worktree is detached at f495a750.

## Findings

### P2-1: On a merge route, serialization ownership lasts from the route's first write until the pull request merges

- **Lines:** 532–545 and 550–553 (`docs/specs/support-depth-contract.md`).
- **What the text says:** ownership is held "before its first write" (535). It excludes every other domain member "from changing those outputs or inputs until the gated operation ends" (543–544). Then the delta adds: "When a route's writes reach the integration branch by merge, the gated operation ends at the merge" (550–551). Nothing says that writes on a branch are only candidate preparation.
- **Failure scenario:**
  - `parity-promote.yml` writes pointers, version metadata and `support-matrix` on its runner checkout, then opens a pull request that the maintainer merges by hand. A packet pull request built by `agent-maintenance-open-pr.yml` and `parity-acquire.yml` waits on a closeout that the workflow comments say "spans days".
  - Every agent's acquisition, promotion and `refresh-publication` writes the shared `cli_manifests/support_matrix/current.json`. So they all share one domain.
  - Read literally, one open depth-enrolled pull request therefore locks every other agent's support-matrix changes for days.
  - The nightly re-dispatch resets the packet branch, so the earlier generation never merges and its ownership has no defined end.
  - Lines 556–558 assume no cross-process lock. An implementer must either build a lock that spans pull-request lifetimes or conclude that no route can demonstrate serialization. In that case every route must refuse, and Path enablement item 3 can never hold.
- **Aggravating factor:** the domain rule at 542 counts two routes as overlapping when they merely read the same inputs (registry, `Cargo.lock`, the build graph). That puts every route into one global domain.
- **Status:** introduced by the delta. It is the A3 fix colliding with the A4/D13 serialization rule.
- **Remediation:** for a merge route, make the serialized gated operation the integration step only: read the branch tip, compute the merge result, check it, merge (for example a merge queue that checks the exact merge commit). State that branch-side writes are candidate preparation whose admission is "a result obtained earlier" and is checked again at the merge result. Build domain overlap only from writer-versus-writer and writer-versus-reader pairs.

### P2-2: Overwriting evidence in place is not a depth-gated effect, so accepted evidence can be replaced without admission or detection

- **Lines:** 447 and 457.
- **What the text says:**
  - The values that belong to a tuple for the evidence output are "Every result, mapping and dependency identity recorded for the tuple, and the presence of the evidence itself" (447).
  - Line 457 forbids only removal: "MUST NOT be removed while any published output relies on it".
  - Evidence is not in the invalidation set at 306–308 (implementation, tests, fixtures, requirements, upstream inputs, dependencies).
- **Failure scenario:**
  - V1 is depth-enrolled, accepted and published as qualified.
  - Someone dispatches `parity-acquire` for V1 again. The `parity-promote.yml` migration note tells maintainers to do exactly this.
  - That workflow regenerates `wrapper_coverage.json` from the current wrapper source, then runs `manifest-report --version V1`.
  - `crates/xtask/src/manifest_report.rs:119` overwrites `reports/V1/*` in place and never removes the directory. Content changes; presence and recorded identities do not.
  - So no tuple value changes, no admission is required, and the validation at 514–517 sees no contradiction.
  - The published qualification now cites committed evidence that no longer matches the identities it recorded.
- **Status:** introduced by the delta (the A5 value column and the A1 fix). The A1 root cause is fixed for deletion only.
- **Remediation:** make the content identity of the evidence a tuple value, not only its presence. Or extend 457 to "removed or replaced".

### P2-3: "records a closeout as closed" does not fit maintenance closeouts, and the new catch-all makes the gap permissive

- **Lines:** 466 and 471, with the 454 row.
- **What the text says:**
  - `MaintenanceCloseout` (`crates/xtask/src/agent_maintenance/closeout/types.rs:71`) has no open/closed state. Only the proving-run closeout has `closed`.
  - `prepare-agent-closeout` writes the canonical `maintenance-closeout.json`, which `close-agent-maintenance` "accepts unmodified". `close-agent-maintenance` then "rewrites the closeout from `serialize_closeout_json` on every run" (`prepare_closeout.rs:3-4`, `:44`).
  - The maintenance contract says "`close-agent-maintenance` remains the only closeout writer" (`maintenance-request-contract-v1.md:469`).
  - Line 471 is new: "Every other depth-gated effect is a **reporting effect**", and reporting effects need no authority, currency or due-work check.
- **Failure scenario:**
  - An implementer follows the maintenance contract and gates `close-agent-maintenance` as the closing writer.
  - `prepare-agent-closeout` sets no "closed" value, so it is classified as reporting. It writes the full record (and on refusal leaves a new file behind).
  - `close-agent-maintenance` then rewrites identical bytes, so the closeout record changes no value. Its other writes are not depth-gated for a maintenance-path tuple: `HANDOFF.md`, `remediation-log.md`, and the lifecycle record (`MaintenanceCloseoutWritten`, clearing `Drifted`). The lifecycle row at 455 covers only the onboarding path.
  - Result: a depth-enrolled maintenance generation is closed without items 3–5 ever being checked.
  - The older text required authority and currency for every protected effect, so this misclassification is newly consequential.
- **Status:** a regression caused by the delta; A7 is only partially met.
- **Remediation:** phrase the item by effect, not by state. For example: "creates or changes a maintenance closeout record, or records a proving-run closeout as `closed`".

## P3 (non-blocking)

- **Line 551, "Depth admission, currency included".** This can be read as imposing item 4 (P/O/E current) on reporting merges. That reading would refuse truthful pending assessments, against A2 and Proof rule 5. Suggested wording: "depth admission, evaluated against the merge result".
- **Lines 451 and 445–455, missing pointer-like outputs.** `parity-promote.yml` also writes the root `latest_validated.txt` (read by `watch.rs:319` and `prepare.rs:177`) and `current.json` (a neutral-root-intake input, read in `support_matrix/derive.rs:504`). Neither has a target, so the per-target value column never assigns them to a tuple. The impact is small because the same operation's per-target writes are gated.
- **Lines 355 and 399 versus 519, term collision.** The new `### Supersession` section (pointer movement, "a reporting effect") collides with "re-freeze or supersession" of P (maintainer authority) in the same document.
- **Line 515, undefined "admission records".** The term appears nowhere else. It could push an implementation into a new admission log, which conflicts with Minimum machinery.
- **Deferred.** `manifest-retain --apply` (`manifest_retain.rs:78-123`) deletes snapshots and reports for any version that no pointer names and that is outside the last-N validated, which includes a pending V1's in-flight evidence. Line 457 protects only accepted tuples. Results stay truthful, but the evidence for that generation is lost.

## Objective status

| Obj | Status | Obj | Status |
| --- | --- | --- | --- |
| A1 | partially fixed (P2-2) | A16 | fixed |
| A2 | fixed (P3 at 551) | A17 | fixed (P3 Supersession collision) |
| A3 | partially fixed (P2-1) | A18 | fixed |
| A4 | partially fixed (P2-1) | A19 | fixed |
| A5 | partially fixed (P2-2; P3 root pointer and `current.json`) | A20 | fixed |
| A6 | fixed | A21 | fixed |
| A7 | partially fixed (P2-3) | A22 | fixed |
| A8 | fixed | A23 | fixed |
| A9 | fixed | A24 | fixed |
| A10 | fixed | A25 | fixed |
| A11 | fixed | A26 | fixed |
| A12 | fixed (retirement conditions confirmed at `maintenance-request-contract-v1.md:289`) | A27 | fixed |
| A13 | fixed (P3 "admission records") | A28 | fixed |
| A14 | fixed | A29 | fixed |
| A15 | fixed | | |

## Areas examined

- **The contract:** `docs/specs/support-depth-contract.md` at f495a750, and its diff from e60664e2.
- **Supersession abuse:** I found no way to rewrite an enrolled tuple's results or to publish an enrolled version without its own acceptance.
- **Merge-route evasion:** a direct push via `parity-acquire` with `ref=staging` is still a route, and its fast-forward-only push acts as compare-and-swap.
- **Workflows:** `.github/workflows/parity-acquire.yml`, `parity-promote.yml`, `agent-maintenance-open-pr.yml` and the capability-matrix guard in `ci.yml`.
- **xtask writers:** `support_matrix/derive.rs`, `publication_refresh.rs`, `manifest_retain.rs`, `manifest_report.rs`, `manifest_version_metadata.rs`, `manifest_validate/pointer_consistency.rs`, `agent_maintenance/closeout/{write,types}.rs`, `agent_maintenance/prepare_closeout.rs`, `historical_lifecycle_backfill.rs`, `agent_lifecycle.rs` and `approval_artifact.rs`.
- **Controlling specs:** `maintenance-request-contract-v1.md`, `unified-agent-api/non-tui-support-debt.md`, `unified-agent-api/support-matrix.md` and `cli-agent-onboarding-charter.md`.
