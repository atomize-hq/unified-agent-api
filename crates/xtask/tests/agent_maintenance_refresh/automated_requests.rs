use super::*;
use sha2::Digest;

const OPENCODE_PACKET_PR_WORKFLOW: &str = ".github/workflows/agent-maintenance-open-pr.yml";

fn seed_opencode_packet_pr_workflow(fixture: &std::path::Path) {
    write_text(
        &fixture.join(OPENCODE_PACKET_PR_WORKFLOW),
        "name: Packet PR opener\n",
    );
}

fn seed_opencode_packet_owned_contract_inputs(fixture: &std::path::Path) {
    let registry = agent_registry::AgentRegistry::load(fixture).expect("load registry");
    let entry = registry.find("opencode").expect("opencode registry entry");
    let maintenance_root = "docs/agents/lifecycle/opencode-maintenance";
    write_text(
        &fixture.join(contract_policy::packet_owned_ops_playbook_path(
            maintenance_root,
        )),
        "# Ops playbook\n",
    );
    write_text(
        &fixture.join(contract_policy::packet_owned_workflow_plan_path(
            maintenance_root,
        )),
        "# CI workflows plan\n",
    );
    write_text(
        &fixture.join(contract_policy::packet_owned_prompt_template_path(
            maintenance_root,
        )),
        &contract_policy::packet_pr_prompt_template(entry, maintenance_root),
    );
}

fn opencode_automated_request_toml(basis_ref: &str) -> String {
    automated_request_toml("opencode", basis_ref)
        .replace(
            ".github/workflows/codex-cli-update-snapshot.yml",
            OPENCODE_PACKET_PR_WORKFLOW,
        )
        .replace(
            "source_ref = \"openai/codex\"",
            "source_ref = \"anomalyco/opencode\"",
        )
        .replace(
            "dispatch_kind = \"workflow_dispatch\"",
            "dispatch_kind = \"packet_pr\"",
        )
        .replace(
            "dispatch_workflow = \"codex-cli-update-snapshot.yml\"",
            "dispatch_workflow = \"agent-maintenance-open-pr.yml\"",
        )
}

fn opencode_automated_request_with_execution_contract_toml(
    fixture: &std::path::Path,
    basis_ref: &str,
) -> String {
    let registry = agent_registry::AgentRegistry::load(fixture).expect("load registry");
    let entry = registry.find("opencode").expect("opencode registry entry");
    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    let maintenance_root = "docs/agents/lifecycle/opencode-maintenance";
    let branch_name = "automation/opencode-maintenance-0.98.0";
    let contract = contract_policy::build_execution_contract(
        fixture,
        entry,
        request_path,
        maintenance_root,
        OPENCODE_PACKET_PR_WORKFLOW,
        "0.98.0",
        branch_name,
    )
    .expect("build packet-pr execution contract");

    let quote_array = |values: &[String]| -> String {
        values
            .iter()
            .map(|value| format!("  \"{value}\","))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        concat!(
            "{}\n",
            "[execution_contract]\n",
            "executor = \"{}\"\n",
            "prompt_template_path = \"{}\"\n",
            "prompt_sha256 = \"{}\"\n",
            "pr_summary_path = \"{}\"\n",
            "closeout_path = \"{}\"\n",
            "requires_manual_closeout = {}\n",
            "writable_surfaces = [\n{}\n]\n",
            "read_only_inputs = [\n{}\n]\n",
            "ordered_commands = [\n{}\n]\n",
            "green_gates = [\n{}\n]\n",
            "\n",
            "[execution_contract.recovery]\n",
            "recreate_packet_command = \"{}\"\n",
            "reopen_pr_body_path = \"{}\"\n",
            "reopen_pr_branch = \"{}\"\n",
            "notes = [\n{}\n]\n"
        ),
        opencode_automated_request_toml(basis_ref),
        contract.executor,
        contract.prompt_template_path,
        contract.prompt_sha256,
        contract.pr_summary_path,
        contract.closeout_path,
        if contract.requires_manual_closeout {
            "true"
        } else {
            "false"
        },
        quote_array(&contract.writable_surfaces),
        quote_array(&contract.read_only_inputs),
        quote_array(&contract.ordered_commands),
        quote_array(&contract.green_gates),
        contract.recovery.recreate_packet_command,
        contract.recovery.reopen_pr_body_path,
        contract.recovery.reopen_pr_branch,
        quote_array(&contract.recovery.notes),
    )
}

#[test]
fn automated_request_v2_with_detected_release_parses() {
    let fixture = fixture_root("agent-maintenance-automated-request-v2");
    seed_publication_inputs(&fixture);
    seed_opencode_packet_pr_workflow(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &opencode_automated_request_toml(
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let request = load_request(&fixture, Path::new(request_path)).expect("load request");
    assert!(request.is_automated_watch_request());
    let detected = request.detected_release.expect("detected release");
    assert_eq!(detected.target_version, "0.98.0");
    assert_eq!(detected.dispatch_workflow, "agent-maintenance-open-pr.yml");
    assert_eq!(
        detected.branch_name,
        "automation/opencode-maintenance-0.98.0"
    );
}

#[test]
fn automated_request_with_execution_contract_parses_and_validates() {
    let fixture = fixture_root("agent-maintenance-automated-request-with-contract");
    seed_publication_inputs(&fixture);
    seed_opencode_packet_pr_workflow(&fixture);
    seed_opencode_packet_owned_contract_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &opencode_automated_request_with_execution_contract_toml(
            &fixture,
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
        "docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md"
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
    assert!(contract
        .writable_surfaces
        .contains(&"crates/agent_api/src/runtime_support_data.rs".to_string()));
}

#[test]
fn historical_automated_request_without_execution_contract_still_loads_for_read_only() {
    let fixture = fixture_root("agent-maintenance-automated-request-historical-read-only");
    seed_publication_inputs(&fixture);
    seed_opencode_packet_pr_workflow(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &opencode_automated_request_toml(
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
    seed_opencode_packet_pr_workflow(&fixture);
    seed_opencode_packet_owned_contract_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &opencode_automated_request_with_execution_contract_toml(
            &fixture,
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
    seed_opencode_packet_pr_workflow(&fixture);
    seed_opencode_packet_owned_contract_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &opencode_automated_request_with_execution_contract_toml(
            &fixture,
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
    seed_opencode_packet_pr_workflow(&fixture);
    seed_opencode_packet_owned_contract_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &opencode_automated_request_with_execution_contract_toml(
            &fixture,
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
    seed_opencode_packet_pr_workflow(&fixture);
    seed_opencode_packet_owned_contract_inputs(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &opencode_automated_request_with_execution_contract_toml(
            &fixture,
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
    seed_opencode_packet_pr_workflow(&fixture);

    let request_path =
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml";
    write_text(
        &fixture.join(request_path),
        &opencode_automated_request_toml(
            "docs/integrations/opencode/governance/seam-2-closeout.md",
        ),
    );

    let plan = build_refresh_plan(&fixture, Path::new(request_path)).expect("build refresh plan");
    assert!(plan
        .planned_paths()
        .contains(&"docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md"));
    assert!(plan
        .planned_paths()
        .contains(&"docs/agents/lifecycle/opencode-maintenance/OPS_PLAYBOOK.md"));
    assert!(plan
        .planned_paths()
        .contains(&"docs/agents/lifecycle/opencode-maintenance/CI_WORKFLOWS_PLAN.md"));
    assert!(plan.planned_paths().contains(
        &"docs/agents/lifecycle/opencode-maintenance/governance/execute-agent-maintenance-prompt.md"
    ));

    let handoff = planned_utf8(
        &plan,
        "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md",
    );
    assert!(handoff.contains("This file is the canonical contributor execution contract"));
    assert!(handoff.contains(
        "<!-- generated-by: xtask agent-maintenance renderer; source-of-truth: governance/maintenance-request.toml -->"
    ));
    assert!(handoff.contains("maintained agent packet: `opencode`"));
    assert!(handoff
        .contains("local execution host: `local Codex CLI host via execute-agent-maintenance`"));
    assert!(handoff.contains("## Writable surfaces"));
    assert!(handoff.contains("## Read-only inputs"));
    assert!(handoff.contains("## Ordered repo commands"));
    assert!(handoff.contains("## Exact green gates"));
    assert!(handoff.contains("## Exact closeout command"));
    assert!(handoff.contains("## Exact maintained-agent prompt"));
    assert!(handoff
        .contains("Execute the automated maintenance packet for `opencode` target `0.98.0`."));
    assert!(handoff.contains(
        "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-request.toml"
    ));

    let pr_summary = planned_utf8(
        &plan,
        "docs/agents/lifecycle/opencode-maintenance/governance/pr-summary.md",
    );
    assert!(pr_summary.contains("Automated maintenance packet for `opencode` target `0.98.0`."));
    assert!(pr_summary
        .contains("Execute the automated maintenance packet for `opencode` target `0.98.0`."));
    assert!(pr_summary.contains("## Exact maintained-agent prompt"));
    assert!(!pr_summary.contains("## Work Queue Summary (autogenerated)"));
}
