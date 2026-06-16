use std::{fs, path::Path};

use serde_json::json;
use xtask::{agent_lifecycle, support_matrix};

use crate::harness::{sha256_hex, write_text};

pub(super) fn seed_green_publication_surfaces(root: &Path) {
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        "# Support matrix\n\nManual contract text.\n",
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/codex",
        &["aarch64-apple-darwin", "x86_64-unknown-linux-musl"],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/claude_code",
        &["linux-x64", "darwin-arm64", "win32-x64"],
    );
    seed_cli_manifest_root(
        root,
        "cli_manifests/opencode",
        &["linux-x64", "darwin-arm64", "win32-x64"],
    );
    seed_cli_manifest_root(root, "cli_manifests/gemini_cli", &["darwin-arm64"]);
    seed_cli_manifest_root(root, "cli_manifests/aider", &["darwin-arm64"]);

    let bundle =
        support_matrix::generate_publication_artifacts(root).expect("generate support publication");
    write_text(
        &root.join("cli_manifests/support_matrix/current.json"),
        &bundle.json,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/support-matrix.md"),
        &bundle.markdown,
    );
    write_text(
        &root.join("crates/agent_api/src/runtime_support_data.rs"),
        &bundle.runtime_support_data,
    );
    write_text(
        &root.join("docs/specs/unified-agent-api/capability-matrix.md"),
        include_str!("../../../../docs/specs/unified-agent-api/capability-matrix.md"),
    );
    seed_lifecycle_eligible_audit_peers(root);
}

pub(super) fn seed_published_baseline(root: &Path, approval_path: &str) {
    let approval_sha256 = sha256_hex(&root.join(approval_path));
    let lifecycle_path =
        root.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json");
    let packet_path =
        root.join("docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json");
    let packet_rel =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/publication-ready.json";
    let lifecycle_rel =
        "docs/agents/lifecycle/gemini-cli-onboarding/governance/lifecycle-state.json";
    let packet_json = |lifecycle_sha256: &str| {
        json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": approval_sha256,
            "lifecycle_state_path": lifecycle_rel,
            "lifecycle_state_sha256": lifecycle_sha256,
            "lifecycle_stage": "publication_ready",
            "support_tier_at_emit": "baseline_runtime",
            "manifest_root": "cli_manifests/gemini_cli",
            "expected_targets": ["darwin-arm64"],
            "capability_publication_enabled": true,
            "support_publication_enabled": true,
            "capability_matrix_target": serde_json::Value::Null,
            "required_commands": [
                "cargo run -p xtask -- support-matrix --check",
                "cargo run -p xtask -- capability-matrix --check",
                "cargo run -p xtask -- capability-matrix-audit",
                "make preflight"
            ],
            "required_publication_outputs": [
                "cli_manifests/support_matrix/current.json",
                "docs/specs/unified-agent-api/support-matrix.md",
                "crates/agent_api/src/runtime_support_data.rs",
                "docs/specs/unified-agent-api/capability-matrix.md"
            ],
            "runtime_evidence_paths": [
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/input-contract.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/run-status.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/run-summary.md",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/validation-report.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/written-paths.json",
                "docs/agents/.uaa-temp/runtime-follow-on/runs/rtfo-publication/handoff.json"
            ],
            "publication_owned_paths": [lifecycle_rel, packet_rel],
            "blocking_issues": [],
            "implementation_summary": {
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
            }
        })
    };
    let lifecycle_json = |packet_sha256: &str| {
        json!({
            "schema_version": "1",
            "agent_id": "gemini_cli",
            "onboarding_pack_prefix": "gemini-cli-onboarding",
            "approval_artifact_path": approval_path,
            "approval_artifact_sha256": approval_sha256,
            "lifecycle_stage": "published",
            "support_tier": "publication_backed",
            "side_states": [],
            "current_owner_command": "refresh-publication --write",
            "expected_next_command": agent_lifecycle::published_prepare_closeout_command(
                approval_path
            ),
            "last_transition_at": "2026-05-01T00:00:00Z",
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
            "implementation_summary": {
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
            },
            "publication_packet_path": packet_rel,
            "publication_packet_sha256": packet_sha256,
            "closeout_baseline_path": serde_json::Value::Null
        })
    };

    write_text(
        &packet_path,
        &serde_json::to_string_pretty(&packet_json(
            "0000000000000000000000000000000000000000000000000000000000000000",
        ))
        .expect("serialize publication-ready packet"),
    );
    let mut packet_sha256 = sha256_hex(&packet_path);
    write_text(
        &lifecycle_path,
        &serde_json::to_string_pretty(&lifecycle_json(&packet_sha256))
            .expect("serialize lifecycle state"),
    );
    let lifecycle_sha256 = sha256_hex(&lifecycle_path);
    write_text(
        &packet_path,
        &serde_json::to_string_pretty(&packet_json(&lifecycle_sha256))
            .expect("serialize publication-ready packet"),
    );
    packet_sha256 = sha256_hex(&packet_path);
    write_text(
        &lifecycle_path,
        &serde_json::to_string_pretty(&lifecycle_json(&packet_sha256))
            .expect("serialize lifecycle state"),
    );
}

fn seed_cli_manifest_root(root: &Path, manifest_root: &str, canonical_targets: &[&str]) {
    let current = serde_json::json!({
        "expected_targets": canonical_targets,
        "inputs": canonical_targets
            .iter()
            .map(|target| serde_json::json!({
                "target_triple": target,
                "binary": { "semantic_version": "1.0.0" }
            }))
            .collect::<Vec<_>>(),
        "commands": []
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
                "wrapper_only_args": []
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

fn approval_with_release_watch_enrolled_maintenance(
    contents: &str,
    dispatch_workflow: &str,
    upstream_block: &str,
) -> String {
    let mut updated = contents.trim_end().to_string();
    updated.push_str(&format!(
        concat!(
            "\n\n",
            "[descriptor.maintenance]\n",
            "mode = \"release_watch_enrolled\"\n",
            "\n",
            "[descriptor.maintenance.release_watch]\n",
            "enabled = true\n",
            "version_policy = \"latest_stable_minus_one\"\n",
            "dispatch_kind = \"workflow_dispatch\"\n",
            "dispatch_workflow = \"{dispatch_workflow}\"\n",
            "\n",
            "[descriptor.maintenance.release_watch.upstream]\n",
            "{upstream_block}\n",
        ),
        dispatch_workflow = dispatch_workflow,
        upstream_block = upstream_block.trim_end(),
    ));
    updated.push('\n');
    updated
}

fn refresh_publication_continuity(
    root: &Path,
    approval_rel: &str,
    lifecycle_rel: &str,
    packet_rel: &str,
) {
    let approval_sha256 = sha256_hex(&root.join(approval_rel));
    let lifecycle_path = root.join(lifecycle_rel);
    let packet_path = root.join(packet_rel);
    let base_lifecycle: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&lifecycle_path).expect("read lifecycle state fixture"),
    )
    .expect("parse lifecycle state fixture");
    let base_packet: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&packet_path).expect("read packet fixture"))
            .expect("parse packet fixture");

    let lifecycle_json = |packet_sha256: &str| {
        let mut lifecycle = base_lifecycle.clone();
        lifecycle["approval_artifact_sha256"] = json!(approval_sha256);
        lifecycle["publication_packet_sha256"] = json!(packet_sha256);
        if lifecycle
            .get("lifecycle_stage")
            .and_then(serde_json::Value::as_str)
            == Some("closed_baseline")
        {
            for field in ["required_evidence", "satisfied_evidence"] {
                let entries = lifecycle[field]
                    .as_array_mut()
                    .expect("closed baseline evidence arrays");
                if !entries
                    .iter()
                    .any(|entry| entry.as_str() == Some("maintenance_readiness_settled"))
                {
                    entries.push(json!("maintenance_readiness_settled"));
                }
            }
        }
        lifecycle
    };
    let packet_json = |lifecycle_sha256: &str| {
        let mut packet = base_packet.clone();
        packet["approval_artifact_sha256"] = json!(approval_sha256);
        packet["lifecycle_state_sha256"] = json!(lifecycle_sha256);
        packet
    };

    write_text(
        &packet_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&packet_json(
                "0000000000000000000000000000000000000000000000000000000000000000",
            ))
            .expect("serialize packet fixture")
        ),
    );
    let mut packet_sha256 = sha256_hex(&packet_path);
    write_text(
        &lifecycle_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&lifecycle_json(&packet_sha256))
                .expect("serialize lifecycle fixture")
        ),
    );
    let lifecycle_sha256 = sha256_hex(&lifecycle_path);
    write_text(
        &packet_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&packet_json(&lifecycle_sha256))
                .expect("serialize packet fixture")
        ),
    );
    packet_sha256 = sha256_hex(&packet_path);
    write_text(
        &lifecycle_path,
        &format!(
            "{}\n",
            serde_json::to_string_pretty(&lifecycle_json(&packet_sha256))
                .expect("serialize lifecycle fixture")
        ),
    );
}

fn seed_lifecycle_eligible_audit_peers(root: &Path) {
    write_text(
        &root.join("docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml"),
        &approval_with_release_watch_enrolled_maintenance(
            include_str!(
                "../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml"
            ),
            "codex-cli-update-snapshot.yml",
            concat!(
                "source_kind = \"github_releases\"\n",
                "owner = \"openai\"\n",
                "repo = \"codex\"\n",
                "tag_prefix = \"rust-v\""
            ),
        ),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json"),
        include_str!("../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-cli-onboarding/governance/publication-ready.json"),
        include_str!("../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/publication-ready.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/codex-cli-onboarding/governance/proving-run-closeout.json"),
        include_str!("../../../../docs/agents/lifecycle/codex-cli-onboarding/governance/proving-run-closeout.json"),
    );

    write_text(
        &root.join("docs/agents/lifecycle/claude-code-cli-onboarding/governance/approved-agent.toml"),
        &approval_with_release_watch_enrolled_maintenance(
            include_str!("../../../../docs/agents/lifecycle/claude-code-cli-onboarding/governance/approved-agent.toml"),
            "claude-code-update-snapshot.yml",
            concat!(
                "source_kind = \"gcs_object_listing\"\n",
                "bucket = \"claude-code-dist-86c565f3-f756-42ad-8dfa-d59b1c096819\"\n",
                "prefix = \"claude-code-releases\"\n",
                "version_marker = \"manifest.json\""
            ),
        ),
    );
    write_text(
        &root.join("docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json"),
        include_str!("../../../../docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/claude-code-cli-onboarding/governance/publication-ready.json"),
        include_str!("../../../../docs/agents/lifecycle/claude-code-cli-onboarding/governance/publication-ready.json"),
    );
    write_text(
        &root.join("docs/agents/lifecycle/claude-code-cli-onboarding/governance/proving-run-closeout.json"),
        include_str!("../../../../docs/agents/lifecycle/claude-code-cli-onboarding/governance/proving-run-closeout.json"),
    );

    refresh_publication_continuity(
        root,
        "docs/agents/lifecycle/codex-cli-onboarding/governance/approved-agent.toml",
        "docs/agents/lifecycle/codex-cli-onboarding/governance/lifecycle-state.json",
        "docs/agents/lifecycle/codex-cli-onboarding/governance/publication-ready.json",
    );
    refresh_publication_continuity(
        root,
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/approved-agent.toml",
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/lifecycle-state.json",
        "docs/agents/lifecycle/claude-code-cli-onboarding/governance/publication-ready.json",
    );
}
