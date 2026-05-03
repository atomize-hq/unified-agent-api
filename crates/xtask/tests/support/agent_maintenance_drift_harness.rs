use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use xtask::{release_doc, support_matrix};

use crate::harness::{
    seed_gemini_approval_artifact, seed_release_touchpoints, sha256_hex, write_text,
};

const GEMINI_APPROVAL_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml";
const GEMINI_LIFECYCLE_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json";
const GEMINI_PUBLICATION_READY_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json";
const GEMINI_CLOSEOUT_PATH: &str =
    "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json";
const GEMINI_PACK_PREFIX: &str = "gemini-cli-onboarding";
const GEMINI_REQUIRED_COMMANDS: &[&str] = &[
    "cargo run -p xtask -- support-matrix --check",
    "cargo run -p xtask -- capability-matrix --check",
    "cargo run -p xtask -- capability-matrix-audit",
    "make preflight",
];
const GEMINI_REQUIRED_PUBLICATION_OUTPUTS: &[&str] = &[
    "cli_manifests/support_matrix/current.json",
    "docs/specs/unified-agent-api/support-matrix.md",
    "docs/specs/unified-agent-api/capability-matrix.md",
];
const GEMINI_HISTORICAL_RUNTIME_EVIDENCE_PATHS: &[&str] = &[
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/input-contract.json",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/run-status.json",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/run-summary.md",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/validation-report.json",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/written-paths.json",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-gemini_cli-runtime-follow-on/handoff.json",
];

pub fn seed_publication_inputs(root: &Path) {
    write_text(
        &root.join("Cargo.toml"),
        "[workspace]\nmembers = [\n  \"crates/agent_api\",\n  \"crates/codex\",\n  \"crates/claude_code\",\n  \"crates/opencode\",\n  \"crates/gemini_cli\",\n  \"crates/aider\",\n  \"crates/wrapper_events\",\n  \"crates/xtask\",\n]\n",
    );
    seed_release_touchpoints(root);
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nManual contract text.\n",
    );

    seed_publishable_workspace_member(root, "crates/gemini_cli", "unified-agent-api-gemini-cli");
    seed_cli_manifest_root(
        root,
        "cli_manifests/codex",
        &["x86_64-unknown-linux-musl"],
        &[(&["mcp", "list"], &["x86_64-unknown-linux-musl"])],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/claude_code",
        &["linux-x64"],
        &[(&["mcp", "list"], &["linux-x64"])],
    );
    seed_cli_manifest_root(root, "cli_manifests/opencode", &["linux-x64"], &[]);
    seed_cli_manifest_root(root, "cli_manifests/gemini_cli", &["darwin-arm64"], &[]);
    seed_cli_manifest_root(root, "cli_manifests/aider", &["darwin-arm64"], &[]);

    let support_bundle =
        support_matrix::generate_publication_artifacts(root).expect("generate support publication");
    write_text(
        &root.join("cli_manifests/support_matrix/current.json"),
        &support_bundle.json,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        &support_bundle.markdown,
    );

    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        &default_capability_matrix_markdown(),
    );

    seed_gemini_approval_artifact(
        root,
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/approved-agent.toml",
        "gemini-cli-onboarding",
    );

    let release_doc = release_doc::render_release_doc(root).expect("render release doc");
    write_text(&root.join(release_doc::RELEASE_DOC_PATH), &release_doc);
}

fn gemini_implementation_summary() -> serde_json::Value {
    serde_json::json!({
        "requested_runtime_profile": "default",
        "achieved_runtime_profile": "default",
        "primary_template": "gemini_cli",
        "template_lineage": ["gemini_cli"],
        "landed_surfaces": [
            "wrapper_runtime",
            "backend_harness",
            "runtime_manifest_evidence"
        ],
        "deferred_surfaces": [],
        "minimal_profile_justification": serde_json::Value::Null
    })
}

fn gemini_publication_packet(
    approval_sha: &str,
    lifecycle_rel: &str,
    lifecycle_sha: &str,
    publication_owned_paths: &[&str],
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": "1",
        "agent_id": "gemini_cli",
        "approval_artifact_path": GEMINI_APPROVAL_PATH,
        "approval_artifact_sha256": approval_sha,
        "lifecycle_state_path": lifecycle_rel,
        "lifecycle_state_sha256": lifecycle_sha,
        "lifecycle_stage": "publication_ready",
        "support_tier_at_emit": "baseline_runtime",
        "manifest_root": "cli_manifests/gemini_cli",
        "expected_targets": ["darwin-arm64"],
        "capability_publication_enabled": true,
        "support_publication_enabled": true,
        "capability_matrix_target": serde_json::Value::Null,
        "required_commands": GEMINI_REQUIRED_COMMANDS,
        "required_publication_outputs": GEMINI_REQUIRED_PUBLICATION_OUTPUTS,
        "runtime_evidence_paths": GEMINI_HISTORICAL_RUNTIME_EVIDENCE_PATHS,
        "publication_owned_paths": publication_owned_paths,
        "blocking_issues": [],
        "retryable_failures": [],
        "active_runtime_evidence_run_id": serde_json::Value::Null,
        "implementation_summary": gemini_implementation_summary()
    })
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum LifecycleBaselineGap {
    None,
    MissingPublicationPacketPath,
    MissingPublicationPacketSha,
    MissingCloseoutBaselinePath,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum RuntimeEvidenceTruth {
    Stale,
    Truthful,
}

#[allow(dead_code)]
pub fn seed_gemini_lifecycle_baseline(root: &Path, gap: LifecycleBaselineGap) {
    let approval_sha = sha256_hex(&root.join(GEMINI_APPROVAL_PATH));
    let lifecycle_path = root.join(GEMINI_LIFECYCLE_PATH);
    let packet_path = root.join(GEMINI_PUBLICATION_READY_PATH);
    let closeout_path = root.join(GEMINI_CLOSEOUT_PATH);

    write_text(
        &packet_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&gemini_publication_packet(
                &approval_sha,
                GEMINI_LIFECYCLE_PATH,
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                &[GEMINI_LIFECYCLE_PATH, GEMINI_PUBLICATION_READY_PATH],
            ))
            .expect("serialize publication-ready packet")
        ),
    );
    write_text(
        &closeout_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "state": "closed",
                "approval_ref": GEMINI_APPROVAL_PATH,
                "approval_sha256": approval_sha,
                "approval_source": "historical-lifecycle-backfill",
                "manual_control_plane_edits": 0,
                "partial_write_incidents": 0,
                "ambiguous_ownership_incidents": 0,
                "duration_missing_reason": "Exact duration not recoverable from committed evidence.",
                "explicit_none_reason": "No residual friction remained in the committed proving-run evidence.",
                "preflight_passed": true,
                "recorded_at": "2026-04-21T11:23:09Z",
                "commit": "6b7d5f6e9cf2bf54933659f5700bb59d1f8a95e8"
            }))
            .expect("serialize closeout")
        ),
    );
    let packet_sha = sha256_hex(&packet_path);
    let publication_packet_path = match gap {
        LifecycleBaselineGap::MissingPublicationPacketPath => serde_json::Value::Null,
        _ => serde_json::Value::String(
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json"
                .to_string(),
        ),
    };
    let publication_packet_sha = match gap {
        LifecycleBaselineGap::MissingPublicationPacketSha => serde_json::Value::Null,
        _ => serde_json::Value::String(packet_sha),
    };
    let closeout_baseline_path = match gap {
        LifecycleBaselineGap::MissingCloseoutBaselinePath => serde_json::Value::Null,
        _ => serde_json::Value::String(
            "docs/agents/lifecycle/gemini-cli-onboarding/governance/proving-run-closeout.json"
                .to_string(),
        ),
    };
    write_text(
        &lifecycle_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "1",
                "agent_id": "gemini_cli",
                "onboarding_pack_prefix": GEMINI_PACK_PREFIX,
                "approval_artifact_path": GEMINI_APPROVAL_PATH,
                "approval_artifact_sha256": approval_sha,
                "lifecycle_stage": "closed_baseline",
                "support_tier": "publication_backed",
                "side_states": [],
                "current_owner_command": "close-proving-run --write",
                "expected_next_command": "check-agent-drift --agent gemini_cli",
                "last_transition_at": "2026-04-21T11:23:09Z",
                "last_transition_by": "historical-lifecycle-backfill",
                "required_evidence": [
                    "registry_entry",
                    "docs_pack",
                    "manifest_root_skeleton",
                    "runtime_write_complete",
                    "implementation_summary_present",
                    "publication_packet_written",
                    "support_matrix_check_green",
                    "capability_matrix_check_green",
                    "capability_matrix_audit_green",
                    "preflight_green",
                    "proving_run_closeout_written"
                ],
                "satisfied_evidence": [
                    "registry_entry",
                    "docs_pack",
                    "manifest_root_skeleton",
                    "runtime_write_complete",
                    "implementation_summary_present",
                    "publication_packet_written",
                    "support_matrix_check_green",
                    "capability_matrix_check_green",
                    "capability_matrix_audit_green",
                    "preflight_green",
                    "proving_run_closeout_written"
                ],
                "blocking_issues": [],
                "retryable_failures": [],
                "active_runtime_evidence_run_id": serde_json::Value::Null,
                "implementation_summary": gemini_implementation_summary(),
                "publication_packet_path": publication_packet_path,
                "publication_packet_sha256": publication_packet_sha,
                "closeout_baseline_path": closeout_baseline_path
            }))
            .expect("serialize lifecycle state")
        ),
    );
}

#[allow(dead_code)]
pub fn seed_gemini_published_baseline(root: &Path) {
    let approval_sha = sha256_hex(&root.join(GEMINI_APPROVAL_PATH));
    let lifecycle_path = root.join(GEMINI_LIFECYCLE_PATH);
    let packet_path = root.join(GEMINI_PUBLICATION_READY_PATH);

    let packet_json = |lifecycle_sha: &str| {
        gemini_publication_packet(
            &approval_sha,
            GEMINI_LIFECYCLE_PATH,
            lifecycle_sha,
            &[GEMINI_LIFECYCLE_PATH, GEMINI_PUBLICATION_READY_PATH],
        )
    };
    let lifecycle_json = |packet_sha: &str| {
        serde_json::json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "onboarding_pack_prefix": GEMINI_PACK_PREFIX,
            "approval_artifact_path": GEMINI_APPROVAL_PATH,
            "approval_artifact_sha256": approval_sha,
            "lifecycle_stage": "published",
            "support_tier": "publication_backed",
            "side_states": [],
            "current_owner_command": "refresh-publication --write",
            "expected_next_command": format!(
                "close-proving-run --approval {GEMINI_APPROVAL_PATH} --closeout {GEMINI_CLOSEOUT_PATH}"
            ),
            "last_transition_at": "2026-04-21T11:23:09Z",
            "last_transition_by": "xtask refresh-publication --write",
            "required_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton",
                "runtime_write_complete",
                "implementation_summary_present",
                "publication_packet_written",
                "support_matrix_check_green",
                "capability_matrix_check_green",
                "capability_matrix_audit_green",
                "preflight_green"
            ],
            "satisfied_evidence": [
                "registry_entry",
                "docs_pack",
                "manifest_root_skeleton",
                "runtime_write_complete",
                "implementation_summary_present",
                "publication_packet_written",
                "support_matrix_check_green",
                "capability_matrix_check_green",
                "capability_matrix_audit_green",
                "preflight_green"
            ],
            "blocking_issues": [],
            "retryable_failures": [],
            "active_runtime_evidence_run_id": serde_json::Value::Null,
            "implementation_summary": gemini_implementation_summary(),
            "publication_packet_path": GEMINI_PUBLICATION_READY_PATH,
            "publication_packet_sha256": packet_sha,
            "closeout_baseline_path": serde_json::Value::Null
        })
    };

    write_text(
        &packet_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&packet_json(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ))
            .expect("serialize published packet")
        ),
    );
    let mut packet_sha = sha256_hex(&packet_path);
    write_text(
        &lifecycle_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&lifecycle_json(&packet_sha))
                .expect("serialize published lifecycle")
        ),
    );
    let lifecycle_sha = sha256_hex(&lifecycle_path);
    write_text(
        &packet_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&packet_json(&lifecycle_sha))
                .expect("serialize published packet")
        ),
    );
    packet_sha = sha256_hex(&packet_path);
    write_text(
        &lifecycle_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&lifecycle_json(&packet_sha))
                .expect("serialize published lifecycle")
        ),
    );
}

pub fn seed_gemini_runtime_integrated_state(root: &Path) {
    let approval_sha = sha256_hex(&root.join(GEMINI_APPROVAL_PATH));
    write_text(
        &root.join(GEMINI_LIFECYCLE_PATH),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": "1",
                "agent_id": "gemini_cli",
                "onboarding_pack_prefix": GEMINI_PACK_PREFIX,
                "approval_artifact_path": GEMINI_APPROVAL_PATH,
                "approval_artifact_sha256": approval_sha,
                "lifecycle_stage": "runtime_integrated",
                "support_tier": "baseline_runtime",
                "side_states": [],
                "current_owner_command": "runtime-follow-on --write",
                "expected_next_command": format!(
                    "prepare-publication --approval {GEMINI_APPROVAL_PATH} --write"
                ),
                "last_transition_at": "2026-05-01T00:00:00Z",
                "last_transition_by": "agent-maintenance-drift-test",
                "required_evidence": [
                    "registry_entry",
                    "docs_pack",
                    "manifest_root_skeleton",
                    "runtime_write_complete",
                    "implementation_summary_present"
                ],
                "satisfied_evidence": [
                    "registry_entry",
                    "docs_pack",
                    "manifest_root_skeleton",
                    "runtime_write_complete",
                    "implementation_summary_present"
                ],
                "blocking_issues": [],
                "retryable_failures": [],
                "active_runtime_evidence_run_id": "repair-gemini_cli-runtime-follow-on",
                "implementation_summary": gemini_implementation_summary(),
                "publication_packet_path": serde_json::Value::Null,
                "publication_packet_sha256": serde_json::Value::Null,
                "closeout_baseline_path": serde_json::Value::Null
            }))
            .expect("serialize runtime integrated lifecycle state")
        ),
    );
}

pub fn seed_runtime_evidence_run(root: &Path, truth: RuntimeEvidenceTruth) {
    let approval_sha = sha256_hex(&root.join(GEMINI_APPROVAL_PATH));
    let run_root = root
        .join("docs/agents/.uaa-temp/runtime-follow-on/runs/repair-gemini_cli-runtime-follow-on");
    let input_sha = match truth {
        RuntimeEvidenceTruth::Stale => {
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
        }
        RuntimeEvidenceTruth::Truthful => &approval_sha,
    };

    write_text(
        &run_root.join("run-status.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "approval_artifact_path": GEMINI_APPROVAL_PATH,
                "agent_id": "gemini_cli",
                "status": "write_validated",
                "validation_passed": true,
                "handoff_ready": true,
                "run_dir": run_root.to_string_lossy(),
            }))
            .expect("serialize runtime run status")
        ),
    );
    write_text(
        &run_root.join("input-contract.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "approval_artifact_path": GEMINI_APPROVAL_PATH,
                "approval_artifact_sha256": input_sha,
                "agent_id": "gemini_cli",
                "manifest_root": "cli_manifests/gemini_cli",
                "required_handoff_commands": GEMINI_REQUIRED_COMMANDS
            }))
            .expect("serialize runtime input contract")
        ),
    );
    write_text(
        &run_root.join("validation-report.json"),
        "{\n  \"status\": \"pass\"\n}\n",
    );
    write_text(
        &run_root.join("handoff.json"),
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::json!({
                "agent_id": "gemini_cli",
                "manifest_root": "cli_manifests/gemini_cli",
                "runtime_lane_complete": true,
                "publication_refresh_required": true,
                "required_commands": GEMINI_REQUIRED_COMMANDS,
                "blockers": []
            }))
            .expect("serialize runtime handoff")
        ),
    );
    write_text(
        &run_root.join("written-paths.json"),
        concat!(
            "[\n",
            "  \"crates/gemini_cli/src/lib.rs\",\n",
            "  \"cli_manifests/gemini_cli/snapshots/default.json\"\n",
            "]\n"
        ),
    );
    write_text(
        &run_root.join("run-summary.md"),
        "# Runtime follow-on\n\nCommitted runtime evidence recorded.\n",
    );
}

#[allow(dead_code)]
pub fn seed_governance_closeouts(
    root: &Path,
    opencode_capabilities: &[&str],
    support_unsupported: bool,
) {
    let capability_lines = opencode_capabilities
        .iter()
        .map(|capability| format!("`{capability}`"))
        .collect::<Vec<_>>()
        .join(", ");
    write_text(
        &root.join("docs/integrations/opencode/governance/seam-2-closeout.md"),
        &format!(
            "# Closeout\n\n- capability advertisement is intentionally conservative and now matches the landed backend contract and generated capability inventory:\n  <!-- xtask-governance-check:opencode-capabilities:start -->\n  {capability_lines}\n  <!-- xtask-governance-check:opencode-capabilities:end -->\n  are the claimed OpenCode v1 capability ids under the current runtime evidence\n"
        ),
    );

    let seam3_text = if support_unsupported {
        "# Closeout\n\n- the support publication artifacts now show OpenCode as manifest-supported only where committed root evidence justifies it, while\n  <!-- xtask-governance-check:opencode-support:start -->\n  backend_support = unsupported\n  uaa_support = unsupported\n  <!-- xtask-governance-check:opencode-support:end -->\n  under the current backend evidence and pointer posture\n"
    } else {
        "# Closeout\n\n- the support publication artifacts now show OpenCode as manifest-supported only where committed root evidence justifies it, while\n  <!-- xtask-governance-check:opencode-support:start -->\n  backend_support = supported\n  uaa_support = supported\n  <!-- xtask-governance-check:opencode-support:end -->\n  under the current backend evidence and pointer posture\n"
    };
    write_text(
        &root.join("docs/integrations/opencode/governance/seam-3-closeout.md"),
        seam3_text,
    );
}

fn default_capability_matrix_markdown() -> String {
    include_str!("../../../../docs/specs/unified-agent-api/capability-matrix.md").to_string()
}

#[allow(dead_code)]
pub fn seed_closed_governance_maintenance(root: &Path, resolved_surface: &str) {
    write_text(
        &root.join(
            "docs/agents/lifecycle/opencode-maintenance/governance/maintenance-closeout.json",
        ),
        &serde_json::to_string_pretty(&serde_json::json!({
            "resolved_findings": [{
                "category_id": "governance_doc_drift",
                "surfaces": [resolved_surface],
            }]
        }))
        .expect("serialize maintenance closeout"),
    );
}

#[allow(dead_code)]
pub fn run_xtask_check_agent_drift(fixture_root: &Path, agent_id: &str) -> std::process::Output {
    let xtask_bin = PathBuf::from(env!("CARGO_BIN_EXE_xtask"));
    Command::new(xtask_bin)
        .arg("check-agent-drift")
        .arg("--agent")
        .arg(agent_id)
        .current_dir(fixture_root)
        .output()
        .expect("spawn xtask check-agent-drift")
}

#[allow(dead_code)]
pub fn release_doc_with_packages(packages: &[&str]) -> String {
    let packages = packages
        .iter()
        .map(|package| package.to_string())
        .collect::<Vec<_>>();
    release_doc::splice_release_doc_block(
        "# Release docs\n\nManual contract text.\n",
        &release_doc::render_release_doc_block(&packages),
    )
}

#[allow(dead_code)]
pub fn set_support_matrix_enabled(root: &Path, agent_id: &str, enabled: bool) {
    let registry_path = root.join("crates/xtask/data/agent_registry.toml");
    let registry = fs::read_to_string(&registry_path).expect("read registry");
    let agent_marker = format!("agent_id = \"{agent_id}\"");
    let agent_start = registry.find(&agent_marker).expect("agent in registry");
    let support_marker = "support_matrix_enabled = true";
    let support_start = registry[agent_start..]
        .find(support_marker)
        .map(|offset| agent_start + offset)
        .expect("support flag after agent entry");
    let mut updated = registry;
    updated.replace_range(
        support_start..support_start + support_marker.len(),
        if enabled {
            "support_matrix_enabled = true"
        } else {
            "support_matrix_enabled = false"
        },
    );
    write_text(&registry_path, &updated);
}

fn seed_publishable_workspace_member(root: &Path, member_path: &str, package_name: &str) {
    write_text(
        &root.join(member_path).join("Cargo.toml"),
        &format!("[package]\nname = \"{package_name}\"\nversion = \"0.2.3\"\nedition = \"2021\"\n"),
    );
}

pub fn seed_cli_manifest_root(
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

    let version = serde_json::json!({
        "semantic_version": "1.0.0",
        "status": "supported",
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
                serde_json::to_string_pretty(&report).expect("serialize report")
            ),
        );
    }
}
