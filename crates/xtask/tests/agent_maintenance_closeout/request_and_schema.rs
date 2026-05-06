use super::*;

#[test]
fn close_agent_maintenance_requires_request_linkage() {
    let fixture = fixture_root("close-agent-maintenance-request-linkage");
    maintenance_harness::seed_opencode_basis(&fixture);
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
            "request_ref": "docs/agents/lifecycle/opencode-maintenance/governance/not-the-request.toml",
            "request_sha256": sha256_hex(&request_absolute),
            "resolved_findings": [finding_json(
                "governance_doc_drift",
                "SEAM-2 closeout now matches the landed capability advertisement boundary.",
                &[
                    "docs/integrations/opencode/governance/seam-2-closeout.md",
                    "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md"
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
        .contains("`request_ref` must equal `docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml`"));
}

#[test]
fn close_agent_maintenance_requires_resolved_and_deferred_truth() {
    let fixture = fixture_root("close-agent-maintenance-truth");
    maintenance_harness::seed_opencode_basis(&fixture);
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
                    "docs/integrations/opencode/governance/seam-2-closeout.md",
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
                "governance_doc_drift",
                "SEAM-2 closeout still matches live governance drift.",
                &[
                    "docs/integrations/opencode/governance/seam-2-closeout.md",
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

    let handoff_path = fixture.join("docs/agents/lifecycle/opencode-maintenance/HANDOFF.md");
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
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &maintenance_request_toml_with_refs(
            "opencode",
            "docs/agents/lifecycle/opencode-maintenance/governance/missing-basis.md",
            "docs/agents/lifecycle/opencode-maintenance/governance/missing-opened-from.md",
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
        .expect_err("missing request evidence refs should fail");
    let message = err.to_string();
    assert!(message.contains("unable to load linked request"));
    assert!(message.contains("field `basis_ref`"));
    assert!(message.contains("must point to an existing file"));
}
