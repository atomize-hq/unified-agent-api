use super::*;

#[test]
fn onboarding_producer_refuses_descendant_lifecycle_rollback_in_either_api_order() {
    let (mut f, _) = onboarding();
    let frozen = git(
        f.root.path(),
        &["show", &format!("on2-freeze:{}", inputs::STATE)],
    );
    write(f.root.path(), inputs::STATE, frozen);
    let w = f.w.clone();
    later(
        &mut f,
        2,
        "push",
        "restore enrolled lifecycle after implementation",
        &w,
    );
    let rollback = git(f.root.path(), &["rev-parse", "HEAD"]);
    let host = EvidenceHost::for_test(ISOLATED, &rollback);
    f.api
        .put("git/ref/heads/staging", json!({"object":{"sha":rollback}}));
    // Both sources are individually valid, and only lifecycle bytes changed. The producer
    // closure and immutable execution/adoption graph therefore cannot detect this rollback.
    profile::approved(f.root.path(), ISOLATED, &rollback, &rollback, &f.commit, P).unwrap();
    for commit in [&f.commit, &rollback] {
        let at = source(f.root.path(), ISOLATED, commit).unwrap();
        profile::original_for(&at, P).unwrap();
    }
    for reverse in [false, true] {
        if reverse {
            f.api.change(RUNS, |v| {
                v["workflow_runs"].as_array_mut().unwrap().reverse()
            });
        }
        let error = resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).unwrap_err();
        assert!(error.contains("lifecycle progression"), "{error}");
    }
}

#[test]
fn onboarding_genuine_freeze_attempt_remains_applicable_and_blocks_later_proof() {
    let (mut f, host) = onboarding();
    let freeze = git(f.root.path(), &["rev-parse", "on2-freeze"]);
    let original = P
        .record(&fs::read_to_string(f.root.path().join(inputs::RECORD)).unwrap())
        .unwrap();
    let at = source(f.root.path(), ISOLATED, &freeze).unwrap();
    assert_eq!(profile::original_for(&at, P).unwrap(), original);
    // Progression itself must accept the genuine historical freeze in either discovery
    // order, including retries observed after implementation. Closure approval is separate.
    for commits in [[&freeze, &f.commit], [&f.commit, &freeze]] {
        let mut progression = progression::Onboarding::default();
        for commit in commits.into_iter().chain([&freeze]) {
            progression
                .observe(
                    f.root.path(),
                    commit,
                    &source(f.root.path(), ISOLATED, commit).unwrap(),
                    P,
                    &original,
                )
                .unwrap();
        }
    }
    // No fabricated witness for the pre-implementation source. Matching Event/P/O suffices
    // for applicability, and approval refuses before any attempt/artifact can be inspected.
    f.api.change(RUNS, |v| {
        let mut run = v["workflow_runs"][0].clone();
        run["id"] = json!(2);
        run["head_sha"] = json!(freeze);
        v["workflow_runs"].as_array_mut().unwrap().insert(0, run);
        v["total_count"] = json!(2);
    });
    for attempts in [1, 2] {
        f.api.change(RUNS, |v| {
            v["workflow_runs"][0]["run_attempt"] = json!(attempts)
        });
        let error = resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).unwrap_err();
        assert!(
            error.contains("producer closure is not approved on protected staging: crates"),
            "{error}"
        );
    }
}
