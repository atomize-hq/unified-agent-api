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
    // CI never ran on the implementation ancestor. Its independent adoption stays present.
    f.api.change(RUNS, |v| {
        v["workflow_runs"]
            .as_array_mut()
            .unwrap()
            .retain(|run| run["id"] == 2);
        v["total_count"] = json!(1);
    });
    let error = resolve_profile_in(&f.api, f.root.path(), P, Some(&host))
        .expect_err("rollback authenticated without its implementation ancestor's CI run");
    assert!(error.contains("lifecycle progression"), "{error}");
    // Restoring the higher stage again cannot erase the intervening invalid transition.
    let implemented = git(
        f.root.path(),
        &["show", &format!("{}:{}", f.commit, inputs::STATE)],
    );
    write(f.root.path(), inputs::STATE, implemented);
    later(
        &mut f,
        3,
        "push",
        "restore implemented state after rollback",
        &w,
    );
    f.api.change(RUNS, |v| {
        v["workflow_runs"]
            .as_array_mut()
            .unwrap()
            .retain(|run| run["id"] == 3);
        v["total_count"] = json!(1);
    });
    let error = resolve_profile_in(&f.api, f.root.path(), P, Some(&host))
        .expect_err("later forward transition erased a historical rollback");
    assert!(error.contains("lifecycle progression"), "{error}");
}

fn merge(root: &Path, tree_source: &str, first: &str, second: &str) -> String {
    let tree = git(root, &["rev-parse", &format!("{tree_source}^{{tree}}")]);
    git(
        root,
        &[
            "commit-tree",
            &tree,
            "-p",
            first,
            "-p",
            second,
            "-m",
            "merge staging and PR branch",
        ],
    )
}

#[test]
fn onboarding_producer_checks_all_merge_parents_for_heads_and_test_checkouts() {
    let (mut f, host) = onboarding();
    let root = f.root.path();
    let before = git(root, &["rev-parse", "on2-freeze^"]);
    let tree = git(root, &["rev-parse", &format!("{before}^{{tree}}")]);
    let staging = git(
        root,
        &[
            "commit-tree",
            &tree,
            "-p",
            &before,
            "-m",
            "staging advances",
        ],
    );
    let integrated = f.commit.clone();
    let w = f.w.clone();
    // Ordinary staging history predates this freeze. The merge retains the PR's state,
    // regardless of which parent carries this generation (PR merge vs GitHub test merge).
    for (first, second) in [(&integrated, &staging), (&staging, &integrated)] {
        let valid = merge(root, &integrated, first, second);
        let mut progression = progression::Onboarding::default();
        let at = source(root, ISOLATED, &valid).unwrap();
        progression
            .observe(
                root,
                &valid,
                &at,
                P,
                &profile::original_for(&at, P).unwrap(),
            )
            .unwrap();
    }
    let valid = merge(root, &integrated, &integrated, &staging);
    add_run(&mut f, 2, "pull_request", &valid, &w);
    resolve_profile_in(&f.api, f.root.path(), P, Some(&host)).unwrap();
    let api = f.api.clone();
    let freeze = git(f.root.path(), &["rev-parse", "producer-freeze"]);
    // First-parent-only traversal sees enrolled -> enrolled. The implementation ancestor
    // with no listed workflow run is reachable only through the second parent.
    let rollback = merge(f.root.path(), &freeze, &freeze, &integrated);
    for checkout in [false, true] {
        f.api = api.clone();
        add_run(&mut f, 3, "pull_request", &rollback, &w);
        f.api.change(RUNS, |v| {
            v["workflow_runs"]
                .as_array_mut()
                .unwrap()
                .retain(|run| run["id"] == 3);
            v["total_count"] = json!(1);
            if checkout {
                v["workflow_runs"][0]["head_sha"] = json!(integrated);
            }
        });
        if checkout {
            // Real PR checkout shape: parent 1 is staging, parent 2 is the API head.
            // The witness still names the merge checkout, and adoption remains independent.
            for attempt in 1..=2 {
                f.api
                    .change(&format!("actions/runs/3/attempts/{attempt}"), |v| {
                        v["head_sha"] = json!(integrated)
                    });
            }
            f.api
                .change("actions/runs/3/artifacts?per_page=100&page=1", |v| {
                    for artifact in v["artifacts"].as_array_mut().unwrap() {
                        artifact["workflow_run"]["head_sha"] = json!(integrated);
                    }
                });
        }
        let error = resolve_profile_in(&f.api, f.root.path(), P, Some(&host))
            .expect_err("merge restored enrolled despite its implemented parent");
        assert!(error.contains("lifecycle progression"), "{error}");
    }
}

#[test]
fn onboarding_producer_refuses_interrupted_or_ambiguous_generation_history() {
    let (f, _) = onboarding();
    let root = f.root.path();
    let original = profile::original_for(&source(root, ISOLATED, &f.commit).unwrap(), P).unwrap();
    let before = git(root, &["rev-parse", "on2-freeze^"]);
    let tree = git(root, &["rev-parse", &format!("{before}^{{tree}}")]);
    let gap = git(
        root,
        &[
            "commit-tree",
            &tree,
            "-p",
            &f.commit,
            "-m",
            "remove binding",
        ],
    );
    let resumed = merge(root, &f.commit, &gap, &f.commit);
    let freeze_tree = git(root, &["rev-parse", "producer-freeze^{tree}"]);
    let parallel = git(
        root,
        &[
            "commit-tree",
            &freeze_tree,
            "-p",
            &before,
            "-m",
            "independent freeze with identical Event/P/O",
        ],
    );
    let ambiguous = merge(root, &f.commit, &f.commit, &parallel);
    for (commit, expected) in [
        (resumed, "history interrupts frozen Event/P/O"),
        (ambiguous, "ambiguous onboarding history"),
    ] {
        let error = progression::Onboarding::default()
            .observe(
                root,
                &commit,
                &source(root, ISOLATED, &commit).unwrap(),
                P,
                &original,
            )
            .unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
    let mut progression = progression::Onboarding::default();
    for (commit, ambiguous) in [(&f.commit, false), (&parallel, true)] {
        let result = progression.observe(
            root,
            commit,
            &source(root, ISOLATED, commit).unwrap(),
            P,
            &original,
        );
        if ambiguous {
            assert!(result.unwrap_err().contains("ambiguous onboarding history"));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn onboarding_producer_refuses_missing_parents_hidden_by_shallow_metadata() {
    let (f, _) = onboarding();
    let root = f.root.path();
    let tree = git(root, &["rev-parse", &format!("{}^{{tree}}", f.commit)]);
    write(
        root,
        "missing-history.commit",
        format!(
            "tree {tree}\nparent {}\nauthor Test <test@example.com> 1 +0000\ncommitter Test <test@example.com> 1 +0000\n\nmissing parent\n",
            "f".repeat(40)
        ),
    );
    let commit = git(
        root,
        &[
            "hash-object",
            "-t",
            "commit",
            "-w",
            "missing-history.commit",
        ],
    );
    write(root, ".git/shallow", format!("{commit}\n"));
    let at = source(root, ISOLATED, &commit).unwrap();
    let error = progression::Onboarding::default()
        .observe(
            root,
            &commit,
            &at,
            P,
            &profile::original_for(&at, P).unwrap(),
        )
        .unwrap_err();
    assert!(
        error.contains("cannot read complete onboarding history"),
        "{error}"
    );
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
