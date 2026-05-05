#![allow(dead_code, unused_imports)]

use std::{fs, path::Path};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

mod agent_lifecycle {
    pub use xtask::agent_lifecycle::*;
}
mod agent_registry {
    pub use xtask::agent_registry::*;
}
#[path = "../src/agent_maintenance/request.rs"]
mod request;
#[path = "../src/agent_maintenance/docs.rs"]
mod docs;
#[path = "../src/workspace_mutation.rs"]
mod workspace_mutation;
#[path = "../src/agent_maintenance/prepare.rs"]
mod prepare;

use harness::{fixture_root, write_text};
use prepare::{apply_prepare_plan, build_prepare_plan, Args};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");

#[test]
fn prepare_agent_maintenance_builds_packet_first_plan() {
    let fixture = fixture_root("prepare-agent-maintenance-plan");
    seed_registry(&fixture);
    seed_support_files(&fixture);

    let plan = build_prepare_plan(&fixture, &args()).expect("build plan");
    assert_eq!(
        plan.request.relative_path,
        "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml"
    );
    let request_text = String::from_utf8(plan.files[0].contents.clone()).expect("utf8");
    assert!(request_text.contains("artifact_version = \"2\""));
    assert!(request_text.contains("trigger_kind = \"upstream_release_detected\""));
    assert!(request_text.contains("branch_name = \"automation/codex-maintenance-0.98.0\""));
    assert!(plan
        .planned_paths()
        .contains(&"docs/agents/lifecycle/codex-maintenance/HANDOFF.md"));
}

#[test]
fn prepare_agent_maintenance_write_creates_packet_root() {
    let fixture = fixture_root("prepare-agent-maintenance-write");
    seed_registry(&fixture);
    seed_support_files(&fixture);

    let plan = build_prepare_plan(&fixture, &args()).expect("build plan");
    let summary = apply_prepare_plan(&fixture, &plan).expect("apply plan");
    assert_eq!(summary.total, plan.files.len());

    let handoff = fs::read_to_string(
        fixture.join("docs/agents/lifecycle/codex-maintenance/HANDOFF.md"),
    )
    .expect("read handoff");
    assert!(handoff.contains("detected_by"));
    assert!(handoff.contains("automation/codex-maintenance-0.98.0"));
}

#[test]
fn prepare_agent_maintenance_packet_pr_defaults_to_generic_open_pr_workflow() {
    let fixture = fixture_root("prepare-agent-maintenance-packet-pr");
    seed_registry(&fixture);
    seed_support_files(&fixture);

    let mut args = args();
    args.dispatch_kind = "packet_pr".to_string();
    args.dispatch_workflow = None;

    let plan = build_prepare_plan(&fixture, &args).expect("build plan");
    let request_text = String::from_utf8(plan.files[0].contents.clone()).expect("utf8");
    assert!(request_text.contains("dispatch_kind = \"packet_pr\""));
    assert!(request_text.contains("dispatch_workflow = \"agent-maintenance-open-pr.yml\""));
}

fn args() -> Args {
    Args {
        agent: "codex".to_string(),
        current_version: "0.97.0".to_string(),
        latest_stable: "0.99.0".to_string(),
        target_version: "0.98.0".to_string(),
        opened_from: Path::new(".github/workflows/codex-cli-update-snapshot.yml").to_path_buf(),
        detected_by: ".github/workflows/agent-maintenance-release-watch.yml".to_string(),
        dispatch_kind: "workflow_dispatch".to_string(),
        dispatch_workflow: Some("codex-cli-update-snapshot.yml".to_string()),
        branch_name: "automation/codex-maintenance-0.98.0".to_string(),
        request_recorded_at: "2026-05-05T15:00:00Z".to_string(),
        request_commit: "abcdef1".to_string(),
        dry_run: true,
        write: false,
    }
}

fn seed_registry(root: &Path) {
    write_text(
        &root.join("crates/xtask/data/agent_registry.toml"),
        SEEDED_REGISTRY,
    );
}

fn seed_support_files(root: &Path) {
    write_text(
        &root.join(".github/workflows/codex-cli-update-snapshot.yml"),
        "name: Codex worker\n",
    );
    write_text(
        &root.join("cli_manifests/codex/latest_validated.txt"),
        "0.97.0\n",
    );
}
