### S3 — Codex `agent_api.session.resume.v1` mapping + SA-C05 (control + env overrides)

- This slice was decomposed into sub-slices in this directory:
  - `slice-3-codex-resume-v1-mapping-and-sa-c05/`
- Archived original: `archive/slice-3-codex-resume-v1-mapping-and-sa-c05.md`

#### Sub-slices

- `subslice-1-sa-c05-stream-resume-control.md` (S3a): implement SA-C05 resume streaming control + env overrides in `crates/codex`.
- `subslice-2-sa-c05-tests-env-and-termination.md` (S3b): pin SA-C05 env-override + termination behavior with `crates/codex` tests.
- `subslice-3-agent-api-codex-resume-mapping.md` (S3c): validate + map `resume.v1` in the `agent_api` Codex backend and call SA-C05.
- `subslice-4-agent-api-codex-selection-failure-translation.md` (S3d): implement pinned “not found” translation and terminal `Error` event rule.
- `subslice-5-agent-api-fake-codex-integration-tests-and-advertise.md` (S3e): fake-binary integration tests, then enable allowlist + advertise capability id.
