# Proof Notes

- Proof was captured on parent `staging` after `C2` merged both worker lanes.
- The live queue contained three stale enrolled agents: `codex`, `claude_code`, and `opencode`.
- `opencode` is the only packet-PR enrolled agent in that queue.
- The watcher result moved during implementation. The final proof uses the exact merged-parent queue truth `latest_stable = 1.14.48` and `target_version = 1.14.47`.
- The regenerated `maintenance-request.toml` now carries `request_commit = 1673a34b6eb1e2cf7d6a3bfef229f668c02746f9` so the packet matches the proof-stable parent base.
- `execute-agent-maintenance --dry-run` succeeded against the regenerated live request, proving the frozen packet is relay-executable before any write-mode closeout.
