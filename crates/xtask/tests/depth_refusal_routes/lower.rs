//! LW1: the public lower delivery boundaries (`workspace_mutation::apply_mutations` and the
//! lifecycle/publication-packet storage writers) refuse declared or retained depth, and proposed
//! record or enrollment destinations, before any write. `tests/depth_promotion.rs` already covers
//! retained/nested/proposed cases for `apply_mutations` and the publication packet writer on
//! codex-named paths; these cases add the declared-table workspace, the lifecycle-state writer and
//! the synthetic subject's own destinations.

use std::fs;

use xtask::{
    agent_lifecycle::{
        write_lifecycle_state, write_publication_ready_packet, LifecycleState,
        PublicationReadyPacket,
    },
    workspace_mutation::{apply_mutations, PlannedMutation},
};

use super::support::{
    registry_with_subject, repo, snapshot, synthetic_record, Depth, Workspace, MANIFEST_ROOT,
    RECORD, REGISTRY,
};

const LIFECYCLE: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json";
const PACKET: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/publication-ready.json";

fn lifecycle_state() -> LifecycleState {
    let raw = fs::read(
        repo()
            .join("docs/agents/lifecycle/opencode-cli-onboarding/governance/lifecycle-state.json"),
    )
    .unwrap();
    serde_json::from_slice(&raw).unwrap()
}

fn publication_packet() -> PublicationReadyPacket {
    let raw =
        fs::read(repo().join(
            "docs/agents/lifecycle/opencode-cli-onboarding/governance/publication-ready.json",
        ))
        .unwrap();
    let packet: PublicationReadyPacket = serde_json::from_slice(&raw).unwrap();
    packet.validate().unwrap();
    packet
}

fn pointer() -> PlannedMutation {
    PlannedMutation::create(
        format!("{MANIFEST_ROOT}/pointers/latest_validated/x86_64-unknown-linux-gnu.txt.new"),
        b"0.2.0\n".to_vec(),
    )
}

#[test]
fn lw1_apply_mutations_refuses_depth_workspace() {
    for depth in Depth::ALL {
        let ws = Workspace::with(depth);
        let before = snapshot(ws.root());
        let error = apply_mutations(ws.root(), &[pointer()])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(&format!(
                "neutral mutation writer: {}",
                depth.guard_reason()
            )),
            "{error}"
        );
        // A caller-supplied inner root cannot hide the enclosing workspace's depth.
        let inner = ws.root().join(MANIFEST_ROOT);
        let nested = PlannedMutation::create("latest_validated.txt.new", b"0.2.0\n".to_vec());
        assert!(apply_mutations(&inner, &[nested]).is_err());
        ws.assert_unchanged(&before, "apply_mutations");
    }
}

#[test]
fn lw1_apply_mutations_refuses_proposed_record_and_enrollment_destinations() {
    let ws = Workspace::plain();
    let before = snapshot(ws.root());
    let record = PlannedMutation::create(RECORD, synthetic_record());
    let error = apply_mutations(ws.root(), &[pointer(), record])
        .unwrap_err()
        .to_string();
    assert!(error.contains("proposed depth record"), "{error}");
    let current = fs::read(ws.root().join(REGISTRY)).unwrap();
    let declaration =
        PlannedMutation::replace(REGISTRY, current, registry_with_subject(true).into_bytes());
    let error = apply_mutations(ws.root(), &[declaration])
        .unwrap_err()
        .to_string();
    assert!(error.contains("declared enrollment"), "{error}");
    ws.assert_unchanged(&before, "apply_mutations proposals");
}

#[test]
fn lw1_lifecycle_and_packet_storage_writers_refuse_depth_workspace() {
    let (state, packet) = (lifecycle_state(), publication_packet());
    for depth in Depth::ALL {
        let ws = Workspace::with(depth);
        let before = snapshot(ws.root());
        let error = write_lifecycle_state(ws.root(), LIFECYCLE, &state)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(&format!("lifecycle writer: {}", depth.guard_reason())),
            "{error}"
        );
        let error = write_publication_ready_packet(ws.root(), PACKET, &packet)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(&format!(
                "publication packet writer: {}",
                depth.guard_reason()
            )),
            "{error}"
        );
        ws.assert_unchanged(&before, "lifecycle storage");
    }
}

#[test]
fn lw1_lifecycle_and_packet_storage_writers_refuse_a_record_destination() {
    let (state, packet) = (lifecycle_state(), publication_packet());
    let ws = Workspace::plain();
    let before = snapshot(ws.root());
    for error in [
        write_lifecycle_state(ws.root(), RECORD, &state)
            .unwrap_err()
            .to_string(),
        write_publication_ready_packet(ws.root(), RECORD, &packet)
            .unwrap_err()
            .to_string(),
    ] {
        assert!(error.contains("proposed depth record"), "{error}");
    }
    ws.assert_unchanged(&before, "lifecycle storage record destination");
}
