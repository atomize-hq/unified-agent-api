# Support-depth policy — maintainer dispositions for P1 (2026-10-02)

- Recorded: 2026-10-02, Claude Code session (lead), from maintainer decisions made in-session.
- Authority: maintainer (repository owner), stated in chat. This file is a local record
  (`.codex/` is gitignored); the P1 contracts carry the decisions as rules.
- P0 package: branch `docs/support-depth-policy-proposal` @ `2797b8d9` (local, unpushed).
  Proposal `690dac5d…`, companion `b35b0e79…`, backlog `c5f776b1…`.
- Supersedes: nothing. Extends `.codex/dossiers/support-depth-policy-decisions-2026-09-28.md`,
  whose legacy/migration treatment was already superseded by the greenfield drafts.
- Pro consult: `.codex/guidance/2026-10-02-support-depth-p1-dispositions-approach-review.md`
  (ADJUST, keep architecture and dispositions; prose-only evidence base).

## Authorizations given

| What | Scope |
| --- | --- |
| Amend `docs/specs/**` | P1 contract inventory only; rules only. No evaluator code, runtime work, enrollment or pilot execution. |
| Land P0 | Done as local commit `2797b8d9`. Not pushed; no PR. |
| Pro review of corrected bytes | Waived: prior review accepted as sufficient for the line-351 delta. |

## Dispositions

| ID | Disposition | Status |
| --- | --- | --- |
| D1 families/selectors | As recommended (BASE + templates, exact effect selectors, explicit opaque exceptions). | Accepted as recommended |
| D2 query/output promises | As recommended (keep opaque MCP output; verified schema for new record queries). | Accepted as recommended |
| D3 identity/serialization | As recommended (four logical bindings through existing fields first). | Accepted as recommended |
| D4 receipts/adequacy | As recommended (reuse existing tests/runners/CI plus content-bound adequacy review). | Accepted as recommended |
| D5 dependency closure/reuse | As recommended (affected crate + transitive build closure; exact version/target default). | Accepted as recommended |
| D6 grants/transition storage | As recommended (existing debt inventory authoritative; transitions stored with run/closeout). | Accepted as recommended |
| D7 clean adoption/enrollment | As recommended (positive selection, explicit unenrolled remainder, no enrollment subsystem). | Accepted as recommended |
| D8 pilot + onboarding agent | **Deferred** to the enrollment checkpoints. Not a P1 input. | Deferred (see triggers) |
| D9 additive interface/lineage | As recommended (derived candidates + approved selection + existing packets). | Accepted as recommended |
| D10 capability allowlist | Amend the charter allowlist to the eight entries the audit already reads; record rationale. | Decided |
| D11 path-specific activation | P4-M may precede P3, conditional on writer/effect closure. | Decided |
| D12 enrollment recurrence | Pilot enrolls one exact generation. Successor-generation rule deferred. | Decided |
| D13 final-write control | Serialize affected publication first; per-writer compare-and-refuse later. | Decided |
| D14 unified-support roll-up | Re-derive `uaa_support` from capability + depth truth under a schema version bump, for depth-enrolled generations only. Historical rows keep their original meaning. **Reaffirmed** after the lead corrected its framing: this amends the support-matrix spec's deliberate separation from capability advertising, it is not a drift fix. Constraint: capability linkage travels through depth facts under the manifest root, so the support matrix never reads the capability inventory. Second consumer to account for: `markdown_support_claim` drift checks read `uaa_support`. | Decided, reaffirmed |

## Refinements accepted from the Pro review (P1 content, not new decisions)

1. D11's condition is complete mediation, not gate coverage: every supported route to a protected
   effect enforces the fresh admission predicate **with the authority correct for that route**, or
   refuses before changing authoritative state. Sharing a gate implementation is insufficient when
   it checks the wrong lifecycle authority.
2. Inventory method: effects backward to writers, then entrypoints forward, reconciled. Aggregate
   regeneration and lower-level writers with unguarded callers are in scope.
3. D13 narrows H5 and does not discharge it. Stale admission, changed authority/inputs and stale
   compensation (snapshot restoration is a writer) all survive serialization. Define the protected
   operation and its final-admission boundary.
4. D12 needs fail-closed non-matches: not matching an enrollment never licenses overwriting an
   enrolled acceptance result. Admission keys on effects, not on the request's version.
5. State proof / enrollment / qualification / acceptance as explicit distinctions, not new status
   fields. Truthful pending or failed assessment must remain publishable.
6. Payload shape and write authorization are separate: keeping depth facts out of the runtime
   payload does not exempt enrolled pointer changes from depth admission.
7. Four-binding transition rules: what may change, who authorizes it, what it invalidates.
8. Capability matrix and capability audit documents join the P1 scope inventory.
9. Contract-to-schema and contract-to-test impact (the `c0`–`c11` spec tests) is assessed per
   changed rule; no runtime change is smuggled in as test alignment.
10. P1 distinguishes present behavior from required behavior; the writer inventory is final for a
    named source snapshot and is rechecked against landed enforcement before P4-M.

## Findings to carry into the contracts

- **D10 identifiers** (from `AGENT_API_ORTHOGONALITY_ALLOWLIST`, `crates/xtask/src/capability_publication.rs`):
  `agent_api.run`, `agent_api.events`, `agent_api.events.live`, `agent_api.exec.non_interactive`,
  `agent_api.tools.mcp.list.v1`, `agent_api.tools.mcp.get.v1`, `agent_api.tools.mcp.add.v1`,
  `agent_api.tools.mcp.remove.v1`. Published matrix: `mcp.list` two backends, `mcp.get` codex only,
  `mcp.add`/`remove` gated on target support and `allow_mcp_write` and absent by default.
- **`uaa_support` is name-derived.** `classify_uaa_support` in
  `crates/xtask/src/support_matrix/derive.rs` reads only manifest support, backend support and
  evidence notes. It never reads a capability. Rows as of staging `f61534be`: codex partial ×16 /
  unsupported ×20; claude_code partial ×4 / supported ×2; opencode unsupported ×18; aider and
  gemini_cli unsupported, while the capability matrix advertises 15 / 14 / 6 / 4 / 4 universal
  capabilities respectively. Neither P0 draft mentions `uaa_support`.
- **Naming collision.** "family" already means runtime support family in
  `crates/xtask/src/support_matrix/derive.rs` (`runtime_family`,
  `EMBEDDED_RUNTIME_SUPPORT_FAMILIES`). P1 uses "obligation template" for the policy concept.
  Recommended by the lead and not objected to; not an explicit maintainer disposition.

## Deferrals and where each will be hit

| Deferred item | Trigger |
| --- | --- |
| D8 pilot upstream version and target | P4-M maintenance enrollment checkpoint. Note the gap: no `linux-x64` OpenCode snapshot newer than `1.4.11`; `latest_supported` is `none` on every target. |
| D8 genuinely-new onboarding agent (Goose is a candidate only) | P4-O onboarding enrollment checkpoint; feasibility packet first. |
| D12 successor-generation rule | After the first enrolled generation has run once. |
| D13 per-writer compare-and-refuse | When serialized publication is shown to be a throughput problem. |
| D14 option C (proof status in capability matrix cells) | After the pilot produces a first qualified capability. |
| Companion §10 verification script asserts pre-revision hashes | Any future edit of the companion; one-sentence fix. |

## Working arrangement for this effort

Maintainer override of the CLAUDE.md worker policy: Codex is excluded. Authoring and landing by
the lead Opus session; dual review by an Opus lane and a Sonnet lane. Requested effort tiers are
Opus "extra" and Sonnet "max"; agent effort comes from the agent definition, so this needs either
acceptance of the existing reviewer definition's `high` or new definitions. Maintainer chose new definitions: `opus-xhigh-adversarial-reviewer` (claude-opus-5-5, xhigh) and `sonnet-max-defect-reviewer` (claude-sonnet-5-5, max), created uncommitted in `.claude/agents/`.
