# round1-sonnet: reviewer report

Lane: Sonnet 5.5 max, defects and conformance. Candidate: `e60664e2`. Scope: full.
Verbatim reviewer output; the lead's adjudication is in
`../support-depth-p1a-review-adjudication-2026-10-02.md`.

---

**Review of candidate e60664e2 (base 2797b8d9). Verdict: not CLEAN. 0 P1, 8 P2, 13 P3.**

`git diff --stat` confirms exactly one new file, 580 lines. `git diff --check` is clean, and every relative link and in-document anchor resolves. I ran no builds or tests.

Paths below are relative to the worktree root `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-a22751ed5dc7079f5`. "C:" means `docs/specs/support-depth-contract.md`.

## P2 findings

**1. Final admission can be satisfied with per-route locks (safety rule; C:436, 444-446, 451-455).**
- The text says "hold serialization ownership of the protected operation" and "with every invalidating change excluded until the effect completes ... need not be repeated". It then says "The serialization boundary MUST be demonstrated for each route."
- Scenario: `refresh-publication` takes its own file lock, checks once inside it and writes. The `parity-promote.yml` promote job writes pointers, runs `manifest-version-metadata` and removes the marker in plain shell under a GitHub `concurrency` group. Each route "demonstrates" its own boundary and neither excludes the other, so interleaved acceptance or restoration (the failure the rule exists to prevent) passes the literal text.
- "Invalidating change" is undefined (bindings? authority files? any writer's output?), so any route can claim the no-repeat clause.
- Remediation: say that every route able to write or restore an output a protected effect may change shares one serialization domain (D13). Define "invalidating change" as any change by any writer to the bindings, authority, relevant inputs or expected output state. Define "demonstrated" as an Annex A entry naming the interfering writers plus a conformance test.

**2. The two effect kinds are not total, so an unlisted effect can skip admission item 5 (C:374-391, 402-404).**
- Protected outputs include "version metadata", "the embedded runtime-support projection" and "the publication stages of the lifecycle record". Writes to them are named in neither kind. Pointer retreat or removal and marker creation are also unnamed.
- Evidence: `.github/workflows/parity-promote.yml` writes `manifest-version-metadata --status validated` in the same step as the pointer advance and marker retirement. `crates/xtask/src/manifest_version_metadata.rs:52-57` has `Validated` and `Supported`. `crates/xtask/src/support_matrix/derive.rs:325-329` derives `manifest_support` from version metadata.
- Consequence: an implementation can treat a `validated`/`supported` flip or a `published` stage commit as a reporting effect and skip item 5, while due depth obligations remain.
- Remediation: define an acceptance effect as "any protected effect that is not a reporting effect", or enumerate these writes.

**3. Replacing an enrolled result by a successor generation is undecided (C:344-345, 413-416, 395-404, 571).**
- C:415-416 says a version outside every depth enrollment that would replace a depth-enrolled pointer or row "touches that tuple". C:344-345 gives a later version "no publication authority".
- Evidence: `derive.rs:641-662` recomputes every version's `pointer_promotion` from current pointers. `docs/specs/unified-agent-api/support-matrix.md:245-246` shows predecessor rows reading `none` after a successor is promoted. So every V2 promotion rewrites V1's row and pointer.
- Consequence: V2's route holds no authority over V1's tuple (item 2). V2's P/O/E are not V1's (item 3), and V2's code changes stale V1's evidence (C:253-257, item 5). Literal reading refuses every later promotion for that (agent, target). The alternative is an implementer carve-out ("successor may replace predecessor"), which C:417 forbids.
- This follows from D12 (no successor rule) plus rule 1. "Result for a depth scope tuple" (C:72-73, 374-375) is never defined, so a posture-field change cannot be classified.
- Remediation: define "result for a depth scope tuple", and add one rule for how replacing an enrolled pointer or row by a different generation is admitted (or that it needs explicit supersession).

**4. Reporting effects are gated by items 1-3, against the accepted "truthful pending or failed ... must stay publishable" refinement (C:395-404, 285-286, 555).**
- Only item 5 is limited to acceptance effects. Item 2 requires authority "under the lifecycle path that owns its depth enrollment".
- The research expects direct commands to report. `docs/agents/lifecycle/support-depth-policy-research-decisions.md:418` says "Allow truthful partial operator reporting without upgrading unenrolled/unverified/debt to verified or advancing acceptance effects". Proposal L159 says "partial operator publication remain possible before promotion".
- Consequence: a direct `support-matrix` or `capability-matrix` run holds no path authority, so it must refuse even a truthful pending row. A tuple whose enrollment no longer resolves, or whose bindings are inconsistent (items 1, 3), cannot have its truthful state written to any protected output. The earlier "qualified" row stays until hand-edited, which C:426-427 puts outside mediation. The sixth outcome, "Unresolved" (C:555), is unreportable through protected outputs.
- Remediation: state which predicate items apply to reporting effects, give operator-run reporting a defined authority or an explicit refusal, and say how an unresolved tuple is reported.

**5. "Validated reuse binding" is used but never defined (C:229, 253-257, 297-298, 576).**
- C:229 rejects verification when "the bindings differ and no validated reuse binding covers the difference". The table says a P or O change invalidates E wholesale, while C:253-257 says "An unrelated change invalidates nothing".
- The only bridge is the undefined term. Its definition lives in non-normative text (research-decisions L199, proposal L109: unchanged requirements and trust rules, exact version/target or approved compatibility rule, matching dependencies, per obligation), and C:31-33 disclaims that text.
- Consequence: implementers either rerun everything after every re-freeze or invent reuse criteria. C:576's "re-bound historical result" cannot be tested.
- Remediation: carry the definition into the contract, or delete the term and say re-freeze invalidates all evidence.

**6. The "does not change" table misattributes owners (criterion 6; C:44, 46, 48).**
- Runtime profiles: the charter has no "runtime profile", "profile" or "minimum integration". Its only hit is "support tier" at L166. `RuntimeProfile` lives in `crates/xtask/src/agent_lifecycle.rs:145` and the non-normative operator guide (L381-382).
- The `enrolled` stage (C:68-69) is likewise only in code (`agent_lifecycle.rs:94`). The charter and `capabilities-schema-spec.md:86-87` name other stages.
- Separate promotion: `maintenance-request-contract-v1.md` contains no "promot". The charter says only "Promotion remains maintainer-gated for every agent, at every tier." (L280). "Promotion is outside the maintenance contract by design" is in the Draft lifecycle spec (L38), which is context only.
- C:46's effect cell reads "None. Depth admission is added to them."
- Consequence: the single-owner rule (C:48) is false for these surfaces.
- Remediation: cite the real owner or drop the row, describe row 8 as "maintainer-gated promotion", and rephrase C:46.

**7. The accepted refinement "who authorizes it" is missing for O (C:298).**
- The O row reads "Acquisition at its freeze; afterwards only explicit re-freeze". P names its authorizer ("Maintainer approval", C:297) and the debt table has a Who column (C:330-333). O's re-freeze authorizer is not named.
- Rule 2 (C:305-306) only bars E from rewriting O. Proposal L122 requires an "explicit acquisition re-freeze".
- Remediation: name who may re-freeze O.

**8. Classification rule 6 contradicts itself (C:154-155).**
- The text is "`EXCLUDED` applies to one mode. It MUST NOT cover a command that also has a headless mode."
- The research's own EXCLUDED example, bare `claude` (research-decisions L104: "by mode only"), belongs to a command that also has `--print` headless modes. Proposal L77 intends only "whole subtree" exclusion to be barred. Literal reading forbids the example.
- Remediation: "It MUST NOT extend to a headless mode of the same command or to the command as a whole."

## P3 observations (non-blocking)

- **Collisions with existing repository terms.**
  - "Admission" is already the `close-agent-maintenance` stand-down-marker refusal (`crates/xtask/src/agent_maintenance/closeout.rs:155`, lifecycle spec L777).
  - "Obligation" is the request contract's "remaining obligation `G - A`" (request contract L213-214, 254-257). The debt table (C:331, 333) mixes the two meanings; "obligation coverage" has no field in the closed debt row shape (debt spec L15-31, request contract L246-247).
  - "Template" collides with `TemplateId`, `primary_template` and `template_lineage` (`agent_lifecycle.rs:153, 294-295`) and with prompt templates.
  - C:68-69's list of existing "enrolled" meanings omits "enrolled for publication" (`runtime-support-contract.md:222`) and "enrolled agent registry entry" (request contract L77).
- **Internal collisions.**
  - Dimension id `O` (C:102) versus binding **O** (C:298).
  - "Candidate" in C:400 versus C:512.
  - "Selector" in two senses (C:142-148 versus C:297, 360-362), with different outcomes: rule 4 says record it, rule 8 says error.
  - Unqualified "enrollment" at C:89, 263, 342, 344, 425, 544, 555, 569 despite C:68.
  - Lowercase "may" in a rule at C:312.
- **Generation and Event versus nightly regeneration (C:65-66, 296).**
  - `.github/workflows/agent-maintenance-open-pr.yml:162-163` regenerates the request with a fresh `request_recorded_at` and `github.sha` each dispatch until a marker exists.
  - Either each regeneration is a new generation (needing a predecessor reference the request has no field for) or the Event changes "inside a generation".
  - C:296 names only maintenance fields; the onboarding analogue is `approval_commit`/`approval_recorded_at` (`approval_artifact.rs:246-249`).
- **Lanes (C:314-320).** It is undecided what O is for the docs-only lane or a stand-down that skips the second freeze, yet rule 8 demands all four bindings.
- **Binding rule 9 (C:321-324).** Existing "supersession" (`agent-maintenance-open-pr.yml:273-356`) closes older-version PRs. The workflow refuses to regenerate or replace a stood-down packet (L133), so no existing re-preparation path exists for that case.
- **Final admission leftovers (C:447-448, 456, 425).**
  - Restoration is a protected effect that must pass admission, yet an interrupted operation must not read as accepted. It is unstated which wins when restoration admission fails.
  - "Reads as a completed acceptance" is undefined.
  - C:425 addresses "A caller" although only a route computes the touched set.
- **Evidence (C:230, 254-256).**
  - "Transitive build dependencies" collides with Cargo `[build-dependencies]`; D5 means the full resolved build graph.
  - "Producer is unsupported" has no list.
  - Research-decisions L197 ("do not select a favorable run silently") is dropped.
- **Overrides (C:156-158).** Research-decisions L248 ("cannot erase an existing promise or evade approved debt policy"; expiry condition) is omitted. As written an override may replace `M` or BASE obligations.
- **"The shared evaluator" (C:513).** It is never established. Proposal L143's one-evaluator-for-both-lifecycles rule is not carried.
- **Debt table (C:330).** "The same blocker" versus debt spec L55 "`blocker_class` still holds". "Frozen delegation" and "initial authorization baseline" are undefined. Proposal L247 ("name-level historical debt cannot automatically defer every new semantic requirement") is not stated.
- **Conformance table (C:557-576).**
  - It has diverged from the rules: EOF (C:567; CORE row C:128 lacks the research's "EOF is not proof of success"), "Enrolling an existing operation requires its full proof" (C:569), "Reprepare" (C:576) and "partial acceptance" (C:570) have no source rule. It has no rows for capability qualification (C:179-184), Path enablement or Additive integration.
  - Safe simplification: reduce it to C:559's sentence.
- **Hand-edit backstop (C:426-429).** "Lacks its bindings" cannot apply to pointer, marker or version-metadata files. Research-decisions L420 also asks to detect output from unsupported tooling.
- **Scope-of-refusal and wording items.**
  - C:503-504 is not limited to the enabled bounded scope, so it could block unenrolled agents.
  - C:501-502 says "both paths are enabled"; proposal L225 and research-decisions L403 say "proven and landed" (stricter than the research; confirm intent).
  - C:94 "R1 through X4" includes `O` and `L`, which research-decisions L141-151 keeps out of BASE.
  - C:112 says "Each applicable obligation ... `not_applicable`".
  - C:357-359 "approved request": maintenance requests are frozen, not approved.
  - Template cells have no obligation ids or granularity (C:60 versus C:126-138).
- **MCP template (C:136).** It cites a Draft spec (`mcp-management-spec.md:3`) whose isolated-home rule is SHOULD (L362). Scope and Purpose (C:5, 22-28) omit bindings, publication and additive integration. C:43 row 5 lists the capabilities spec for "minimum semantics", but C:173-174 correctly points to the capability's owner document. How EXCLUDED relates to `RULES.json` `parity_exclusions` is unstated (risk of a second exclusion ledger; research §4).

## Safety rules (criterion 3)

Complete mediation covers every failure the brief names: version-keyed bypass (C:413-418), wrong-path authority (C:418-422), omitted arguments (C:425) and lower-level writers (C:423-424). Its residual gaps are findings 2, 3 and 4.

Final admission names stale admission, changed authority or inputs, and snapshot restoration (C:437-448). Its residual gap is finding 1.

## Confirmed

- **Decisions.** D1-D7, D9, D11-D13 and the refinements (complete mediation, final freshness, effects-not-version keying, truthful reporting, pointer admission) are encoded. D14 is left to the support-matrix contract (C:192). D8 and D10 are absent, as required.
- **Existing-behavior claims verified.** Coverage level names (generator contract L51-61). Name identity (request contract L229-243). Renewable debt fields (debt spec L54-56 and `contract_policy.rs:234-240`). Freeze points (request contract L59-68). Neutral intake categories (`support-matrix.md:168-175`). CI association (`closeout/evidence.rs:34-36`).
- **Table rows 1-5 and 7** match their owners. No rule contradicts a controlling spec beyond the attributions in finding 6.

Reviewed file: `~/__Active_Code/atomize-hq/unified-agent-api/.claude/worktrees/agent-a22751ed5dc7079f5/docs/specs/support-depth-contract.md`
