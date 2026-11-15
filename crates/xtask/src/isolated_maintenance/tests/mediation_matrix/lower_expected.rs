//! Independently pinned delivery bytes, including pretty formatting and the terminal newline.
pub(super) const STATE: &str = r#"{
  "schema_version": "1",
  "agent_id": "opencode",
  "onboarding_pack_prefix": "opencode-cli-onboarding",
  "approval_artifact_path": "docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml",
  "approval_artifact_sha256": "905bdfd0cd7ee72169fb36c546c8448619ecd5e72c98b59421c584386b1feb1f",
  "lifecycle_stage": "closed_baseline",
  "support_tier": "publication_backed",
  "side_states": [],
  "current_owner_command": "close-proving-run --write",
  "expected_next_command": "check-agent-drift --agent opencode",
  "last_transition_at": "2026-04-18T19:31:24-05:00",
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
    "proving_run_closeout_written",
    "maintenance_readiness_settled",
    "maintenance_closeout_written"
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
    "proving_run_closeout_written",
    "maintenance_readiness_settled",
    "maintenance_closeout_written"
  ],
  "blocking_issues": [],
  "retryable_failures": [],
  "active_runtime_evidence_run_id": null,
  "implementation_summary": {
    "requested_runtime_profile": "default",
    "achieved_runtime_profile": "default",
    "primary_template": "opencode",
    "template_lineage": [
      "opencode"
    ],
    "landed_surfaces": [
      "wrapper_runtime",
      "backend_harness",
      "wrapper_coverage_source",
      "runtime_manifest_evidence",
      "session_resume",
      "session_fork"
    ],
    "deferred_surfaces": [],
    "minimal_profile_justification": null
  },
  "publication_packet_path": "docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json",
  "publication_packet_sha256": "bfaa4b191a8dc832c5891c5bf4b74646b1085e06737fe1fcf37fc012018f0f95",
  "closeout_baseline_path": "docs/agents/lifecycle/opencode-cli-onboarding/governance/proving-run-closeout.json"
}
"#;

pub(super) const PACKET: &str = r#"{
  "schema_version": "1",
  "agent_id": "opencode",
  "approval_artifact_path": "docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml",
  "approval_artifact_sha256": "905bdfd0cd7ee72169fb36c546c8448619ecd5e72c98b59421c584386b1feb1f",
  "lifecycle_state_path": "docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json",
  "lifecycle_state_sha256": "51f2e4dc373ccccb5cd528bd0d573cc170e715f6947dfa0b0202322eedd8aa36",
  "lifecycle_stage": "publication_ready",
  "support_tier_at_emit": "baseline_runtime",
  "manifest_root": "cli_manifests/opencode",
  "expected_targets": [
    "linux-x64",
    "darwin-arm64",
    "win32-x64"
  ],
  "capability_publication_enabled": true,
  "support_publication_enabled": true,
  "capability_matrix_target": null,
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
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-opencode-runtime-follow-on/input-contract.json",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-opencode-runtime-follow-on/run-status.json",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-opencode-runtime-follow-on/run-summary.md",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-opencode-runtime-follow-on/validation-report.json",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-opencode-runtime-follow-on/written-paths.json",
    "docs/agents/.uaa-temp/runtime-follow-on/runs/historical-opencode-runtime-follow-on/handoff.json"
  ],
  "publication_owned_paths": [
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json",
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json"
  ],
  "blocking_issues": [],
  "implementation_summary": {
    "requested_runtime_profile": "default",
    "achieved_runtime_profile": "default",
    "primary_template": "opencode",
    "template_lineage": [
      "opencode"
    ],
    "landed_surfaces": [
      "wrapper_runtime",
      "backend_harness",
      "wrapper_coverage_source",
      "runtime_manifest_evidence",
      "session_resume",
      "session_fork"
    ],
    "deferred_surfaces": [],
    "minimal_profile_justification": null
  }
}
"#;
