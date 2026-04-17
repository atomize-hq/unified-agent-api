# Session Log - OpenCode CLI Onboarding

Use short dated START/END entries to record planning progress for this pack. This pack is
pre-implementation, so entries should describe runtime-lock, wrapper-plan, backend-plan, and
promotion-review work rather than code changes.

## [2026-04-16 00:00 UTC] Planning Review - packet-alignment
- Reviewed:
  - `docs/project_management/next/cli-agent-onboarding-charter.md`
  - `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
  - `docs/project_management/next/_templates/cli-agent-onboarding-packet-template.md`
- Result:
  - confirmed the charter requires wrapper-crate-first sequencing
  - updated the packet/template so `crates/<agent>` work is explicitly prior to `agent_api`
  - removed the duplicate onboarding template so there is one canonical template
  - created this feature directory to keep the next steps recorded in-repo
- Commands: none (docs-only review/update)
- Blockers:
  - real maintainer smoke evaluation for `OpenCode` is still pending
  - canonical runtime surface for the wrapper crate is still unresolved

## [2026-04-16 00:30 UTC] Planning Review - triad-scaffold-bootstrap
- Reviewed:
  - `docs/project_management/task-triads-feature-setup-standard.md`
  - `docs/project_management/next/codex-cli-parity/**`
  - `docs/project_management/next/unified-agent-api/**`
  - `docs/project_management/next/cli-agent-onboarding-charter.md`
  - `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
- Result:
  - expanded this directory from a placeholder into a repo-standard pre-implementation triad pack
  - added C0-C3 specs, task registry, and kickoff prompts aligned to crate-first sequencing
  - kept all planning scope confined to this directory and left downstream crate/manifest paths
    untouched
- Commands:
  - `rg --files docs/project_management/next/opencode-cli-onboarding`
  - `sed -n ...` against the source charter, packet, and reference packs
- Blockers:
  - this entry predates the packet closeout now recorded below

## [2026-04-17 03:00 UTC] Planning Review - packet-closeout-complete
- Reviewed:
  - `docs/project_management/next/cli-agent-onboarding-third-agent-packet.md`
  - `docs/project_management/next/cli-agent-onboarding-charter.md`
  - `docs/specs/unified-agent-api/support-matrix.md`
- Result:
  - installed `opencode-ai` and captured real maintainer-environment smoke evidence
  - confirmed structured `run --format json` output plus `--model`, `--session`, `--fork`, and `--dir`
  - classified `serve` and `acp` as deferred helper surfaces
  - froze `opencode run --format json` as the presumptive v1 wrapper surface
- Commands:
  - `npm install -g opencode-ai`
  - `opencode --version`
  - `opencode --help`
  - `opencode auth list`
  - `opencode models`
  - `opencode run --format json -m opencode/gpt-5-nano ...`
  - `opencode serve --port 4101`
  - `opencode acp --cwd .`
- Blockers:
  - provider health was mixed; the default OpenRouter route and several configured provider paths were not stable enough to assume as default v1 posture
  - wrapper parser and fixture implementation still belongs to later execution work
