use super::*;

#[test]
fn close_agent_maintenance_rejects_resolved_findings_that_still_match_live_drift() {
    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    let closeout = valid_closeout_struct(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml",
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
                    "docs/integrations/opencode/governance/seam-2-closeout.md".to_string(),
                    "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md".to_string(),
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
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );
    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
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
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );
    let closeout_path = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
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
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    let closeout = valid_closeout_struct(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml",
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
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    );
    let closeout = valid_closeout_struct(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml",
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
