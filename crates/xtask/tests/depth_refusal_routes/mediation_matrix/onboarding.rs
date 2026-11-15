use super::*;

const APPROVAL: &str =
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/approved-agent.toml";
const CLOSEOUT: &str =
    "docs/agents/lifecycle/opencode-cli-onboarding/governance/proving-run-closeout.json";

#[test]
fn every_on_entry_refuses_onboarding_carriers_without_scope_authority() {
    let routes: [(&str, Vec<&str>); 8] = [
        (
            "onboard_agent",
            vec!["onboard-agent", "--write", "--approval", APPROVAL],
        ),
        (
            "runtime_follow_on",
            vec![
                "runtime-follow-on",
                "--write",
                "--approval",
                APPROVAL,
                "--run-id",
                "s11-matrix",
            ],
        ),
        (
            "runtime_follow_on",
            vec![
                "runtime-follow-on",
                "--dry-run",
                "--approval",
                APPROVAL,
                "--run-id",
                "s11-dry-run",
            ],
        ),
        (
            "repair_runtime_evidence",
            vec!["repair-runtime-evidence", "--write", "--approval", APPROVAL],
        ),
        (
            "prepare_publication",
            vec!["prepare-publication", "--write", "--approval", APPROVAL],
        ),
        (
            "publication_refresh",
            vec!["refresh-publication", "--write", "--approval", APPROVAL],
        ),
        (
            "prepare_proving_run_closeout",
            vec![
                "prepare-proving-run-closeout",
                "--write",
                "--approval",
                APPROVAL,
            ],
        ),
        (
            "close_proving_run",
            vec![
                "close-proving-run",
                "--approval",
                APPROVAL,
                "--closeout",
                CLOSEOUT,
            ],
        ),
    ];
    assert_eq!(routes.len(), 8);
    for carrier in Carrier::ALL {
        let ws = workspace(carrier);
        for (route, args) in &routes {
            ws.assert_refuses(args, &format!("{route}: {}", carrier.reason()));
        }
    }
}

#[test]
fn ordinary_neutral_command_reaches_its_production_result() {
    let ws = Workspace::plain();
    let output = ws.xtask(&["capability-matrix", "--out", "ordinary/matrix.md"]);
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{error}");
    let written = fs::read_to_string(ws.root().join("ordinary/matrix.md")).unwrap();
    assert!(written.contains("# Capability matrix"));
}

fn ordinary_runtime_ready(ws: &Workspace) {
    use sha2::{Digest, Sha256};
    let state = serde_json::json!({
        "schema_version": "1",
        "agent_id": "opencode",
        "onboarding_pack_prefix": "opencode-cli-onboarding",
        "approval_artifact_path": APPROVAL,
        "approval_artifact_sha256": hex::encode(Sha256::digest(fs::read(ws.root().join(APPROVAL)).unwrap())),
        "lifecycle_stage": "enrolled",
        "support_tier": "bootstrap",
        "side_states": [],
        "current_owner_command": "runtime-follow-on --write",
        "expected_next_command": "runtime-follow-on --dry-run",
        "last_transition_at": "2026-11-15T00:00:00Z",
        "last_transition_by": "s11-test",
        "required_evidence": ["registry_entry", "docs_pack", "manifest_root_skeleton"],
        "satisfied_evidence": ["registry_entry", "docs_pack", "manifest_root_skeleton"],
        "blocking_issues": [],
        "retryable_failures": [],
        "active_runtime_evidence_run_id": null,
        "implementation_summary": null,
        "publication_packet_path": null,
        "publication_packet_sha256": null,
        "closeout_baseline_path": null
    });
    write(
        ws.root(),
        "docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json",
        serde_json::to_vec_pretty(&state).unwrap(),
    );
}

#[test]
fn on2_dry_run_guards_persistence_for_foreign_approval_and_keeps_ordinary_delivery() {
    let args = [
        "runtime-follow-on",
        "--dry-run",
        "--approval",
        APPROVAL,
        "--run-id",
        "s11-persistence",
    ];
    let ordinary = Workspace::plain();
    ordinary_runtime_ready(&ordinary);
    let output = ordinary.xtask(&args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = ordinary
        .root()
        .join("docs/agents/.uaa-temp/runtime-follow-on/runs/s11-persistence");
    for name in [
        "input-contract.json",
        "codex-prompt.md",
        "run-status.json",
        "run-summary.md",
        "validation-report.json",
        "written-paths.json",
        "handoff.json",
    ] {
        assert!(!fs::read(run.join(name)).unwrap().is_empty(), "{name}");
    }
    for carrier in [
        Carrier::Omitted,
        Carrier::Orphaned,
        Carrier::Declared,
        Carrier::Disabled,
        Carrier::RemovedRecord,
    ] {
        let ws = workspace(carrier);
        ordinary_runtime_ready(&ws);
        ws.assert_refuses(&args, &format!("runtime_follow_on: {}", carrier.reason()));
    }
}
