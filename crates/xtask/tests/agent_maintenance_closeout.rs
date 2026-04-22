#![allow(dead_code, unused_imports, clippy::enum_variant_names)]

use std::{fs, os::unix::fs::symlink, path::Path};

use serde_json::json;
use sha2::Digest;

mod agent_registry {
    pub use xtask::agent_registry::*;
}

#[path = "../src/agent_maintenance/finding_signature.rs"]
mod finding_signature;

mod workspace_mutation {
    pub use xtask::workspace_mutation::*;
}

#[path = "../src/approval_artifact.rs"]
mod approval_artifact;
#[path = "../src/agent_maintenance/closeout.rs"]
mod closeout;
#[path = "../src/agent_maintenance/drift/mod.rs"]
mod drift;
#[path = "../src/release_doc.rs"]
mod release_doc;
#[path = "../src/agent_maintenance/request.rs"]
mod request;
#[path = "../src/root_intake_layout.rs"]
mod root_intake_layout;
#[path = "../src/support_matrix.rs"]
mod support_matrix;

#[path = "support/onboard_agent_harness.rs"]
mod harness;
#[path = "support/agent_maintenance_harness.rs"]
mod maintenance_harness;

use closeout::{
    load_linked_closeout, validate_live_drift_report, validate_live_drift_truth,
    write_closeout_outputs,
};
use harness::{fixture_root, write_text};

#[test]
fn close_agent_maintenance_requires_request_linkage() {
    let fixture = fixture_root("close-agent-maintenance-request-linkage");
    maintenance_harness::seed_opencode_basis(&fixture);
    let request_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
    );
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": "docs/project_management/next/opencode-maintenance/governance/not-the-request.toml",
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [finding_json(
                "governance_doc_drift",
                "SEAM-2 closeout now matches the landed capability advertisement boundary.",
                &[
                    "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
                    "docs/project_management/next/opencode-maintenance/HANDOFF.md"
                ],
            )],
            "explicit_none_reason": "No deferred maintenance findings remain after packet refresh.",
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("request linkage mismatch should fail");
    assert!(err
        .to_string()
        .contains("`request_ref` must equal `docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml`"));
}

#[test]
fn close_agent_maintenance_requires_resolved_and_deferred_truth() {
    let fixture = fixture_root("close-agent-maintenance-truth");
    maintenance_harness::seed_opencode_basis(&fixture);
    let request_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
    );
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": request_path.display().to_string(),
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [],
            "explicit_none_reason": "No deferred maintenance findings remain after packet refresh.",
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("empty resolved findings should fail");
    assert!(err
        .to_string()
        .contains("`resolved_findings` must not be empty"));

    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": request_path.display().to_string(),
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [finding_json(
                "governance_doc_drift",
                "SEAM-2 closeout now matches the landed capability advertisement boundary.",
                &[
                    "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
                ],
            )],
            "deferred_findings": [finding_json(
                "support_publication_drift",
                "Support publication still needs follow-up.",
                &[
                    "docs/specs/unified-agent-api/support-matrix.md",
                ],
            )],
            "explicit_none_reason": "No deferred maintenance findings remain after packet refresh.",
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("deferred findings xor explicit-none is required");
    assert!(err
        .to_string()
        .contains("exactly one of `deferred_findings` or `explicit_none_reason` is required"));
}

#[test]
fn close_agent_maintenance_rejects_symlinked_output() {
    let fixture = fixture_root("close-agent-maintenance-symlink-output");
    maintenance_harness::seed_opencode_basis(&fixture);
    let request_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
    );
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": request_path.display().to_string(),
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [finding_json(
                "governance_doc_drift",
                "SEAM-2 closeout still matches live governance drift.",
                &[
                    "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
                    "docs/specs/unified-agent-api/capability-matrix.md"
                ],
            )],
            "explicit_none_reason": "No deferred maintenance findings remain after publication and packet refresh.",
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let handoff_path = fixture.join("docs/project_management/next/opencode-maintenance/HANDOFF.md");
    let outside = fixture_root("close-agent-maintenance-symlink-target");
    let outside_target = outside.join("handoff.md");
    write_text(&outside_target, "outside handoff\n");
    if let Some(parent) = handoff_path.parent() {
        fs::create_dir_all(parent).expect("create handoff parent");
    }
    symlink(&outside_target, &handoff_path).expect("create handoff symlink");

    let err = write_closeout_outputs(&fixture, request_path, closeout_path)
        .expect_err("symlinked output should fail");
    let message = err.to_string();
    assert!(message.contains("HANDOFF.md"));
    assert!(message.contains("symlink"));
}

#[test]
fn close_agent_maintenance_rejects_missing_request_evidence_refs() {
    let fixture = fixture_root("close-agent-maintenance-missing-request-evidence");
    maintenance_harness::seed_opencode_basis(&fixture);
    let request_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
    );
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml_with_refs(
            "opencode",
            "docs/project_management/next/opencode-maintenance/governance/missing-basis.md",
            "docs/project_management/next/opencode-maintenance/governance/missing-opened-from.md",
        ),
    );

    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &valid_closeout_json(&request_absolute, request_path),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("missing request evidence refs should fail");
    let message = err.to_string();
    assert!(message.contains("unable to load linked request"));
    assert!(message.contains("field `basis_ref`"));
    assert!(message.contains("must point to an existing file"));
}

#[test]
fn close_agent_maintenance_rejects_resolved_findings_that_still_match_live_drift() {
    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    let closeout = valid_closeout_struct(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );

    let err = validate_live_drift_report(
        closeout_path,
        "opencode",
        &closeout,
        Ok(drift::AgentDriftReport {
            agent_id: "opencode".to_string(),
            findings: vec![drift::DriftFinding {
                category: drift::DriftCategory::GovernanceDoc,
                summary: "Live governance drift is still present.".to_string(),
                surfaces: vec![
                    "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md"
                        .to_string(),
                    "docs/project_management/next/opencode-maintenance/HANDOFF.md".to_string(),
                ],
            }],
        }),
    )
    .expect_err("live drift cannot also be marked resolved");
    assert!(err
        .to_string()
        .contains("`resolved_findings` still matches live drift"));
}

#[test]
fn close_agent_maintenance_rejects_explicit_none_when_live_drift_exists() {
    let fixture = fixture_root("close-agent-maintenance-live-explicit-none");
    maintenance_harness::seed_opencode_basis(&fixture);
    maintenance_harness::overwrite_opencode_governance_with_stale_claim(&fixture);
    let request_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
    );
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
        ),
    );
    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &valid_closeout_json(&request_absolute, request_path),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("live drift cannot use explicit-none");
    assert!(err
        .to_string()
        .contains("`explicit_none_reason` is only allowed when the live drift report is clean"));
}

#[test]
fn close_agent_maintenance_rejects_unaccounted_live_deferred_drift() {
    let fixture = fixture_root("close-agent-maintenance-live-deferred-missing");
    maintenance_harness::seed_opencode_basis(&fixture);
    maintenance_harness::overwrite_opencode_governance_with_stale_claim(&fixture);
    let request_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
    );
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
        ),
    );
    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &serde_json::to_string_pretty(&json!({
            "request_ref": request_path.display().to_string(),
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [finding_json(
                "release_doc_drift",
                "Historical release-doc drift was resolved.",
                &["docs/crates-io-release.md"],
            )],
            "deferred_findings": [finding_json(
                "support_publication_drift",
                "Support publication still needs follow-up.",
                &["docs/specs/unified-agent-api/support-matrix.md"],
            )],
            "preflight_passed": true,
            "recorded_at": "2026-04-22T01:45:00Z",
            "commit": "4adefdf"
        }))
        .expect("serialize closeout"),
    );

    let err = load_linked_closeout(&fixture, request_path, closeout_path)
        .expect_err("all live drift must be deferred if still present");
    assert!(err
        .to_string()
        .contains("is not accounted for in `deferred_findings`"));
}

#[test]
fn close_agent_maintenance_rejects_deferred_findings_when_live_report_is_clean() {
    let fixture = fixture_root("close-agent-maintenance-clean-deferred");
    maintenance_harness::seed_opencode_basis(&fixture);
    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    let closeout = valid_closeout_struct(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );

    let err = validate_live_drift_truth(
        &fixture,
        closeout_path,
        "opencode",
        &closeout_with_deferred(closeout),
    )
    .expect_err("clean live report cannot keep deferred findings");
    assert!(err
        .to_string()
        .contains("`deferred_findings` must be empty when the live drift report is clean"));
}

#[test]
fn close_agent_maintenance_blocks_when_live_drift_recheck_returns_error() {
    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    let closeout = valid_closeout_struct(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );

    let err = validate_live_drift_report(
        closeout_path,
        "opencode",
        &closeout,
        Err(drift::DriftCheckError::Internal(
            "synthetic live re-check failure".to_string(),
        )),
    )
    .expect_err("live drift re-check errors must block closeout");
    assert!(err
        .to_string()
        .contains("live drift re-check failed for `opencode`"));
}

#[test]
fn opencode_maintenance_closeout_writes_only_owned_outputs_after_refresh_state() {
    let fixture = fixture_root("opencode-maintenance-closeout-write");
    maintenance_harness::seed_opencode_basis(&fixture);
    let request_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml",
    );
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
        ),
    );

    let packet_root = fixture.join("docs/project_management/next/opencode-maintenance");
    write_text(
        &packet_root.join("README.md"),
        "historical maintenance readme\n",
    );
    write_text(
        &packet_root.join("scope_brief.md"),
        "historical maintenance scope\n",
    );
    write_text(
        &packet_root.join("governance/remediation-log.md"),
        "old remediation log\n",
    );
    write_text(&packet_root.join("HANDOFF.md"), "old handoff\n");

    let closeout_path = Path::new(
        "docs/project_management/next/opencode-maintenance/governance/maintenance-closeout.json",
    );
    write_text(
        &fixture.join(closeout_path),
        &valid_closeout_json(&request_absolute, request_path),
    );

    let summary = write_closeout_outputs(&fixture, request_path, closeout_path)
        .expect("closeout write should succeed");
    assert_eq!(summary.agent_id, "opencode");
    assert_eq!(summary.apply.total, 3);

    let handoff = fs::read_to_string(packet_root.join("HANDOFF.md")).expect("read handoff");
    assert!(handoff.contains("closed maintenance run for `opencode`"));
    assert!(handoff.contains("governance_doc_drift"));
    assert!(handoff.contains("No deferred findings remain"));

    let remediation_log = fs::read_to_string(packet_root.join("governance/remediation-log.md"))
        .expect("read remediation log");
    assert!(remediation_log.contains("request sha256"));
    assert!(remediation_log
        .contains("SEAM-2 closeout now matches the landed capability advertisement boundary."));

    let closeout = fs::read_to_string(fixture.join(closeout_path)).expect("read closeout");
    assert!(closeout.contains("\"request_ref\": \"docs/project_management/next/opencode-maintenance/governance/maintenance-request.toml\""));
    assert!(closeout.contains("\"explicit_none_reason\": \"No deferred maintenance findings remain after publication and packet refresh.\""));

    assert_eq!(
        fs::read_to_string(packet_root.join("README.md")).expect("read readme"),
        "historical maintenance readme\n"
    );
    assert_eq!(
        fs::read_to_string(packet_root.join("scope_brief.md")).expect("read scope"),
        "historical maintenance scope\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.join(
            "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md"
        ))
        .expect("read onboarding closeout"),
        "# Closeout\n\n- capability advertisement is intentionally conservative and now matches the landed backend contract and generated capability inventory:\n  <!-- xtask-governance-check:opencode-capabilities:start -->\n  `agent_api.config.model.v1`, `agent_api.events`, `agent_api.events.live`, `agent_api.run`, `agent_api.session.fork.v1`, `agent_api.session.resume.v1`\n  <!-- xtask-governance-check:opencode-capabilities:end -->\n  are the claimed OpenCode v1 capability ids under the current runtime evidence\n"
    );
}

fn maintenance_request_toml(agent_id: &str, basis_ref: &str) -> String {
    maintenance_request_toml_with_refs(agent_id, basis_ref, basis_ref)
}

fn maintenance_request_toml_with_refs(
    agent_id: &str,
    basis_ref: &str,
    opened_from: &str,
) -> String {
    format!(
        concat!(
            "artifact_version = \"1\"\n",
            "agent_id = \"{agent_id}\"\n",
            "trigger_kind = \"drift_detected\"\n",
            "basis_ref = \"{basis_ref}\"\n",
            "opened_from = \"{opened_from}\"\n",
            "requested_control_plane_actions = [\n",
            "  \"packet_doc_refresh\",\n",
            "  \"capability_matrix_refresh\",\n",
            "]\n",
            "request_recorded_at = \"2026-04-22T01:15:00Z\"\n",
            "request_commit = \"1adb8f1\"\n",
            "\n",
            "[runtime_followup_required]\n",
            "required = false\n",
            "items = []\n"
        ),
        agent_id = agent_id,
        basis_ref = basis_ref,
        opened_from = opened_from,
    )
}

fn valid_closeout_json(request_absolute: &Path, request_path: &Path) -> String {
    serde_json::to_string_pretty(&valid_closeout(
        &request_path.display().to_string(),
        &sha256_hex(request_absolute),
    ))
    .expect("serialize closeout")
}

fn finding_json(category_id: &str, summary: &str, surfaces: &[&str]) -> serde_json::Value {
    json!({
        "category_id": category_id,
        "summary": summary,
        "surfaces": surfaces,
    })
}

fn valid_closeout(request_ref: &str, request_sha256: &str) -> serde_json::Value {
    json!({
        "request_ref": request_ref,
        "request_sha256": request_sha256,
        "resolved_findings": [finding_json(
            "governance_doc_drift",
            "SEAM-2 closeout now matches the landed capability advertisement boundary.",
            &[
                "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
                "docs/project_management/next/opencode-maintenance/HANDOFF.md"
            ],
        )],
        "explicit_none_reason": "No deferred maintenance findings remain after publication and packet refresh.",
        "preflight_passed": true,
        "recorded_at": "2026-04-22T01:45:00Z",
        "commit": "4adefdf"
    })
}

fn valid_closeout_struct(request_ref: &str, request_sha256: &str) -> closeout::MaintenanceCloseout {
    closeout::MaintenanceCloseout {
        request_ref: request_ref.to_string(),
        request_sha256: request_sha256.to_string(),
        resolved_findings: vec![closeout::MaintenanceFinding {
            category_id: closeout::MaintenanceDriftCategory::GovernanceDoc,
            summary: "SEAM-2 closeout now matches the landed capability advertisement boundary."
                .to_string(),
            surfaces: vec![
                "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md"
                    .to_string(),
                "docs/project_management/next/opencode-maintenance/HANDOFF.md".to_string(),
            ],
        }],
        deferred_findings: closeout::DeferredFindingsTruth::Findings(vec![
            closeout::MaintenanceFinding {
                category_id: closeout::MaintenanceDriftCategory::GovernanceDoc,
                summary: "Governance drift remains deferred.".to_string(),
                surfaces: vec![
                    "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md"
                        .to_string(),
                ],
            },
        ]),
        preflight_passed: true,
        recorded_at: "2026-04-22T01:45:00Z".to_string(),
        commit: "4adefdf".to_string(),
    }
}

fn closeout_with_deferred(
    closeout: closeout::MaintenanceCloseout,
) -> closeout::MaintenanceCloseout {
    closeout::MaintenanceCloseout {
        deferred_findings: closeout::DeferredFindingsTruth::Findings(vec![
            closeout::MaintenanceFinding {
                category_id: closeout::MaintenanceDriftCategory::GovernanceDoc,
                summary: "Governance drift remains deferred.".to_string(),
                surfaces: vec![
                    "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md"
                        .to_string(),
                ],
            },
        ]),
        ..closeout
    }
}

fn sha256_hex(path: &Path) -> String {
    let bytes = fs::read(path).expect("read request artifact");
    hex::encode(sha2::Sha256::digest(bytes))
}
