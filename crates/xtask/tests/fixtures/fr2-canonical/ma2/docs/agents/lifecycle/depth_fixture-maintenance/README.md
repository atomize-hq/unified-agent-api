<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# depth_fixture maintenance

This packet tracks automated upstream-release maintenance for `depth_fixture`.

## Request

- request artifact: `docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml`
- trigger kind: `upstream_release_detected`
- basis ref: `cli_manifests/depth_fixture/latest_validated.txt`
- opened from: `.github/workflows/agent-maintenance-open-pr.yml`
- recorded at: `2026-11-03T00:00:00Z`
- request commit: `abcdef0`

## Trigger context

- detected_by: `fixture`
- current_validated: `0.1.0`
- target_version: `0.2.0`
- latest_stable: `0.3.0`
- version_policy: `latest_stable_minus_one`
- source_kind: `github_releases`
- source_ref: `fixture/isolated`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- branch_name: `automation/depth_fixture-maintenance-0.2.0`

## Support-surface audit

- required: `true`
- pre-run debt count: `0`
- expected post-run debt count: `0`
- discovered upstream surface rows: `0`
- preexisting unsupported rows: `0`
- required uplifts this run:
- none
- deferred preexisting gaps:
- none


## Canonical execution contract

Use `docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md` as the exact contributor execution contract for this lane. The PR body summary under `docs/agents/lifecycle/depth_fixture-maintenance/governance/pr-summary.md` is derivative only.
