# OpenCode implementation - seam extraction

Source:
- `docs/project_management/next/opencode-cli-onboarding/next-steps-handoff.md`
- `docs/project_management/next/opencode-cli-onboarding/threading.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-1-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-2-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-3-closeout.md`
- `docs/project_management/next/opencode-cli-onboarding/governance/seam-4-closeout.md`

This pack is the code-facing plan-of-record for landing OpenCode. It consumes the already-closed
onboarding/contracts work directly, preserves the no-new-bridge rule, and keeps the pack one level
above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: `SEAM-3`
- Next seam: none

Policy:

- only the active seam is eligible for authoritative downstream sub-slices by default
- no additional queued seam remains after `SEAM-3` activation; any follow-on work must come from
  a stale-trigger-driven reopen rather than preview planning inside this pack
- active and next seams must eventually terminate in a dedicated final `S99` seam-exit-gate slice
  once seam-local planning begins
- seams that still need a contract-definition boundary may reserve `S00` during seam-local
  planning
- future seams remain seam briefs only
- the authoritative inbound bridge is the existing onboarding `THR-04` plus the four onboarding
  seam closeouts; this pack does not create a new bridge ledger, sidecar manifest, or lifecycle
  doc
- canonical OpenCode contract refs for this pack live under `docs/specs/**`, not `docs/contracts/**`
- UAA promotion is out of scope unless the published stale triggers from the closed onboarding pack
  fire

Scope restatement:

- Land the first OpenCode code-facing implementation plan across `cli_manifests/opencode/`,
  `crates/opencode/`, and the `crates/agent_api` OpenCode backend.
- Preserve the repo's four support layers: manifest support, backend support, UAA unified support,
  and passthrough visibility.
- Treat deterministic replay, fake-binary, fixture, and offline-parser evidence as the default
  proof path. Live provider-backed OpenCode smoke remains basis-lock or stale-trigger
  revalidation evidence only.

## Verification matrix

| Surface | Owner seam | Default proof path | Acceptance gate | Live-only revalidation |
|---|---|---|---|---|
| `crates/opencode/**` wrapper spawn, stream, completion, parser, redaction | `SEAM-1` | targeted fake-binary, transcript-fixture, and offline-parser tests; expected package name follows the existing pattern (`unified-agent-api-opencode`) unless seam-local review finds a blocking naming conflict | `cargo test -p unified-agent-api-opencode`; accepted controls `--model`, `--session` / `--continue`, `--fork`, and `--dir` are covered; helper surfaces fail closed | only if the `opencode run --format json` basis or prerequisite record from the evidence contract goes stale |
| `cli_manifests/opencode/**` manifest root, pointers, reports, current snapshot, and wrapper coverage artifacts | `SEAM-1` | deterministic artifact generation plus root validation against committed schema/rules | `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`; root passes schema, pointer, report, `current.json`, and support-matrix consistency checks | only if upstream CLI surface or target inventory drift invalidates the closed onboarding evidence basis |
| `crates/agent_api/**` OpenCode backend request mapping, event/completion translation, capability advertisement, and extension fail-closed behavior | `SEAM-2` | targeted backend tests with fixtures or fake wrapper input, plus harness-backed DR-0012 gating checks | `cargo test -p unified-agent-api --features opencode`; redaction, bounded payloads, unsupported extensions, capability advertisement, and DR-0012 completion gating are explicitly covered | only if wrapper inputs, capability registry rules, or stale triggers from `THR-04` reopen the basis |
| support publication and capability inventory outputs | `SEAM-3` | deterministic regeneration plus drift checks over committed evidence only | `cargo run -p xtask -- support-matrix --check`; `cargo run -p xtask -- capability-matrix`; OpenCode is added to the committed root/backends set without implying UAA promotion | only if new multi-backend evidence or universal registry changes reopen the promotion boundary |
| workspace integration | pack closeout | standard repo hygiene/build/test gates | `make fmt-check`; `make clippy`; `make check`; `make test`; `make preflight` before final landing | none |

Live basis-lock command set, when stale triggers require it:

- `opencode run --format json -m opencode/gpt-5-nano "Reply with the word OK."`
- `opencode run --format json -m opencode/gpt-5-nano "Summarize this repository structure in 5 bullets."`
- `opencode run --format json -m opencode/gpt-5-nano --session <session_id> "Reply with CONTINUED."`
- `opencode run --format json -m opencode/gpt-5-nano --session <session_id> --fork "Reply with FORKED."`
- `opencode run --format json -m opencode/gpt-5-nano --dir . "Reply with DIR_OK."`

These commands are not routine done-ness gates for slice completion. They are only for basis-lock
confirmation or stale-trigger revalidation.
