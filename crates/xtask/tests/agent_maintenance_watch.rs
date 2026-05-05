#![allow(dead_code)]

use std::{fs, path::PathBuf};

#[path = "support/onboard_agent_harness.rs"]
mod harness;

mod agent_lifecycle {
    pub use xtask::agent_lifecycle::*;
}

mod agent_registry {
    pub use xtask::agent_registry::*;
}

#[path = "../src/agent_maintenance/watch.rs"]
mod watch;

use harness::{fixture_root, write_text};
use watch::{build_watch_queue_with_resolver, run_in_workspace_with_resolver, Args, Error};

const SEEDED_REGISTRY: &str = include_str!("../data/agent_registry.toml");

#[test]
fn build_watch_queue_emits_frozen_fields_for_stale_agents() {
    let fixture = fixture_root("agent-maintenance-watch-queue");
    seed_registry(&fixture);
    seed_latest_validated(&fixture, "cli_manifests/codex", "0.97.0");
    seed_latest_validated(&fixture, "cli_manifests/claude_code", "1.2.3");

    let queue = build_watch_queue_with_resolver(&fixture, resolver_for_queue).expect("queue");

    assert_eq!(queue.schema_version, 1);
    assert!(!queue.generated_at.is_empty());
    assert_eq!(
        queue.stale_agents,
        vec![
            watch::MaintenanceWatchQueueEntry {
                agent_id: "codex".to_string(),
                manifest_root: "cli_manifests/codex".to_string(),
                current_validated: "0.97.0".to_string(),
                latest_stable: "0.99.0".to_string(),
                target_version: "0.98.0".to_string(),
                version_policy: "latest_stable_minus_one".to_string(),
                dispatch_kind: "workflow_dispatch".to_string(),
                dispatch_workflow: "codex-cli-update-snapshot.yml".to_string(),
                maintenance_root: "docs/agents/lifecycle/codex-maintenance".to_string(),
                request_path:
                    "docs/agents/lifecycle/codex-maintenance/governance/maintenance-request.toml"
                        .to_string(),
                opened_from: ".github/workflows/codex-cli-update-snapshot.yml".to_string(),
                detected_by: ".github/workflows/agent-maintenance-release-watch.yml".to_string(),
                branch_name: "automation/codex-maintenance-0.98.0".to_string(),
            },
            watch::MaintenanceWatchQueueEntry {
                agent_id: "claude_code".to_string(),
                manifest_root: "cli_manifests/claude_code".to_string(),
                current_validated: "1.2.3".to_string(),
                latest_stable: "1.2.5".to_string(),
                target_version: "1.2.4".to_string(),
                version_policy: "latest_stable_minus_one".to_string(),
                dispatch_kind: "workflow_dispatch".to_string(),
                dispatch_workflow: "claude-code-update-snapshot.yml".to_string(),
                maintenance_root: "docs/agents/lifecycle/claude_code-maintenance".to_string(),
                request_path:
                    "docs/agents/lifecycle/claude_code-maintenance/governance/maintenance-request.toml"
                        .to_string(),
                opened_from: ".github/workflows/claude-code-update-snapshot.yml".to_string(),
                detected_by: ".github/workflows/agent-maintenance-release-watch.yml".to_string(),
                branch_name: "automation/claude_code-maintenance-1.2.4".to_string(),
            }
        ]
    );
}

#[test]
fn run_in_workspace_emits_json_queue_file() {
    let fixture = fixture_root("agent-maintenance-watch-emit-json");
    seed_registry(&fixture);
    seed_latest_validated(&fixture, "cli_manifests/codex", "0.97.0");
    seed_latest_validated(&fixture, "cli_manifests/claude_code", "1.2.3");

    let mut stdout = Vec::new();
    run_in_workspace_with_resolver(
        &fixture,
        Args {
            check: false,
            emit_json: Some(PathBuf::from("_ci_tmp/maintenance-watch.json")),
        },
        &mut stdout,
        resolver_for_queue,
    )
    .expect("emit queue");

    let output = String::from_utf8(stdout).expect("stdout utf8");
    assert!(output.contains("stale_agents: 2"));
    assert!(output.contains("emitted_json: _ci_tmp/maintenance-watch.json"));

    let written = fs::read_to_string(fixture.join("_ci_tmp/maintenance-watch.json"))
        .expect("read queue json");
    let parsed: watch::MaintenanceWatchQueue =
        serde_json::from_str(&written).expect("parse queue json");
    assert_eq!(parsed.schema_version, 1);
    assert_eq!(parsed.stale_agents.len(), 2);
}

#[test]
fn clean_or_not_newer_agents_are_not_emitted() {
    let fixture = fixture_root("agent-maintenance-watch-clean");
    seed_registry(&fixture);
    seed_latest_validated(&fixture, "cli_manifests/codex", "0.98.0");
    seed_latest_validated(&fixture, "cli_manifests/claude_code", "1.2.4");

    let queue = build_watch_queue_with_resolver(&fixture, resolver_for_queue).expect("queue");
    assert!(queue.stale_agents.is_empty());
}

#[test]
fn malformed_upstream_history_fails_closed() {
    let fixture = fixture_root("agent-maintenance-watch-malformed");
    seed_registry(&fixture);
    seed_latest_validated(&fixture, "cli_manifests/codex", "0.97.0");
    seed_latest_validated(&fixture, "cli_manifests/claude_code", "1.2.3");

    let err = build_watch_queue_with_resolver(&fixture, |entry, _| {
        if entry.agent_id == "codex" {
            Err(Error::Validation("synthetic upstream failure".to_string()))
        } else {
            Ok(vec!["1.2.5".parse().unwrap(), "1.2.4".parse().unwrap()])
        }
    })
    .unwrap_err();

    assert!(err.to_string().contains("synthetic upstream failure"));
}

#[test]
fn packet_pr_enrollment_uses_generic_open_pr_workflow() {
    let fixture = fixture_root("agent-maintenance-watch-packet-pr");
    seed_registry_with(
        &fixture,
        &SEEDED_REGISTRY.replace(
            "dispatch_kind = \"workflow_dispatch\"\ndispatch_workflow = \"codex-cli-update-snapshot.yml\"",
            "dispatch_kind = \"packet_pr\"",
        ),
    );
    seed_latest_validated(&fixture, "cli_manifests/codex", "0.97.0");
    seed_latest_validated(&fixture, "cli_manifests/claude_code", "1.2.4");

    let queue = build_watch_queue_with_resolver(&fixture, resolver_for_queue).expect("queue");
    let codex = queue
        .stale_agents
        .iter()
        .find(|entry| entry.agent_id == "codex")
        .expect("codex stale agent");
    assert_eq!(codex.dispatch_kind, "packet_pr");
    assert_eq!(codex.dispatch_workflow, "agent-maintenance-open-pr.yml");
    assert_eq!(codex.opened_from, ".github/workflows/agent-maintenance-open-pr.yml");
}

fn resolver_for_queue(
    entry: &xtask::agent_registry::AgentRegistryEntry,
    _release_watch: &xtask::agent_registry::ReleaseWatchMetadata,
) -> Result<Vec<semver::Version>, Error> {
    let versions = match entry.agent_id.as_str() {
        "codex" => vec!["0.99.0", "0.98.0", "0.97.0"],
        "claude_code" => vec!["1.2.5", "1.2.4", "1.2.3"],
        other => panic!("unexpected agent {other}"),
    };
    Ok(versions
        .into_iter()
        .map(|value| value.parse().expect("valid semver"))
        .collect())
}

fn seed_registry(root: &std::path::Path) {
    seed_registry_with(root, SEEDED_REGISTRY);
}

fn seed_registry_with(root: &std::path::Path, registry: &str) {
    write_text(
        &root.join("crates/xtask/data/agent_registry.toml"),
        registry,
    );
}

fn seed_latest_validated(root: &std::path::Path, manifest_root: &str, version: &str) {
    write_text(
        &root.join(manifest_root).join("latest_validated.txt"),
        &format!("{version}\n"),
    );
}
