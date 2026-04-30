# UAA-0022 Narrowness Report

Date: 2026-04-30
Branch: `codex/recommend-next-agent`
Status: review memo

## Purpose

This report explains how the shipped `uaa-0022` runtime-follow-on lane is still narrower than the original intended plan, even after the follow-up that closed the biggest execution gap.

The key point is simple:

The repo now has a real bounded runtime lane. That is good.

But the lane still implements the minimum viable closure of the seam, not the full review and throughput contract that the planning docs originally described.

## The intended plan

The intended shape in:

- `docs/backlog/uaa-0022-runtime-follow-on-codex-runner.md`
- `PLAN.md`

was not just "add a command that can run Codex safely."

It aimed for a stronger contract:

1. A repo-owned runtime lane after `onboard-agent --write` and `scaffold-wrapper-crate --write`.
2. Codex execution owned by the repo command, not by an informal operator step.
3. A pinned input contract and pinned write boundary.
4. A reviewable output contract that tells maintainers what tier landed, what template was used, what was deferred, and why.
5. A clean handoff into the later publication-refresh lane.

That is the benchmark this report compares against.

## What now exists

The shipped lane now does all of these things correctly:

1. `xtask runtime-follow-on --write` actually owns the Codex execution step.
2. The command records execution evidence and validates the post-run diff against the frozen baseline.
3. Success now requires real runtime-owned output changes.
4. The onboarding handoff text no longer tells maintainers to mutate publication-owned manifest files during the runtime lane.
5. The operator guide now matches the runtime/publication boundary much better than before.

This means the repo has crossed the important line from:

"prepare a packet and hope the human drives the middle seam correctly"

to:

"the repo owns the bounded execution seam and can validate whether that seam was followed."

That is the major win.

## How the shipped lane is still narrower than the intended plan

### 1. The review artifact is thinner than the planned output contract

The original plan wanted the runtime lane to produce a richer implementation summary, including:

- achieved tier
- primary template used
- minimal-tier justification when applicable
- richer surfaces intentionally deferred

The shipped implementation records:

- whether Codex ran
- whether boundary checks passed
- whether required test and handoff checks passed
- what files changed

That is enough to validate control and safety.
It is not yet the full maintainer-facing review summary the plan originally described.

So the lane is operationally closed, but still lighter than the intended reviewer experience.

### 2. The lane validates outputs, but it does not deeply classify implementation quality

The current validator checks:

- write boundary
- manifest ownership split
- no generated `wrapper_coverage.json` edit
- required default-tier onboarding test presence
- handoff contract validity
- non-zero runtime-owned writes

That is a good enforcement surface for lane ownership.

But it does not yet enforce more semantic judgments from the plan, such as:

- whether the implementation truly matches `default` versus only looking structurally similar
- whether the chosen template lineage was the right one
- whether deferred rich surfaces were crisply called out

In other words, the current lane is stronger on control than on qualitative review semantics.

### 3. The handoff into the next lane is cleaner, but still minimal

The plan described a machine-readable handoff that would clearly tell the next lane:

- what runtime evidence exists
- what publication-refresh commands remain
- whether the lane is ready for publication refresh
- what blockers remain

The shipped handoff now satisfies the minimum semantic contract needed for transition.

What it does not yet provide is a richer structured transition packet that fully explains the runtime result in the maintainers' review language. It is sufficient for orchestration, but still narrower than the broader handoff vision in the planning docs.

### 4. The implementation optimizes for seam closure, not maximum throughput instrumentation

The planning discussion also pointed at throughput concerns:

- shorter onboarding lead time
- lower review time
- fewer regressions escaping into later publication work

The shipped lane improves those outcomes indirectly, but it does not yet measure or report them directly.

There is no deeper instrumentation layer here yet. The implementation closes the control seam first.

That is a reasonable sequencing decision, but it is narrower than the larger plan language.

### 5. The lane remains intentionally narrow on publication refresh

This one is deliberate, not a miss.

The original planning work explicitly said publication refresh should stay out of scope for this milestone. The shipped lane preserves that boundary.

That means the create flow is still split across:

- runtime lane
- publication refresh lane
- proving-run closeout lane

This is correct relative to scope control, but it also means the implementation is narrower than any imagined "one command from approval to green" version of the plan.

It is a bounded seam, not the whole create-mode story.

## Bottom line

The runtime-follow-on lane is now narrower than the original plan in one specific way:

it closes the execution-control seam, but not the full richness of the intended review-and-summary contract.

Put more plainly:

- The repo now owns the runtime execution lane.
- The repo now enforces the important boundary rules.
- The repo now hands off cleanly into the next lane.

But:

- the review artifact is still lean
- the semantic classification is still shallow
- the handoff is still minimal
- the broader throughput/reporting ambitions from the planning phase are not fully realized yet

## Practical conclusion

This should be viewed as:

**the narrow successful version of the plan**

not

**the maximal version of the plan**

That is not a failure.

It means the repo implemented the smallest complete seam closure first:

- command-owned execution
- enforced runtime boundary
- no-op rejection
- runtime/publication separation

and left the richer reviewer-facing summary layer for a later improvement if it proves worth the added complexity.

## Recommendation

Treat `uaa-0022` as operationally closed for the seam it targeted.

If the repo wants to continue toward the original fuller vision, the next incremental enhancement should be:

1. add a structured implementation summary with achieved tier, template lineage, and deferred rich surfaces
2. make that summary part of the machine-validated runtime handoff
3. keep publication refresh as a separate lane

That would expand the lane toward the original plan without reopening the already-closed execution seam.
