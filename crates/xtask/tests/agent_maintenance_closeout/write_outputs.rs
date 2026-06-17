use super::*;

#[test]
fn opencode_maintenance_closeout_writes_only_owned_outputs_after_refresh_state() {
    let fixture = fixture_root("opencode-maintenance-closeout-write");
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

    let packet_root = fixture.join("docs/agents/lifecycle/opencode-maintenance");
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
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
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
    assert!(closeout.contains("\"request_ref\": \"docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml\""));
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
            "docs/integrations/opencode/governance/seam-2-closeout.md"
        ))
        .expect("read onboarding closeout"),
        "# Closeout\n\n- capability advertisement is intentionally conservative and now matches the landed backend contract and generated capability inventory:\n  <!-- xtask-governance-check:opencode-capabilities:start -->\n  `agent_api.config.model.v1`, `agent_api.events`, `agent_api.events.live`, `agent_api.run`, `agent_api.session.fork.v1`, `agent_api.session.resume.v1`\n  <!-- xtask-governance-check:opencode-capabilities:end -->\n  are the claimed OpenCode v1 capability ids under the current runtime evidence\n"
    );
}

#[test]
fn closeout_write_adds_maintenance_closeout_evidence_to_required_and_satisfied_sets() {
    let fixture = fixture_root("opencode-maintenance-closeout-lifecycle");
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
        &valid_closeout_json(&request_absolute, request_path),
    );
    write_text(
        &fixture.join("docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json"),
        include_str!("../../../../docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json"),
    );
    write_text(
        &fixture.join("docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml"),
        include_str!("../../../../docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml"),
    );
    write_text(
        &fixture.join("docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json"),
        include_str!("../../../../docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json"),
    );
    write_text(
        &fixture.join("docs/agents/lifecycle/opencode-cli-onboarding/governance/proving-run-closeout.json"),
        include_str!("../../../../docs/agents/lifecycle/opencode-cli-onboarding/governance/proving-run-closeout.json"),
    );

    write_closeout_outputs(&fixture, request_path, closeout_path).expect("closeout write");

    let state = agent_lifecycle::load_lifecycle_state(
        &fixture,
        "docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json",
    )
    .expect("load lifecycle state");
    assert!(state
        .required_evidence
        .contains(&agent_lifecycle::EvidenceId::MaintenanceCloseoutWritten));
    assert!(state
        .satisfied_evidence
        .contains(&agent_lifecycle::EvidenceId::MaintenanceCloseoutWritten));
}

#[test]
fn automated_request_closeout_preserves_trigger_truth_in_handoff() {
    let fixture = fixture_root("automated-request-closeout-handoff");
    maintenance_harness::seed_opencode_basis(&fixture);
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Generic maintenance opener\n",
    );
    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &automated_maintenance_request_toml(
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

    write_closeout_outputs(&fixture, request_path, closeout_path).expect("closeout write");
    let handoff =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/opencode-maintenance/HANDOFF.md"))
            .expect("read handoff");
    assert!(handoff.contains("upstream_release_detected"));
    assert!(handoff.contains("automation/opencode-maintenance-1.14.47"));
}

#[test]
fn automated_request_execution_contract_still_supports_manual_closeout() {
    let fixture = fixture_root("automated-request-closeout-execution-contract");
    maintenance_harness::seed_opencode_basis(&fixture);
    let registry =
        agent_registry::AgentRegistry::parse(include_str!("../../data/agent_registry.toml"))
            .expect("parse seeded registry");
    let entry = registry.find("opencode").expect("find opencode entry");
    let maintenance_root = "docs/agents/lifecycle/opencode-maintenance";
    write_text(
        &fixture.join(
            "docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md",
        ),
        &contract_policy::packet_pr_prompt_template(entry, maintenance_root),
    );
    write_text(
        &fixture.join("docs/agents/lifecycle/opencode-maintenance/OPS_PLAYBOOK.md"),
        "# Ops playbook\n",
    );
    write_text(
        &fixture.join("docs/agents/lifecycle/opencode-maintenance/CI_WORKFLOWS_PLAN.md"),
        "# CI workflows\n",
    );
    write_text(
        &fixture.join(".github/workflows/agent-maintenance-open-pr.yml"),
        "name: Generic maintenance opener\n",
    );

    let request_path =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml");
    let request_absolute = fixture.join(request_path);
    write_text(
        &request_absolute,
        &automated_maintenance_request_with_execution_contract_toml(
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

    write_closeout_outputs(&fixture, request_path, closeout_path).expect("closeout write");
    let handoff =
        fs::read_to_string(fixture.join("docs/agents/lifecycle/opencode-maintenance/HANDOFF.md"))
            .expect("read handoff");
    assert!(handoff.contains("upstream_release_detected"));
    assert!(handoff.contains("automation/opencode-maintenance-1.14.47"));
    assert!(handoff.contains(
        "Manual closeout remained an explicit maintainer action recorded with `close-agent-maintenance`"
    ));
}
