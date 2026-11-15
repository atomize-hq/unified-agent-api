//! E.7 composes the actual bootstrap writers; no installed positive record is seeded here.
use super::*;

mod checkpoints;
mod insufficient;
pub(super) use checkpoints::freeze;

#[test]
fn e7_connected_onboarding_qualified_chain() {
    connected_with(false, true, Some(public_on4::Phase::Closeout));
}

#[test]
fn e7_connected_onboarding_repaired_qualified_chain() {
    connected_with(true, true, Some(public_on4::Phase::Closeout));
}

#[test]
fn e7_connected_onboarding_release_watch_chain() {
    connected_with(false, true, Some(public_on4::Phase::CloseoutReleaseWatch));
}

/// Final invariants run after the actual closed candidate's IN1 and post-tip observation.
pub(super) fn final_state(root: &Path, host: &MemoryHost, release_watch: bool) {
    let record = crate::depth_record::DepthRecord::parse(
        &fs::read_to_string(root.join(RECORD)).unwrap(),
        "depth_fixture",
        "0.2.0",
    )
    .unwrap();
    let approval: Value =
        toml_edit::de::from_str(&fs::read_to_string(root.join(APPROVAL)).unwrap()).unwrap();
    assert_eq!(
        record.event,
        crate::depth_record::Event::Onboarding {
            approval_commit: approval["approval_commit"].as_str().unwrap().into(),
            approval_recorded_at: approval["approval_recorded_at"].as_str().unwrap().into(),
        }
    );
    assert_eq!(record.policy.selection.enrollment.version, "0.2.0");
    let due = crate::depth_record::due(&record).unwrap();
    assert_eq!(due.len(), 11);
    assert_eq!(record.execution.as_ref().unwrap().results.len(), 11);
    assert_eq!(record.acceptance_entries.len(), 2);
    let route = crate::isolated_maintenance::route(root).unwrap().unwrap();
    let captured = route.capture(root).unwrap();
    let verdicts =
        EvidenceVerdicts::authenticate(root, &record, &captured.evidence_host(root).unwrap())
            .unwrap();
    for obligation in due {
        assert_eq!(
            verdicts.result(&record, &obligation),
            crate::depth_record::DepthResult::Verified
        );
    }
    let registry = captured.registry();
    let entry = registry.find("depth_fixture").unwrap();
    assert!(entry.backend_module.is_none());
    assert!(entry.capability_declaration.always_on.is_empty());
    assert!(entry.capability_declaration.backend_extensions.is_empty());
    assert!(entry.capability_declaration.target_gated.is_empty());
    assert!(entry.capability_declaration.config_gated.is_empty());
    assert!(!root
        .join("crates/agent_api/src/backends/depth_fixture")
        .exists());
    let metadata: Value =
        serde_json::from_slice(&fs::read(root.join(ROOT).join("versions/0.2.0.json")).unwrap())
            .unwrap();
    assert_eq!(metadata["semantic_version"], "0.2.0");
    assert_eq!(metadata["status"], "snapshotted");
    assert!(metadata["validation"].is_null());
    for path in [
        "min_supported.txt",
        "latest_validated.txt",
        "pointers/latest_supported/x86_64-unknown-linux-gnu.txt",
        "pointers/latest_validated/x86_64-unknown-linux-gnu.txt",
    ] {
        assert_eq!(
            fs::read_to_string(root.join(ROOT).join(path)).unwrap(),
            "none\n"
        );
    }
    let state = life::load_lifecycle_state(root, STATE).unwrap();
    assert_eq!(state.lifecycle_stage, life::LifecycleStage::ClosedBaseline);
    let closeout: Value = serde_json::from_slice(&fs::read(root.join(
        "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/proving-run-closeout.json"
    )).unwrap()).unwrap();
    assert_eq!(closeout["state"], "closed");
    if release_watch {
        assert_eq!(
            approval["descriptor"]["maintenance"]["mode"],
            "release_watch_enrolled"
        );
    } else {
        assert_eq!(
            approval["descriptor"]["maintenance"]["mode"],
            "explicitly_deferred"
        );
        assert_eq!(
            closeout["maintenance_settlement"]["mode"],
            "explicitly_deferred"
        );
        assert!(entry.maintenance.release_watch.is_none());
    }
    assert_eq!(
        entry.support_depth.as_ref().unwrap().enabled_paths,
        Some(vec![
            crate::agent_registry::support_depth::LifecyclePath::Onboarding
        ])
    );
    let rows = host.records();
    // Require the expected checkpoints before inspecting them: an empty history cannot pass.
    for stage in [
        life::LifecycleStage::RuntimeIntegrated,
        life::LifecycleStage::PublicationReady,
        life::LifecycleStage::Published,
        life::LifecycleStage::ClosedBaseline,
    ] {
        assert!(
            rows.iter()
                .any(|r| r.successful() && r.new_state.as_ref().is_some_and(|s| s.stage == stage)),
            "missing {stage:?} admission"
        );
    }
    let artifact: Value = serde_json::from_slice(
        &fs::read(root.join(crate::support_matrix::JSON_OUTPUT_PATH)).unwrap(),
    )
    .unwrap();
    let row = artifact["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["agent"] == "depth_fixture")
        .unwrap();
    assert_eq!(row["depth"]["promises"].as_array().unwrap().len(), 1);
    assert_eq!(row["depth"]["promises"][0]["qualified"], true);
}
