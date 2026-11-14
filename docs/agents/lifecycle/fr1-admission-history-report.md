# FR1 admission history implementation report

Status: FR1 code and docs delivered in `feat/fr1-admission-history`; changes intentionally
uncommitted. All required groups and the final other-PR-only proof have completed. Existing
sandbox restrictions prevent a fully green validation gate. No live profile, production declaration, network/credentials, commit
or push was introduced. Compilation uses the default TMPDIR without an override.

Original scope: the FR1 sub-slice in `support-depth-freeze-readiness-design.md`.

## Files changed

- New `crates/xtask/src/depth_acceptance/admission/`: `record.rs`, `policy.rs`, `objects.rs`,
  `transport.rs`, `history.rs`, `emitter.rs`, `mod.rs`, fake host `fake.rs`, `tests.rs` and
  `tests/policy.rs`.
- IN1 integration: `depth_acceptance/{mod.rs,integration.rs,integration/generation.rs}`.
- Evidence integration: `depth_evidence.rs`, `depth_evidence/verdict.rs`,
  `depth_evidence/verdict/github/{mod.rs,population.rs,tests.rs,tests/onboarding.rs}` and new
  `tests/onboarding/admissions.rs`.
- Isolated route: new `isolated_maintenance/admission.rs`, `host.rs`, `mod.rs`; test wiring in
  `tests.rs`, `tests/fixture.rs`, `tests/in1_integration.rs`,
  `tests/in1_integration/regressions.rs`, new `tests/in1_integration/admission.rs`, and
  `tests/bootstrap/{public_on1.rs,public_on2.rs}`.
- Transport: `scripts/depth_ci_archive.py`, `scripts/test_depth_ci_archive.py`.
- Docs: this report, `support-depth-freeze-readiness-design.md`,
  `support-depth-onboarding-producer-freeze-question.md`, and
  `docs/specs/{support-depth-contract.md,support-depth-evidence-v1.md}` (including Annex A).

Paths in the first four bullets are relative to `crates/xtask/src/`; design docs are under
`docs/agents/lifecycle/`. Removed the inherited untracked `crates/xtask/build.rs` and restored
`crates/xtask/Cargo.toml`: there is no build script, new dependency or manifest/lockfile diff.

## Record schema

Independent schema version 1, kind `in1_admission`. Typed, unknown-field-rejecting canonical JSON
uses deterministic ordered members, exact round-trip byte equality, a maximum of 65,536 bytes,
and SHA-256 over `support-depth-in1-admission-v1` + NUL + canonical bytes.

| Fields | Meaning |
| --- | --- |
| `repository` | Canonical name, repository id and owner id |
| `generation`, `generation_sha256` | Original Event/P/O/enrollment identity; nullable for pre-generation bootstrap |
| `candidate`, `parent`, `checkout`, `head_relationship` | Commit/tree objects and exact-head evaluated context |
| `old_state`, `new_state` | Lifecycle path/blob/stage; complete state remains content addressed |
| `checkpoint`, `decision`, `reasons` | Bootstrap or unclassified; admitted/refused; bounded reason enums |
| `predecessors`, `enrollment_anchor` | Sorted unique successful parent-frontier decisions and independent first-record anchor |
| `policy` | Approved implementation/workflow/closure and fixed workflow/job/app identities |
| `protected_tip`, `canonical_tip`, `accepted_origin`, `registry_blob`, `inputs` | Observed roots, accepted repository and authoritative input blobs |
| `producer_context`, `execution_evidence` | Nullable context and consumed evidence identities; producer absence is representable |
| `claims` | Run, attempt, emitting job, check and event |

Generation and evaluated-input identities have their own digest domains. Input identity includes
root snapshots and excludes retry association and outcome, allowing conflict detection without
conflating distinct observations. Refusals retain bounded reason enums, not arbitrary error text.

## Runtime policy identity and approval

The source-installed isolated profile supplies the authenticated protected installation commit
(`Profile.baseline`), independently of candidate JSON, local working files and producer discovery.
That evaluated policy source is pinned separately from subsequent candidate and protected-tip
observations. Runtime Git reads resolve its commit/tree/blob identities and verify object hashes.

The fixed policy closure includes `crates`, `Cargo.toml`, `Cargo.lock`, `.github`, the exact
`.github/workflows/ci.yml`, `scripts`, the exact `scripts/depth_ci_archive.py`, plus installed
profile scopes. Whole tree ids bind additions/deletions and nested modes, not just a chosen file
list. Root Cargo/toolchain configuration overrides remain unsupported as in producer approval.
`implementation` summarizes canonical closure JSON with domain
`support-depth-in1-git-policy-v1` + NUL; the map itself records Git object ids.

The evaluator reconstructs the approved closure from the independent protected source and
compares evaluated source tree/blob objects with that anchor, using the same object-equality
rule as producer closure approval. It refuses a copied approved map over changed source, a valid
recomputed candidate map/digest, omitted closure members, unsupported configuration or a host
record claiming another policy. Candidate-supplied identities never approve themselves. Source
installation also pins workflow/app/job provenance. The fake policy tree is explicitly modeled;
phase C must establish the deployed executable/source/runtime binding and real protected-host
provenance. FR1 claims no compiled-binary attestation or production qualification.

## Archive reader decision

Kept Python `in1` as the existing proof transport's ZIP reader. It returns bytes from exactly one
regular, unencrypted `admission.json`; input archive bound is 2 MiB, declared and streamed expanded
payload bounds are 64 KiB. No extraction or Git lookup occurs. Rust owns canonical JSON, typed
schema, record digest, host association and policy/history authentication. This preserves the
existing transport boundary without adding a Rust ZIP dependency. Eight Python tests include
both existing witness/registry cases and new IN1 boundary, duplicate, traversal, extra-entry,
nonregular and encrypted-entry refusals.

## Proving-test audit

All listed FR1 tests pass in the completed depth-acceptance group or the connected ON2 proof.
The corrected bootstrap group also exercises producer-discovery refusal after the admission
population becomes unavailable, even with retained payload bytes and advanced tips. All 59
bootstrap tests pass. The final focused connected proof also passes with only the other PR's run
present, so refusal on the original PR cannot mask a missing cross-PR check. Runtime references
below are measured containing suites, not invented individual timings: A = 62 depth-acceptance tests, 41.79 s (49.93 s wall including
compile); C = final connected ON2 test, 398.78 s (407.19 s wall including compile).
The earlier connected run took 132.87 s (142.67 s wall including compile/lock wait).
The focused 17-test admission group before the final three adversarial additions took 7.95 s.

Names without a prefix below are under `depth_acceptance::admission::tests::`.

| Required item | Executable coverage | Measured runtime |
| --- | --- | --- |
| Actual IN1 decision; first-record anchor; no producer context | `authenticates_actual_in1_and_first_record_enrollment_anchor` uses `admission_fixture`, which calls real `Route::with_admitted_publication`; bad anchor tree refuses | A |
| Canonical schema, versions, unknown fields, domains, 64 KiB | `canonical_record_roundtrip_bound_versions_unknown_fields_and_domains` | A |
| Candidate JSON, same-name check, missing publication | `refuses_candidate_json_same_name_wrong_job_attempt_and_missing_publication`; an empty decision stream confers no eligibility | A |
| Same-run wrong-job artifact despite valid digest and copied commitment | `policy::same_run_wrong_job_artifact_with_valid_digest_and_copied_commitment_refuses` | A |
| Wrong attempt and every retry separately authenticated | `every_retry_is_authenticated_and_wrong_attempt_commitment_refuses` | A |
| Missing predecessors; input/commit/tree mismatches; first decision without bootstrap | `missing_predecessors_object_mismatches_and_history_gaps_refuse` | A |
| Missing parent branch and raw object corruption | `verifies_raw_object_hashes_and_traverses_second_parent_to_the_anchor` | A |
| 1,000-run/stream pagination, 100-attempt overflow, job/artifact population overflow | `each_host_stream_and_control_bound_overflow_refuses`, `host_pagination_accepts_the_run_bound_and_refuses_gaps_duplicates_and_page_overflow` | A |
| 1,000 unique commits accepted, 1,001 refused without truncation | `unique_commit_traversal_accepts_1000_and_refuses_1001_without_truncation` | A |
| Archive/log/object bounds | `oversized_archives_logs_and_objects_refuse_without_truncation`; Python IN1 tests additionally exercise 64 KiB exact boundary | A; Python 0.018 s |
| Conflicting decisions; distinct roots and retries | `retries_are_distinct_conflicts_refuse_and_roots_distinguish_inputs` | A |
| Late retry cannot rewrite a child's predecessor | `late_parent_retry_does_not_rewrite_existing_child_predecessors` (both admission orders) | A |
| Revocation preserves historical authority | `revocation_preserves_immutable_history_and_changes_present_eligibility`; connected proof also revokes higher-stage authority and still refuses rollback | A, C |
| Changing roots/populations, including consumption recheck | `roots_and_populations_are_rechecked_after_history_and_at_consumption`; `concurrent_host_attempt_registration_preserves_the_complete_population` | A |
| Complete runtime Git policy identity; forged source/map/digest; overrides | `policy_identity_covers_git_policy_and_dependency_closure`, `policy::runtime_git_policy_authentication_rejects_self_selected_closure_and_source`, `policy::runtime_policy_refuses_root_toolchain_configuration_even_with_matching_claims` | A |
| Publication required for success; incomplete attempts; retained refusal | `isolated_maintenance::tests::in1_integration::admission::{in1_publication_failure_refuses_success_and_retains_an_incomplete_attempt,in1_missing_job_commitment_leaves_an_uploaded_attempt_incomplete,in1_retains_reached_refusals_without_granting_admission}` | IN1: 37.92 s tests |
| Admitted higher-stage ancestor with no producer; both producer API orders; another PR | `isolated_maintenance::tests::bootstrap::public_on2::public_on2_connected_freeze_write_admission_and_observation` calls `depth_evidence::...::onboarding::admissions::prove_unobserved_admission`, which invokes actual producer discovery; the producer-only progression observations alone succeed, independently retained higher-stage IN1 authority causes refusal; a separate inventory containing only the other PR prevents the original PR from masking a missing cross-PR check | C; full bootstrap 240.79 s |
| Fake tip movement cannot manufacture admission | `fake_tip_advance_has_no_admission_issuance_side_effect`; connected proof hides the admission population with advanced tips and retained payloads intact and requires history-gap refusal, including from producer discovery | A, C; full bootstrap 240.79 s |
| Existing IN1/MA/ON behavior | Existing manual/closeout/promotion/integration tests pass in A; unchanged GitHub maintenance/onboarding evidence tests pass; runtime-follow-on 45 pass; IN1's socket-dependent cases are blocked; bootstrap 59 pass | See validation |

## Interfaces left for FR2/FR3

- `Record` reserves `request_freeze` / `execution_required` checkpoint enums. FR1 emits bootstrap
  or unclassified; it does not infer a reporting waiver or readiness transition.
- `producer_context` remains nullable; execution-evidence references are representable separately.
  FR3 must supply authenticated reporting context and joins to retained admission/history.
- `Publication` and `Host` expose attempt-specific begin/upload/commit and independent json/bytes
  reads. `History::{check_candidate,check_progression,candidate_predecessors,recheck,eligible}`
  expose authenticated progression, complete parent frontiers, freshness and present eligibility.
- FR2 must add pure writer-derived complete-tree planning and IN1 freeze equality. FR3 owns
  canonical MA2 request/report collection-discovery binding and population classification.

## Deviations and proof limits

No FR2/FR3 behavior was added. The connected unobserved-ancestor proof extends the existing real
ON2 test instead of duplicating expensive writer setup. Small record/host tests share one immutable
real-IN1 fixture and fork independent host facts; they do not depend on test order or thread-local
host state. Existing fixture-installed public-route tests retain their scoped installation guards.
The Python reader remains transport-only. Source policy identity uses runtime Git objects instead
of the inherited unreviewed compile-time hash. The first new closure-coverage test demonstrably
failed against inherited code (missing `crates`, 0.91 s test / 8.66 s wall).

Production `LIVE_PROFILE` remains `None`. Real transport publication, protected-policy app/source
provenance, emitter isolation, retention and deployed runtime qualification remain phase C. The
actual reporting exception, connected ON3–ON5 writers and source-stable publication remain outside
FR1. Historical admission does not grant current eligibility after revocation.

## Commands and results

Full captured logs and machine-readable wall timings are under ignored `target/fr1-validation/`.
`results.json` contains this continuation's runs. Isolated-group logs use the full group name with
`::` replaced by `_` (for example `isolated_maintenance_tests_in1_integration.txt`); older worker
logs with abbreviated names in that directory are not the results reported here.
Times use default TMPDIR; no sandbox restrictions were bypassed.

| Command | Result | Runtime |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | Final 3.22 s wall |
| `git diff --check` | Pass | Below 1 s |
| `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | Pass | Final 5.84 s wall (earlier 9.56 s / 13.91 s) |
| `cargo test -p xtask --lib depth_acceptance` | 62 passed | 41.79 s tests / 49.93 s wall |
| `cargo test -p xtask --lib depth_evidence` | 91 passed, 12 environment failures, 1 existing ignored subprocess entry | 77.16 s tests / 77.24 s wall |
| `cargo test -p xtask --lib runtime_follow_on` | 45 passed | 42.77 s tests / 42.86 s wall |
| `cargo test -p xtask --lib isolated_maintenance::tests::in1_integration` | 23 passed, 4 socket-restricted failures | 37.92 s tests / 37.99 s wall |
| `cargo test -p xtask --lib isolated_maintenance::tests::bootstrap` | Corrected run: 59 passed | 240.79 s tests / 247.88 s wall |
| Final focused connected ON2 test (`--exact`, only other PR also tested) | 1 passed | 398.78 s tests / 407.19 s wall |
| `python3 -m unittest discover -s scripts -p test_depth_ci_archive.py` | 8 passed | Final 0.013 s tests / 0.119 s wall |
| `make loc-check` | Pass, no Rust file above 700 code lines | Final 0.130 s wall |
| `make hygiene` | Pass | Final 0.112 s wall |

The evidence failures reproduce documented sandbox restrictions: ten staged-channel tests cannot
bind Unix sockets (`Operation not permitted`, including the forged-channel child that never binds);
two native local-guard tests fail `ForeignLoader` because explicitly `/tmp`-rooted fixtures see
injected `/tmp/.codex` and `/tmp/.agents` ancestor loaders. Those tests and guards remain unchanged.
The lead must rerun the full affected groups unrestricted; these results are not a fully green gate.

The four IN1 failures are the unchanged connected closeout cases
`in1_closeout_candidate_admission_and_post_push_observation`,
`in1_prepared_closeout_candidate_admission_and_post_push_observation`,
`in1_reauthenticates_population_in_admission_and_post_push_observation`, and
`in1_refuses_forged_closeout_entries_in_admission_and_observation`. Each fails while opening the
MA4 Unix-socket evidence channel, before its connected admission assertions. All three new
admission publication tests pass, as do the other existing IN1 tests (including the parent race).

Bootstrap's initial run exposed an incorrect new test assertion, not an implementation failure:
hiding the admission population preserves payload bytes in the fake artifact store. Discovery
correctly refused the gap, but the assertion incorrectly expected an empty payload store. The
assertion now requires the payload store to remain unchanged across discovery. Full bootstrap
rerun passed all 59 tests. Initial run took 129.13 s tests / 129.21 s wall; the rerun took
240.79 s tests / 247.88 s wall. The later standalone other-PR assertion passed in the focused
connected-test rerun (398.78 s tests / 407.19 s wall).

Final audit tightened the cross-PR proof: after both listing orders, producer discovery is also
called with only the other PR's run. This ensures a refusal on the original PR cannot mask missing
cross-PR progression. This adds no writer setup; the final connected proof passed after this
assertion was added.

Final outcome: all FR1 proving cases pass; all 59 bootstrap tests pass. Across the requested full
groups, 280 tests pass, 16 reproduce environment restrictions, and one pre-existing subprocess
entry remains intentionally ignored by the original suite. Required fmt, clippy, LOC and hygiene
checks pass. No assertions or tests were removed to hide those restrictions. The lead still needs
to rerun the affected evidence and IN1 groups unrestricted.

Runtime limit: small admission cases share immutable setup (the initial 17-test group took 7.95 s).
The connected proof reuses the existing real ON2/IN1 chain, but its final focused run took 398.78 s,
compared with 132.87 s earlier; the full bootstrap rerun took 240.79 s. This variability was not
profiled, and the final focused timing exceeds the earlier five-minute evidence-suite target.
Do not infer a stable small CI runtime from the earlier faster run. No extra writer fixture was
added for the other-PR assertion.
