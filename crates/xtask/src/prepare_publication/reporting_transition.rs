//! IN1 independently checks the value-based ON3 checkpoint against its pinned predecessor.
use super::*;
use crate::runtime_advertising::source::Source;

pub(crate) fn check(
    root: &Path,
    before: &Source,
    record: &crate::depth_record::DepthRecord,
    entry: &AgentRegistryEntry,
) -> Result<(), String> {
    let captured = xtask::isolated_maintenance::route(root)?
        .ok_or("ON3 reporting transition requires captured onboarding authority")?
        .capture(root)?;
    let approval_path = agent_lifecycle::approval_artifact_path_for_entry(entry);
    let raw = Source::workspace(root)?
        .optional_bytes(&approval_path)?
        .ok_or("ON3 reporting transition lacks approval")?;
    if before.optional_bytes(&approval_path)?.as_ref() != Some(&raw) {
        return Err("ON3 reporting approval differs from pinned predecessor".into());
    }
    let approval = crate::runtime_follow_on::depth::approval::execution(
        root,
        &raw,
        &approval_path,
        &captured,
    )?;
    let context = context_for(root, approval, entry.clone()).map_err(|e| e.to_string())?;
    let prior: LifecycleState = serde_json::from_str(&before.read(&context.lifecycle_state_path)?)
        .map_err(|e| e.to_string())?;
    if !matches!(
        prior.lifecycle_stage,
        LifecycleStage::RuntimeIntegrated | LifecycleStage::PublicationReady
    ) || context.lifecycle_state.lifecycle_stage != LifecycleStage::PublicationReady
        || prior.support_depth != context.lifecycle_state.support_depth
        || prior.written_paths_sha256 != context.lifecycle_state.written_paths_sha256
        || record.policy.selection.enrollment.version
            != context
                .approval
                .support_depth_version
                .as_deref()
                .ok_or("missing approved depth version")?
    {
        return Err(
            "ON3 reporting E requires the preserved publication_ready lifecycle checkpoint".into(),
        );
    }
    acquired_inputs::check(root, &approval_path)?;
    validate_check_mode(root, &context).map_err(|e| e.to_string())?;
    if prior.lifecycle_stage == LifecycleStage::RuntimeIntegrated
        && prior.active_runtime_evidence_run_id.as_deref()
            != Some(context.runtime_evidence.run_id.as_str())
    {
        return Err("ON3 packet does not select its predecessor's active runtime evidence".into());
    }
    captured.recheck(root)
}
