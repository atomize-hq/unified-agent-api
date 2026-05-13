# Proof Notes

- Final request sha256: `f8fa17dc42ca05bf3ec09e7f01423240234db0fdf2553a45e39b98b90c71f570`
- Final successful `run_id`: `20260512T235319Z`
- Final write result: `write_validated` with `validation_passed = true`
- Final closeout result: `cargo run -p xtask -- close-agent-maintenance --request docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml --closeout docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json` succeeded after a bounded `crates/xtask` closeout-state fix
- Retained pre-open evidence remains truthful for the final run:
  - `watch-queue.json`
  - `workflow-dispatch-summary.md`
- Superseded attempts were not archived as final proof:
  - `20260512T174458Z` wrote the packet but failed `make preflight` once; the same tree later passed `make preflight`, so the run was discarded and replaced with a fresh packet
  - `20260512T181930Z` failed inside managed Codex execution after `npm exec` timed out fetching `opencode-ai@1.14.47`; the local npm path was warmed before the final rerun
- Final repo truth for the successful run:
  - target version stayed `1.14.47`
  - branch name stayed `automation/opencode-maintenance-1.14.47`
  - closeout stayed manual during write mode and was authored only after the successful write
