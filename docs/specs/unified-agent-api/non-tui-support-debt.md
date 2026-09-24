# Non-TUI Support Debt Inventory

Status: Normative  
Scope: committed baseline inventory for enrolled automated-maintenance non-TUI support debt

This document is the machine-checkable baseline inventory for temporary enrolled non-TUI support
blockers.

## Normative language

This document uses RFC 2119 requirement keywords (`MUST`, `MUST NOT`, `SHOULD`).

## Row shape

Each debt row MUST:

- use one level-3 heading as the canonical row id
- provide exactly these required bullet keys:
  - `agent_id`
  - `surface_kind`
  - `command_path`
  - `surface_id`
  - `current_reason`
  - `blocker_class`
  - `owner`
  - `milestone`
  - `follow_on`
  - `evidence_ref`
  - `scope_target_triples`
  - `authorized_at_version`
  - `authorization_evidence_ref`

`scope_target_triples` MUST be a non-empty comma-separated list of canonical target triples from
the row agent's `union.expected_targets`; platform names, wildcards, ranges, and omission are not
valid. `authorized_at_version` MUST be exactly one canonical upstream semantic version, never a
range, list, or moving value. `authorization_evidence_ref` MUST name the coverage report for that
agent and version whose same-surface `upstream_available_on` observations contain every granted
target. `evidence_ref` remains historical provenance and is not authorization evidence.

Rows with the same surface identity MAY grant disjoint targets at one version or grants at
different versions. Rows MUST NOT overlap on one `(surface identity, authorized_at_version,
target triple)`.

Allowed `blocker_class` values are aligned to the maintenance-request contract:

- `upstream_not_machine_exposed`
- `platform_evidence_missing`
- `requires_new_infra`
- `requires_new_architectural_seam`
- `outside_registry_maintenance_write_envelope`

Row ids are the canonical `debt_ref` anchors used by `support_surface_audit`.

A maintenance run carries a preexisting row to its target version by updating that row in place:
`scope_target_triples`, `authorized_at_version`, and `authorization_evidence_ref`. It does so only
while the row's `blocker_class` still holds; otherwise the surface is uplifted. Adding a second row
for the same identity remains valid. Both gates render the lexicographically smallest row id, but
they compare it differently: `maintenance-audit-status` compares `debt_ref` (the row id), while
strict reconciliation compares only the deferral reason and follow-on. Until `uaa-0059` is resolved,
an added row whose id sorts first therefore makes the gates disagree, so maintenance runs update in
place.

## Contract marker

### `support-debt-authorization-contract-target-version-v1`

This marker is required. Readers that do not understand target- and version-scoped authorization
interpret it as an incomplete debt row and reject the inventory rather than broadening grants.

## Inventory

### `claude-code-install-command`

- `agent_id`: `claude_code`
- `surface_kind`: `commands`
- `command_path`: `claude_code install`
- `surface_id`: `install`
- `current_reason`: `Current wrapper contract excludes installation flows even though the upstream non-TUI surface exists on win32-x64.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post packet-pr convergence follow-on`
- `follow_on`: `TODOS.md#close-claude-code-install-maintenance-gap`
- `evidence_ref`: `cli_manifests/claude_code/reports/2.1.29/coverage.any.json`
- `scope_target_triples`: `win32-x64`
- `authorized_at_version`: `2.1.29`
- `authorization_evidence_ref`: `cli_manifests/claude_code/reports/2.1.29/coverage.any.json`

### `claude-code-install-force-flag`

- `agent_id`: `claude_code`
- `surface_kind`: `flags`
- `command_path`: `claude_code install`
- `surface_id`: `--force`
- `current_reason`: `The wrapper excludes the Windows installation force path along with the install command seam.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post packet-pr convergence follow-on`
- `follow_on`: `TODOS.md#close-claude-code-install-maintenance-gap`
- `evidence_ref`: `cli_manifests/claude_code/reports/2.1.29/coverage.any.json`
- `scope_target_triples`: `win32-x64`
- `authorized_at_version`: `2.1.29`
- `authorization_evidence_ref`: `cli_manifests/claude_code/reports/2.1.29/coverage.any.json`

### `codex-completion-command`

- `agent_id`: `codex`
- `surface_kind`: `commands`
- `command_path`: `codex completion`
- `surface_id`: `completion`
- `current_reason`: `Shell completion generation remains outside the current shared wrapper execution seam.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-codex-completion-maintenance-gap`
- `evidence_ref`: `cli_manifests/codex/reports/0.129.0/coverage.any.json`
- `scope_target_triples`: `x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, aarch64-apple-darwin, x86_64-pc-windows-msvc`
- `authorized_at_version`: `0.144.6`
- `authorization_evidence_ref`: `cli_manifests/codex/reports/0.144.6/coverage.any.json`

### `codex-completion-shell-arg`

- `agent_id`: `codex`
- `surface_kind`: `positional_args`
- `command_path`: `codex completion`
- `surface_id`: `SHELL`
- `current_reason`: `The shell selector argument is blocked on the same completion-generation seam as the parent command.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-codex-completion-maintenance-gap`
- `evidence_ref`: `cli_manifests/codex/reports/0.129.0/coverage.any.json`
- `scope_target_triples`: `x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, aarch64-apple-darwin, x86_64-pc-windows-msvc`
- `authorized_at_version`: `0.144.6`
- `authorization_evidence_ref`: `cli_manifests/codex/reports/0.144.6/coverage.any.json`

### `opencode-acp-command`

- `agent_id`: `opencode`
- `surface_kind`: `commands`
- `command_path`: `opencode acp`
- `surface_id`: `acp`
- `current_reason`: `OpenCode maintenance still centers the narrow v1 seam and has not absorbed the wider non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`
- `scope_target_triples`: `linux-x64`
- `authorized_at_version`: `1.4.11`
- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.4.11/coverage.any.json`

### `opencode-attach-command`

- `agent_id`: `opencode`
- `surface_kind`: `commands`
- `command_path`: `opencode attach`
- `surface_id`: `attach`
- `current_reason`: `OpenCode maintenance still centers the narrow v1 seam and has not absorbed the wider non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`
- `scope_target_triples`: `linux-x64`
- `authorized_at_version`: `1.4.11`
- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.4.11/coverage.any.json`

### `opencode-models-command`

- `agent_id`: `opencode`
- `surface_kind`: `commands`
- `command_path`: `opencode models`
- `surface_id`: `models`
- `current_reason`: `OpenCode maintenance still centers the narrow v1 seam and has not absorbed the wider non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`
- `scope_target_triples`: `linux-x64`
- `authorized_at_version`: `1.4.11`
- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.4.11/coverage.any.json`

### `opencode-providers-command`

- `agent_id`: `opencode`
- `surface_kind`: `commands`
- `command_path`: `opencode providers`
- `surface_id`: `providers`
- `current_reason`: `OpenCode maintenance still centers the narrow v1 seam and has not absorbed the wider non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`
- `scope_target_triples`: `linux-x64`
- `authorized_at_version`: `1.4.11`
- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.4.11/coverage.any.json`

### `opencode-serve-command`

- `agent_id`: `opencode`
- `surface_kind`: `commands`
- `command_path`: `opencode serve`
- `surface_id`: `serve`
- `current_reason`: `OpenCode maintenance still centers the narrow v1 seam and has not absorbed the wider non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`
- `scope_target_triples`: `linux-x64`
- `authorized_at_version`: `1.4.11`
- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.4.11/coverage.any.json`

### `opencode-web-command`

- `agent_id`: `opencode`
- `surface_kind`: `commands`
- `command_path`: `opencode web`
- `surface_id`: `web`
- `current_reason`: `OpenCode maintenance still centers the narrow v1 seam and has not absorbed the wider non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`
- `scope_target_triples`: `linux-x64`
- `authorized_at_version`: `1.4.11`
- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.4.11/coverage.any.json`

### `opencode-run-attach-flag`

- `agent_id`: `opencode`
- `surface_kind`: `flags`
- `command_path`: `opencode run`
- `surface_id`: `--attach`
- `current_reason`: `The wider OpenCode run surface remains blocked on the same architectural seam as the broader non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`
- `scope_target_triples`: `linux-x64`
- `authorized_at_version`: `1.4.11`
- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.4.11/coverage.any.json`

### `opencode-run-agent-flag`

- `agent_id`: `opencode`
- `surface_kind`: `flags`
- `command_path`: `opencode run`
- `surface_id`: `--agent`
- `current_reason`: `The wider OpenCode run surface remains blocked on the same architectural seam as the broader non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`
- `scope_target_triples`: `linux-x64`
- `authorized_at_version`: `1.4.11`
- `authorization_evidence_ref`: `cli_manifests/opencode/reports/1.4.11/coverage.any.json`
