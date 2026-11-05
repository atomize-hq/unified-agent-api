# Support depth: machinery-proof rescope and remaining roadmap

Status: MP-P1 and MP-P2 approved by the maintainer on 2026-11-02.
Source baseline: staging `0cb08241` (PR #309).
The [support-depth contract](../../specs/support-depth-contract.md) remains normative and is not
amended here. This document changes no code, enrollment, host setting or credential, and creates
no background job.

## Why this rescope

The approved [proposal sequence](support-depth-policy-proposal.md) keeps two facts separate:

1. **Path machinery proof.** The complete maintenance chain works end to end in an isolated
   workspace. The contract lists "Qualification of any real operation" as something it does not
   establish. The proposal allows controlled fixtures, provided "synthetic proof certifies no
   nonexistent API".
   Maintenance may be enabled without onboarding proof.
2. **Qualification of a real operation.** For example, Codex `run`, which is ordinary support work
   that comes after the paths are enabled.

E1-P1 picked a real Codex operation as the proof subject, and that joined the two facts. Then:
- PI-P1 to PI-P3 required complete-mapping truthfulness: initial, resume and fork, ADMIN mutation,
  and 24 unresolved flags.
- IR-P2 required the musl tuple.
- NE-P1/P2 and NC-P1/P2 required a native producer and six runtime-context facts.

Each step was correct *for qualifying Codex*. None of it is needed to prove that the maintenance
machinery works. Since 2026-10-25 the isolated chain has been waiting on Codex truth work.

## Approved decisions

| Id | Recommendation | Reason / boundary |
| --- | --- | --- |
| MP-P1 — proof subject | Replace E1-P1's proof subject with one **synthetic agent** that exists only in the isolated repository. It has a small fixture CLI with a real tagged release and manifest root, one wrapper-only operation that claims no shared capability, its own RULES and validator spec, and a credential-free CI evidence profile. Run the unchanged E1 chain (steps 0–7 and the rejection matrix) on it. The run is labeled a **machinery proof that qualifies nothing**. | This is the contract's own separation. Nothing in it is fabricated: acquisition, P/O freeze, evidence, adequacy adoption, closeout, promotion and IN1 all run for real on a subject whose whole API is known. No real agent's truth decisions apply. The same subject can serve as the onboarding proof's new agent, which the contract allows. Production registries never contain it. |
| MP-P2 — Codex rules | Keep PI-P1 to PI-P3, IR-P2, NE-P1/P2 and NC-P1/P2 as approved, and **re-label them as Codex qualification rules**. They are prerequisites for accepting depth results for real Codex operations, not for the machinery proof. Depth enrollment itself may still begin with this work outstanding (contract, "Depth enrollment MAY begin with implementation and evidence work outstanding"). Park PR #310 (layer inputs) as Codex support work. | This removes no safeguard. A real Codex enrollment must still meet every one of them. E1-P2's threshold applies unchanged to the synthetic subject. IR-P1 (isolated authority) and IR-P3 (producer terminality) are agent-general and stay in force for the proof. |

Unchanged:
- Annex A remains the sole route inventory.
- Each step must use real public entrypoints.
- There is no generic `--allow-depth` and no blanket NA.
- Closeout and promotion stay separate, and the native merge queue is used.
- Per-agent production authorization stays separate.

## Code that is Codex-specific today

These parts must become driven by the selected subject:
- `isolated_maintenance/scope.rs` (162 lines): fixed `"codex"`, `0.156.1`, musl, `codex exec`, CORE.
- `isolated_maintenance/mod.rs`: `LIVE_PROFILE = None`, the single compiled profile.
- `agent_registry/support_depth/pilot_scope.rs` (132 lines): the Codex pilot modes and flows. This
  stays Codex-only, as qualification policy.
- `depth_ci/profile.rs` (197 lines): the CORE profile is hard-coded to the Codex paths, artifact,
  trust and version.

Pilot scope remains Codex qualification policy. The proof binding gains a second, synthetic subject
alongside it, not a general bypass.

## Remaining roadmap

The critical path is A → B → C → D → E. F and G follow and do not block the workflows.

**A. Decision revisions.** This document.

**B. Maintenance machinery.** Each item is a reviewed PR into `staging`.
1. **Synthetic subject.** The fixture CLI and release source, the registry entry template for the
   isolated repository, RULES and a validator spec, and a CI evidence profile with obligations and
   case mapping.
2. **Subject-driven isolated binding.**
   - First, a bounded amendment to the [evidence contract](../../specs/support-depth-evidence-v1.md).
     Today it allows only the two compiled profiles, and its isolated binding admits only Codex
     `0.156.1` `run`/CORE on musl. The amendment adds the synthetic subject as one more compiled
     profile. Profiles still come only from compiled source, and unsupported selections still
     refuse.
   - Then replace the fixed Codex constants in `scope.rs` and `depth_ci/profile.rs`.
   - The authority boundary (IR-P1) is unchanged.
3. **Depth intake and acquisition to request freeze** (MA1→MA2), through the public entrypoints.
4. **Execution on CI evidence** (MA4).
5. **Closeout only when every due obligation is proven, through IN1** (MA5).
6. **Evidence-backed promotion** through the native merge queue (MA6).
7. **Executed depth-refusal tests** at every ON1–ON5, NE1–NE8 and LW1 entrypoint.
8. **Registry acceptance of `support_depth` tables** for the isolated profile only. Production
   parsing still refuses them until D.
   Status: implemented. `AgentRegistry::parse_isolated` is the one accepting parse; only the
   installed profile mints its admission, for its compiled subject. `AgentRegistry::parse` still
   refuses every table.

**C. Isolated end-to-end run.** A disposable repository and a temporary runner carry the synthetic
subject through E1 steps 0–7 and the rejection matrix. The repository and the credential reference
need separate maintainer authorization before any host write.

**D. Maintenance enablement.** Land the positive and negative evidence. Then per-agent maintainer
authorization enables the maintenance path for each agent (contract "Path enablement").

**E. Onboarding.**
1. The charter defines the create-lane P/O freeze point and the committed artifact that records it.
2. `onboard-agent` writes version metadata, RULES and a validator spec.
3. The runtime follow-on carries depth identity, and `written-paths.json` integrity is fixed.
4. Validator specs for `aider` and `gemini_cli`. Written: each states the shared depth checks
   and lists the truth decisions its root still needs. Neither root has a `RULES.json`, so neither
   adopts Annex B yet.
5. Gap-memo items: runtime handoff capability and publication truth, and the hard-coded backend
   list in `capability_matrix.rs`. The backend list is closed: the generator reads the registry.
6. Wire ON1–ON5 for depth in the isolated workspace only.
7. Independent new-agent proof in that workspace, reusing the synthetic subject. Maintenance
   evidence does not substitute.
8. Establish route mediation (item 3) again for the maintenance path, as the contract requires
   when onboarding is enabled.
9. Per-agent maintainer authorization, given once items 1–8 hold. Each authorization is what
   enables the onboarding path for that agent.

**F. Real-agent qualification (support work).**
- Codex: PI-P3 declarations (resume and fork), the 24 unresolved flags, the IR-P2 musl packet, and
  the NE/NC native producer and six context facts (PR #310).
- Then Claude Code and OpenCode.

**G. Additive shared integration**, last, as the proposal orders it.
