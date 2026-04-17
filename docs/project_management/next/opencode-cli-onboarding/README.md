# OpenCode CLI onboarding - closed seam pack

Source:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`

This pack captures the closed seam briefs, authoritative threading, pack-level review surfaces,
and governance records that onboarded `OpenCode` as the first real third CLI agent in this repo.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: none
- Next seam: none
- Future seams: none

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- the next seam may later receive seam-local review + slices, and only provisional deeper planning
- active and next seams must eventually terminate in a dedicated final `S99` seam-exit gate slice
  once seam-local planning begins
- seams that still need a contract-definition boundary may reserve `S00` during seam-local planning
- future seams remain seam briefs only
- pack-level `review_surfaces.md` is orientation only; active and next seams still need seam-local
  `review.md` later
- this repo's normative contract surfaces live under `docs/specs/**`; when downstream work creates
  later OpenCode-specific canonical contracts, they should also live under `docs/specs/**`

Closed-pack posture:

- the source packet's maintainer-backed smoke evidence was strong enough to lock
  `opencode run --format json` as the current v1 wrapper seam
- the critical path remains crate-first: runtime/evidence lock first, wrapper + manifest planning
  second, backend mapping third, UAA promotion review last
- the landed `SEAM-4` closeout now records the bounded promotion-review outcome, and no further
  queued seam remains in this pack
- `serve`, `acp`, `run --attach`, and direct interactive TUI surfaces remain deferred until an
  upstream seam explicitly reopens them
- `SEAM-1` now serves as closeout-backed upstream evidence for the downstream seams and their
  published closeout records
- legacy triad artifacts in this directory are retained as source provenance, but the seam-pack
  files referenced from this README are now the canonical closed planning surface for downstream
  work
