use super::*;

#[test]
fn runtime_git_policy_authentication_rejects_self_selected_closure_and_source() {
    let root = tempfile::tempdir().unwrap();
    git(
        root.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            fixture().root.path().to_str().unwrap(),
            ".",
        ],
    );
    git(root.path(), &["config", "user.name", "Policy"]);
    git(
        root.path(),
        &["config", "user.email", "policy@example.invalid"],
    );
    let approved = &fixture().publication.authority.policy;
    super::super::policy::authenticate(root.path(), approved, approved).unwrap();
    std::fs::write(
        root.path().join("crates/policy.rs"),
        "candidate-selected policy\n",
    )
    .unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "unapproved policy"]);
    let commit = git(root.path(), &["rev-parse", "HEAD"]);
    let source = emitter::object(root.path(), &commit).unwrap();
    let mut claimed = approved.clone();
    claimed.workflow_commit = commit;
    // Copying approved identities does not approve the actual evaluated source.
    assert!(
        super::super::policy::authenticate(root.path(), &claimed, approved)
            .unwrap_err()
            .contains("not approved on protected staging")
    );
    // Recomputing a perfectly valid digest over candidate objects cannot approve them either.
    claimed.closure = super::super::policy::closure(root.path(), &source, ["producer"]).unwrap();
    claimed.implementation = super::super::policy::identity(&claimed.closure).unwrap();
    assert!(
        super::super::policy::authenticate(root.path(), &claimed, approved)
            .unwrap_err()
            .contains("differs from protected anchor")
    );
    let mut missing = approved.clone();
    missing.closure.remove("Cargo.lock");
    assert!(super::super::policy::authenticate(root.path(), &missing, approved).is_err());
    // A record copied from an arbitrary host cannot select a new evaluated policy commit.
    let mut record = retained();
    record.policy = claimed;
    let host = fixture().host.fork();
    publish(&host, record);
    assert!(resolve(host).is_err());
}

#[test]
fn runtime_policy_refuses_root_toolchain_configuration_even_with_matching_claims() {
    let root = tempfile::tempdir().unwrap();
    git(
        root.path(),
        &[
            "clone",
            "--quiet",
            "--no-hardlinks",
            fixture().root.path().to_str().unwrap(),
            ".",
        ],
    );
    git(root.path(), &["config", "user.name", "Policy"]);
    git(
        root.path(),
        &["config", "user.email", "policy@example.invalid"],
    );
    std::fs::write(
        root.path().join("rust-toolchain.toml"),
        "[toolchain]\nchannel = 'nightly'\n",
    )
    .unwrap();
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "unapproved toolchain"]);
    let source = emitter::object(root.path(), &git(root.path(), &["rev-parse", "HEAD"])).unwrap();
    assert!(
        super::super::policy::closure(root.path(), &source, ["producer"])
            .unwrap_err()
            .contains("absent root Cargo/toolchain")
    );
}

#[test]
fn same_run_wrong_job_artifact_with_valid_digest_and_copied_commitment_refuses() {
    let host = fixture().host.fork();
    let mut record = retained();
    // The same run retains both jobs' artifacts. Its trusted job log even names the wrong
    // artifact; only exact payload job/check binding distinguishes this attack.
    record.claims.job += 1;
    let mut upload_claims = record.claims.clone();
    upload_claims.attempt += 1;
    let artifact = host
        .upload(&upload_claims, &record.canonical().unwrap())
        .unwrap();
    host.facts.lock().unwrap().insert(
        format!("actions/jobs/{}/logs", retained().claims.job),
        format!(
            "2026-11-13T00:00:01Z {}\n",
            transport::commitment(&record.identity().unwrap(), artifact)
        )
        .into_bytes(),
    );
    assert!(resolve(host)
        .err()
        .unwrap()
        .contains("independent host association"));
}

#[test]
fn incomplete_observation_is_never_eligible_or_a_conflict_side() {
    let first = retained();
    let host = fixture().host.fork();
    let mut incomplete = first.clone();
    incomplete.decision = Decision::Refused;
    incomplete.reasons = vec![Reason::ObservationIncomplete];
    assert_eq!(
        incomplete.inputs_identity(&[]).unwrap(),
        first.inputs_identity(&[]).unwrap()
    );
    let refused = publish(&host, incomplete.clone());
    publish(&host, first.clone());
    let history = resolve(host.fork()).unwrap();
    assert!(history.eligible(&first.identity().unwrap()).unwrap());
    assert!(!history.eligible(&refused.identity().unwrap()).unwrap());
    // An incomplete refusal must also preserve detection of a later reached contradiction.
    incomplete.reasons = vec![
        Reason::OwningPredicateRefused,
        Reason::ObservationIncomplete,
    ];
    publish(&host, incomplete);
    assert!(resolve(host)
        .err()
        .unwrap()
        .contains("conflicting decisions over identical inputs"));
    let mut malformed = first;
    malformed.reasons = vec![Reason::ObservationIncomplete];
    assert!(malformed.canonical().is_err());
    malformed.decision = Decision::Refused;
    malformed.reasons = vec![
        Reason::OwningPredicatesSatisfied,
        Reason::ObservationIncomplete,
    ];
    assert!(malformed.canonical().is_err());
}

#[test]
fn history_transport_unavailability_is_sticky_while_reached_refusal_is_complete() {
    let first = retained();
    for path in [
        String::new(),
        format!("actions/jobs/{}/logs", first.claims.job),
    ] {
        let host = fixture().host.fork();
        let original = host.facts.lock().unwrap().remove(&path).unwrap();
        let capture = observations::Capture::start().unwrap();
        assert!(resolve(host.fork()).is_err());
        host.facts.lock().unwrap().insert(path, original);
        resolve(host).unwrap();
        assert!(capture.finish().unwrap().incomplete);
    }
    let host = fixture().host.fork();
    host.change("", |value| value["id"] = json!(999));
    let capture = observations::Capture::start().unwrap();
    assert!(resolve(host)
        .err()
        .unwrap()
        .contains("repository identity mismatch"));
    assert!(!capture.finish().unwrap().incomplete);
}

#[test]
fn reached_admission_with_incomplete_closing_is_not_successful_authority() {
    let first = retained();
    let host = fixture().host.fork();
    host.page(transport::RUNS, "workflow_runs", vec![]);
    let mut incomplete = first.clone();
    incomplete.reasons.push(Reason::ObservationIncomplete);
    let incomplete = publish(&host, incomplete);
    let history = resolve(host.fork()).unwrap();
    assert!(!history.eligible(&incomplete.identity().unwrap()).unwrap());
    assert!(history
        .predecessors(&incomplete.candidate.commit)
        .unwrap()
        .is_empty());
    let conflicting = host.fork();
    let mut refusal = first.clone();
    refusal.decision = Decision::Refused;
    refusal.reasons = vec![Reason::OwningPredicateRefused];
    publish(&conflicting, refusal);
    assert!(resolve(conflicting)
        .err()
        .unwrap()
        .contains("conflicting decisions over identical inputs"));
    let complete = publish(&host, first);
    let history = resolve(host).unwrap();
    assert!(history.eligible(&complete.identity().unwrap()).unwrap());
}
