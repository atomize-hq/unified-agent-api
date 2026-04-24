<!-- generated-by: xtask refresh-agent; owner: control-plane -->

# Threading

1. Run `check-agent-drift --agent opencode`.
2. Record the maintainer-authored request at `docs/reports/agent-lifecycle/opencode-maintenance/governance/maintenance-request.toml`.
3. Apply `refresh-agent --dry-run` and `refresh-agent --write` using that request.
4. Close the maintenance run with `close-agent-maintenance` once findings are resolved or explicitly deferred.
