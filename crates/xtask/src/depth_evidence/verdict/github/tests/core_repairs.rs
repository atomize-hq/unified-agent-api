use super::*;

#[test]
fn core_shared_lifecycle_failures_invalidate_every_supported_obligation() {
    for name in [
        "spawn_and_timeout_are_distinct",
        "cancel::explicit_cancel_terminates_blocking_backend_and_completion_is_cancelled",
    ] {
        let mut f = Fixture::new_for(Profile::Core);
        f.w.cases
            .executed
            .iter_mut()
            .find(|r| r.case.name == name)
            .unwrap()
            .outcome = CaseOutcome::Failed;
        f.install(1);
        let result: Value =
            serde_json::from_str(&prove_profile(f.root.path(), &f.api, Profile::Core).unwrap())
                .unwrap();
        let rows = result["obligations"].as_array().unwrap();
        for id in ["E1", "LC2"] {
            let row = rows.iter().find(|r| r["obligation"]["id"] == id).unwrap();
            assert_eq!(
                row["result"], "failed",
                "{name} must invalidate {id}: {result}"
            );
        }
        assert_eq!(
            rows.iter().find(|r| r["obligation"]["id"] == "A1").unwrap()["result"],
            "verified"
        );
    }
}

#[test]
fn core_requirement_only_index_change_invalidates_matching_old_artifact() {
    let f = Fixture::new_for(Profile::Core);
    let before: Value =
        serde_json::from_str(&prove_profile(f.root.path(), &f.api, Profile::Core).unwrap())
            .unwrap();
    assert!(before["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|o| o["result"] == "verified"));
    let resolved = resolve_profile(&f.api, f.root.path(), Profile::Core).unwrap();
    let run = &resolved.runs[0];
    let mut view = crate::git_inputs::GitInputs::new(f.root.path()).unwrap();
    crate::depth_evidence::currentness::check(
        f.root.path(),
        &mut view,
        &run.receipt,
        &run.witness.graph,
        true,
    )
    .unwrap();
    write(
        f.root.path(),
        "docs/specs/unified-agent-api/run-protocol-spec.md",
        "changed cancellation requirements",
    );
    git(f.root.path(), &["add", "."]);
    // The old artifact still matches its authenticated host metadata, but current callers
    // must refuse it before the preview can clone that old built checkout.
    assert!(prove_profile(f.root.path(), &f.api, Profile::Core)
        .unwrap_err()
        .contains("reviewed staging"));
    let mut view = crate::git_inputs::GitInputs::new(f.root.path()).unwrap();
    let error = crate::depth_evidence::currentness::check(
        f.root.path(),
        &mut view,
        &run.receipt,
        &run.witness.graph,
        true,
    )
    .unwrap_err();
    assert!(
        error.contains(
            "changed built/current dependency: docs/specs/unified-agent-api/run-protocol-spec.md"
        ),
        "{error}"
    );
    // Counterfactual: omitting canonical requirements is precisely what hid this drift.
    let mut legacy = run.witness.graph.clone();
    legacy.scopes.retain(|s| s != "docs/specs");
    legacy.inputs.retain(|f| !f.path.starts_with("docs/specs/"));
    let mut view = crate::git_inputs::GitInputs::new(f.root.path()).unwrap();
    crate::depth_evidence::currentness::check(
        f.root.path(),
        &mut view,
        &run.receipt,
        &legacy,
        true,
    )
    .unwrap();
}

#[test]
fn core_requires_complete_built_and_reviewed_canonical_requirements() {
    let mut f = Fixture::new_for(Profile::Core);
    let run = json!({"id":1, "head_sha":f.commit, "event":"workflow_dispatch"});
    for missing_scope in [false, true] {
        let mut w = f.w.clone();
        if missing_scope {
            w.graph.scopes.retain(|s| s != "docs/specs");
        } else {
            w.graph
                .inputs
                .retain(|i| i.path != "docs/specs/unified-agent-api/run-protocol-spec.md");
        }
        assert!(
            profile::validate_witness(f.root.path(), &w, &run, 1, &f.commit, Profile::Core)
                .unwrap_err()
                .contains("complete canonical requirement scope")
        );
    }
    for path in [
        "docs/specs/unified-agent-api/run-protocol-spec.md",
        "docs/specs/new-core-requirement.md",
    ] {
        git(f.root.path(), &["reset", "--hard", &f.commit]);
        write(f.root.path(), path, "changed canonical requirement");
        git(f.root.path(), &["add", "."]);
        git(f.root.path(), &["commit", "-qm", "requirement change"]);
        let changed = git(f.root.path(), &["rev-parse", "HEAD"]);
        let error = prove_profile(f.root.path(), &f.api, Profile::Core).unwrap_err();
        assert!(
            error.contains(if path.ends_with("new-core-requirement.md") {
                "inventory"
            } else {
                "reviewed staging"
            }),
            "{error}"
        );
        assert!(
            profile::approved(f.root.path(), &changed, &changed, &f.commit, Profile::Core)
                .unwrap_err()
                .contains("docs/specs")
        );
        f.api.change("issues/9/comments?per_page=100&page=1", |v| {
            let body = v[0]["body"].as_str().unwrap().replace(&f.commit, &changed);
            v[0]["body"] = json!(body);
        });
        assert!(
            adoption::resolve(&f.api, f.root.path(), 9, &f.w, Profile::Core)
                .unwrap()
                .is_empty()
        );
        // Restore the old endorsement before testing a distinct reviewed source change.
        f.install(1);
    }
}
