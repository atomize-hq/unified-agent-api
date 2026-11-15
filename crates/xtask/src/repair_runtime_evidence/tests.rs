use super::*;
use std::fs;

use serde_json::json;

use crate::approval_artifact::{
    ApprovalArtifact, ApprovalDescriptor, ApprovalMaintenance, ApprovalMaintenanceDeferral,
    ApprovalMaintenanceMode, APPROVED_SCOPE_CREATE_LANE_CLOSEOUT,
};

#[test]
fn promote_repair_bundle_restores_backup_when_canonical_validation_fails() {
    let workspace_root = std::env::temp_dir().join(format!(
        "repair-runtime-evidence-rollback-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&workspace_root).expect("create workspace root");

    let approval = ApprovalArtifact {
        support_depth_version: None,
        relative_path: "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml"
            .to_string(),
        canonical_path: workspace_root
            .join("docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml"),
        sha256: "approval-sha".to_string(),
        descriptor: ApprovalDescriptor {
            agent_id: "gemini_cli".to_string(),
            display_name: "Gemini CLI".to_string(),
            crate_path: "crates/gemini_cli".to_string(),
            backend_module: "crates/agent_api/src/backends/gemini_cli".to_string(),
            manifest_root: "cli_manifests/gemini_cli".to_string(),
            package_name: "unified-agent-api-gemini-cli".to_string(),
            canonical_targets: vec!["darwin-arm64".to_string()],
            wrapper_coverage_binding_kind: "generated_from_wrapper_crate".to_string(),
            wrapper_coverage_source_path: "crates/gemini_cli".to_string(),
            always_on_capabilities: vec![],
            target_gated_capabilities: vec![],
            config_gated_capabilities: vec![],
            backend_extensions: vec![],
            support_matrix_enabled: true,
            capability_matrix_enabled: true,
            capability_matrix_target: None,
            docs_release_track: "crates-io".to_string(),
            onboarding_pack_prefix: "gemini-cli-onboarding".to_string(),
        },
        maintenance: ApprovalMaintenance {
            mode: ApprovalMaintenanceMode::ExplicitlyDeferred {
                deferral: ApprovalMaintenanceDeferral {
                    reason: "release watch is deferred".to_string(),
                    follow_up: "revisit after closeout".to_string(),
                    approved_scope: APPROVED_SCOPE_CREATE_LANE_CLOSEOUT.to_string(),
                },
                deferral_sha256: "maintenance-deferral-sha".to_string(),
            },
            section_sha256: "maintenance-section-sha".to_string(),
        },
    };

    let run_id = repair_run_id(&approval.descriptor.agent_id);
    let canonical_run_root = workspace_root.join(format!("{RUNTIME_RUNS_ROOT}/{run_id}"));
    let temp_run_root = workspace_root.join(format!("{RUNTIME_RUNS_ROOT}/.tmp-{run_id}-rollback"));
    let backup_run_root =
        workspace_root.join(format!("{RUNTIME_RUNS_ROOT}/.bak-{run_id}-rollback"));

    fs::create_dir_all(&canonical_run_root).expect("create canonical root");
    fs::write(
        canonical_run_root.join("run-summary.md"),
        "# Previous Repair Bundle\n",
    )
    .expect("write prior summary");

    fs::create_dir_all(&temp_run_root).expect("create temp root");
    let canonical_run_dir = canonical_run_root.to_string_lossy().into_owned();
    write_test_json(
        &temp_run_root.join("input-contract.json"),
        &json!({
            "approval_artifact_path": approval.relative_path,
            "approval_artifact_sha256": approval.sha256,
            "agent_id": approval.descriptor.agent_id,
            "manifest_root": approval.descriptor.manifest_root,
            "required_handoff_commands": crate::agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS,
        }),
    );
    write_test_json(
        &temp_run_root.join("run-status.json"),
        &json!({
            "approval_artifact_path": approval.relative_path,
            "agent_id": approval.descriptor.agent_id,
            "status": "write_validated",
            "validation_passed": true,
            "handoff_ready": true,
            "run_dir": canonical_run_dir,
        }),
    );
    write_test_json(
        &temp_run_root.join("validation-report.json"),
        &json!({
            "status": "pass"
        }),
    );
    write_test_json(
        &temp_run_root.join("handoff.json"),
        &json!({
            "agent_id": approval.descriptor.agent_id,
            "manifest_root": approval.descriptor.manifest_root,
            "runtime_lane_complete": true,
            "publication_refresh_required": true,
            "required_commands": crate::agent_lifecycle::REQUIRED_PUBLICATION_COMMANDS,
            "blockers": []
        }),
    );
    write_test_json(&temp_run_root.join("written-paths.json"), &json!([]));
    fs::write(
        temp_run_root.join("run-summary.md"),
        "# Invalid Replacement Bundle\n",
    )
    .expect("write invalid summary");

    let err = promote_repair_bundle(
        &workspace_root,
        &approval,
        &run_id,
        &canonical_run_root,
        &temp_run_root,
        &backup_run_root,
    )
    .expect_err("invalid canonical bundle should fail validation");
    assert!(
        matches!(err, Error::Validation(message) if message.contains("did not record any written paths"))
    );

    let restored_summary = fs::read_to_string(canonical_run_root.join("run-summary.md"))
        .expect("read restored summary");
    assert!(restored_summary.contains("Previous Repair Bundle"));
    assert!(
        !temp_run_root.exists(),
        "temp staging dir should be cleaned up"
    );
    assert!(
        !backup_run_root.exists(),
        "backup dir should be restored back into canonical location"
    );

    fs::remove_dir_all(&workspace_root).expect("remove workspace root");
}

fn write_test_json(path: &Path, value: &serde_json::Value) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    let mut bytes = serde_json::to_vec_pretty(value).expect("serialize json");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write json");
}
