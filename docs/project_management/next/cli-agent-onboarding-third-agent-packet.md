# Provenance Stub — Third-Agent Comparison Packet

The canonical comparison packet moved to:

- `docs/reports/verification/cli-agent-selection/third-agent-packet.md`

This legacy path remains only as a provenance pointer for historical planning references.

UAA promotion-gate expectations:
- capability-matrix impact review only after the wrapper crate and backend adapter surfaces are concrete
- explicit decision on which OpenCode behaviors remain backend-specific passthrough versus UAA-promoted support

Docs/spec artifact expectations:
- backend contract/spec docs if new extension or event semantics are introduced
- capability-matrix regeneration if new built-in backend capability coverage changes

Evidence/fixture expectations:
- deterministic fixture or fake-binary approach for tests that do not depend on a live provider account
- smoke protocol for manual maintainer evaluation when provider-backed access is required

## 9. Maintainer Smoke Evidence Addendum

Provenance: `observed maintainer environment evidence + maintainer inference`

Environment used:
- host repo: `atomize-hq/unified-agent-api`
- install path used for closeout: `npm install -g opencode-ai`
- observed CLI version: `1.4.7`

Observed credential posture:
- `opencode auth list` showed configured credentials for `Azure`, `LMStudio`, `MiniMax Coding Plan (minimaxi.com)`, and `OpenRouter`
- `opencode models` succeeded and returned provider/model inventory, including `opencode/*`, `azure/*`, `lmstudio/*`, `minimax-cn-coding-plan/*`, and `openrouter/*`

Observed successful runtime evidence:
- `opencode run --format json -m opencode/gpt-5-nano "Reply with the word OK."`
  - emitted structured `step_start`, `text`, and `step_finish` events
  - proved the CLI can produce line-delimited raw JSON events on a single headless run surface
- `opencode run --format json -m opencode/gpt-5-nano "Summarize this repository structure in 5 bullets."`
  - produced a non-trivial structured response against the local repo
  - confirmed the run surface is suitable for typed-event mapping beyond trivial echo behavior
- `opencode run --format json -m opencode/gpt-5-nano --session <session_id> "Reply with CONTINUED."`
  - reused the same session and returned a structured continued response
- `opencode run --format json -m opencode/gpt-5-nano --session <session_id> --fork "Reply with FORKED."`
  - created a new session and returned a structured forked response
- `opencode run --format json -m opencode/gpt-5-nano --dir . "Reply with DIR_OK."`
  - confirmed the canonical surface supports explicit working-directory control

Observed helper-surface evidence:
- `opencode serve --port 4101`
  - started an HTTP server and printed `opencode server listening on http://127.0.0.1:4101`
  - warned that `OPENCODE_SERVER_PASSWORD` was not set and the server was unsecured
- `opencode acp --cwd .`
  - produced no TUI/run-style output and did not present itself as the primary prompt-driven run surface
  - behaved like a protocol-oriented helper surface rather than the canonical wrapper run surface

Observed provider/auth failures and caveats:
- default provider routing through OpenRouter failed with `401 User not found`
- `azure/gpt-5.4-mini` failed because `AZURE_RESOURCE_NAME` was not configured
- `minimax-cn-coding-plan/MiniMax-M2.5-highspeed` failed due missing API secret/header
- `lmstudio/openai/gpt-oss-20b` did not return within the probe window, consistent with an unavailable local server

Closeout conclusion:
- real authenticated maintainer-backed smoke evidence now exists for `OpenCode`
- the evidence is sufficient to freeze the initial wrapper-surface choice
- the closeout does **not** justify bundling `serve` or `acp` into v1 wrapper scope

## 10. Canonical V1 Runtime-Surface Decision

Provenance: `observed maintainer environment evidence + committed repo evidence + maintainer inference`

Chosen v1 canonical wrapper surface:
- `opencode run --format json`

Why this surface wins:
- it is a single-command, headless, prompt-driven run surface analogous to `codex exec --json` and Claude Code `--print --output-format stream-json`
- it emits machine-parseable raw JSON events directly from the run without requiring `serve`, `attach`, ACP, or the TUI
- it proved prompt, model, session reuse, fork, and explicit working-directory control on the same surface
- it is the narrowest surface that satisfies the wrapper-crate-first charter without prematurely expanding into protocol/server work

Pass criteria satisfied by observed evidence:
- deterministic headless spawn surface exists
- structured live events appear before process exit
- a completion event is distinct from streamed text output
- model selection works on the same surface
- session continuation and fork work on the same surface
- explicit working directory works on the same surface

V1 explicit decisions:
- `crates/opencode/` should target `opencode run --format json` first and only
- `serve` is classified as a secondary backend-owned HTTP/helper surface
- `acp` is classified as a secondary backend-owned protocol/helper surface
- plain formatted stdout/stderr output is **not** acceptable as the canonical wrapper transport
- `opencode run --attach ...` is deferred until after the core run surface is stable

Pinned safe rejections / wrapper-owned behavior:
- multi-directory add-on semantics comparable to Codex `add_dirs` are out of scope for v1 and should fail closed until explicitly specified
- wrapper timeouts remain wrapper-owned behavior; the canonical CLI surface does not need a native timeout flag to be acceptable
- any OpenCode-only controls exposed only through `serve` or `acp` remain backend-specific and deferred

What would have invalidated this choice:
- if `--format json` had been only a debug/log dump rather than a stable run-event transport
- if `serve` or `acp` had been required to obtain structured events or completion
- if stdout had mixed human text and raw JSON in a way that prevented a robust parser/replay strategy
- if model/session/fork/dir behavior had fragmented across multiple incompatible run paths

## 11. Initial Target And Support Posture

Provenance: `committed repo evidence + observed maintainer environment evidence + maintainer inference`

Initial target/platform posture:
- `cli_manifests/opencode/` should start with a three-target root shape:
  - `linux-x64` as the only v1 required target for promotion
  - `darwin-arm64` as an expected but initially optional target
  - `win32-x64` as an expected but initially optional target
- manifest support must remain target-scoped first; Linux may be supported while macOS and Windows remain unsupported or absent

Required evidence artifacts for initial support posture:
- `current.json`
- `versions/*.json`
- `pointers/latest_supported/*.txt`
- `pointers/latest_validated/*.txt`
- `reports/**`
- `wrapper_coverage.json`
- one wrapper-owned protocol evidence artifact under `cli_manifests/opencode/reports/**` capturing raw `run --format json` transcript samples and parser expectations

Wrapper evidence model decision:
- `wrapper_coverage.json` remains necessary for help-surface parity evidence
- `wrapper_coverage.json` is **not** sufficient by itself for OpenCode backend support because the chosen runtime seam is protocol-like and must be evidenced separately
- v1 therefore requires a second committed protocol evidence artifact tied to `run --format json` so backend support can be published without conflating help coverage with runtime/protocol behavior

Support-layer separation to publish:
- `manifest support`
  - means committed `cli_manifests/opencode/**` evidence says a version/target tuple exists at the manifest layer
- `wrapper/backend support`
  - means `crates/opencode` exists and produces wrapper-derived committed evidence for that same target, including wrapper coverage and protocol evidence
- `UAA support`
  - means the Unified Agent API can make a deterministic cross-agent claim after the wrapper surface is frozen, the backend adapter exists, and the relevant contracts/capabilities are actually satisfied
- backend-only passthrough visibility must stay explicit in notes and must not be promoted into UAA support

## 12. V1 Wrapper Scope

Provenance: `observed maintainer environment evidence + committed repo evidence + maintainer inference`

| Aspect | V1 In | V1 Out / Deferred |
|---|---|---|
| Canonical spawn surface | `opencode run --format json` | formatted stdout/stderr scraping, TUI-first flows |
| Streaming source | raw JSON events emitted by `run --format json` | `serve` HTTP event transport, ACP protocol transport |
| Completion semantics | explicit wrapper completion derived after streamed events finish | any design that exposes OpenCode-native protocol types directly |
| Supported run controls | prompt, `--model`, `--session`, `--continue`, `--fork`, `--dir` | `--attach`, share/web/import-export session workflows |
| Add-dir posture | fail closed in v1 unless separately specified | Codex-style multi-add-dir support |
| Parser surface | line-oriented parser for `run --format json` transcripts | parser for `serve` or ACP payloads |
| Fixture strategy | committed replay fixtures captured from `run --format json` plus a fake-binary/process emitter for wrapper tests | live dependency on provider-backed accounts for deterministic tests |
| Helper surfaces | documented as deferred | `serve`, ACP, HTTP attach, protocol/client libraries |
| Capability posture | backend-specific first, UAA promotion later | new promoted `agent_api.*` capabilities derived from `serve`/ACP in v1 |

## 13. Workstreams, Deliverables, Risks, And Gates

Provenance: `maintainer inference grounded in repo constraints`

### Required workstreams

1. `C0` evidence lock-in and fixture strategy
   - confirm the packet closeout evidence in the execution scaffold
   - pin the replay-fixture and fake-binary strategy for `run --format json`

2. `C1` manifest-root and wrapper planning
   - define `cli_manifests/opencode/` artifact rules
   - define `crates/opencode/` wrapper contract around the chosen runtime surface

3. `C2` `agent_api` adapter planning
   - map the chosen OpenCode wrapper events into universal run/event/completion contracts
   - keep scope bounded to the chosen wrapper surface and explicit backend-specific extensions

4. `C3` UAA promotion review
   - assess which OpenCode behaviors remain backend-specific
   - review whether any capability or extension promotion is justified after wrapper/backend scope is concrete

### Required deliverables

- one closed packet with real smoke evidence and an explicit runtime-surface decision
- one triad scaffold under `docs/project_management/next/opencode-cli-onboarding/`
- one manifest-root plan for `cli_manifests/opencode/`
- one wrapper parser/fixture strategy for `run --format json`
- one bounded `agent_api` planning phase that starts only after wrapper decisions are fixed
- one explicit UAA promotion review phase separated from backend support

### Blocking risks

- OpenCode default provider routing may vary by environment even when an `opencode/*` model works
- the `run --format json` event shape may still need normalization rules before wrapper parsing is stable
- `serve` and ACP may tempt scope growth if not held behind the v1-out boundary
- target naming and publication shape must stay target-first rather than collapsing into version-global support claims

### Acceptance gates

- packet closeout evidence is committed in this planning artifact
- chosen wrapper runtime surface is explicit and defended
- parser and fixture strategy is explicit for the chosen surface
- initial target/platform posture is explicit
- wrapper evidence model is explicit and separate from help-surface coverage
- triad scaffold exists before any wrapper or `agent_api` implementation begins
- backend support vs UAA unified support boundary remains explicit

## 14. Dated Evidence Appendix

Provenance: `dated external snapshot evidence`

### Appendix A. `OpenCode`
- Snapshot date: `2026-04-16`
- Official links:
  - [GitHub repo](https://github.com/anomalyco/opencode)
  - [Docs intro](https://dev.opencode.ai/docs/)
  - [CLI docs](https://opencode.ai/docs/cli/)
  - [Providers docs](https://opencode.ai/docs/providers)
- Install / distribution:
  - install script: `curl -fsSL https://opencode.ai/install | bash`
  - npm package: `opencode-ai` latest `1.4.7`, modified `2026-04-16`
  - multiple package-manager paths and desktop binaries documented
- Adoption / community:
  - GitHub stars: `144,473`
  - GitHub forks: `16,344`
  - npm downloads last month: `3,029,913`
- Release activity:
  - latest stable release observed: `v1.4.7`, published `2026-04-16`
  - multiple stable releases published in the prior week
- Access prerequisites:
  - provider credential required
  - official docs recommend provider login via `opencode auth login`
  - official docs recommend Anthropic / Claude Pro or Max, but other providers are supported
- Normalized notes:
  - strongest current adoption signal in this shortlist
  - explicit `run`, `serve`, and `acp` surfaces are unusually attractive for this repo
  - provider-auth friction is real, but documented
  - maintainer smoke closeout on `2026-04-17 UTC` succeeded on `opencode/gpt-5-nano` using `run --format json`
  - default OpenRouter routing and several configured provider paths were not healthy enough to use as the canonical v1 assumption

### Appendix B. `Gemini CLI`
- Snapshot date: `2026-04-16`
- Official links:
  - [GitHub repo](https://github.com/google-gemini/gemini-cli)
  - [GitHub Action using Gemini CLI](https://github.com/google-github-actions/run-gemini-cli)
- Install / distribution:
  - npm package: `@google/gemini-cli` latest `0.38.1`, modified `2026-04-15`
  - install via npm, npx, or other official paths documented in repo/docs
  - Node.js `>=20`
- Adoption / community:
  - GitHub stars: `101,499`
  - GitHub forks: `13,165`
  - npm downloads last month: `3,243,755`
- Release activity:
  - latest stable release observed: `v0.38.1`, published `2026-04-15`
  - preview/nightly cadence visible alongside stable releases
- Access prerequisites:
  - Google auth and/or API configuration required
  - GitHub Action docs show service auth requirements for automation
- Normalized notes:
  - strongest official-major-vendor candidate in the shortlist
  - strong headless and automation story
  - less architecturally distinctive than OpenCode for the first non-Codex/non-Claude onboarding

### Appendix C. `aider`
- Snapshot date: `2026-04-16`
- Official links:
  - [GitHub repo](https://github.com/Aider-AI/aider)
  - [Official site](https://aider.chat/)
- Install / distribution:
  - install helper: `python -m pip install aider-install`
  - PyPI package: `aider-chat` latest `0.86.2`
  - Python requirement: `>=3.10,<3.13`
- Adoption / community:
  - GitHub stars: `43,443`
  - GitHub forks: `4,219`
  - pypistats last month downloads: `866,024`
- Release activity:
  - PyPI latest observed: `0.86.2`
  - recent GitHub stable releases shown through `v0.86.0` in the observed release feed
- Access prerequisites:
  - provider/model credentials required depending on chosen backend
  - works across multiple model providers
- Normalized notes:
  - mature and well-liked terminal coding assistant
  - broad model flexibility is a strength
  - pair-programming orientation may require more adaptation to fit this repo’s current backend/event assumptions

## 15. Acceptance Checklist

Provenance: `maintainer inference`

- [x] Exactly 3 real candidates are compared.
- [x] The fixed per-dimension comparison table is present.
- [x] No weighted total score is used.
- [x] The recommendation explains the winner and tie-break reasoning.
- [x] The recommended agent includes a concrete evaluation recipe.
- [x] Every judgment-heavy section has a provenance line.
- [x] The dated evidence appendix includes all 3 candidates.
- [x] Commercial or gated access requirements are explicit where applicable.
- [x] The packet records real maintainer-backed smoke evidence for the chosen runtime surface.
- [x] The packet names one canonical v1 wrapper surface and explicitly defers `serve` and `acp`.
- [x] The packet includes an explicit v1 in/out boundary for the wrapper crate.
- [x] The packet pins an initial target/platform posture for `cli_manifests/opencode/`.
- [x] The packet makes the wrapper evidence model explicit and keeps it separate from help-surface coverage.
- [x] The packet keeps `crates/opencode/` work ahead of `agent_api` adapter work.
- [x] UAA promotion is treated as a later gate, not bundled into initial backend support.
- [x] Required workstreams, deliverables, risks, and acceptance gates are present and aligned to the crate-first ladder.
