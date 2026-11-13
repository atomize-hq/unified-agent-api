//! Finite ON2 producer inputs. Reporting records and lifecycle progression are projections, while
//! approval and acquired bytes are immutable dependencies of the existing synthetic execution.
use super::profile::Profile;
use crate::{
    agent_lifecycle::{LifecycleStage, LifecycleState},
    agent_registry::AgentRegistry,
    depth_record::DepthRecord,
    runtime_advertising::source::Source,
};

pub(crate) const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
pub(crate) const APPROVAL: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/approved-agent.toml";
pub(crate) const STATE: &str =
    "docs/agents/lifecycle/depth-fixture-cli-onboarding/governance/lifecycle-state.json";

pub(crate) fn check(source: &Source, p: Profile, record: &DepthRecord) -> Result<(), String> {
    check_inputs(p, record, |path| source.read(path))
}

pub(crate) fn check_inputs(
    p: Profile,
    record: &DepthRecord,
    mut read: impl FnMut(&str) -> Result<String, String>,
) -> Result<(), String> {
    if p != Profile::SyntheticOnboarding {
        return Ok(());
    }
    let template = include_str!(
        "../../tests/fixtures/depth-synthetic/onboarding/registry-entry-deferred.toml"
    );
    let mut entry = AgentRegistry::parse_unenforced(template)
        .map_err(|e| e.to_string())?
        .agents
        .remove(0);
    let enrollment = &entry
        .support_depth
        .as_ref()
        .unwrap()
        .enrollments
        .as_ref()
        .unwrap()[0];
    let mut selected = record.policy.selection.enrollment.clone();
    for operation in &mut selected.operations {
        if operation.classifications.as_ref().is_some_and(|rows| {
            rows.iter().any(|row| {
                row.templates != [crate::agent_registry::support_depth::ObligationTemplate::Core]
            })
        }) {
            return Err("ON2 producer permits only reviewed CORE classifications".into());
        }
        operation.classifications = None;
        operation.classification_basis = None;
    }
    let expected = Profile::Synthetic.record(include_str!(
        "../../tests/fixtures/depth-synthetic/record.json"
    ))?;
    if record.policy.selection.agent_id != p.agent_id()
        || &selected != enrollment
        || !record.policy.path_enabled
        || p.obligations(record) != Profile::Synthetic.obligations(&expected)
    {
        return Err("ON2 producer inputs differ from the finite wrapper selection".into());
    }
    let raw = read(APPROVAL)?;
    let document: serde_json::Value = toml_edit::de::from_str(&raw).map_err(|e| e.to_string())?;
    if document["descriptor"]["maintenance"]["mode"] == "release_watch_enrolled" {
        let registry =
            AgentRegistry::parse_unenforced(&read(crate::agent_registry::REGISTRY_RELATIVE_PATH)?)
                .map_err(|e| e.to_string())?;
        entry.maintenance = registry
            .find(p.agent_id())
            .ok_or("missing ON2 registry entry")?
            .maintenance
            .clone();
    }
    let approval =
        crate::isolated_maintenance::onboarding::approval::read(raw.as_bytes(), APPROVAL, &entry)?;
    let state: LifecycleState = serde_json::from_str(&read(STATE)?).map_err(|e| e.to_string())?;
    state.validate().map_err(|e| e.to_string())?;
    let object = state
        .support_depth
        .as_deref()
        .ok_or("missing ON2 frozen lifecycle object")?;
    if approval.event != record.event
        || state.agent_id != p.agent_id()
        || state.onboarding_pack_prefix != entry.scaffold.onboarding_pack_prefix
        || state.approval_artifact_path != APPROVAL
        || state.approval_artifact_sha256 != super::reference(APPROVAL, raw.as_bytes()).sha256
        || state.lifecycle_stage < LifecycleStage::RuntimeIntegrated
        || state.active_runtime_evidence_run_id.is_none()
        || state.written_paths_sha256.is_none()
        || state.implementation_summary.is_none()
        || record
            .request_mismatch(&object.binding(approval.event))
            .is_some()
    {
        return Err(
            "committed ON2 approval, implementation or frozen lifecycle binding differs".into(),
        );
    }
    for input in &record
        .obligations
        .as_ref()
        .ok_or("missing ON2 frozen O")?
        .acquired_inputs
    {
        if super::reference(&input.path, read(&input.path)?.as_bytes()) != *input {
            return Err("acquired ON2 producer input differs from frozen O".into());
        }
    }
    Ok(())
}

/// Check the collector's immutable onboarding dependencies independently of its claimed graph.
/// The mutable reporting record and lifecycle object are checked by `check` at each source.
pub(crate) fn check_graph(
    source: &Source,
    p: Profile,
    record: &DepthRecord,
    graph: &crate::depth_evidence::payload::DependencyGraph,
) -> Result<(), String> {
    if p != Profile::SyntheticOnboarding {
        return Ok(());
    }
    let mut required = vec![APPROVAL, p.contract()];
    required.extend(
        record
            .obligations
            .as_ref()
            .ok_or("missing ON2 O")?
            .acquired_inputs
            .iter()
            .map(|f| f.path.as_str()),
    );
    for path in required {
        let expected = super::reference(path, source.read(path)?.as_bytes());
        if !graph.inputs.contains(&expected) {
            return Err(format!(
                "onboarding graph lacks immutable producer input: {path}"
            ));
        }
    }
    if !graph.scopes.iter().any(|scope| scope == p.directory()) {
        return Err("onboarding graph lacks complete synthetic fixture scope".into());
    }
    Ok(())
}
