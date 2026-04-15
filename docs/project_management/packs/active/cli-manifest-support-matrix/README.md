# CLI manifest support matrix - seam extraction

Source: `docs/project_management/next/cli-manifest-support-matrix/plan.md`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds for the CLI manifest support-matrix feature. It stays one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-4`
- Next seam: `SEAM-5`
- Future seams: none
- `SEAM-3` has landed and closed, so it is now out of the forward window.

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- the next seam may later receive seam-local review + slices, and only provisional deeper planning
- active and next seams must eventually terminate in a dedicated final `S99` seam-exit gate slice once seam-local planning begins
- seams that still need a contract-definition boundary may reserve `S00` during seam-local planning
- future seams remain seam briefs only
- pack-level `review_surfaces.md` is orientation only; active and next seams still need seam-local `review.md` later
- this repo's normative contract surfaces live under `docs/specs/**`; those repo-stable spec paths are the durable contract refs used by this pack

Assumptions captured for extraction:

- phase 1 stays tooling-only and does not change runtime `agent_api` behavior
- support truth remains target-scoped first, with per-version summaries derived from target rows
- the existing capability matrix remains separate from the new support matrix
- existing manifest evidence under `cli_manifests/codex/**` and `cli_manifests/claude_code/**` stays the only evidence layer
