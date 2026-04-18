# Threading - OpenCode implementation

This document makes the OpenCode implementation control plane explicit: authoritative inbound
basis, concrete contract ownership for this pack, dependency edges, revalidation triggers, and
bounded publication follow-through.

Repo-specific contract note: this pack uses the existing normative OpenCode contracts under
`docs/specs/**` as the canonical refs. It does not create new `docs/contracts/**` artifacts for
OpenCode.

## Execution horizon summary

- Active seam: `SEAM-2`
- Next seam: `SEAM-3`
- Future seam: none
- `THR-04` from the closed onboarding pack is the authoritative inbound handoff for this pack.

## Inbound authoritative handoff

- **Thread ID**: `THR-04`
  - **Producer seam**: upstream `SEAM-4` in `docs/project_management/next/opencode-cli-onboarding/`
  - **Consumer seam(s)**: `SEAM-1`, `SEAM-2`, `SEAM-3`
  - **Carried contract IDs**: upstream `C-07`
  - **Purpose**: carry forward the closeout-backed recommendation that OpenCode landing should
    proceed as backend-support work and must not spin up active UAA promotion planning under the
    current evidence basis.
  - **State**: `revalidated`
  - **Revalidation trigger**: new multi-backend evidence, spec-registry changes, or additional
    OpenCode behavior that materially changes the promotion case
  - **Satisfied by**: `docs/project_management/next/opencode-cli-onboarding/threading.md` and
    `docs/project_management/next/opencode-cli-onboarding/governance/seam-4-closeout.md`
  - **Notes**: `SEAM-1` and `SEAM-2` have now revalidated against this thread; this pack still
    consumes the published onboarding recommendation directly and does not replace it with a new
    bridge artifact

## Contract registry

- **Contract ID**: `C-01`
  - **Type**: API
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`
  - **Derived consumers**: future OpenCode regression and release review work
  - **Thread IDs**: `THR-05`
  - **Definition**: the concrete `crates/opencode/` wrapper implementation surface remains pinned
    to `opencode run --format json`, accepted controls stay limited to `--model`, `--session` /
    `--continue`, `--fork`, and `--dir`, and the wrapper owns parsing, event typing, completion
    finality handoff, and redaction.
  - **Canonical contract ref**: `docs/specs/opencode-wrapper-run-contract.md`
  - **Versioning / compat**: helper-surface expansion is a reopen event, not additive scope inside
    this pack

- **Contract ID**: `C-02`
  - **Type**: state
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`
  - **Derived consumers**: future validation, support publication, and release review flows
  - **Thread IDs**: `THR-05`
  - **Definition**: `cli_manifests/opencode/` owns the committed OpenCode root inventory,
    pointer/update rules, version metadata posture, report layout, and root-validator expectations
    needed to represent manifest support and backend support separately.
  - **Canonical contract ref**: `docs/specs/opencode-cli-manifest-contract.md`
  - **Versioning / compat**: new artifacts must remain compatible with the repo's existing
    manifest-evidence model and four-layer support separation

- **Contract ID**: `C-03`
  - **Type**: schema
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`
  - **Derived consumers**: future backend regression, capability inventory, and stale-trigger
    revalidation work
  - **Thread IDs**: `THR-06`
  - **Definition**: the OpenCode backend maps wrapper-owned events into the universal envelope,
    advertises only capabilities it can honor deterministically, fails closed on unsupported
    extensions, preserves DR-0012 completion gating, and keeps public payloads bounded and
    redacted.
  - **Canonical contract ref**: `docs/specs/opencode-agent-api-backend-contract.md`
  - **Versioning / compat**: backend-specific behavior remains under backend-owned visibility until
    a separate stale-trigger-driven promotion effort proves otherwise

- **Contract ID**: `C-04`
  - **Type**: config
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**: pack closeout and future stale-trigger follow-on packs
  - **Derived consumers**: support publication, capability inventory, and future promotion review
  - **Thread IDs**: `THR-07`
  - **Definition**: OpenCode publication work extends the repo's committed root and backend sets so
    support and capability inventories can include OpenCode while preserving manifest support,
    backend support, UAA unified support, and passthrough visibility as distinct layers.
  - **Canonical contract ref**: `docs/specs/unified-agent-api/support-matrix.md`
  - **Versioning / compat**: publication changes must not imply UAA promotion or collapse
    passthrough visibility into unified support

## Thread registry

- **Thread ID**: `THR-05`
  - **Producer seam**: `SEAM-1`
  - **Consumer seam(s)**: `SEAM-2`, `SEAM-3`
  - **Carried contract IDs**: `C-01`, `C-02`
  - **Purpose**: publish a closeout-backed wrapper and manifest implementation handoff so backend
    work and publication follow-through consume landed root truth instead of inferring it from
    planning prose.
  - **State**: `revalidated`
  - **Revalidation trigger**:
    - any change to the canonical OpenCode run surface or accepted controls
    - any change to wrapper-owned parser, event typing, completion-finality, or redaction behavior
    - any change to manifest-root inventory, pointer/update rules, wrapper-coverage posture, or
      root-validator semantics
    - any change that weakens deterministic fake-binary, fixture, transcript, offline-parser, or
      root-validation proof as the default done-ness path
  - **Satisfied by**:
    - `SEAM-1` closeout plus the landed wrapper and manifest artifacts
    - deterministic wrapper proof under `crates/opencode/tests/**` and
      `crates/opencode/src/bin/fake_opencode_run_json.rs`
    - committed manifest-root evidence under `cli_manifests/opencode/**`
    - mechanical root validation via `cargo run -p xtask -- codex-validate --root cli_manifests/opencode`
  - **Notes**:
    - `SEAM-2` has now revalidated against this thread and can treat the wrapper and manifest
      handoff as current input rather than provisional planning prose
    - `SEAM-2` consumes wrapper-owned runtime truth from landed `crates/opencode/**` code and its
      deterministic proof surfaces, not from live provider-backed smoke or planning prose
    - `SEAM-3` consumes manifest-root truth from landed `cli_manifests/opencode/**` evidence and
      the root validator posture, not from backend-planning assumptions or packet-era notes
    - the thread remains the authoritative upstream backend-consumer handoff until `THR-06`
      publishes from `SEAM-2`

- **Thread ID**: `THR-06`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-3`
  - **Carried contract IDs**: `C-03`
  - **Purpose**: expose the actual OpenCode backend behavior, capability posture, extension
    ownership, and validation evidence needed for bounded publication follow-through.
  - **State**: `identified`
  - **Revalidation trigger**: any change in wrapper inputs, event/completion mapping, payload
    bounds, redaction, or capability advertisement
  - **Satisfied by**: `SEAM-2` closeout plus landed backend tests and publication-facing evidence
  - **Notes**: this thread is intentionally backend-support scoped; it does not authorize UAA
    promotion by itself

- **Thread ID**: `THR-07`
  - **Producer seam**: `SEAM-3`
  - **Consumer seam(s)**: pack closeout and future stale-trigger follow-on packs
  - **Carried contract IDs**: `C-04`
  - **Purpose**: publish the explicit support/publication answer for OpenCode implementation work:
    which rows, root sets, and backend inventories were updated, what remained backend-specific,
    and which stale triggers would reopen promotion review later.
  - **State**: `identified`
  - **Revalidation trigger**: any `THR-04` stale trigger fires, support-matrix semantics change,
    capability inventory assumptions change, or publication evidence starts implying UAA promotion
  - **Satisfied by**: `SEAM-3` closeout and the committed support publication artifacts
  - **Notes**: this thread exists to keep publication follow-through explicit and bounded, not to
    create a new generic lifecycle framework

## Dependency graph

- upstream onboarding `THR-04` -> `SEAM-1`: the active seam inherits the no-new-bridge and
  no-active-UAA-promotion posture directly from the closed onboarding pack
- `SEAM-1 -> SEAM-2`: backend implementation must consume landed wrapper and manifest behavior, not
  redefine it
- `SEAM-1 -> SEAM-3`: support publication cannot be trusted until the OpenCode root and its
  deterministic validation posture exist
- `SEAM-2 -> SEAM-3`: publication follow-through must derive from the actual landed backend
  capabilities and evidence

## Critical path

`THR-04 (closed onboarding handoff)` -> `SEAM-1 (wrapper crate + manifest foundation)` ->
`SEAM-2 (agent_api backend implementation)` -> `SEAM-3 (backend support publication and validation
follow-through)`

## Workstreams

- `WS-WRAPPER-MANIFEST-FOUNDATION`: `SEAM-1`
- `WS-AGENT-API-OPENCODE`: `SEAM-2`
- `WS-BACKEND-SUPPORT-PUBLICATION`: `SEAM-3`
