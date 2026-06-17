# Workflow Dispatch Summary

- watcher entrypoint: `.github/workflows/agent-maintenance-release-watch.yml`
- queue artifact: `docs/agents/lifecycle/opencode-maintenance/governance/proof/watch-queue.json`
- proven agent: `opencode`
- current_validated: `1.4.11`
- latest_stable: `1.14.48`
- target_version: `1.14.47`
- dispatch_kind: `packet_pr`
- dispatch_workflow: `agent-maintenance-open-pr.yml`
- opened_from: `.github/workflows/agent-maintenance-open-pr.yml`
- branch_name: `automation/opencode-maintenance-1.14.47`
- generated request: `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`
- generated handoff: `docs/agents/lifecycle/opencode-maintenance/HANDOFF.md`
- generated pr summary: `docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md`

The shared watcher emitted an `opencode` queue entry that resolved directly to the generic packet-PR opener. The parent branch then regenerated the live `opencode` maintenance packet from that queue truth without introducing a bespoke workflow surface.
