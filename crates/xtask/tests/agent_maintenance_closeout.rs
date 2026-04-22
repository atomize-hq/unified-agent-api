#![allow(dead_code, unused_imports, clippy::enum_variant_names)]

use std::{fs, os::unix::fs::symlink, path::Path};

use serde_json::json;
use sha2::Digest;

mod agent_registry {
    pub use xtask::agent_registry::*;
}

mod workspace_mutation {
    pub use xtask::workspace_mutation::*;
}

#[path = "../src/agent_maintenance/closeout.rs"]
mod closeout;

#[path = "support/onboard_agent_harness.rs"]
mod harness;

use closeout::{load_linked_closeout, write_closeout_outputs};
use harness::{fixture_root, write_text};

#[test]
fn close_agent_maintenance_requires_request_linkage() {
    let fixture = fixture_root("close-agent-maintenance-request-linkage");
    seed_opencode_basis(&fixture);
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
    seed_opencode_basis(&fixture);
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
    seed_opencode_basis(&fixture);
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
fn opencode_maintenance_closeout_writes_only_owned_outputs_after_refresh_state() {
    let fixture = fixture_root("opencode-maintenance-closeout-write");
    seed_opencode_basis(&fixture);
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
        "# SEAM-2 closeout\n\nThis stale capability claim triggered maintenance.\n"
    );
}

fn maintenance_request_toml(agent_id: &str, basis_ref: &str) -> String {
    format!(
        concat!(
            "artifact_version = \"1\"\n",
            "agent_id = \"{agent_id}\"\n",
            "trigger_kind = \"drift_detected\"\n",
            "basis_ref = \"{basis_ref}\"\n",
            "opened_from = \"{basis_ref}\"\n",
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
    )
}

fn valid_closeout_json(request_absolute: &Path, request_path: &Path) -> String {
    serde_json::to_string_pretty(&json!({
        "request_ref": request_path.display().to_string(),
        "request_sha256": sha256_hex(request_absolute),
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
    }))
    .expect("serialize closeout")
}

fn finding_json(category_id: &str, summary: &str, surfaces: &[&str]) -> serde_json::Value {
    json!({
        "category_id": category_id,
        "summary": summary,
        "surfaces": surfaces,
    })
}

fn seed_opencode_basis(root: &Path) {
    write_text(
        &root.join(
            "docs/project_management/next/opencode-implementation/governance/seam-2-closeout.md",
        ),
        "# SEAM-2 closeout\n\nThis stale capability claim triggered maintenance.\n",
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        "# Capability matrix\n\nOpenCode capability publication truth.\n",
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nOpenCode support publication truth.\n",
    );
}

fn sha256_hex(path: &Path) -> String {
    let bytes = fs::read(path).expect("read request artifact");
    hex::encode(sha2::Sha256::digest(bytes))
}
