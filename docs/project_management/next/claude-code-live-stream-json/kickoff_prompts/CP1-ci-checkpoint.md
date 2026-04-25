# Kickoff Prompt — CP1-ci-checkpoint (GitHub-hosted multi-OS)

## Scope
- Run the bounded CI checkpoint gates defined in:
  - `docs/project_management/next/claude-code-live-stream-json/ci_checkpoint_plan.md`

## Start Checklist
1. `git checkout feat/claude-code-live-stream-json && git pull --ff-only`
2. Read: `ci_checkpoint_plan.md`, `session_log.md`, and this prompt.
3. Set `CP1-ci-checkpoint` status to `in_progress` in `tasks.json` (orchestration branch only).
4. Add START entry to `session_log.md`; commit docs (`docs: start CP1-ci-checkpoint`).

## Requirements
- Tested SHA: the HEAD commit on `feat/claude-code-live-stream-json` after `C1-integ` is completed.
- Trigger the dedicated workflow:
  - `.github/workflows/claude-code-live-stream-json-smoke.yml`
- The workflow MUST run the multi-OS smoke scripts on GitHub-hosted runners:
  - Linux: `scripts/smoke/claude-code-live-stream-json/linux-smoke.sh`
  - macOS: `scripts/smoke/claude-code-live-stream-json/macos-smoke.sh`
  - Windows: `scripts/smoke/claude-code-live-stream-json/windows-smoke.ps1`
- Run Linux-only gate:
  - `make preflight`

## End Checklist
1. Record evidence in `session_log.md`:
   - tested SHA
   - workflow run ids/links (or command output if run manually)
   - per-OS pass/fail
2. Set `CP1-ci-checkpoint` status to `completed` in `tasks.json`.
3. Commit docs (`docs: finish CP1-ci-checkpoint`).

