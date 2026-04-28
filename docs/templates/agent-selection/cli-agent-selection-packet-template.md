# Template — CLI Agent Selection Packet

Status: Canonical reusable template  
Date (UTC): `<fill>`  
Owner(s): wrappers team / packet author  
Related source docs:
- `docs/specs/cli-agent-recommendation-dossier-contract.md`
- `docs/specs/cli-agent-onboarding-charter.md`
- `docs/specs/unified-agent-api/support-matrix.md`
- `docs/specs/**` for any normative contract this packet cites

Template lock:
- generated preview packets and promoted canonical packets must preserve this template's title block shape, section numbering, section headings, section order, `Provenance:` lines, and fixed 3-candidate table shape exactly
- content may replace prompts and placeholders, but required sections must not be renamed, reordered, or collapsed

## Purpose

Use this template to evaluate exactly 3 real CLI agent candidates and produce one implementation-ready CLI agent selection packet for the recommended winner.

This artifact is informative, not normative. It must point at authoritative specs, contracts, and committed repo evidence instead of becoming a second source of truth.

The implementation handoff in a filled packet must preserve the repo's crate-first onboarding ladder:
- wrapper crate first at the registry-owned `crate_path` under `crates/`
- `agent_api` backend adapter second
- UAA promotion assessment last

## Scope Lock

Use this packet when the goal is to compare viable CLI agent candidates and prepare the implementation handoff for the recommended new CLI agent.

In scope:
- candidate shortlist and recommendation
- dated external evidence capture
- repo-fit analysis against existing wrapper and `agent_api` seams
- concrete evaluation recipe for the recommended agent
- shape-agnostic implementation handoff that keeps wrapper-crate work ahead of `agent_api` work

Out of scope:
- implementing the new agent
- changing phase-1 support semantics unless a concrete hole is discovered
- inventing helper tooling or a packet generator
- hardcoding a downstream task-pack format

## Required Core Sections

Every required core section below must stay in this order.
Each section must include:
- a `Provenance:` line
- the required outputs listed under the section

Optional appendices may follow after the required core.

---

## 1. Candidate Summary

Provenance: `<committed repo evidence | dated external snapshot evidence | maintainer inference>`

Prompt:
- Name the 3 shortlisted candidates.
- State why these 3 were chosen for comparison now.
- State the recommendation in one sentence only.

Required outputs:
- exact candidate names
- one-sentence shortlist rationale
- one-sentence recommendation

## 2. What Already Exists

Provenance: `committed repo evidence`

Prompt:
- List the repo surfaces this onboarding should reuse rather than rebuild.
- Call out any prior charter or planning doc that already constrains this work.

Required outputs:
- bullet list of reusable repo surfaces
- bullet list of authoritative docs/contracts

## 3. Selection Rubric

Provenance: `maintainer inference informed by dated external snapshot evidence`

Prompt:
- Use the fixed per-dimension score buckets below.
- Do not compute a weighted total.
- Product-value signals are primary. Differentiation signals are secondary tie-breakers.

Score buckets:
- `0` = weak / missing / materially blocked
- `1` = partial / notable caveats
- `2` = solid / usable with caveats
- `3` = strong / clearly favorable

Primary dimensions:
- `Adoption & community pull`
- `CLI product maturity & release activity`
- `Installability & docs quality`
- `Reproducibility & access friction`

Secondary dimensions:
- `Architecture fit for this repo`
- `Capability expansion / future leverage`

Required outputs:
- one short paragraph explaining the rubric philosophy
- one fixed comparison table for all 3 candidates using the dimensions above
- one short recommendation paragraph with tie-break reasoning

## 4. Fixed 3-Candidate Comparison Table

Provenance: `dated external snapshot evidence + maintainer inference`

Prompt:
- Compare exactly 3 candidates.
- Use one row per candidate.
- Use one column per required dimension.
- Add a short `Notes` column for caveats.

Required outputs:
- one markdown table with exactly 3 candidate rows
- per-dimension scores only, no total column
- short notes per candidate

Recommended table shape:

| Candidate | Adoption & community pull | CLI product maturity & release activity | Installability & docs quality | Reproducibility & access friction | Architecture fit for this repo | Capability expansion / future leverage | Notes |
|---|---:|---:|---:|---:|---:|---:|---|

## 5. Recommendation

Provenance: `maintainer inference grounded in the comparison table`

Prompt:
- Name the winner.
- Explain why it wins without using a weighted total.
- Explain why the runners-up did not win.

Required outputs:
- winning candidate
- short rationale paragraph
- 1-2 bullets for each non-winning candidate explaining why it lost
- explicit decision block with:
  - `Approve recommended agent`
  - `Override to shortlisted alternative`
  - `Stop and expand research`

## 6. Recommended Agent Evaluation Recipe

Provenance: `dated external snapshot evidence + maintainer inference`

Prompt:
- Describe how another maintainer can evaluate the recommended agent now.
- Include gated/commercial prerequisites explicitly.
- Separate what is reproducible immediately from what remains blocked.

Required outputs:
- `reproducible now` subsection
- `blocked until later` subsection
- install path(s)
- auth / account / billing prerequisites
- runnable commands
- evidence gatherable without paid or elevated access
- blocked steps that require paid or elevated access
- expected artifacts to save during evaluation

## 7. Repo-Fit Analysis

Provenance: `committed repo evidence + maintainer inference`

Prompt:
- Explain how the recommended agent maps onto the current repo architecture.
- Keep the wrapper crate as the first implementation stage and separate it from the later `agent_api` backend stage.
- Call out where current phase-1 seams appear sufficient and where they might crack.

Required outputs:
- manifest-root expectations
- wrapper crate expectations
- `agent_api` backend expectations
- UAA promotion expectations
- support/publication expectations
- likely seam risks

## 8. Required Artifacts

Provenance: `committed repo evidence + maintainer inference`

Prompt:
- List the concrete artifact types a real onboarding would eventually need.
- Keep wrapper-crate artifacts explicit and prior to `agent_api` artifacts.
- Stay shape-agnostic about downstream planning packs.

Required outputs:
- manifest-root artifact expectations
- wrapper-crate artifact expectations
- `agent_api` artifact expectations
- UAA promotion-gate expectations
- docs/spec artifact expectations
- evidence/fixture expectations

## 9. Workstreams, Deliverables, Risks, And Gates

Provenance: `maintainer inference grounded in repo constraints`

Prompt:
- End with an implementation-ready handoff.
- Do not prescribe one task-pack or feature-pack format.
- Name the workstreams another maintainer would need to execute.
- Keep the workstreams explicitly gated: packet closeout, wrapper crate, `agent_api`, then UAA promotion review.

Required outputs:
- required workstreams
- required deliverables
- blocking risks
- acceptance gates

Recommended subsections:
- `Required workstreams`
- `Required deliverables`
- `Blocking risks`
- `Acceptance gates`

## 10. Dated Evidence Appendix

Provenance: `dated external snapshot evidence`

Prompt:
- Include one structured appendix entry for each of the 3 candidates.
- Preserve source links and normalized notes.
- Keep recommendation evidence self-contained in this packet.

Required outputs for each candidate:
- snapshot date
- official home / repo / docs links
- install/distribution notes
- adoption/community signals
- release activity notes
- access prerequisites
- normalized notes

Appendix requirements:
- include loser rationale for the two non-winning shortlisted candidates
- include strategic contenders if any

Suggested shape:

### Appendix A. `<candidate>`
- Snapshot date: `<YYYY-MM-DD>`
- Official links:
  - `<url>`
  - `<url>`
- Install / distribution:
  - `<note>`
- Adoption / community:
  - `<note>`
- Release activity:
  - `<note>`
- Access prerequisites:
  - `<note>`
- Normalized notes:
  - `<note>`

## 11. Acceptance Checklist

Provenance: `maintainer inference`

Prompt:
- Make the checklist mechanically reviewable.
- Do not rely on reader interpretation.

Required outputs:
- checklist items for template conformance
- checklist items for candidate comparison completeness
- checklist items for evidence/provenance completeness
- checklist items for implementation-handoff completeness

Required checklist:
- [ ] Exactly 3 real candidates are compared.
- [ ] The fixed per-dimension comparison table is present.
- [ ] No weighted total score is used.
- [ ] The recommendation explains the winner and tie-break reasoning.
- [ ] The recommended agent includes a concrete evaluation recipe.
- [ ] Every judgment-heavy section has a provenance line.
- [ ] The handoff keeps registry-owned wrapper-crate-path work ahead of `agent_api` adapter work.
- [ ] UAA promotion is treated as a later gate, not bundled into initial backend support.
- [ ] The dated evidence appendix includes all 3 candidates.
- [ ] Commercial or gated access requirements are explicit where applicable.
- [ ] Required workstreams, deliverables, risks, and acceptance gates are present.
- [ ] The packet stays shape-agnostic about downstream planning-pack format.

---

## Optional Appendices

Use appendices only for candidate-specific material that would otherwise bloat the core:
- auth/provider quirks
- unusual install caveats
- protocol or output-shape caveats
- extra evaluation notes
- future follow-on ideas that are not part of the required handoff
