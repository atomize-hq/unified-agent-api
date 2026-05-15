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

Allowed `blocker_class` values are aligned to the maintenance-request contract:

- `upstream_not_machine_exposed`
- `platform_evidence_missing`
- `requires_new_infra`
- `requires_new_architectural_seam`
- `outside_registry_maintenance_write_envelope`

Row ids are the canonical `debt_ref` anchors used by `support_surface_audit`.

## Inventory

### `claude-code-install-command`

- `agent_id`: `claude_code`
- `surface_kind`: `commands`
- `command_path`: `claude install`
- `surface_id`: `install`
- `current_reason`: `Current wrapper contract excludes installation flows even though the upstream non-TUI surface exists on win32-x64.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post packet-pr convergence follow-on`
- `follow_on`: `TODOS.md#close-claude-code-install-maintenance-gap`
- `evidence_ref`: `cli_manifests/claude_code/reports/2.1.29/coverage.any.json`

### `claude-code-install-force-flag`

- `agent_id`: `claude_code`
- `surface_kind`: `flags`
- `command_path`: `claude install`
- `surface_id`: `--force`
- `current_reason`: `The wrapper excludes the Windows installation force path along with the install command seam.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post packet-pr convergence follow-on`
- `follow_on`: `TODOS.md#close-claude-code-install-maintenance-gap`
- `evidence_ref`: `cli_manifests/claude_code/reports/2.1.29/coverage.any.json`

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

### `opencode-run-command`

- `agent_id`: `opencode`
- `surface_kind`: `commands`
- `command_path`: `opencode run`
- `surface_id`: `run`
- `current_reason`: `OpenCode maintenance still centers the narrow v1 seam and has not absorbed the wider non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`

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

### `opencode-run-format-flag`

- `agent_id`: `opencode`
- `surface_kind`: `flags`
- `command_path`: `opencode run`
- `surface_id`: `--format`
- `current_reason`: `The wider OpenCode run surface remains blocked on the same architectural seam as the broader non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`

### `opencode-run-dir-flag`

- `agent_id`: `opencode`
- `surface_kind`: `flags`
- `command_path`: `opencode run`
- `surface_id`: `--dir`
- `current_reason`: `The wider OpenCode run surface remains blocked on the same architectural seam as the broader non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`

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

### `opencode-run-model-flag`

- `agent_id`: `opencode`
- `surface_kind`: `flags`
- `command_path`: `opencode run`
- `surface_id`: `--model`
- `current_reason`: `The wider OpenCode run surface remains blocked on the same architectural seam as the broader non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`

### `opencode-run-continue-flag`

- `agent_id`: `opencode`
- `surface_kind`: `flags`
- `command_path`: `opencode run`
- `surface_id`: `--continue`
- `current_reason`: `The wider OpenCode run surface remains blocked on the same architectural seam as the broader non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`

### `opencode-run-session-flag`

- `agent_id`: `opencode`
- `surface_kind`: `flags`
- `command_path`: `opencode run`
- `surface_id`: `--session`
- `current_reason`: `The wider OpenCode run surface remains blocked on the same architectural seam as the broader non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`

### `opencode-run-fork-flag`

- `agent_id`: `opencode`
- `surface_kind`: `flags`
- `command_path`: `opencode run`
- `surface_id`: `--fork`
- `current_reason`: `The wider OpenCode run surface remains blocked on the same architectural seam as the broader non-TUI command set.`
- `blocker_class`: `requires_new_architectural_seam`
- `owner`: `wrappers team`
- `milestone`: `post shared maintenance proof follow-on`
- `follow_on`: `TODOS.md#close-opencode-non-tui-maintenance-gaps`
- `evidence_ref`: `cli_manifests/opencode/reports/1.14.47/coverage.any.json`

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
