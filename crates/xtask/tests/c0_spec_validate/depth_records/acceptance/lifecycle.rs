use super::*;
pub(super) fn state_path(temp: &Path) -> PathBuf {
    temp.join(GOVERNANCE).join("lifecycle-state.json")
}
fn proving_path(temp: &Path) -> PathBuf {
    temp.join(GOVERNANCE).join("proving-run-closeout.json")
}
pub(super) fn setup(temp: &Path, root: &Path, depth: bool) -> Value {
    let approval = format!("{GOVERNANCE}/approved-agent.toml");
    let mut text = fs::read_to_string(workspace_root().join(&approval)).unwrap();
    if depth {
        text.push_str(&format!("\n[support_depth]\nversion = \"{VERSION}\"\n"));
    }
    write_text(&temp.join(&approval), &text);
    let mut packet: Value = serde_json::from_str(
        &fs::read_to_string(
            workspace_root()
                .join(GOVERNANCE)
                .join("publication-ready.json"),
        )
        .unwrap(),
    )
    .unwrap();
    packet["approval_artifact_sha256"] = json!(sha(&text));
    write_json(
        &temp.join(GOVERNANCE).join("publication-ready.json"),
        &packet,
    );
    let mut state: Value = serde_json::from_str(
        &fs::read_to_string(
            workspace_root()
                .join(GOVERNANCE)
                .join("lifecycle-state.json"),
        )
        .unwrap(),
    )
    .unwrap();
    state["approval_artifact_sha256"] = json!(sha(&text));
    state["publication_packet_sha256"] = json!(sha(&fs::read_to_string(
        temp.join(GOVERNANCE).join("publication-ready.json")
    )
    .unwrap()));
    for key in ["required_evidence", "satisfied_evidence"] {
        state[key]
            .as_array_mut()
            .unwrap()
            .retain(|v| v != "maintenance_closeout_written");
    }
    write_json(&state_path(temp), &state);
    // Orthogonality audit needs the second publisher as in the actual repository.
    let peer = "docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json";
    write_text(
        &temp.join(peer),
        &fs::read_to_string(workspace_root().join(peer)).unwrap(),
    );
    fs::remove_dir_all(temp.join("cli_manifests/claude_code")).unwrap();
    copy_dir_recursive(
        &workspace_root().join("cli_manifests/claude_code"),
        &temp.join("cli_manifests/claude_code"),
    );
    republish(root);
    state
}
fn proving(temp: &Path, state: &str) -> Value {
    json!({"state":state,"approval_ref":format!("{GOVERNANCE}/approved-agent.toml"),"approval_sha256":sha(&fs::read_to_string(temp.join(GOVERNANCE).join("approved-agent.toml")).unwrap()),"approval_source":"fixture","manual_control_plane_edits":0,"partial_write_incidents":0,"ambiguous_ownership_incidents":0,"duration_seconds":1,"explicit_none_reason":"none","preflight_passed":true,"recorded_at":RECORDED_AT,"commit":COMMIT})
}

#[test]
fn closed_proving_edits_count_but_prepared_and_serialization_only_edits_do_not() {
    let (temp, root, _) = baseline(Some(&record()));
    setup(temp.path(), &root, true);
    let tip = commit_all(temp.path());
    let mut closeout = proving(temp.path(), "prepared");
    write_json(&proving_path(temp.path()), &closeout);
    assert!(requirements(temp.path(), &tip).is_empty());
    closeout["state"] = json!("closed");
    write_json(&proving_path(temp.path()), &closeout);
    assert!(requirements(temp.path(), &tip)[0]
        .effects
        .iter()
        .any(|e| e.kind == "closeout semantic change"));
    let tip = commit_all(temp.path());
    fs::write(proving_path(temp.path()), closeout.to_string()).unwrap();
    assert!(requirements(temp.path(), &tip).is_empty());
    closeout["recorded_at"] = json!("2026-10-21T00:00:00Z");
    write_json(&proving_path(temp.path()), &closeout);
    assert_eq!(requirements(temp.path(), &tip).len(), 1);
    let mut rec = record();
    matching(&mut rec);
    write_record(&root, &rec);
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
}

#[test]
fn lifecycle_publication_is_acceptance_even_after_separate_prepare_and_unchanged_packet() {
    use xtask::agent_lifecycle::{required_evidence_for_stage, LifecycleStage};
    let (temp, root, _) = baseline(Some(&record()));
    let published = setup(temp.path(), &root, true);
    let mut prepared = published.clone();
    prepared["lifecycle_stage"] = json!("publication_ready");
    prepared["support_tier"] = json!("baseline_runtime");
    let evidence = serde_json::to_value(required_evidence_for_stage(
        LifecycleStage::PublicationReady,
    ))
    .unwrap();
    prepared["required_evidence"] = evidence.clone();
    prepared["satisfied_evidence"] = evidence;
    write_json(&state_path(temp.path()), &prepared);
    republish(&root);
    let tip = commit_all(temp.path());
    write_json(&state_path(temp.path()), &published);
    republish(&root);
    assert!(requirements(temp.path(), &tip)[0]
        .effects
        .iter()
        .any(|e| e.kind == "published"));
    let mut rec = record();
    matching(&mut rec);
    write_record(&root, &rec);
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
}

#[test]
fn drift_clearance_requires_candidate_attribution_even_if_the_lifecycle_file_is_removed() {
    let (temp, root, _) = baseline(Some(&record()));
    let mut state = setup(temp.path(), &root, false);
    state["side_states"] = json!(["drifted"]);
    write_json(&state_path(temp.path()), &state);
    let tip = commit_all(temp.path());
    state["side_states"] = json!([]);
    write_json(&state_path(temp.path()), &state);
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap_err()
        .contains("scope unresolved"));
    let mut closeout = maintenance_closeout(temp.path());
    write_json(&closeout_path(temp.path()), &closeout);
    assert_eq!(requirements(temp.path(), &tip).len(), 1);
    // A positively resolved outside-version request is ordinary, without charging retained V.
    let request = fs::read_to_string(temp.path().join(request_path()))
        .unwrap()
        .replace(VERSION, "9.9.9");
    write_text(&temp.path().join(request_path()), &request);
    closeout["request_sha256"] = json!(sha(&request));
    write_json(&closeout_path(temp.path()), &closeout);
    assert!(requirements(temp.path(), &tip).is_empty());
    fs::remove_file(state_path(temp.path())).unwrap();
    republish(&root);
    let report = xtask::depth_acceptance::compare(temp.path(), &tip, "codex").unwrap();
    assert!(report.missing_entries.is_empty());
    // Removing a publisher independently violates the ordinary orthogonality audit.
    assert!(!report.input_issues.is_empty());
    fs::remove_file(closeout_path(temp.path())).unwrap();
    assert!(xtask::depth_acceptance::compare(temp.path(), &tip, "codex")
        .unwrap_err()
        .contains("scope unresolved"));
}

#[test]
fn closed_stage_and_each_closeout_evidence_assertion_charge_their_own_destination() {
    use xtask::agent_lifecycle::{required_evidence_for_stage, LifecycleStage};
    let (temp, root, _) = baseline(Some(&record()));
    let published = setup(temp.path(), &root, true);
    write_json(&proving_path(temp.path()), &proving(temp.path(), "closed"));
    write_json(
        &closeout_path(temp.path()),
        &maintenance_closeout(temp.path()),
    );
    let tip = commit_all(temp.path());
    let mut maintained = published.clone();
    for field in ["required_evidence", "satisfied_evidence"] {
        maintained[field]
            .as_array_mut()
            .unwrap()
            .push(json!("maintenance_closeout_written"));
    }
    write_json(&state_path(temp.path()), &maintained);
    assert!(requirements(temp.path(), &tip)[0]
        .effects
        .iter()
        .any(|e| e.kind == "maintenance_closeout_written"));
    let mut closed = published;
    closed["lifecycle_stage"] = json!("closed_baseline");
    closed["closeout_baseline_path"] = json!(format!("{GOVERNANCE}/proving-run-closeout.json"));
    let evidence =
        serde_json::to_value(required_evidence_for_stage(LifecycleStage::ClosedBaseline)).unwrap();
    closed["required_evidence"] = evidence.clone();
    closed["satisfied_evidence"] = evidence;
    write_json(&state_path(temp.path()), &closed);
    let missing = requirements(temp.path(), &tip);
    for kind in ["closed_baseline", "proving_run_closeout_written"] {
        assert!(missing[0].effects.iter().any(|e| e.kind == kind));
    }
    let mut rec = record();
    matching(&mut rec);
    write_record(&root, &rec);
    republish(&root);
    assert!(requirements(temp.path(), &tip).is_empty());
}
