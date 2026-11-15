//! Extends the existing real ON2 writer/IN1 chain, avoiding another expensive writer setup.
use super::*;
use crate::depth_acceptance::admission::{fake::MemoryHost, Decision, Generation, Publication};
use std::sync::Arc;

pub(crate) fn prove_unobserved_admission(
    root: &Path,
    publication: Publication,
    host: Arc<MemoryHost>,
) {
    let scratch = tempfile::tempdir().unwrap();
    git(
        scratch.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            root.to_str().unwrap(),
            ".",
        ],
    );
    let root = scratch.path();
    git(root, &["config", "user.name", "Admission history"]);
    git(root, &["config", "user.email", "history@example.invalid"]);
    let records = host.records();
    let implementation = records
        .iter()
        .find(|r| {
            r.decision == Decision::Admitted
                && r.new_state.as_ref().is_some_and(|s| {
                    s.stage == crate::agent_lifecycle::LifecycleStage::RuntimeIntegrated
                })
        })
        .unwrap();
    let freeze = records
        .iter()
        .find(|r| {
            r.decision == Decision::Admitted
                && r.generation.is_some()
                && r.new_state
                    .as_ref()
                    .is_some_and(|s| s.stage == crate::agent_lifecycle::LifecycleStage::Enrolled)
        })
        .unwrap();
    let enrolled = git(
        root,
        &[
            "show",
            &format!("{}:{}", freeze.candidate.commit, inputs::STATE),
        ],
    );
    let (mut w, built) = build(root, P);
    unrelated_pre_anchor_dispatch_is_inapplicable(root, &publication, &host, &w, &built);
    write(root, inputs::STATE, enrolled);
    git(root, &["add", "."]);
    git(
        root,
        &[
            "commit",
            "-qm",
            "rollback on another PR with no producer on admitted ancestor",
        ],
    );
    let lower = git(root, &["rev-parse", "HEAD"]);
    w.workflow_commit = lower.clone();
    w.build.source_commit = lower.clone();
    w.build.source_tree = git(root, &["rev-parse", "HEAD^{tree}"]);
    let api = endpoints(root, P, &w, &lower, 1);
    let mut f = Fixture {
        profile: P,
        root: scratch,
        api,
        w: w.clone(),
        commit: lower.clone(),
    };
    write(
        f.root.path(),
        "docs/other-pr.md",
        "another PR in the same generation\n",
    );
    later(
        &mut f,
        2,
        "merge_group",
        "another PR repeats the rollback",
        &w,
    );
    let tip = git(f.root.path(), &["rev-parse", "HEAD"]);
    f.api
        .put("git/ref/heads/staging", json!({"object":{"sha":tip}}));
    let original = P
        .record(&fs::read_to_string(f.root.path().join(inputs::RECORD)).unwrap())
        .unwrap();
    let candidates = vec![lower.clone(), tip.clone()];
    let host = Arc::new(host.fork());
    let before = host.records();
    host.root(&tip);
    assert_eq!(
        host.records(),
        before,
        "tip movement must not issue admissions"
    );
    let publication = Publication {
        authority: publication.authority,
        host: host.clone(),
    };
    let bound = EvidenceHost::for_admission_test(ISOLATED, &tip, publication.clone());
    for reverse in [false, true] {
        if reverse {
            f.api.change(RUNS, |v| {
                v["workflow_runs"].as_array_mut().unwrap().reverse()
            });
        }
        let rows = f.api.json(&f.api.endpoint(RUNS)).unwrap();
        assert!(rows["workflow_runs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["head_sha"] != implementation.candidate.commit));
        // These two producer sources alone have identical enrolled stages. The independently
        // retained real IN1 admission is the only observation of the higher stage.
        let mut observed = progression::Onboarding::default();
        for candidate in &candidates {
            observed
                .observe(
                    f.root.path(),
                    candidate,
                    &source(f.root.path(), ISOLATED, candidate).unwrap(),
                    P,
                    &original,
                )
                .unwrap();
        }
        let error = resolve_profile_in(&f.api, f.root.path(), P, Some(&bound)).unwrap_err();
        assert!(error.contains("lifecycle progression"), "{error}");
    }
    // Isolate the other PR so a refusal on the original PR cannot mask missing cross-PR
    // progression. Generation discovery must not be narrowed to the approval-linked PR.
    f.api.change(RUNS, |v| {
        v["workflow_runs"]
            .as_array_mut()
            .unwrap()
            .retain(|run| run["id"] == 2);
        v["total_count"] = json!(1);
    });
    let error = resolve_profile_in(&f.api, f.root.path(), P, Some(&bound)).unwrap_err();
    assert!(
        error.contains("lifecycle progression"),
        "other PR alone: {error}"
    );
    // Revocation removes present eligibility while preserving the irreversible historical stage.
    let revoked: Vec<_> = records
        .iter()
        .filter(|r| r.candidate.commit == implementation.candidate.commit)
        .map(|r| r.identity().unwrap())
        .collect();
    host.put("in1/revocations", json!({"records":revoked}));
    let history = publication
        .resolve(f.root.path(), &[tip.clone(), lower.clone()])
        .unwrap();
    assert!(!history
        .eligible(&implementation.identity().unwrap())
        .unwrap());
    assert!(history
        .check_progression(
            &lower,
            &Generation::from_record(&original),
            crate::agent_lifecycle::LifecycleStage::Enrolled
        )
        .unwrap_err()
        .contains("lifecycle progression"));
    // Hiding the admission population leaves a protected generation gap, even though retained
    // payload bytes, the fake protected branch and valid candidate files are still present.
    host.page("actions/workflows/ci.yml/runs", "workflow_runs", vec![]);
    let history = publication
        .resolve(f.root.path(), &[tip, lower.clone()])
        .unwrap();
    assert!(history
        .check_candidate(f.root.path(), &lower, &original, inputs::STATE)
        .unwrap_err()
        .contains("history gap"));
    let retained_payloads = host.records();
    let error = resolve_profile_in(&f.api, f.root.path(), P, Some(&bound)).unwrap_err();
    assert!(error.contains("history gap"), "{error}");
    assert_eq!(
        host.records(),
        retained_payloads,
        "discovery must not issue admissions"
    );
}

fn unrelated_pre_anchor_dispatch_is_inapplicable(
    root: &Path,
    publication: &Publication,
    host: &MemoryHost,
    witness: &Witness,
    tip: &str,
) {
    // An older branch with no depth record has no enrollment lineage. It still belongs in
    // the producer inventory, and must be skipped by Event/P/O applicability before history.
    let empty = git(root, &["mktree"]);
    let output = crate::git_inputs::command(root)
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .args(["commit-tree", &empty, "-m", "pre-enrollment branch"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let old = String::from_utf8(output.stdout).unwrap().trim().to_owned();
    let mut api = endpoints(root, P, witness, tip, 1);
    api.change(RUNS, |value| {
        value["workflow_runs"].as_array_mut().unwrap().push(json!({
            "id":999,"head_sha":old,"path":profile::WORKFLOW,
            "head_repository":{"full_name":ISOLATED},"event":"workflow_dispatch","run_attempt":1
        }));
        value["total_count"] = json!(value["workflow_runs"].as_array().unwrap().len());
    });
    let host = Arc::new(host.fork());
    host.root(tip);
    let publication = Publication {
        authority: publication.authority.clone(),
        host,
    };
    // Confirm that supplying the unrelated root to the history resolver reproduces the gap.
    assert!(publication
        .resolve(root, &[tip.into(), old])
        .err()
        .unwrap()
        .contains("history gap"));
    let bound = EvidenceHost::for_admission_test(ISOLATED, tip, publication);
    let resolved = resolve_profile_in(&api, root, P, Some(&bound)).unwrap();
    assert_eq!(resolved.runs.len(), 11);
}
