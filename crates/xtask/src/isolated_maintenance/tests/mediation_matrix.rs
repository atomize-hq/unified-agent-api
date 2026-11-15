//! E.8 uses both committed ON2 vectors and actual ON3/ON5 retained onboarding output.
use super::*;
use crate::workspace_mutation::{apply_mutations, PlannedMutation};

mod aggregate;
#[path = "../../../tests/support/onboarding_guard_record.rs"]
#[allow(clippy::duplicate_mod)]
mod guard_record;
mod integration;
mod lower;
mod lower_expected;
mod maintenance;

thread_local! {
    static CONNECTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(super) fn connected_enabled() -> bool {
    CONNECTED.with(std::cell::Cell::get)
}

const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const STATE: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json";
const PACKET: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json";
const DECLARED: &str =
    include_str!("../../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml");

fn unchanged(root: &Path, expected: &str, invoke: impl FnOnce() -> String) {
    let before = super::unsupported_routes::tree(root);
    let error = invoke();
    assert!(
        error.contains(expected),
        "expected {expected:?}, got {error:?}"
    );
    assert_eq!(super::unsupported_routes::tree(root), before, "{expected}");
}

/// The connected chain calls this with authentic producer output, including retained entries.
pub(super) fn onboarding_checkpoint(root: &Path) {
    let record = fs::read(root.join(RECORD)).unwrap();
    let parsed = crate::depth_record::DepthRecord::parse(
        std::str::from_utf8(&record).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    assert!(matches!(
        parsed.event,
        crate::depth_record::Event::Onboarding { .. }
    ));
    assert_eq!(parsed.policy.selection.enrollment.version, "0.2.0");
    assert_eq!(parsed.execution.as_ref().unwrap().results.len(), 11);
    let state = crate::agent_lifecycle::load_lifecycle_state(root, STATE).unwrap();
    // After ON4/ON5 the lifecycle state legitimately advances past the packet's pinned
    // `lifecycle_state_sha256`; the lower-writer carriers need only a structurally valid packet.
    let packet: crate::agent_lifecycle::PublicationReadyPacket =
        serde_json::from_slice(&fs::read(root.join(PACKET)).unwrap()).unwrap();
    packet.validate().unwrap();
    lower::carriers(&record, &state, &packet);
    let bundle = crate::support_matrix::PublicationBundle {
        json: fs::read_to_string(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
        markdown: fs::read_to_string(root.join(crate::support_matrix::MARKDOWN_OUTPUT_PATH))
            .unwrap(),
        runtime_support_data: fs::read_to_string(
            root.join(crate::support_matrix::AGENT_API_RUNTIME_SUPPORT_DATA_OUTPUT_PATH),
        )
        .unwrap(),
    };
    let artifact: serde_json::Value = serde_json::from_str(&bundle.json).unwrap();
    let subject = artifact["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["agent"] == "depth_fixture")
        .unwrap();
    assert_eq!(subject["depth"]["promises"].as_array().unwrap().len(), 1);
    unchanged(
        root,
        "support-matrix writer: unsupported depth write (declared enrollment)",
        || crate::support_matrix::write_publication_artifacts(root, &bundle).unwrap_err(),
    );
    let head = git(root, &["rev-parse", "HEAD"]);
    unchanged(
        root,
        "promotion has no maintenance lifecycle-path authority",
        || {
            crate::manifest_promotion::build_plan(
                root,
                &crate::manifest_promotion::Args {
                    agent: "depth_fixture".into(),
                    version: "0.2.0".into(),
                    passed_targets: vec!["x86_64-unknown-linux-gnu".into()],
                    validation_commit: head,
                    write: true,
                },
            )
            .err()
            .unwrap()
        },
    );
    integration::relay(root);
    let route = crate::isolated_maintenance::route(root).unwrap().unwrap();
    let (proposal, captured) = route.closeout_publication(root).unwrap();
    unchanged(root, "manual closeout proposed an unowned output", || {
        proposal
            .apply_closeout_publication(
                root,
                Some(&captured),
                vec![(RECORD.into(), record.clone())],
            )
            .unwrap_err()
    });
}

pub(super) fn integration_checkpoint(root: &Path, parent: &str, commit: impl Fn()) {
    integration::committed(root, parent, commit);
}

pub(super) fn on4_aggregate(root: &Path) {
    aggregate::check(root);
}

#[test]
fn onboarding_retained_and_proposed_lower_writer_matrix() {
    let record = guard_record::record();
    let state = super::unsupported_routes::repo_json(
        "docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json",
    );
    let packet = super::unsupported_routes::repo_json(
        "docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json",
    );
    let parsed = crate::depth_record::DepthRecord::parse(
        std::str::from_utf8(&record).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    assert!(matches!(
        parsed.event,
        crate::depth_record::Event::Onboarding { .. }
    ));
    lower::carriers(&record, &state, &packet);
    lower::proposed(&state, &packet, &record);
}

#[cfg(target_os = "linux")]
#[test]
fn connected_onboarding_mediation_and_admitted_writers() {
    CONNECTED.with(|active| assert!(!active.replace(true)));
    super::bootstrap::public_on3::mediation_chain();
    CONNECTED.with(|active| active.set(false));
}

#[cfg(target_os = "linux")]
#[test]
fn connected_on4_aggregate_and_restoration() {
    CONNECTED.with(|active| assert!(!active.replace(true)));
    super::bootstrap::public_on3::mediation_on4_preparation();
    CONNECTED.with(|active| active.set(false));
}
