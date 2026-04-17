# Packet — First Real Third CLI Agent Onboarding

Status: Draft  
Date (UTC): 2026-04-16  
Prepared for: post-phase-1 third-agent selection and implementation handoff  
Related source docs:
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/cli-manifest-support-matrix/plan.md`
- `docs/project_management/next/_templates/cli-agent-onboarding-packet-template.md`

## Purpose

Select the first real third CLI agent to onboard after phase 1, using a reproducible 3-candidate comparison, then hand off the winning choice as bounded implementation work without locking the repo into one permanent downstream planning-pack format.

This packet is informative, not normative. Specs under `docs/specs/**` remain authoritative for contract semantics.

## Scope Lock

In scope:
- compare exactly 3 real candidate CLI agents
- preserve dated external evidence inline
- recommend one winner
- define a concrete evaluation recipe for that winner
- hand off required workstreams, deliverables, risks, and acceptance gates

Out of scope:
- implementing the chosen agent
- adding helper tooling or validators for packet generation
- reopening phase-1 support semantics
- mandating one downstream feature-pack shape

## 1. Candidate Summary

Provenance: `dated external snapshot evidence + maintainer inference`

Shortlisted candidates:
- `OpenCode`
- `Gemini CLI`
- `aider`

Why these 3:
- each is a real terminal-first coding agent with strong current adoption signals
- each has a documented install and usage path we can evaluate today
- together they cover three useful shapes:
  - open-source, provider-agnostic, strongly agentic terminal workflow (`OpenCode`)
  - official major-vendor terminal agent (`Gemini CLI`)
  - established pair-programming terminal workflow with broad model flexibility (`aider`)

Recommendation in one sentence:
- `OpenCode` is the best first real third agent because it combines the strongest current pull with a terminal-native agent workflow, explicit plan/build agent split, and a documented non-interactive/server posture that should stress the repo in useful but tractable ways.

## 2. What Already Exists

Provenance: `committed repo evidence`

Reusable repo surfaces:
- `cli_manifests/codex/**`
- `cli_manifests/claude_code/**`
- `docs/project_management/next/cli-agent-onboarding-charter.md`
- `docs/project_management/next/cli-manifest-support-matrix/plan.md`
- `docs/specs/unified-agent-api/**`
- `crates/codex/`, `crates/claude_code/`, `crates/agent_api/`
- `crates/xtask/**`

Existing constraints this packet must respect:
- phase 1 already separated manifest support, backend support, and UAA unified support
- phase 1 proved future-agent readiness structurally, but deferred real third-agent onboarding
- onboarding must reuse the manifest evidence model rather than inventing a second truth store

## 3. Selection Rubric

Provenance: `maintainer inference informed by dated external snapshot evidence`

Rubric philosophy:
- Product-value signals are primary. The first real third agent should be one users actually want.
- Capability differentiation is secondary. Overlap with current unified API support is expected and good.
- Commercial or gated access is allowed, but access friction and reproducibility cost must be scored explicitly as negatives.
- Scores are per-dimension only. There is no weighted total.

Score buckets:
- `0` = weak / missing / materially blocked
- `1` = partial / notable caveats
- `2` = solid / usable with caveats
- `3` = strong / clearly favorable

Primary dimensions:
- `Adoption & community pull`
- `CLI product maturity & release activity`
- `Installability & docs quality`
- `Reproducibility & access friction`

Secondary dimensions:
- `Architecture fit for this repo`
- `Capability expansion / future leverage`

## 4. Fixed 3-Candidate Comparison Table

Provenance: `dated external snapshot evidence + maintainer inference`

| Candidate | Adoption & community pull | CLI product maturity & release activity | Installability & docs quality | Reproducibility & access friction | Architecture fit for this repo | Capability expansion / future leverage | Notes |
|---|---:|---:|---:|---:|---:|---:|---|
| `OpenCode` | 3 | 3 | 3 | 2 | 3 | 3 | Extremely strong current repo and package pull; provider-auth friction is real, but docs are strong and the terminal agent model is close to this repo’s wrapper/backends shape. |
| `Gemini CLI` | 3 | 3 | 3 | 2 | 2 | 2 | Huge official adoption and strong docs; slightly less attractive as the first third agent because it adds less architectural novelty than OpenCode while still carrying provider-auth friction. |
| `aider` | 2 | 2 | 3 | 3 | 2 | 2 | Mature and well-liked, with broad model flexibility and simple install, but it is more pair-programming oriented and less obviously aligned with the repo’s current event/backend posture than OpenCode. |

## 5. Recommendation

Provenance: `maintainer inference grounded in the comparison table`

Recommended winner:
- `OpenCode`

Why it wins:
- It clears the product-value bar first. Its current GitHub and package-registry pull are extremely strong, so this is not a niche architectural pet project.
- It looks like a real terminal coding agent users already want, not just a thin provider shell.
- It provides useful secondary leverage:
  - built-in `plan` and `build` agents
  - internal subagent support
  - explicit non-interactive `run`
  - headless `serve`
  - ACP server mode over stdin/stdout with nd-JSON
- Those traits should teach the repo more than Gemini CLI or aider without forcing us into a science project.

Why the others did not win:
- `Gemini CLI`
  - Very strong candidate, and likely the cleanest official major-vendor follow-on after Codex and Claude.
  - Lost mainly because `OpenCode` has comparable or better current pull plus a richer agent/runtime surface for the first non-Codex/non-Claude integration.
- `aider`
  - Strong demand and reproducibility. It remains a good candidate.
  - Lost because its workflow is somewhat less aligned with the repo’s current wrapper/backends model and less likely to stress the newly landed neutral seams in the ways we most want to validate first.

## 6. Recommended Agent Evaluation Recipe

Provenance: `dated external snapshot evidence + maintainer inference`

Recommended agent:
- `OpenCode`

Install paths:
- install script:
  - `curl -fsSL https://opencode.ai/install | bash`
- npm:
  - `npm install -g opencode-ai`
- Homebrew:
  - `brew install sst/tap/opencode`

Auth / account / billing prerequisites:
- `OpenCode` is provider-agnostic and requires provider credentials.
- Official docs recommend `opencode auth login` and selecting a provider.
- The docs specifically recommend Anthropic / Claude Pro or Max as the cost-effective path.

Runnable commands for initial evaluation:
```bash
# Verify install
opencode --help

# Configure credentials
opencode auth login

# Explore available models after auth
opencode models --refresh

# Interactive smoke
opencode

# Non-interactive smoke
opencode run "Summarize this repository structure in 5 bullets."

# Optional attached/headless flow
opencode serve
opencode run --attach http://localhost:4096 "List likely Rust workspace crates in this repo."

# Optional ACP surface smoke
opencode acp --cwd .
```

Evidence gatherable without paid or elevated access:
- installability
- CLI help surface
- command topology
- local startup behavior
- whether `run`, `serve`, and `acp` surfaces exist as documented

Likely blocked or degraded without paid/elevated access:
- high-quality model-backed evaluation
- realistic multi-turn coding performance
- provider-specific auth flows that require subscription or API billing

Artifacts to save during evaluation:
- `opencode --help` output
- command inventory for `run`, `auth`, `models`, `serve`, `acp`
- notes on install friction by platform
- notes on credential prerequisites
- notes on whether non-interactive and ACP/server surfaces look strong enough for typed-event mapping

## 7. Repo-Fit Analysis

Provenance: `committed repo evidence + maintainer inference`

Manifest-root expectations:
- a new root under `cli_manifests/opencode/`
- root-local evidence should mirror the phase-1 pattern:
  - version metadata
  - pointer files
  - current snapshot / union expectations
  - reports / coverage artifacts

Wrapper-crate expectations:
- new wrapper crate for terminal spawn + typed streaming + completion
- offline parser or fixture-backed parse surface if OpenCode output/event shapes permit it
- explicit non-interactive mode handling via `opencode run`
- likely need to assess ACP / nd-JSON versus CLI/stdout as the safer wrapper seam

`agent_api` backend expectations:
- backend adapter mapping typed OpenCode events into the universal envelope
- capability and extension advertisement rules must still follow the charter
- redaction, bounded payloads, and completion-finality invariants remain mandatory

Support/publication expectations:
- third-agent onboarding should fit into the existing phase-1 support publication architecture, not bypass it
- manifest support, backend support, and UAA unified support must remain distinct

Likely seam risks:
- OpenCode’s provider-agnostic auth may complicate reproducible fake-binary or fixture flows
- client/server and ACP surfaces may be richer than the current wrappers assume
- output/event shape may push us toward one surface being canonical and the others being secondary

## 8. Required Artifacts

Provenance: `committed repo evidence + maintainer inference`

Manifest-root artifact expectations:
- `cli_manifests/opencode/README.md`
- `artifacts.lock.json`
- `wrapper_coverage.json` or equivalent coverage declaration
- `versions/*.json`
- `current.json`
- `pointers/**`
- `reports/**`

Backend-crate artifact expectations:
- `crates/opencode/` wrapper crate
- fake-binary, fixture, or parser acceptance strategy
- typed event and completion surfaces

`agent_api` artifact expectations:
- backend adapter under `crates/agent_api/`
- capability advertisement updates
- backend-specific extension docs if required

Docs/spec artifact expectations:
- backend contract/spec docs if new extension or event semantics are introduced
- capability-matrix regeneration if new built-in backend capability coverage changes

Evidence/fixture expectations:
- deterministic fixture or fake-binary approach for tests that do not depend on a live provider account
- smoke protocol for manual maintainer evaluation when provider-backed access is required

## 9. Workstreams, Deliverables, Risks, And Gates

Provenance: `maintainer inference grounded in repo constraints`

### Required workstreams

1. Candidate validation closeout
   - confirm the OpenCode evaluation recipe against a real maintainer environment
   - capture actual command/output evidence for the chosen surface

2. Manifest-root onboarding design
   - define `cli_manifests/opencode/` artifact expectations
   - determine target/platform posture
   - define version/pointer/report rules

3. Wrapper-crate contract design
   - select canonical runtime surface:
     - CLI stdout/stderr
     - `run`
     - `serve`
     - ACP / nd-JSON
   - define offline testability strategy

4. `agent_api` backend adapter design
   - map OpenCode behavior into universal run/event/completion contracts
   - identify any backend-specific extensions

5. Validation and fixture plan
   - define fake-binary, parser-fixture, or manual smoke obligations
   - define which gates belong in tests versus human evaluation

### Required deliverables

- one implementation plan that names the canonical OpenCode runtime surface
- one manifest-root artifact plan
- one wrapper/backend test strategy
- one capability/extension impact assessment
- one maintainer smoke protocol for gated/provider-backed evaluation

### Blocking risks

- OpenCode’s most useful runtime surface may depend on provider-backed auth that is hard to fixture
- ACP/server mode may be attractive but could widen scope if treated as mandatory from day one
- provider-agnostic auth may create more configuration variance than Codex/Claude-specific wrappers

### Acceptance gates

- chosen runtime surface is explicit and defended
- fake-binary / fixture / smoke strategy is explicit
- manifest-root artifact expectations are explicit
- backend support vs UAA unified support boundary remains explicit
- enough evidence exists that a maintainer can start implementation without rediscovering the packet’s reasoning

## 10. Dated Evidence Appendix

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

## 11. Acceptance Checklist

Provenance: `maintainer inference`

- [x] Exactly 3 real candidates are compared.
- [x] The fixed per-dimension comparison table is present.
- [x] No weighted total score is used.
- [x] The recommendation explains the winner and tie-break reasoning.
- [x] The recommended agent includes a concrete evaluation recipe.
- [x] Every judgment-heavy section has a provenance line.
- [x] The dated evidence appendix includes all 3 candidates.
- [x] Commercial or gated access requirements are explicit where applicable.
- [x] Required workstreams, deliverables, risks, and acceptance gates are present.
- [x] The packet stays shape-agnostic about downstream planning-pack format.
