//! T4 — closeout evidence resolution.
//!
//! These exercise the *selection policy*, not the transport. The policy is the load-bearing part:
//! a resolver that ranks candidate runs can be steered by re-running until the preferred answer
//! exists, and a resolver that guesses on absent evidence produces an artifact that reads as
//! verified. Every case here is a frozen API response, so a change in the policy shows up as a
//! changed verdict rather than a changed network call.

use serde_json::json;

use crate::closeout::{resolve_preflight_with_fetcher, MaintenanceCloseoutError};

const COMMIT: &str = "f51e0b62a1c34d5e6f7a8b9c0d1e2f3a4b5c6d7e";
const CI_PATH: &str = ".github/workflows/ci.yml";

fn run(id: u64, attempt: u64, status: &str, conclusion: Option<&str>) -> serde_json::Value {
    json!({
        "id": id,
        "path": CI_PATH,
        "head_sha": COMMIT,
        "status": status,
        "conclusion": conclusion,
        "run_attempt": attempt,
    })
}

fn page(runs: Vec<serde_json::Value>) -> String {
    json!({ "workflow_runs": runs }).to_string()
}

/// One page of `runs`, then an empty page if the first was full.
fn single_page(
    runs: Vec<serde_json::Value>,
) -> impl FnMut(&str) -> Result<String, MaintenanceCloseoutError> {
    let body = page(runs);
    move |_url: &str| Ok(body.clone())
}

fn expect_refusal(
    result: Result<crate::closeout::ResolvedPreflight, MaintenanceCloseoutError>,
    what: &str,
) -> String {
    match result {
        Ok(resolved) => panic!("{what} must refuse, resolved {resolved:?}"),
        Err(err) => err.to_string(),
    }
}

#[test]
fn a_unanimous_success_resolves_to_passed() {
    let resolved = resolve_preflight_with_fetcher(
        COMMIT,
        single_page(vec![run(1, 1, "completed", Some("success"))]),
    )
    .expect("a completed successful run resolves");
    assert!(resolved.passed);
    assert_eq!(resolved.commit, COMMIT);
    assert_eq!(resolved.runs.len(), 1);
    assert_eq!(resolved.runs[0].conclusion, "success");
}

/// A known failure is a resolved fact, not a failure to resolve. T4 records it; it does not gate.
#[test]
fn a_unanimous_failure_resolves_to_not_passed_rather_than_refusing() {
    let resolved = resolve_preflight_with_fetcher(
        COMMIT,
        single_page(vec![run(1, 1, "completed", Some("failure"))]),
    )
    .expect("a completed failing run resolves");
    assert!(!resolved.passed);
    assert_eq!(resolved.runs[0].conclusion, "failure");
}

#[test]
fn no_associated_run_refuses_and_names_the_merge_commit_case() {
    let message = expect_refusal(
        resolve_preflight_with_fetcher(COMMIT, single_page(vec![])),
        "an unobserved commit",
    );
    assert!(
        message.contains("no .github/workflows/ci.yml run is associated"),
        "{message}"
    );
    assert!(message.contains("merge commit"), "{message}");
}

/// Evidence that is still moving is not evidence.
#[test]
fn a_non_terminal_run_refuses_rather_than_reading_the_finished_ones() {
    let message = expect_refusal(
        resolve_preflight_with_fetcher(
            COMMIT,
            single_page(vec![
                run(1, 1, "completed", Some("success")),
                run(2, 1, "in_progress", None),
            ]),
        ),
        "an in-flight run",
    );
    assert!(message.contains("is not final"), "{message}");
    assert!(message.contains("run 2 is `in_progress`"), "{message}");
}

/// The rule that would resolve this — "prefer the newest" — is the rule that can be steered by
/// re-running until the preferred answer exists.
#[test]
fn distinct_runs_that_disagree_refuse_and_name_every_candidate() {
    let message = expect_refusal(
        resolve_preflight_with_fetcher(
            COMMIT,
            single_page(vec![
                run(1, 1, "completed", Some("success")),
                run(2, 1, "completed", Some("failure")),
            ]),
        ),
        "disagreeing runs",
    );
    assert!(message.contains("disagrees"), "{message}");
    assert!(
        message.contains("run 1 attempt 1 concluded `success`"),
        "{message}"
    );
    assert!(
        message.contains("run 2 attempt 1 concluded `failure`"),
        "{message}"
    );
    assert!(
        message.contains("not resolved by preferring one"),
        "{message}"
    );
}

/// Re-run attempts of one run are that run's history, not separate evidence: the highest attempt is
/// its current conclusion. This is the one place ranking is correct, and it is scoped to a run id.
#[test]
fn rerun_attempts_of_one_run_collapse_to_the_highest_attempt() {
    let resolved = resolve_preflight_with_fetcher(
        COMMIT,
        single_page(vec![
            run(1, 1, "completed", Some("failure")),
            run(1, 2, "completed", Some("success")),
        ]),
    )
    .expect("one run's attempts collapse");
    assert!(resolved.passed);
    assert_eq!(resolved.runs.len(), 1);
    assert_eq!(resolved.runs[0].run_attempt, 2);
}

#[test]
fn attempts_arriving_out_of_order_still_collapse_to_the_highest() {
    let resolved = resolve_preflight_with_fetcher(
        COMMIT,
        single_page(vec![
            run(1, 3, "completed", Some("success")),
            run(1, 2, "completed", Some("failure")),
        ]),
    )
    .expect("attempt order does not matter");
    assert!(resolved.passed);
    assert_eq!(resolved.runs[0].run_attempt, 3);
}

#[test]
fn another_workflow_at_the_same_commit_is_not_preflight_evidence() {
    let mut other = run(9, 1, "completed", Some("failure"));
    other["path"] = json!(".github/workflows/agent-maintenance-release-watch.yml");
    let resolved = resolve_preflight_with_fetcher(
        COMMIT,
        single_page(vec![run(1, 1, "completed", Some("success")), other]),
    )
    .expect("only ci.yml counts");
    assert!(resolved.passed);
    assert_eq!(resolved.runs.len(), 1);
}

/// The query filters server-side. A filter that silently stops filtering is indistinguishable from
/// one that matched, so the field is re-checked here.
#[test]
fn a_run_for_another_commit_is_ignored_even_though_the_query_filtered() {
    let mut foreign = run(9, 1, "completed", Some("failure"));
    foreign["head_sha"] = json!("0123456789abcdef0123456789abcdef01234567");
    let resolved = resolve_preflight_with_fetcher(
        COMMIT,
        single_page(vec![run(1, 1, "completed", Some("success")), foreign]),
    )
    .expect("head_sha is re-checked");
    assert!(resolved.passed);
    assert_eq!(resolved.runs.len(), 1);
}

/// `cancelled` is terminal but establishes neither outcome. Recording it as a failure would report
/// a verdict CI never reached.
#[test]
fn a_terminal_but_inconclusive_run_refuses_rather_than_recording_a_failure() {
    for conclusion in [
        "cancelled",
        "skipped",
        "neutral",
        "stale",
        "action_required",
    ] {
        let message = expect_refusal(
            resolve_preflight_with_fetcher(
                COMMIT,
                single_page(vec![run(1, 1, "completed", Some(conclusion))]),
            ),
            conclusion,
        );
        assert!(message.contains("inconclusive"), "{conclusion}: {message}");
        assert!(
            message.contains("establishes neither a pass nor a failure"),
            "{conclusion}: {message}"
        );
    }
}

#[test]
fn a_completed_run_with_no_conclusion_refuses() {
    let message = expect_refusal(
        resolve_preflight_with_fetcher(COMMIT, single_page(vec![run(1, 1, "completed", None)])),
        "a completed run with no conclusion",
    );
    assert!(message.contains("no conclusion"), "{message}");
}

/// A full page means there may be more. Stopping at the first page would let a disagreeing run on
/// page two go unseen — which is the "search until favourable" failure with extra steps.
#[test]
fn every_page_is_read_and_a_later_page_can_still_refuse() {
    let mut first: Vec<serde_json::Value> = (0..100)
        .map(|i| run(i, 1, "completed", Some("success")))
        .collect();
    first[0] = run(0, 1, "completed", Some("success"));
    let second = vec![run(500, 1, "completed", Some("failure"))];
    let pages = [page(first), page(second)];
    let mut seen = 0usize;

    let message = expect_refusal(
        resolve_preflight_with_fetcher(COMMIT, |_url| {
            let body = pages[seen].clone();
            seen += 1;
            Ok(body)
        }),
        "a disagreement on the second page",
    );
    assert!(message.contains("disagrees"), "{message}");
    assert_eq!(seen, 2, "both pages must be fetched");
}

#[test]
fn a_short_first_page_does_not_fetch_a_second() {
    let mut calls = 0usize;
    let body = page(vec![run(1, 1, "completed", Some("success"))]);
    resolve_preflight_with_fetcher(COMMIT, |_url| {
        calls += 1;
        Ok(body.clone())
    })
    .expect("resolves");
    assert_eq!(calls, 1);
}

#[test]
fn the_canonical_repository_is_queried_not_the_local_remote() {
    let body = page(vec![run(1, 1, "completed", Some("success"))]);
    let mut urls = Vec::new();
    resolve_preflight_with_fetcher(COMMIT, |url| {
        urls.push(url.to_string());
        Ok(body.clone())
    })
    .expect("resolves");
    assert_eq!(urls.len(), 1);
    assert!(
        urls[0]
            .starts_with("https://api.github.com/repos/atomize-hq/unified-agent-api/actions/runs"),
        "{}",
        urls[0]
    );
    assert!(
        urls[0].contains(&format!("head_sha={COMMIT}")),
        "{}",
        urls[0]
    );
}

#[test]
fn a_malformed_commit_refuses_before_any_fetch() {
    let mut calls = 0usize;
    let message = expect_refusal(
        resolve_preflight_with_fetcher("not-a-sha", |_url| {
            calls += 1;
            Ok(page(vec![]))
        }),
        "a malformed commit",
    );
    assert!(message.contains("7-40 lowercase hex"), "{message}");
    assert_eq!(calls, 0, "a malformed commit must not reach the network");
}

#[test]
fn a_transport_failure_refuses_rather_than_resolving_empty() {
    let message = expect_refusal(
        resolve_preflight_with_fetcher(COMMIT, |_url| {
            Err(MaintenanceCloseoutError::Validation(
                "curl failed".to_string(),
            ))
        }),
        "a transport failure",
    );
    assert!(message.contains("curl failed"), "{message}");
}

#[test]
fn an_unparseable_body_refuses() {
    let message = expect_refusal(
        resolve_preflight_with_fetcher(COMMIT, |_url| Ok("<html>rate limited</html>".to_string())),
        "an unparseable body",
    );
    assert!(message.contains("parse workflow runs"), "{message}");
}

/// A merge commit carries zero check runs because `ci.yml` triggers on `pull_request`. Without this
/// the refusal would be "no run is associated", which describes a commit CI never saw and a commit
/// CI *cannot* see identically.
#[test]
fn a_merge_commit_is_refused_by_parent_count_with_the_reason_named() {
    let fixture = crate::harness::fixture_root("t4-merge-commit");
    let head = crate::closeout_harness::init_git_fixture(&fixture);

    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .current_dir(&fixture)
            .args(args)
            .output()
            .expect("git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout)
            .expect("utf-8")
            .trim()
            .to_string()
    };
    git(&["switch", "--quiet", "-c", "side"]);
    git(&["commit", "--quiet", "--allow-empty", "-m", "side"]);
    git(&["switch", "--quiet", "main"]);
    git(&["commit", "--quiet", "--allow-empty", "-m", "main"]);
    git(&["merge", "--quiet", "--no-ff", "-m", "merge", "side"]);
    let merge = git(&["rev-parse", "HEAD"]);

    let message = match crate::closeout::reject_merge_commit(&fixture, &merge) {
        Ok(()) => panic!("a merge commit must be refused"),
        Err(err) => err.to_string(),
    };
    assert!(
        message.contains("is a merge commit (2 parents)"),
        "{message}"
    );
    assert!(message.contains("packet-branch head"), "{message}");

    crate::closeout::reject_merge_commit(&fixture, &head)
        .expect("a single-parent commit is not refused");
}

/// The live read-only proof (§10.5), run 2026-09-23 against the real API.
///
/// A resolver whose only caller is its own tests has never met the real response: nullable fields,
/// fields the parser does not name, and the `total_count`/`workflow_runs` envelope are all places a
/// hand-written fixture agrees with itself. These two bodies are the verbatim responses for the
/// heads of two live packets, frozen so a policy change shows up as a changed verdict here.
///
/// | packet | PR | head | selected | conclusion |
/// |---|---|---|---|---|
/// | claude_code 2.1.267 | #211 | `552f3928` | run 35838332532 attempt 1 | `success` |
/// | opencode 1.18.31 | #223 | `86df54af` | run 35838395186 attempt 1 | `success` |
///
/// Read-only: no stand-down marker is needed to resolve evidence, only to write a closeout.
#[test]
fn the_policy_resolves_two_live_packet_heads_from_their_frozen_responses() {
    let cases = [
        (
            "552f39282647671bb1652808a9bf75520bf29fba",
            include_str!("../fixtures/t4/workflow-runs-claude-code-2.1.267-head.json"),
            35838332532u64,
        ),
        (
            "86df54afa75c3fe9467ca4371c527a186d4faf60",
            include_str!("../fixtures/t4/workflow-runs-opencode-1.18.31-head.json"),
            35838395186u64,
        ),
    ];

    for (head, body, expected_run) in cases {
        let resolved = resolve_preflight_with_fetcher(head, |_url| Ok(body.to_string()))
            .unwrap_or_else(|err| panic!("{head} must resolve: {err}"));
        assert!(resolved.passed, "{head}");
        assert_eq!(resolved.commit, head);
        assert_eq!(resolved.runs.len(), 1, "{head}: {:?}", resolved.runs);
        assert_eq!(resolved.runs[0].run_id, expected_run, "{head}");
        assert_eq!(resolved.runs[0].run_attempt, 1, "{head}");
        assert_eq!(resolved.runs[0].conclusion, "success", "{head}");
    }
}

/// The same frozen response resolved against a different commit must find nothing, not reuse the
/// runs it happens to contain. This is what would break if the `head_sha` re-check were dropped on
/// the grounds that the query already filtered.
#[test]
fn a_frozen_response_does_not_resolve_for_a_commit_it_does_not_describe() {
    let body = include_str!("../fixtures/t4/workflow-runs-claude-code-2.1.267-head.json");
    let message = expect_refusal(
        resolve_preflight_with_fetcher("86df54afa75c3fe9467ca4371c527a186d4faf60", |_url| {
            Ok(body.to_string())
        }),
        "a response for another commit",
    );
    assert!(
        message.contains("no .github/workflows/ci.yml run is associated"),
        "{message}"
    );
}
