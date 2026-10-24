use super::*;
use crate::harness::repo_root;

#[test]
fn opencode_maintenance_closeout_writes_only_owned_outputs_after_refresh_state() {
    let fixture = fixture_root("opencode-maintenance-closeout-write");
    seed_opencode_basis(&fixture);
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
    seed_opencode_basis(&fixture);
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
fn invalid_lifecycle_never_partially_writes_closeout_outputs() {
    let fixture = fixture_root("maintenance-closeout-invalid-lifecycle-transaction");
    seed_opencode_basis(&fixture);
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
    let original_closeout = valid_closeout_json(&request_absolute, request_path);
    write_text(&fixture.join(closeout_path), &original_closeout);
    let handoff_path = fixture.join("docs/agents/lifecycle/opencode-maintenance/HANDOFF.md");
    write_text(&handoff_path, "open handoff\n");

    let onboarding_root = "docs/agents/lifecycle/opencode-cli-onboarding/governance";
    for name in [
        "approved-agent.toml",
        "publication-ready.json",
        "proving-run-closeout.json",
    ] {
        let source = repo_root().join(onboarding_root).join(name);
        write_text(
            &fixture.join(onboarding_root).join(name),
            &fs::read_to_string(source).expect("source"),
        );
    }
    let state_source = repo_root()
        .join(onboarding_root)
        .join("lifecycle-state.json");
    let mut state: serde_json::Value =
        serde_json::from_slice(&fs::read(state_source).expect("state source")).expect("state json");
    for field in ["required_evidence", "satisfied_evidence"] {
        state[field]
            .as_array_mut()
            .expect("evidence array")
            .retain(|value| value.as_str() != Some("maintenance_readiness_settled"));
    }
    write_text(
        &fixture.join(onboarding_root).join("lifecycle-state.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&state).expect("serialize state")
        ),
    );

    let err = write_closeout_outputs(&fixture, request_path, closeout_path)
        .expect_err("invalid lifecycle must prevent all writes");
    assert!(err.to_string().contains("maintenance_readiness_settled"));
    assert_eq!(
        fs::read_to_string(fixture.join(closeout_path)).expect("closeout"),
        original_closeout
    );
    assert_eq!(
        fs::read_to_string(handoff_path).expect("handoff"),
        "open handoff\n"
    );
    assert!(!fixture
        .join("docs/agents/lifecycle/opencode-maintenance/governance/remediation-log.md")
        .exists());
}

#[test]
fn automated_request_closeout_preserves_trigger_truth_in_handoff() {
    let fixture = fixture_root("automated-request-closeout-handoff");
    seed_opencode_basis(&fixture);
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
    seed_opencode_basis(&fixture);
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

fn lower_writer_fixture(
    name: &str,
) -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
    let root = fixture_root(name);
    seed_opencode_basis(&root);
    let request =
        Path::new("docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml")
            .to_path_buf();
    let closeout = Path::new(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
    )
    .to_path_buf();
    write_text(
        &root.join(&request),
        &maintenance_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );
    write_text(
        &root.join(&closeout),
        &valid_closeout_json(&root.join(&request), &request),
    );
    (root, request, closeout)
}

#[test]
fn lower_planner_revalidates_bytes_and_derives_paths_instead_of_trusting_constructed_linkage() {
    let (root, request, closeout) = lower_writer_fixture("d5-lower-planner");
    let mut linked = load_linked_closeout(&root, &request, &closeout).unwrap();
    linked.maintenance_pack_root = "docs/agents/lifecycle/codex-maintenance".into();
    linked.request.agent_id = "codex".into();
    let plans = crate::closeout::plan_closeout_mutations(&root, &linked).unwrap();
    assert!(plans.iter().all(|p| p
        .relative_path()
        .starts_with("docs/agents/lifecycle/opencode-maintenance")));
    let prior = fs::read(root.join(&closeout)).unwrap();
    linked.closeout.recorded_at = "invalid".into();
    let error = crate::closeout::plan_closeout_mutations(&root, &linked).unwrap_err();
    assert!(error.to_string().contains("recorded_at"), "{error}");
    assert_eq!(fs::read(root.join(closeout)).unwrap(), prior);
}

#[test]
fn lower_writer_refuses_unresolved_retained_depth_ownership_and_preserves_outputs() {
    let (root, request, closeout) = lower_writer_fixture("d5-lower-retained-depth");
    let mut record: serde_json::Value = serde_json::from_str(
        &include_str!("../fixtures/depth-ci/record.json").replace("codex", "opencode"),
    )
    .unwrap();
    record["policy"]["identity"] =
        json!(crate::depth_record::part_identity(&record["policy"]).unwrap());
    record["obligations"]["policy_identity"] = record["policy"]["identity"].clone();
    record["obligations"]["identity"] =
        json!(crate::depth_record::part_identity(&record["obligations"]).unwrap());
    let record_path = "cli_manifests/opencode/reports/0.0.0/depth-record.json";
    write_text(
        &root.join(record_path),
        &serde_json::to_string_pretty(&record).unwrap(),
    );
    crate::depth_record::DepthRecord::parse(&record.to_string(), "opencode", "0.0.0").unwrap();
    let handoff = "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md";
    let remediation = "docs/agents/lifecycle/opencode-maintenance/governance/remediation-log.md";
    write_text(&root.join(handoff), "old handoff");
    write_text(&root.join(remediation), "old log");
    let live = crate::drift::check_agent_drift(&root, "opencode").unwrap();
    let mut artifact: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(&closeout)).unwrap()).unwrap();
    artifact
        .as_object_mut()
        .unwrap()
        .remove("explicit_none_reason");
    artifact["deferred_findings"] = json!(live
        .findings
        .iter()
        .map(|f| json!({"category_id":f.category_id(), "summary":f.summary, "surfaces":f.surfaces}))
        .collect::<Vec<_>>());
    write_text(&root.join(&closeout), &artifact.to_string());
    load_linked_closeout(&root, &request, &closeout)
        .expect("legacy closeout validates before the depth guard");
    let paths = [
        &closeout,
        Path::new(record_path),
        Path::new(handoff),
        Path::new(remediation),
    ];
    let before: Vec<_> = paths
        .iter()
        .map(|p| fs::read(root.join(p)).unwrap())
        .collect();
    let error = write_closeout_outputs(&root, &request, &closeout).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("request has no exact target version"),
        "{error}"
    );
    for (path, previous) in paths.into_iter().zip(before) {
        assert_eq!(fs::read(root.join(path)).unwrap(), previous);
    }
}
