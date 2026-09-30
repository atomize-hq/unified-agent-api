<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->

# Threading

1. Review the auto-generated request at `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml` and the canonical contract at `docs/agents/lifecycle/claude_code-maintenance/HANDOFF.md`.
2. Apply the exact coding-agent prompt from `HANDOFF.md` against branch `automation/claude_code-maintenance-2.1.280`.
3. After the green gates pass, the actor handed the packet PR authors `docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-closeout.json` and runs the exact `close-agent-maintenance` command from `HANDOFF.md`. An agent executing this packet inside a relay session must not run that command.
