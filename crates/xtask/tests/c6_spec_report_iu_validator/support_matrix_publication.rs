use super::*;

#[test]
fn c6_validator_rejects_missing_support_matrix_publication_artifact() {
    let temp = make_temp_dir("ccm-c6-support-matrix-artifact-missing");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);

    assert_validation_failure(
        &codex_dir,
        "SUPPORT_MATRIX_ARTIFACT_MISSING",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_rejects_missing_support_matrix_publication_artifact_for_claude_root() {
    let temp = make_temp_dir("ccm-c6-support-matrix-artifact-missing-claude");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    let claude_dir = temp.join("cli_manifests").join("claude_code");
    materialize_minimal_valid_codex_dir(&codex_dir);
    materialize_minimal_valid_claude_dir(&claude_dir);

    assert_validation_failure(
        &claude_dir,
        "SUPPORT_MATRIX_ARTIFACT_MISSING",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_version_status_drift_for_latest_validated_rows() {
    let temp = make_temp_dir("ccm-c6-support-matrix-status");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_version_status(&codex_dir, "reported");
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "latest_supported_and_validated",
                "evidence_notes": [],
            }
        ]),
    );

    assert_validation_failure(
        &codex_dir,
        "SUPPORT_MATRIX_VERSION_STATUS_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_support_matrix_consistency_drift_for_claude_root() {
    let temp = make_temp_dir("ccm-c6-support-matrix-claude-consistency");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    let claude_dir = temp.join("cli_manifests").join("claude_code");
    materialize_minimal_valid_codex_dir(&codex_dir);
    materialize_minimal_valid_claude_dir(&claude_dir);
    write_complete_support_matrix_artifact(&temp);

    assert_validation_failure(
        &claude_dir,
        "SUPPORT_MATRIX_ROW_MISSING",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_pointer_promotion_drift_in_support_matrix_publication() {
    let temp = make_temp_dir("ccm-c6-support-matrix-pointer");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": [],
            }
        ]),
    );

    assert_validation_failure(
        &codex_dir,
        "SUPPORT_MATRIX_POINTER_PROMOTION_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_support_state_drift_in_support_matrix_publication() {
    let temp = make_temp_dir("ccm-c6-support-matrix-support-state");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_complete_support_matrix_artifact(&temp);

    let artifact_path = temp
        .join("cli_manifests")
        .join("support_matrix")
        .join("current.json");
    let mut artifact: Value =
        serde_json::from_str(&fs::read_to_string(&artifact_path).expect("read artifact"))
            .expect("parse artifact");
    artifact["rows"][2]["manifest_support"] = json!("unsupported");
    write_json(&artifact_path, &artifact);

    assert_validation_failure(
        &codex_dir,
        "SUPPORT_MATRIX_MANIFEST_SUPPORT_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_non_canonical_support_matrix_row_order() {
    let temp = make_temp_dir("ccm-c6-support-matrix-order");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_complete_support_matrix_artifact(&temp);

    let artifact_path = temp
        .join("cli_manifests")
        .join("support_matrix")
        .join("current.json");
    let mut artifact: Value =
        serde_json::from_str(&fs::read_to_string(&artifact_path).expect("read artifact"))
            .expect("parse artifact");
    artifact["rows"]
        .as_array_mut()
        .expect("rows array")
        .swap(0, 1);
    write_json(&artifact_path, &artifact);

    assert_validation_failure(
        &codex_dir,
        "SUPPORT_MATRIX_ROW_ORDER_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_rejects_incomplete_support_matrix_publication() {
    let temp = make_temp_dir("ccm-c6-support-matrix-missing");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "latest_supported_and_validated",
                "evidence_notes": [],
            }
        ]),
    );

    assert_validation_failure(
        &codex_dir,
        "SUPPORT_MATRIX_ROW_MISSING",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_rejects_missing_committed_agent_root_even_without_rows() {
    let temp = make_temp_dir("ccm-c6-support-matrix-missing-root");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    let claude_dir = temp.join("cli_manifests").join("claude_code");
    materialize_minimal_valid_codex_dir(&codex_dir);
    materialize_minimal_valid_claude_dir(&claude_dir);
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": "aarch64-apple-darwin",
                "manifest_support": "unsupported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": ["current root snapshot omits this target"],
            },
            {
                "agent": "codex",
                "version": VERSION,
                "target": "x86_64-pc-windows-msvc",
                "manifest_support": "unsupported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": ["current root snapshot omits this target"],
            },
            {
                "agent": "codex",
                "version": VERSION,
                "target": REQUIRED_TARGET,
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "latest_supported_and_validated",
                "evidence_notes": [],
            }
        ]),
    );
    fs::remove_dir_all(&claude_dir).expect("remove committed claude root");

    assert_validation_failure(
        &codex_dir,
        "SUPPORT_MATRIX_ROOT_READ_ERROR",
        "cli_manifests/support_matrix/current.json",
    );
}

#[test]
fn c6_validator_detects_support_claim_drift_for_omitted_target() {
    let temp = make_temp_dir("ccm-c6-support-matrix-omission");
    write_workspace_manifest(&temp);
    let codex_dir = temp.join("cli_manifests").join("codex");
    materialize_minimal_valid_codex_dir(&codex_dir);
    write_support_matrix_artifact(
        &temp,
        json!([
            {
                "agent": "codex",
                "version": VERSION,
                "target": "aarch64-apple-darwin",
                "manifest_support": "supported",
                "backend_support": "unsupported",
                "uaa_support": "unsupported",
                "pointer_promotion": "none",
                "evidence_notes": [],
            }
        ]),
    );

    let output = assert_validation_failure(
        &codex_dir,
        "SUPPORT_MATRIX_CURRENT_SNAPSHOT_OMISSION_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
    assert_violation_surface(
        &output,
        "SUPPORT_MATRIX_EVIDENCE_NOTES_MISMATCH",
        "cli_manifests/support_matrix/current.json",
    );
}
