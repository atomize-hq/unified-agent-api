# OpenCode CLI onboarding - seam extraction

Source:
- `docs/project_management/next/opencode-cli-onboarding/plan.md`
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit
intent, governance scaffolds, and the active seam-local planning needed to onboard `OpenCode` as
the first real third CLI agent in this repo.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-3`
- Next seam: `SEAM-4`
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
  new OpenCode-specific canonical contracts, they should also live under `docs/specs/**`

Assumptions captured for extraction:

- the source packet's maintainer-backed smoke evidence was strong enough to lock
  `opencode run --format json` as the current v1 wrapper seam
- the critical path remains crate-first: runtime/evidence lock first, wrapper + manifest planning
  second, backend mapping third, UAA promotion review last
- the landed `SEAM-2` closeout now publishes the wrapper/manifest handoff needed to activate
  `SEAM-3`, while `SEAM-4` remains queued behind the backend seam
- `serve`, `acp`, `run --attach`, and direct interactive TUI surfaces remain deferred until an
  upstream seam explicitly reopens them
- `SEAM-1` now serves as closeout-backed upstream evidence for the active wrapper/manifest seam
- legacy triad artifacts in this directory are retained as source provenance, but the seam-pack
  files in this README are now the canonical planning surface for downstream work
