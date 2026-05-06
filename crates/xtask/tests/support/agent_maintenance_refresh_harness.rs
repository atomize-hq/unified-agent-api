use std::{collections::BTreeSet, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::harness::{seed_release_touchpoints, write_text};

pub fn normalize_support_matrix_fixture(root: &Path) {
    for manifest_root in [
        "cli_manifests/codex",
        "cli_manifests/claude_code",
        "cli_manifests/opencode",
        "cli_manifests/gemini_cli",
        "cli_manifests/aider",
    ] {
        let current_path = root.join(manifest_root).join("current.json");
        let mut current: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&current_path).expect("read current.json"))
                .expect("parse current.json");
        let expected_targets = current["expected_targets"]
            .as_array()
            .expect("expected_targets array")
            .iter()
            .map(|value| value.as_str().expect("target string").to_string())
            .collect::<Vec<_>>();
        current["inputs"] = serde_json::Value::Array(
            expected_targets
                .iter()
                .map(|target| {
                    serde_json::json!({
                        "target_triple": target,
                        "binary": { "semantic_version": "1.0.0" }
                    })
                })
                .collect(),
        );
        write_text(
            &current_path,
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&current).expect("serialize current.json")
            ),
        );

        let version_path = root.join(manifest_root).join("versions/1.0.0.json");
        let mut version: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&version_path).expect("read version.json"))
                .expect("parse version.json");
        version["status"] = serde_json::Value::String("validated".to_string());
        write_text(
            &version_path,
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&version).expect("serialize version.json")
            ),
        );
    }
}

pub fn seed_publication_inputs(root: &Path) {
    seed_release_touchpoints(root);
    write_text(
        &root.join("docs/integrations/opencode/governance/seam-2-closeout.md"),
        "# Closeout\n\nThis stale capability claim triggered maintenance.\n",
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nManual contract text.\n",
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        "# Capability matrix\n\nStale publication.\n",
    );
    seed_publishable_workspace_member(root, "crates/gemini_cli", "unified-agent-api-gemini-cli");
    seed_cli_manifest_root(
        root,
        "cli_manifests/codex",
        &["x86_64-unknown-linux-musl"],
        &[
            (&["mcp", "list"], &["x86_64-unknown-linux-musl"]),
            (&["mcp", "get"], &["x86_64-unknown-linux-musl"]),
            (&["mcp", "add"], &["x86_64-unknown-linux-musl"]),
            (&["mcp", "remove"], &["x86_64-unknown-linux-musl"]),
        ],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/claude_code",
        &["linux-x64", "darwin-arm64", "win32-x64"],
        &[(
            &["mcp", "list"],
            &["linux-x64", "darwin-arm64", "win32-x64"],
        )],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/opencode",
        &["linux-x64", "darwin-arm64", "win32-x64"],
        &[],
    );
    seed_cli_manifest_root(root, "cli_manifests/gemini_cli", &["darwin-arm64"], &[]);
    seed_cli_manifest_root(root, "cli_manifests/aider", &["darwin-arm64"], &[]);
}

pub fn request_toml(
    agent_id: &str,
    actions: &[&str],
    runtime_required: bool,
    runtime_items: &[&str],
) -> String {
    request_toml_with_refs(
        agent_id,
        "docs/integrations/opencode/governance/seam-2-closeout.md",
        "docs/integrations/opencode/governance/seam-2-closeout.md",
        actions,
        runtime_required,
        runtime_items,
    )
}

pub fn request_toml_with_refs(
    agent_id: &str,
    basis_ref: &str,
    opened_from: &str,
    actions: &[&str],
    runtime_required: bool,
    runtime_items: &[&str],
) -> String {
    let actions_block = actions
        .iter()
        .map(|action| format!("  \"{action}\","))
        .collect::<Vec<_>>()
        .join("\n");
    let runtime_items_block = if runtime_items.is_empty() {
        String::new()
    } else {
        runtime_items
            .iter()
            .map(|item| format!("  \"{item}\","))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        concat!(
            "artifact_version = \"1\"\n",
            "agent_id = \"{agent_id}\"\n",
            "trigger_kind = \"drift_detected\"\n",
            "basis_ref = \"{basis_ref}\"\n",
            "opened_from = \"{opened_from}\"\n",
            "requested_control_plane_actions = [\n",
            "{actions_block}\n",
            "]\n",
            "request_recorded_at = \"2026-04-22T01:15:00Z\"\n",
            "request_commit = \"1adb8f1\"\n",
            "\n",
            "[runtime_followup_required]\n",
            "required = {runtime_required}\n",
            "items = [\n",
            "{runtime_items_block}\n",
            "]\n"
        ),
        agent_id = agent_id,
        basis_ref = basis_ref,
        opened_from = opened_from,
        actions_block = actions_block,
        runtime_required = if runtime_required { "true" } else { "false" },
        runtime_items_block = runtime_items_block
    )
}

pub fn automated_request_toml(agent_id: &str, basis_ref: &str) -> String {
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

pub fn automated_request_with_execution_contract_toml(agent_id: &str, basis_ref: &str) -> String {
    let prompt = "# Goal\n\nFollow the maintained PR template for 0.98.0.\n";
    let prompt_sha256 = hex::encode(Sha256::digest(prompt.as_bytes()));

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
            "recreate_packet_command = \"cargo run -p xtask -- prepare-agent-maintenance --request docs/agents/lifecycle/{agent_id}-maintenance/governance/maintenance-request.toml --write\"\n",
            "reopen_pr_body_path = \"docs/agents/lifecycle/{agent_id}-maintenance/governance/pr-summary.md\"\n",
            "reopen_pr_branch = \"automation/{agent_id}-maintenance-0.98.0\"\n",
            "notes = [\n",
            "  \"If PR creation fails after packet generation, rerun packet creation and reopen the PR from the generated pr-summary path.\",\n",
            "  \"If local Codex preflight fails, fix binary/auth and rerun execute-agent-maintenance --dry-run before write mode.\",\n",
            "]\n"
        ),
        automated_request_toml(agent_id, basis_ref),
        agent_id = agent_id,
        prompt_sha256 = prompt_sha256
    )
}

pub fn diff_paths(
    before: &std::collections::BTreeMap<String, Vec<u8>>,
    after: &std::collections::BTreeMap<String, Vec<u8>>,
) -> BTreeSet<String> {
    before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|path| before.get(path) != after.get(path))
        .collect()
}

pub fn planned_utf8(plan: &crate::refresh::RefreshPlan, relative_path: &str) -> String {
    String::from_utf8(
        plan.files
            .iter()
            .find(|file| file.relative_path == relative_path)
            .unwrap_or_else(|| panic!("missing planned file {relative_path}"))
            .contents
            .clone(),
    )
    .expect("utf8 planned file")
}

fn seed_publishable_workspace_member(root: &Path, member_path: &str, package_name: &str) {
    write_text(
        &root.join(member_path).join("Cargo.toml"),
        &format!("[package]\nname = \"{package_name}\"\nversion = \"0.2.3\"\nedition = \"2021\"\n"),
    );
}

fn seed_cli_manifest_root(
    root: &Path,
    manifest_root: &str,
    canonical_targets: &[&str],
    commands: &[(&[&str], &[&str])],
) {
    let current = serde_json::json!({
        "expected_targets": canonical_targets,
        "inputs": [{
            "target_triple": canonical_targets[0],
            "binary": { "semantic_version": "1.0.0" }
        }],
        "commands": commands
            .iter()
            .map(|(path, available_on)| serde_json::json!({
                "path": path,
                "available_on": available_on,
            }))
            .collect::<Vec<_>>(),
    });
    write_text(
        &root.join(manifest_root).join("current.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&current).expect("serialize current manifest")
        ),
    );
    write_text(
        &root.join(manifest_root).join("PR_BODY_TEMPLATE.md"),
        "# Goal\n\nFollow the maintained PR template for {{VERSION}}.\n",
    );
    write_text(
        &root.join(manifest_root).join("OPS_PLAYBOOK.md"),
        "# Ops playbook\n",
    );
    write_text(
        &root.join(manifest_root).join("CI_WORKFLOWS_PLAN.md"),
        "# CI workflows plan\n",
    );

    let version = serde_json::json!({
        "semantic_version": "1.0.0",
        "status": "latest_validated",
        "coverage": { "supported_targets": canonical_targets },
    });
    write_text(
        &root.join(manifest_root).join("versions/1.0.0.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&version).expect("serialize version metadata")
        ),
    );

    for target in canonical_targets {
        write_text(
            &root
                .join(manifest_root)
                .join(format!("pointers/latest_supported/{target}.txt")),
            "1.0.0\n",
        );
        write_text(
            &root
                .join(manifest_root)
                .join(format!("pointers/latest_validated/{target}.txt")),
            "1.0.0\n",
        );
        let report = serde_json::json!({
            "inputs": { "upstream": { "targets": [target] } },
            "deltas": {
                "missing_commands": [],
                "missing_flags": [],
                "missing_args": [],
                "intentionally_unsupported": [],
                "wrapper_only_commands": [],
                "wrapper_only_flags": [],
                "wrapper_only_args": [],
            }
        });
        write_text(
            &root
                .join(manifest_root)
                .join(format!("reports/1.0.0/coverage.{target}.json")),
            &format!(
                "{}\n",
                serde_json::to_string_pretty(&report).expect("serialize support report")
            ),
        );
    }
}
