use super::*;

#[test]
fn automated_request_v2_with_detected_release_parses() {
    let fixture = fixture_root("agent-maintenance-automated-request-v2");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &automated_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let request = load_request(&fixture, Path::new(request_path)).expect("load request");
    assert!(request.is_automated_watch_request());
    let detected = request.detected_release.expect("detected release");
    assert_eq!(detected.target_version, "0.98.0");
    assert_eq!(detected.dispatch_workflow, "codex-cli-update-snapshot.yml");
    assert_eq!(
        detected.branch_name,
        "automation/opencode-maintenance-0.98.0"
    );
}

#[test]
fn automated_request_with_execution_contract_parses_and_validates() {
    let fixture = fixture_root("agent-maintenance-automated-request-with-contract");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &automated_request_with_execution_contract_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let envelope =
        load_request_envelope(&fixture, Path::new(request_path)).expect("load request envelope");
    let contract = envelope
        .require_execution_contract_for_relay()
        .expect("execution contract");

    assert_eq!(contract.executor, "execute-agent-maintenance");
    assert_eq!(
        contract.prompt_template_path,
        "cli_manifests/opencode/PR_BODY_TEMPLATE.md"
    );
    assert_eq!(
        contract.pr_summary_path,
        "docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md"
    );
    assert_eq!(
        contract.recovery.reopen_pr_branch,
        "automation/opencode-maintenance-0.98.0"
    );
    assert!(contract.requires_manual_closeout);
    assert!(contract
        .writable_surfaces
        .contains(&"docs/agents/lifecycle/opencode-maintenance/**".to_string()));
}

#[test]
fn historical_automated_request_without_execution_contract_still_loads_for_read_only() {
    let fixture = fixture_root("agent-maintenance-automated-request-historical-read-only");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &automated_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let envelope =
        load_request_envelope(&fixture, Path::new(request_path)).expect("load request envelope");
    assert!(envelope.request.is_automated_watch_request());
    assert!(envelope.execution_contract.is_none());

    let err = envelope
        .require_execution_contract_for_relay()
        .expect_err("historical request should fail relay-only requirement");
    assert!(err.to_string().contains("[execution_contract]"));
}

#[test]
fn automated_request_execution_contract_rejects_non_repo_relative_writable_surface() {
    let fixture = fixture_root("agent-maintenance-automated-request-invalid-writable-surface");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &automated_request_with_execution_contract_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        )
        .replace(
            "\"docs/agents/lifecycle/opencode-maintenance/**\"",
            "\"../escape/**\"",
        ),
    );

    let err = load_request_envelope(&fixture, Path::new(request_path))
        .expect_err("invalid writable surface should fail");
    assert!(err
        .to_string()
        .contains("execution_contract.writable_surfaces"));
    assert!(err.to_string().contains("repo-relative path"));
}

#[test]
fn automated_request_execution_contract_rejects_recovery_branch_mismatch() {
    let fixture = fixture_root("agent-maintenance-automated-request-invalid-recovery-branch");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &automated_request_with_execution_contract_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        )
        .replace(
            "reopen_pr_branch = \"automation/opencode-maintenance-0.98.0\"",
            "reopen_pr_branch = \"automation/opencode-maintenance-0.99.0\"",
        ),
    );

    let err = load_request_envelope(&fixture, Path::new(request_path))
        .expect_err("recovery branch mismatch should fail");
    assert!(err
        .to_string()
        .contains("execution_contract.recovery.reopen_pr_branch"));
    assert!(err.to_string().contains("detected_release.branch_name"));
}

#[test]
fn automated_request_execution_contract_rejects_non_codex_executor() {
    let fixture = fixture_root("agent-maintenance-automated-request-invalid-executor");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &automated_request_with_execution_contract_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        )
        .replace(
            "executor = \"execute-agent-maintenance\"",
            "executor = \"claude_code\"",
        ),
    );

    let err = load_request_envelope(&fixture, Path::new(request_path))
        .expect_err("non-codex executor should fail");
    assert!(err.to_string().contains("execution_contract.executor"));
    assert!(err
        .to_string()
        .contains("must be `execute-agent-maintenance`"));
}

#[test]
fn automated_request_execution_contract_rejects_manual_closeout_false() {
    let fixture = fixture_root("agent-maintenance-automated-request-invalid-closeout-flag");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &automated_request_with_execution_contract_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        )
        .replace(
            "requires_manual_closeout = true",
            "requires_manual_closeout = false",
        ),
    );

    let err = load_request_envelope(&fixture, Path::new(request_path))
        .expect_err("manual closeout false should fail");
    assert!(err
        .to_string()
        .contains("execution_contract.requires_manual_closeout"));
    assert!(err.to_string().contains("must be `true`"));
}

#[test]
fn automated_packet_refresh_renders_canonical_handoff_and_pr_summary() {
    let fixture = fixture_root("agent-maintenance-automated-packet-docs");
    seed_publication_inputs(&fixture);
    write_text(
        &fixture.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &automated_request_toml(
            "opencode",
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let plan = build_refresh_plan(&fixture, Path::new(request_path)).expect("build refresh plan");
    assert!(plan
        .planned_paths()
        .contains(&"docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md"));

    let handoff = planned_utf8(
        &plan,
        "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md",
    );
    assert!(handoff.contains("This file is the canonical contributor execution contract"));
    assert!(handoff.contains("## Writable surfaces"));
    assert!(handoff.contains("## Read-only inputs"));
    assert!(handoff.contains("## Ordered repo commands"));
    assert!(handoff.contains("## Exact green gates"));
    assert!(handoff.contains("## Exact closeout command"));
    assert!(handoff.contains("Follow the maintained PR template for 0.98.0."));
    assert!(handoff.contains(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml"
    ));

    let pr_summary = planned_utf8(
        &plan,
        "docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md",
    );
    assert!(pr_summary.contains("Automated maintenance packet for `opencode` target `0.98.0`."));
    assert!(pr_summary.contains("Follow the maintained PR template for 0.98.0."));
    assert!(!pr_summary.contains("## Work Queue Summary (autogenerated)"));
}
