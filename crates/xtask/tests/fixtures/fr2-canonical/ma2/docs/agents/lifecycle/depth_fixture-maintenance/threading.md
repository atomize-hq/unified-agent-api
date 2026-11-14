<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Threading

1. Review the auto-generated request at `docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-request.toml` and the canonical contract at `docs/agents/lifecycle/depth_fixture-maintenance/HANDOFF.md`.
2. Apply the exact coding-agent prompt from `HANDOFF.md` against branch `automation/depth_fixture-maintenance-0.2.0`.
3. After the green gates pass, the actor handed the packet PR authors `docs/agents/lifecycle/depth_fixture-maintenance/governance/maintenance-closeout.json` and runs the exact `close-agent-maintenance` command from `HANDOFF.md`. An agent executing this packet inside a relay session must not run that command.
