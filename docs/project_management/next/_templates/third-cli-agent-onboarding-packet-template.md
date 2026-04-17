# Template — Third CLI Agent Onboarding Packet

Status: Template  
Date (UTC): YYYY-MM-DD  
Owner(s): `<team or maintainer>`  
Packet type: planning / onboarding packet  
Recommended filled-packet location: `docs/project_management/next/<packet-name>.md`

This template exists to make third-agent onboarding boring in the good way.

Use it to compare exactly 3 real CLI agent candidates, recommend one, and hand off bounded implementation work without locking the repo into one permanent downstream planning-pack shape.

## Template Rules

- This template owns invariant packet structure, rubric dimensions, evidence requirements, and handoff gates.
- A filled packet owns candidate-specific content only: candidate names, scores, dated evidence snapshots, recommendation details, evaluation recipe, and implementation handoff specifics.
- The filled packet MUST compare exactly 3 real candidate agents.
- The comparison MUST use fixed per-dimension scores. Do not collapse the decision into one weighted total or one magic number.
- Commercial or gated agents are allowed, but access friction, evaluator prerequisites, and reproducibility cost MUST be scored explicitly.
- Healthy overlap with existing unified API endpoints is expected and good. Do not downgrade a candidate just because its primary capabilities overlap with Codex or Claude Code.
- Capability differentiation is a secondary differentiator. Product demand and quality come first.
- The filled packet MUST stay self-contained in review. Keep candidate evidence snapshots inline in the appendix, not in a sidecar artifact.
- The filled packet MUST end in a shape-agnostic implementation handoff: required workstreams, required deliverables, blocking risks, and acceptance gates.
- This packet is planning and onboarding work only. Do not silently expand it into actual agent implementation, helper tooling, or a generator.

## Score Legend

Use the same score buckets for every candidate and every dimension.

| Score | Meaning |
|------|---------|
| `3` | Strong, clear, low ambiguity |
| `2` | Viable, some caveats |
| `1` | Weak, risky, or materially incomplete |
| `0` | Blocked, unknown, or clearly not a fit |

Do not sum these scores into a single total. The recommendation section must explain the winner in prose, including any tie-breaks.

## Required Core

The following sections are mandatory in every filled packet and should remain in this order.

### 1. Packet Summary

**Prompt**
- What is this packet trying to decide?
- Which repo baseline and earlier decisions does it assume are already fixed?
- What is explicitly out of scope for this packet?

**Required output**
- One short paragraph describing the packet goal.
- A flat list of in-scope outcomes.
- A flat list of out-of-scope items.

**Template**

## Packet Summary

This packet evaluates exactly 3 real CLI agent candidates and recommends the first real third CLI agent to onboard after the phase-1 support-matrix work. It does not implement the agent. It produces a bounded, reusable onboarding handoff.

In scope:
- `<outcome>`
- `<outcome>`
- `<outcome>`

Out of scope:
- `<out-of-scope item>`
- `<out-of-scope item>`
- `<out-of-scope item>`

### 2. Baseline And Fixed Invariants

**Prompt**
- Which phase-1 decisions stay fixed for this packet?
- Which support-layer distinctions must remain explicit?
- Which architectural seams already exist and should be reused instead of rebuilt?

**Required output**
- A short baseline paragraph.
- A flat list of fixed invariants.
- A flat list of existing repo assets or flows that this packet reuses.

**Provenance**
- `Committed repo evidence:` `<files reviewed>`
- `Maintainer inference:` `<if any>`

**Template**

## Baseline And Fixed Invariants

Provenance:
- Committed repo evidence: `<files reviewed>`
- Maintainer inference: `<if any, otherwise "none">`

Baseline:
`<brief baseline statement>`

Fixed invariants:
- Support remains split across:
  - manifest / upstream support
  - backend-crate support
  - UAA unified support
- Target-scoped support rows remain the primitive.
- `cli_manifests/**` stays the evidence layer.
- `docs/specs/unified-agent-api/**` stays the semantics and publication layer.
- The packet must remain implementation-ready without prescribing one permanent downstream pack format.
- `<additional invariant>`

What already exists and should be reused:
- `<existing module / doc / validation flow>`
- `<existing module / doc / validation flow>`
- `<existing module / doc / validation flow>`

### 3. Candidate Shortlist

**Prompt**
- Which 3 real CLI agents are being compared?
- Why are these 3 plausible enough to earn a packet?
- What obvious alternatives were excluded before shortlist lock-in?

**Required output**
- Exactly 3 candidate entries.
- One-line rationale per candidate.
- Optional note for pre-shortlist exclusions if needed.

**Provenance**
- `Dated external snapshot evidence:` `<sources used to assemble shortlist>`
- `Maintainer inference:` `<if any>`

**Template**

## Candidate Shortlist

Provenance:
- Dated external snapshot evidence: `<sources used to assemble shortlist>`
- Maintainer inference: `<if any, otherwise "none">`

Candidates:
1. `<candidate A>` — `<one-line rationale>`
2. `<candidate B>` — `<one-line rationale>`
3. `<candidate C>` — `<one-line rationale>`

Optional pre-shortlist exclusions:
- `<excluded candidate>` — `<why excluded before scoring>`

### 4. Fixed Comparison Contract

**Prompt**
- Score all 3 candidates against the same dimensions.
- Keep primary product-value dimensions separate from secondary differentiation dimensions.
- Make access friction explicit instead of hand-waving it.
- Do not penalize overlap with existing unified support; score differentiation only where it creates additional value or learning.

**Required output**
- One comparison table with exactly 3 candidate columns.
- Fixed per-dimension scores using the shared `0-3` legend.
- One evidence-note column or note block per row if the score would otherwise be unclear.
- No weighted total row.

**Provenance**
- `Dated external snapshot evidence:` `<appendix references>`
- `Committed repo evidence:` `<repo files reviewed for fit / tractability>`
- `Maintainer inference:` `<if any>`

**Scoring dimensions**

Primary dimensions:
- `Product demand and user pull`
- `Quality and operator trust`
- `Distribution and install story`
- `Release activity and maintenance health`
- `Reproducibility and access friction`
- `CLI scope clarity and repo fit`

Secondary dimensions:
- `Capability differentiation and future learning value`
- `Manifest / backend / UAA tractability`

For `Reproducibility and access friction`, score high when a maintainer can evaluate the candidate with low ceremony. Score low when the candidate requires billing, waitlists, special org approvals, unstable auth, or unclear evaluator prerequisites.

For `Capability differentiation and future learning value`, reward useful new seams or capabilities, but do not punish healthy overlap with Codex or Claude Code.

**Template**

## Fixed Comparison Contract

Provenance:
- Dated external snapshot evidence: `<appendix references>`
- Committed repo evidence: `<repo files reviewed>`
- Maintainer inference: `<if any, otherwise "none">`

| Dimension | Tier | `<candidate A>` | `<candidate B>` | `<candidate C>` | Notes |
|-----------|------|-----------------|-----------------|-----------------|-------|
| Product demand and user pull | Primary | `<0-3>` | `<0-3>` | `<0-3>` | `<short note>` |
| Quality and operator trust | Primary | `<0-3>` | `<0-3>` | `<0-3>` | `<short note>` |
| Distribution and install story | Primary | `<0-3>` | `<0-3>` | `<0-3>` | `<short note>` |
| Release activity and maintenance health | Primary | `<0-3>` | `<0-3>` | `<0-3>` | `<short note>` |
| Reproducibility and access friction | Primary | `<0-3>` | `<0-3>` | `<0-3>` | `<short note>` |
| CLI scope clarity and repo fit | Primary | `<0-3>` | `<0-3>` | `<0-3>` | `<short note>` |
| Capability differentiation and future learning value | Secondary | `<0-3>` | `<0-3>` | `<0-3>` | `<short note>` |
| Manifest / backend / UAA tractability | Secondary | `<0-3>` | `<0-3>` | `<0-3>` | `<short note>` |

Comparison notes:
- Do not compute a total score.
- If two candidates are close, explain the tie-break in the recommendation section.
- If any candidate is gated or commercial, the notes must say what specifically causes the reproducibility penalty.

### 5. Recommendation

**Prompt**
- Which candidate wins and why?
- Why did it beat the other two without relying on a hidden total?
- What are the main tradeoffs or caveats?

**Required output**
- Name one recommended agent.
- One short recommendation paragraph.
- A flat list of tie-breaks and caveats.

**Provenance**
- `Dated external snapshot evidence:` `<appendix references>`
- `Committed repo evidence:` `<repo files reviewed>`
- `Maintainer inference:` `<if any>`

**Template**

## Recommendation

Provenance:
- Dated external snapshot evidence: `<appendix references>`
- Committed repo evidence: `<repo files reviewed>`
- Maintainer inference: `<if any, otherwise "none">`

Recommended agent: `<name>`

Recommendation:
`<short paragraph explaining why this candidate is the best first third-agent target>`

Tie-breaks and caveats:
- `<tie-break or caveat>`
- `<tie-break or caveat>`
- `<tie-break or caveat>`

### 6. Recommended-Agent Evaluation Recipe

**Prompt**
- How does another maintainer reproduce the recommendation for the chosen agent?
- What install path, auth setup, or billing prerequisites exist?
- Which commands can be run now, and which steps remain blocked by access?

**Required output**
- One concrete evaluation recipe for the recommended agent.
- Install path.
- Auth and access prerequisites.
- Runnable commands or evidence-gathering steps.
- A flat list separating reproducible-now vs blocked-by-access work.

**Provenance**
- `Dated external snapshot evidence:` `<appendix references>`
- `Maintainer inference:` `<if any>`

**Template**

## Recommended-Agent Evaluation Recipe

Provenance:
- Dated external snapshot evidence: `<appendix references>`
- Maintainer inference: `<if any, otherwise "none">`

Install path:
- `<package manager / binary / release asset>`

Auth and access prerequisites:
- `<login requirement>`
- `<billing / org / waitlist requirement>`
- `<environment or API key requirement>`

Runnable commands and evidence steps:
1. `<command>` — `<what this proves>`
2. `<command>` — `<what this proves>`
3. `<command>` — `<what this proves>`

Reproducible now:
- `<step or evidence that any maintainer can gather now>`
- `<step or evidence that any maintainer can gather now>`

Blocked by access:
- `<blocked step>` — `<what access is missing>`
- `<blocked step>` — `<what access is missing>`

### 7. Onboarding Shape And Scope

**Prompt**
- What manifest-root shape does this agent likely need?
- What snapshot, union, coverage, and validation obligations should the eventual implementation inherit?
- Where do backend-crate support and UAA support diverge for this candidate?

**Required output**
- A flat list of expected manifest-layer work.
- A flat list of expected backend-layer work.
- A flat list of expected UAA promotion requirements.
- A short note on likely architectural holes or unknowns.

**Provenance**
- `Committed repo evidence:` `<repo files reviewed>`
- `Dated external snapshot evidence:` `<appendix references>`
- `Maintainer inference:` `<if any>`

**Template**

## Onboarding Shape And Scope

Provenance:
- Committed repo evidence: `<repo files reviewed>`
- Dated external snapshot evidence: `<appendix references>`
- Maintainer inference: `<if any, otherwise "none">`

Manifest-layer expectations:
- `<manifest root / version metadata / pointer / support-row expectation>`
- `<snapshot / union / wrapper coverage expectation>`
- `<validation or report expectation>`

Backend-crate expectations:
- `<crate boundary or adapter expectation>`
- `<agent-specific parsing / wrapper / thin-adapter expectation>`
- `<backend support constraint>`

UAA promotion expectations:
- `<what must be true before UAA support is claimed>`
- `<what remains backend-only at first>`
- `<promotion gate>`

Likely architectural holes or unknowns:
- `<unknown>`
- `<unknown>`

### 8. Shape-Agnostic Implementation Handoff

**Prompt**
- If another maintainer picks this up next, what workstreams exist?
- What deliverables must exist before the onboarding slice is done?
- Which risks block progress?
- Which acceptance gates define done, without prescribing a specific pack format?

**Required output**
- A workstream list.
- A deliverables list.
- A blocking-risks list.
- An acceptance-gates list.

**Provenance**
- `Committed repo evidence:` `<repo files reviewed>`
- `Maintainer inference:` `<if any>`

**Template**

## Shape-Agnostic Implementation Handoff

Provenance:
- Committed repo evidence: `<repo files reviewed>`
- Maintainer inference: `<if any, otherwise "none">`

Required workstreams:
- `<workstream>`
- `<workstream>`
- `<workstream>`

Required deliverables:
- `<deliverable>`
- `<deliverable>`
- `<deliverable>`

Blocking risks:
- `<risk>`
- `<risk>`
- `<risk>`

Acceptance gates:
- `<gate>`
- `<gate>`
- `<gate>`

### 9. Open Questions And Deferred Work

**Prompt**
- What remains unknown after this packet?
- What should be deferred instead of silently pulled into the onboarding slice?

**Required output**
- A flat list of open questions.
- A flat list of explicit deferrals.

**Provenance**
- `Maintainer inference:` `<if any>`

**Template**

## Open Questions And Deferred Work

Provenance:
- Maintainer inference: `<if any, otherwise "none">`

Open questions:
- `<question>`
- `<question>`

Explicit deferrals:
- `<deferred item>`
- `<deferred item>`

### 10. Final Acceptance Checklist

**Prompt**
- Make conformance reviewable without new tooling.
- Cover template completeness, candidate completeness, evidence completeness, provenance completeness, and handoff completeness.

**Required output**
- One checklist that a reviewer can walk mechanically.

**Template**

## Final Acceptance Checklist

- [ ] The packet compares exactly 3 real CLI agent candidates.
- [ ] The packet keeps template invariants separate from candidate-specific content.
- [ ] The packet includes the fixed comparison table with all required dimensions.
- [ ] The packet uses shared `0-3` score buckets and does not compute a weighted total.
- [ ] Product demand and quality are treated as primary dimensions.
- [ ] Capability differentiation is treated as a secondary dimension and does not penalize healthy overlap with current agents.
- [ ] Commercial or gated access penalties are scored explicitly under reproducibility and access friction.
- [ ] The packet names one recommended agent and explains the decision in prose.
- [ ] The packet includes a concrete evaluation recipe for the recommended agent.
- [ ] The packet clearly separates reproducible-now steps from blocked-by-access steps.
- [ ] The packet documents manifest-layer expectations.
- [ ] The packet documents backend-crate expectations.
- [ ] The packet documents UAA promotion expectations separately from backend support.
- [ ] The packet ends in required workstreams, deliverables, blocking risks, and acceptance gates.
- [ ] Each judgment-heavy core section includes a provenance line.
- [ ] Each external-evidence-based claim is backed by a dated evidence snapshot in the appendix.
- [ ] The packet keeps candidate evidence inline in the appendix instead of a sidecar artifact.
- [ ] The packet lists explicit deferrals instead of silently expanding scope.

## Optional Appendices

Use appendices only when the required core would otherwise become bloated. These are optional, but the evidence appendix below becomes mandatory as soon as the filled packet names real candidates.

### Appendix A. Candidate Evidence Snapshots

This appendix is mandatory in every filled packet.

**Prompt**
- Preserve the dated evidence bundle that supports the shortlist and recommendation.
- Use the same evidence fields for all 3 candidates.
- Keep source links and normalized notes together so the packet is self-contained in review.

**Required output**
- One subsection per candidate.
- Observation date.
- Source links.
- Normalized notes for each evidence signal.
- Access and reproducibility notes.
- Commands used to gather evidence when applicable.

**Template**

## Appendix A. Candidate Evidence Snapshots

### `<candidate A>`

Observation date (UTC): `<YYYY-MM-DD>`

Source links:
- `<url>`
- `<url>`
- `<url>`

Evidence bundle:
- Product demand and user pull: `<normalized note>`
- Quality and operator trust: `<normalized note>`
- Distribution and install story: `<normalized note>`
- Release activity and maintenance health: `<normalized note>`
- Reproducibility and access friction: `<normalized note>`
- CLI scope clarity and repo fit: `<normalized note>`
- Capability differentiation and future learning value: `<normalized note>`
- Manifest / backend / UAA tractability: `<normalized note>`

Access and reproducibility notes:
- `<note>`
- `<note>`

Commands used:
- `<command>`
- `<command>`

### `<candidate B>`

Observation date (UTC): `<YYYY-MM-DD>`

Source links:
- `<url>`
- `<url>`
- `<url>`

Evidence bundle:
- Product demand and user pull: `<normalized note>`
- Quality and operator trust: `<normalized note>`
- Distribution and install story: `<normalized note>`
- Release activity and maintenance health: `<normalized note>`
- Reproducibility and access friction: `<normalized note>`
- CLI scope clarity and repo fit: `<normalized note>`
- Capability differentiation and future learning value: `<normalized note>`
- Manifest / backend / UAA tractability: `<normalized note>`

Access and reproducibility notes:
- `<note>`
- `<note>`

Commands used:
- `<command>`
- `<command>`

### `<candidate C>`

Observation date (UTC): `<YYYY-MM-DD>`

Source links:
- `<url>`
- `<url>`
- `<url>`

Evidence bundle:
- Product demand and user pull: `<normalized note>`
- Quality and operator trust: `<normalized note>`
- Distribution and install story: `<normalized note>`
- Release activity and maintenance health: `<normalized note>`
- Reproducibility and access friction: `<normalized note>`
- CLI scope clarity and repo fit: `<normalized note>`
- Capability differentiation and future learning value: `<normalized note>`
- Manifest / backend / UAA tractability: `<normalized note>`

Access and reproducibility notes:
- `<note>`
- `<note>`

Commands used:
- `<command>`
- `<command>`

### Appendix B. Candidate-Specific Quirks

Use this only for details that would otherwise overload the core sections.

Suggested uses:
- auth oddities
- platform limitations
- CLI invocation quirks
- unstable or ambiguous upstream docs

### Appendix C. Repo Mapping Notes

Use this only when the recommended agent has a non-obvious fit against existing repo seams.

Suggested uses:
- likely crate boundaries
- wrapper normalization edge cases
- snapshot / union / coverage translation concerns

