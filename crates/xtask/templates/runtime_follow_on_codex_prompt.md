# Runtime Follow-On

Run id: `{{RUN_ID}}`
Approval: `{{APPROVAL_PATH}}`
Agent: `{{AGENT_ID}}` (`{{DISPLAY_NAME}}`)
Requested tier: `{{REQUESTED_TIER}}`
Allowed rich surfaces: `{{ALLOW_RICH_SURFACES}}`
Minimal justification: `{{MINIMAL_JUSTIFICATION}}`

Read these inputs first:
- {{DOCS_TO_READ}}

Write only inside these owned surfaces:
- {{ALLOWED_WRITE_PATHS}}

Target-owned runtime surfaces:
- wrapper crate: `{{CRATE_PATH}}`
- backend module: `{{BACKEND_MODULE}}`
- manifest root: `{{MANIFEST_ROOT}}`
- wrapper coverage source: `{{WRAPPER_COVERAGE_SOURCE_PATH}}`
- preferred wrapper coverage file: `{{WRAPPER_COVERAGE_MANIFEST_PATH}}`
- required agent_api onboarding test: `{{REQUIRED_TEST_PATH}}`

Approval capability and publication truth:
- canonical targets:
  - {{CANONICAL_TARGETS}}
- always-on capabilities:
  - {{ALWAYS_ON_CAPABILITIES}}
- target-gated capabilities:
  - {{TARGET_GATED_CAPABILITIES}}
- config-gated capabilities:
  - {{CONFIG_GATED_CAPABILITIES}}
- backend extensions:
  - {{BACKEND_EXTENSIONS}}
- support matrix enabled: `{{SUPPORT_MATRIX_ENABLED}}`
- capability matrix enabled: `{{CAPABILITY_MATRIX_ENABLED}}`
- capability matrix target: `{{CAPABILITY_MATRIX_TARGET}}`

Hard rules:
- Do not edit generated `{{MANIFEST_ROOT}}/wrapper_coverage.json`.
- Keep wrapper coverage truth under `{{WRAPPER_COVERAGE_SOURCE_PATH}}/src/**`.
- Publication-owned manifest files are off-limits in this lane.
- Update `{{RUN_DIR}}/handoff.json` so it passes validation with:
  - `agent_id`
  - `manifest_root`
  - `runtime_lane_complete = true`
  - `publication_refresh_required = true`
  - `required_commands` including:
    - {{REQUIRED_COMMANDS}}
  - `blockers`

This packet is the repo-owned execution contract for the runtime lane. After code edits are in place, run the write validation for the same `run_id`.
