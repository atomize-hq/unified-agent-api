//! Checks 1 and 3 on the onboarding path: an onboarding record against the lifecycle state's
//! `support_depth` object (OF-P2), as `manifest-validate` reports them.

use xtask::agent_lifecycle::LifecycleState;

use super::*;

pub(super) const APPROVAL_COMMIT: &str = "8c2607b451727a39392f6f6d3afe34328b969919";
pub(super) const APPROVAL_RECORDED_AT: &str = "2025-11-23T13:26:12-05:00";

/// An onboarding record of codex's approval, with P only.
pub(super) fn onboarding_record() -> Value {
    let mut record = record();
    record["event"] = json!({ "lifecycle_path": "onboarding", "approval_commit": APPROVAL_COMMIT,
                              "approval_recorded_at": APPROVAL_RECORDED_AT });
    record["policy"]["selection"]["enrollment"]["lifecycle_path"] = json!("onboarding");
    record["policy"]["identity"] = json!(part_identity(&record["policy"]).unwrap());
    record
}

/// Codex's lifecycle record at `enrolled`, the stage a dry run freezes at, bound to the approval
/// already at `GOVERNANCE`, with `object` as its `support_depth` object. It reports no packet or
/// closeout, so it adds no acceptance input.
pub(super) fn bind_lifecycle(workspace: &Path, object: Option<Value>) -> PathBuf {
    let lifecycle = format!("{GOVERNANCE}/lifecycle-state.json");
    write_text(
        &workspace.join(&lifecycle),
        &fs::read_to_string(workspace_root().join(&lifecycle)).unwrap(),
    );
    let approval = workspace.join(GOVERNANCE).join("approved-agent.toml");
    let lifecycle_path = workspace.join(&lifecycle);
    edit_state(&lifecycle_path, |state| {
        let enrolled = ["registry_entry", "docs_pack", "manifest_root_skeleton"];
        state["approval_artifact_sha256"] = json!(crate::harness::sha256_hex(&approval));
        state["lifecycle_stage"] = json!("enrolled");
        state["support_tier"] = json!("bootstrap");
        state["required_evidence"] = json!(enrolled);
        state["satisfied_evidence"] = json!(enrolled);
        for field in [
            "implementation_summary",
            "publication_packet_path",
            "publication_packet_sha256",
            "closeout_baseline_path",
            "active_runtime_evidence_run_id",
        ] {
            state[field] = Value::Null;
        }
        let state = state.as_object_mut().unwrap();
        state.retain(|key, _| !key.starts_with("maintenance_readiness"));
    });
    set_object(&lifecycle_path, object);
    lifecycle_path
}

/// Codex's approval with the `[support_depth]` table approving `VERSION`, and its lifecycle record.
fn seed_onboarding_pack(workspace: &Path) -> PathBuf {
    let approval = format!("{GOVERNANCE}/approved-agent.toml");
    let text = fs::read_to_string(workspace_root().join(&approval)).unwrap();
    write_text(
        &workspace.join(&approval),
        &format!("{text}\n[support_depth]\nversion = \"{VERSION}\"\n"),
    );
    bind_lifecycle(workspace, None)
}

/// Rewrites the lifecycle record with `edit`, through the typed state, as a lifecycle writer
/// writes it.
fn edit_state(lifecycle_path: &Path, edit: impl FnOnce(&mut Value)) {
    let mut lifecycle: Value =
        serde_json::from_str(&fs::read_to_string(lifecycle_path).unwrap()).unwrap();
    edit(&mut lifecycle);
    let state: LifecycleState = serde_json::from_str(&lifecycle.to_string()).unwrap();
    write_text(
        lifecycle_path,
        &format!("{}\n", serde_json::to_string_pretty(&state).unwrap()),
    );
}

fn set_object(lifecycle_path: &Path, object: Option<Value>) {
    edit_state(lifecycle_path, |lifecycle| match object {
        Some(object) => lifecycle["support_depth"] = object,
        None => {
            lifecycle.as_object_mut().unwrap().remove("support_depth");
        }
    });
}

fn lifecycle_rel() -> String {
    format!("{GOVERNANCE}/lifecycle-state.json")
}

#[test]
fn c0_validate_requires_linked_lifecycle_artifacts_only_when_depth_is_present() {
    let temp = make_temp_dir("ccm-c0-depth-onboarding-links");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    let lifecycle_path = seed_onboarding_pack(&temp);
    let approval_path = temp.join(GOVERNANCE).join("approved-agent.toml");
    let approval = fs::read(&approval_path).unwrap();
    fs::remove_file(&approval_path).unwrap();

    let (passed, report) = validate(&codex_dir);
    assert!(
        passed,
        "no record and no object needs no approval link:\n{report}"
    );

    // Local depth-field invariants still apply before the no-depth shortcut.
    edit_state(&lifecycle_path, |state| {
        state["written_paths_sha256"] = json!("0".repeat(64));
    });
    assert_fails_at(&codex_dir, "DEPTH_LIFECYCLE_UNREADABLE", &lifecycle_rel());
    edit_state(&lifecycle_path, |state| {
        state
            .as_object_mut()
            .unwrap()
            .remove("written_paths_sha256");
    });

    // An object alone still requires the authenticated approval, even before a record is found.
    set_object(
        &lifecycle_path,
        Some(json!({ "policy_identity": "0".repeat(64) })),
    );
    assert_fails_at(&codex_dir, "DEPTH_LIFECYCLE_UNREADABLE", &lifecycle_rel());

    // A record alone still runs check 10's full lifecycle validation on either lifecycle path.
    set_object(&lifecycle_path, None);
    for record in [onboarding_record(), record()] {
        write_record(&codex_dir, &record);
        assert_fails_at(&codex_dir, "DEPTH_LIFECYCLE_DANGLING", &lifecycle_rel());
    }
    fs::remove_file(codex_dir.join(format!("reports/{VERSION}/depth-record.json"))).unwrap();

    // An existing but changed approval is also irrelevant until depth is present.
    fs::write(&approval_path, [&approval[..], b"\n# changed\n"].concat()).unwrap();
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "no depth needs no approval digest check:\n{report}");
    set_object(
        &lifecycle_path,
        Some(json!({ "policy_identity": "0".repeat(64) })),
    );
    assert_fails_at(&codex_dir, "DEPTH_LIFECYCLE_UNREADABLE", &lifecycle_rel());
}

#[test]
fn c0_validate_requires_an_onboarding_record_to_state_what_the_lifecycle_object_froze() {
    let temp = make_temp_dir("ccm-c0-depth-onboarding");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    let lifecycle_path = seed_onboarding_pack(&temp);
    let record = onboarding_record();
    let policy = record["policy"]["identity"].clone();
    write_record(&codex_dir, &record);

    // A record without the object is a deleted binding; a missing object is not "not enrolled".
    assert_fails_at(
        &codex_dir,
        "DEPTH_LIFECYCLE_BINDING_MISSING",
        &record_path(),
    );

    set_object(&lifecycle_path, Some(json!({ "policy_identity": policy })));
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "the object states the record's P:\n{report}");

    // Check 3: another P, and an O the record does not state.
    set_object(
        &lifecycle_path,
        Some(json!({ "policy_identity": "0".repeat(64) })),
    );
    assert_fails_at(
        &codex_dir,
        "DEPTH_RECORD_NOT_THE_LIFECYCLES",
        &record_path(),
    );
    set_object(
        &lifecycle_path,
        Some(json!({ "policy_identity": policy, "obligations_identity": "0".repeat(64) })),
    );
    assert_fails_at(
        &codex_dir,
        "DEPTH_RECORD_NOT_THE_LIFECYCLES",
        &record_path(),
    );

    // Check 1: the object states P and the approval's version holds no record.
    set_object(&lifecycle_path, Some(json!({ "policy_identity": policy })));
    fs::remove_file(codex_dir.join(record_path().replace("cli_manifests/codex/", ""))).unwrap();
    assert_fails_at(
        &codex_dir,
        "DEPTH_LIFECYCLE_BINDING_ORPHANED",
        &lifecycle_rel(),
    );
    // No record and no object: not yet frozen, and not an error.
    set_object(&lifecycle_path, None);
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "declared, no record yet:\n{report}");
}

#[test]
fn c0_validate_refuses_a_deleted_or_unreadable_lifecycle_state_of_an_onboarding_record() {
    let temp = make_temp_dir("ccm-c0-depth-onboarding-deleted");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    let lifecycle_path = seed_onboarding_pack(&temp);
    let record = onboarding_record();
    set_object(
        &lifecycle_path,
        Some(json!({ "policy_identity": record["policy"]["identity"] })),
    );
    write_record(&codex_dir, &record);
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "bound:\n{report}");

    // Deleting the whole lifecycle state deletes the object with it.
    let bound = fs::read_to_string(&lifecycle_path).unwrap();
    fs::remove_file(&lifecycle_path).unwrap();
    assert_fails_at(
        &codex_dir,
        "DEPTH_LIFECYCLE_BINDING_MISSING",
        &record_path(),
    );

    // With no record left, a lifecycle state that cannot be read is reported here, not skipped:
    // it may state an object no record states.
    fs::remove_file(codex_dir.join(record_path().replace("cli_manifests/codex/", ""))).unwrap();
    let unreadable = [
        "{ not json".to_string(),
        bound.replace(
            "\"support_depth\": {",
            "\"support_depth\": {\n    \"extra\": 1,",
        ),
        // A repeated member, here written once with an escaped name.
        bound.replacen(
            "\"support_depth\":",
            &format!(
                "\"support_depth\": {{\"policy_identity\": \"{}\"}},\n  \"support\\u005fdepth\":",
                "0".repeat(64)
            ),
            1,
        ),
    ];
    for text in unreadable {
        write_text(&lifecycle_path, &text);
        assert_fails_at(&codex_dir, "DEPTH_LIFECYCLE_UNREADABLE", &lifecycle_rel());
    }
    // Any formatting of the object reads: it is the orphaned object that is reported.
    for text in [
        bound.clone(),
        bound.replace(
            "{\n    \"policy_identity\": ",
            "{\"policy\\u005fidentity\":",
        ),
    ] {
        write_text(&lifecycle_path, &text);
        assert_fails_at(
            &codex_dir,
            "DEPTH_LIFECYCLE_BINDING_ORPHANED",
            &lifecycle_rel(),
        );
    }
}

#[test]
fn c0_validate_reports_a_lifecycle_state_it_cannot_look_up() {
    let temp = make_temp_dir("ccm-c0-depth-onboarding-metadata");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    seed_onboarding_pack(&temp);
    let governance = temp.join(GOVERNANCE);
    let held = temp.join("held-governance");
    fs::rename(&governance, &held).unwrap();

    // A parent that is not a directory is not an absent file, and neither is a symlink loop.
    write_text(&governance, "not a directory\n");
    assert_fails_at(&codex_dir, "DEPTH_LIFECYCLE_UNREADABLE", &lifecycle_rel());
    fs::remove_file(&governance).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("governance", &governance).unwrap();
        assert_fails_at(&codex_dir, "DEPTH_LIFECYCLE_UNREADABLE", &lifecycle_rel());
        fs::remove_file(&governance).unwrap();
    }

    // With the pack gone altogether and no record, there is nothing to report.
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "no pack, no record:\n{report}");
    fs::rename(&held, &governance).unwrap();
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "declared, no record yet:\n{report}");
}

#[test]
fn c0_validate_refuses_another_agents_approval_and_lifecycle_identity() {
    let temp = make_temp_dir("ccm-c0-depth-onboarding-foreign");
    let codex_dir = materialize_minimal_valid_workspace(&temp);
    let lifecycle_path = seed_onboarding_pack(&temp);
    let record = onboarding_record();
    write_record(&codex_dir, &record);
    let object = json!({ "policy_identity": record["policy"]["identity"] });
    set_object(&lifecycle_path, Some(object));

    // The lifecycle state points at another agent's approval, with that approval's digest, and the
    // record copies that approval's Event and version.
    let foreign = "docs/agents/lifecycle/aider-onboarding/governance/approved-agent.toml";
    let text = fs::read_to_string(workspace_root().join(foreign)).unwrap();
    write_text(
        &temp.join(foreign),
        &format!("{text}\n[support_depth]\nversion = \"{VERSION}\"\n"),
    );
    let approval: toml_edit::DocumentMut = text.parse().unwrap();
    let mut copied = record.clone();
    copied["event"]["approval_commit"] = json!(approval["approval_commit"].as_str().unwrap());
    copied["event"]["approval_recorded_at"] =
        json!(approval["approval_recorded_at"].as_str().unwrap());
    write_record(&codex_dir, &copied);
    let sha = crate::harness::sha256_hex(&temp.join(foreign));
    edit_state(&lifecycle_path, |lifecycle| {
        lifecycle["approval_artifact_path"] = json!(foreign);
        lifecycle["approval_artifact_sha256"] = json!(sha);
    });
    assert_fails_at(&codex_dir, "DEPTH_APPROVAL_NOT_THE_AGENTS", foreign);

    // A lifecycle state that names another agent is not this one's either.
    let codex_approval = temp.join(GOVERNANCE).join("approved-agent.toml");
    let sha = crate::harness::sha256_hex(&codex_approval);
    write_record(&codex_dir, &record);
    edit_state(&lifecycle_path, |lifecycle| {
        lifecycle["approval_artifact_path"] = json!(format!("{GOVERNANCE}/approved-agent.toml"));
        lifecycle["approval_artifact_sha256"] = json!(sha);
        lifecycle["agent_id"] = json!("aider");
    });
    assert_fails_at(
        &codex_dir,
        "DEPTH_LIFECYCLE_NOT_THE_AGENTS",
        &lifecycle_rel(),
    );
}
