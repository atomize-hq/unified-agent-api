//! Checks 1 and 3 on the onboarding path: an onboarding record against the lifecycle state's
//! `support_depth` object (OF-P2), as `manifest-validate` reports them.

use super::*;

const APPROVAL_COMMIT: &str = "8c2607b451727a39392f6f6d3afe34328b969919";
const APPROVAL_RECORDED_AT: &str = "2025-11-23T13:26:12-05:00";

/// An onboarding record of codex's approval, with P only.
fn onboarding_record() -> Value {
    let mut record = record();
    record["event"] = json!({ "lifecycle_path": "onboarding", "approval_commit": APPROVAL_COMMIT,
                              "approval_recorded_at": APPROVAL_RECORDED_AT });
    record["policy"]["selection"]["enrollment"]["lifecycle_path"] = json!("onboarding");
    record["policy"]["identity"] = json!(part_identity(&record["policy"]).unwrap());
    record
}

/// Codex's lifecycle record, approval and closeouts, with the approval's `[support_depth]` table
/// approving `VERSION` and the lifecycle record bound to that approval.
fn seed_onboarding_pack(workspace: &Path) -> PathBuf {
    copy_lifecycle(workspace);
    let closeout = MAINTENANCE.to_string() + "/maintenance-closeout.json";
    write_text(
        &workspace.join(&closeout),
        &fs::read_to_string(workspace_root().join(&closeout)).unwrap(),
    );
    let approval = workspace.join(GOVERNANCE).join("approved-agent.toml");
    let text = fs::read_to_string(&approval).unwrap();
    write_text(
        &approval,
        &format!("{text}\n[support_depth]\nversion = \"{VERSION}\"\n"),
    );
    let lifecycle_path = workspace.join(GOVERNANCE).join("lifecycle-state.json");
    let mut lifecycle: Value =
        serde_json::from_str(&fs::read_to_string(&lifecycle_path).unwrap()).unwrap();
    lifecycle["approval_artifact_sha256"] = json!(crate::harness::sha256_hex(&approval));
    write_json(&lifecycle_path, &lifecycle);
    lifecycle_path
}

fn set_object(lifecycle_path: &Path, object: Option<Value>) {
    let mut lifecycle: Value =
        serde_json::from_str(&fs::read_to_string(lifecycle_path).unwrap()).unwrap();
    match object {
        Some(object) => lifecycle["support_depth"] = object,
        None => {
            lifecycle.as_object_mut().unwrap().remove("support_depth");
        }
    }
    write_json(lifecycle_path, &lifecycle);
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
    assert_fails_with(&codex_dir, "DEPTH_RECORD_NOT_THE_LIFECYCLES");

    // Check 1: the object states P and the approval's version holds no record.
    set_object(&lifecycle_path, Some(json!({ "policy_identity": policy })));
    fs::remove_file(codex_dir.join(record_path().replace("cli_manifests/codex/", ""))).unwrap();
    assert_fails_at(
        &codex_dir,
        "DEPTH_LIFECYCLE_BINDING_ORPHANED",
        &format!("{GOVERNANCE}/lifecycle-state.json"),
    );
    // No record and no object: not yet frozen, and not an error.
    set_object(&lifecycle_path, None);
    let (passed, report) = validate(&codex_dir);
    assert!(passed, "declared, no record yet:\n{report}");
}
