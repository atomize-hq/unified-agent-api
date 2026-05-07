use std::path::Path;

use serde_json::json;
use sha2::Digest;

use crate::{closeout, harness::sha256_hex};

pub fn maintenance_request_toml(agent_id: &str, basis_ref: &str) -> String {
    maintenance_request_toml_with_refs(agent_id, basis_ref, basis_ref)
}

pub fn maintenance_request_toml_with_refs(
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

pub fn automated_maintenance_request_toml(agent_id: &str, basis_ref: &str) -> String {
    format!(
        concat!(
            "artifact_version = \"2\"\n",
            "agent_id = \"{agent_id}\"\n",
            "trigger_kind = \"upstream_release_detected\"\n",
            "basis_ref = \"{basis_ref}\"\n",
            "opened_from = \".github/workflows/codex-cli-update-snapshot.yml\"\n",
            "requested_control_plane_actions = [\n",
            "  \"packet_doc_refresh\",\n",
            "]\n",
            "request_recorded_at = \"2026-05-05T15:00:00Z\"\n",
            "request_commit = \"abcdef1\"\n",
            "\n",
            "[runtime_followup_required]\n",
            "required = false\n",
            "items = []\n",
            "\n",
            "[detected_release]\n",
            "detected_by = \".github/workflows/agent-maintenance-release-watch.yml\"\n",
            "current_validated = \"0.97.0\"\n",
            "target_version = \"0.98.0\"\n",
            "latest_stable = \"0.99.0\"\n",
            "version_policy = \"latest_stable_minus_one\"\n",
            "source_kind = \"github_releases\"\n",
            "source_ref = \"openai/codex\"\n",
            "dispatch_kind = \"workflow_dispatch\"\n",
            "dispatch_workflow = \"codex-cli-update-snapshot.yml\"\n",
            "branch_name = \"automation/{agent_id}-maintenance-0.98.0\"\n"
        ),
        agent_id = agent_id,
        basis_ref = basis_ref
    )
}

pub fn automated_maintenance_request_with_execution_contract_toml(
    agent_id: &str,
    basis_ref: &str,
) -> String {
    let prompt = "# Goal\n\nFollow the maintained PR template for 0.98.0.\n";
    let prompt_sha256 = hex::encode(sha2::Sha256::digest(prompt.as_bytes()));

    format!(
        concat!(
            "{}\n",
            "[execution_contract]\n",
            "executor = \"codex\"\n",
            "prompt_template_path = \"cli_manifests/{agent_id}/PR_BODY_TEMPLATE.md\"\n",
            "prompt_sha256 = \"{prompt_sha256}\"\n",
            "pr_summary_path = \"docs/agents/lifecycle/{agent_id}-maintenance/governance/pr-summary.md\"\n",
            "closeout_path = \"docs/agents/lifecycle/{agent_id}-maintenance/governance/maintenance-closeout.json\"\n",
            "requires_manual_closeout = true\n",
            "writable_surfaces = [\n",
            "  \"docs/agents/lifecycle/{agent_id}-maintenance/**\",\n",
            "  \"crates/{agent_id}/**\",\n",
            "  \"crates/agent_api/**\",\n",
            "  \"cli_manifests/{agent_id}/artifacts.lock.json\",\n",
            "  \"cli_manifests/{agent_id}/snapshots/0.98.0/**\",\n",
            "  \"cli_manifests/{agent_id}/reports/0.98.0/**\",\n",
            "  \"cli_manifests/{agent_id}/versions/0.98.0.json\",\n",
            "  \"cli_manifests/{agent_id}/wrapper_coverage.json\",\n",
            "]\n",
            "read_only_inputs = [\n",
            "  \"cli_manifests/{agent_id}/OPS_PLAYBOOK.md\",\n",
            "  \"cli_manifests/{agent_id}/CI_WORKFLOWS_PLAN.md\",\n",
            "  \"cli_manifests/{agent_id}/PR_BODY_TEMPLATE.md\",\n",
            "  \".github/workflows/codex-cli-update-snapshot.yml\",\n",
            "]\n",
            "ordered_commands = [\n",
            "  \"cargo run -p xtask -- codex-validate --root cli_manifests/{agent_id}\",\n",
            "  \"cargo run -p xtask -- support-matrix --check\",\n",
            "  \"cargo run -p xtask -- capability-matrix --check\",\n",
            "  \"cargo run -p xtask -- capability-matrix-audit\",\n",
            "  \"make preflight\",\n",
            "]\n",
            "green_gates = [\n",
            "  \"cargo run -p xtask -- codex-validate --root cli_manifests/{agent_id}\",\n",
            "  \"cargo run -p xtask -- support-matrix --check\",\n",
            "  \"cargo run -p xtask -- capability-matrix --check\",\n",
            "  \"cargo run -p xtask -- capability-matrix-audit\",\n",
            "  \"make preflight\",\n",
            "]\n",
            "\n",
            "[execution_contract.recovery]\n",
            "recreate_packet_command = \"cargo run -p xtask -- refresh-agent --request docs/agents/lifecycle/{agent_id}-maintenance/governance/maintenance-request.toml --write\"\n",
            "reopen_pr_body_path = \"docs/agents/lifecycle/{agent_id}-maintenance/governance/pr-summary.md\"\n",
            "reopen_pr_branch = \"automation/{agent_id}-maintenance-0.98.0\"\n",
            "notes = [\n",
            "  \"If PR creation fails after packet generation, rerun packet creation and reopen the PR from the generated pr-summary path.\",\n",
            "  \"If local Codex preflight fails, fix binary/auth and rerun execute-agent-maintenance --dry-run before write mode.\",\n",
            "]\n"
        ),
        automated_maintenance_request_toml(agent_id, basis_ref),
        agent_id = agent_id,
        prompt_sha256 = prompt_sha256
    )
}

pub fn valid_closeout_json(request_absolute: &Path, request_path: &Path) -> String {
    serde_json::to_string_pretty(&valid_closeout(
        &request_path.display().to_string(),
        &sha256_hex(request_absolute),
    ))
    .expect("serialize closeout")
}

pub fn finding_json(category_id: &str, summary: &str, surfaces: &[&str]) -> serde_json::Value {
    json!({
        "category_id": category_id,
        "summary": summary,
        "surfaces": surfaces,
    })
}

pub fn valid_closeout(request_ref: &str, request_sha256: &str) -> serde_json::Value {
    json!({
        "request_ref": request_ref,
        "request_sha256": request_sha256,
        "resolved_findings": [finding_json(
            "governance_doc_drift",
            "SEAM-2 closeout now matches the landed capability advertisement boundary.",
            &[
                "docs/integrations/opencode/governance/seam-2-closeout.md",
                "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md"
            ],
        )],
        "explicit_none_reason": "No deferred maintenance findings remain after publication and packet refresh.",
        "preflight_passed": true,
        "recorded_at": "2026-04-22T01:45:00Z",
        "commit": "4adefdf"
    })
}

pub fn valid_closeout_struct(
    request_ref: &str,
    request_sha256: &str,
) -> closeout::MaintenanceCloseout {
    closeout::MaintenanceCloseout {
        request_ref: request_ref.to_string(),
        request_sha256: request_sha256.to_string(),
        resolved_findings: vec![closeout::MaintenanceFinding {
            category_id: closeout::MaintenanceDriftCategory::GovernanceDoc,
            summary: "SEAM-2 closeout now matches the landed capability advertisement boundary."
                .to_string(),
            surfaces: vec![
                "docs/integrations/opencode/governance/seam-2-closeout.md".to_string(),
                "docs/agents/lifecycle/opencode-maintenance/HANDOFF.md".to_string(),
            ],
        }],
        deferred_findings: closeout::DeferredFindingsTruth::Findings(vec![
            closeout::MaintenanceFinding {
                category_id: closeout::MaintenanceDriftCategory::GovernanceDoc,
                summary: "Governance drift remains deferred.".to_string(),
                surfaces: vec![
                    "docs/integrations/opencode/governance/seam-2-closeout.md".to_string()
                ],
            },
        ]),
        preflight_passed: true,
        recorded_at: "2026-04-22T01:45:00Z".to_string(),
        commit: "4adefdf".to_string(),
    }
}

pub fn closeout_with_deferred(
    closeout: closeout::MaintenanceCloseout,
) -> closeout::MaintenanceCloseout {
    closeout::MaintenanceCloseout {
        deferred_findings: closeout::DeferredFindingsTruth::Findings(vec![
            closeout::MaintenanceFinding {
                category_id: closeout::MaintenanceDriftCategory::GovernanceDoc,
                summary: "Governance drift remains deferred.".to_string(),
                surfaces: vec![
                    "docs/integrations/opencode/governance/seam-2-closeout.md".to_string()
                ],
            },
        ]),
        ..closeout
    }
}
