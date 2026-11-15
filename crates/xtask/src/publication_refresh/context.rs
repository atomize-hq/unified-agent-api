//! Shared publication-context validation and the ordinary production loader.
use super::*;

pub(super) fn build_context(workspace_root: &Path, args: &Args) -> Result<RefreshContext, Error> {
    crate::prepare_publication::acquired_inputs::check(workspace_root, &args.approval)
        .map_err(Error::Validation)?;
    let approval =
        load_approval_artifact(workspace_root, &args.approval).map_err(map_approval_error)?;
    let registry =
        AgentRegistry::load(workspace_root).map_err(|err| Error::Validation(err.to_string()))?;
    let entry = registry
        .find(&approval.descriptor.agent_id)
        .cloned()
        .ok_or_else(|| {
            Error::Validation(format!(
                "approval/registry mismatch: `{}` is not present in {}",
                approval.descriptor.agent_id,
                crate::agent_registry::REGISTRY_RELATIVE_PATH
            ))
        })?;
    context_for(workspace_root, approval, entry)
}

pub(super) fn context_for(
    workspace_root: &Path,
    approval: ApprovalArtifact,
    entry: AgentRegistryEntry,
) -> Result<RefreshContext, Error> {
    let lifecycle_state_path =
        agent_lifecycle::lifecycle_state_path(&approval.descriptor.onboarding_pack_prefix);
    let lifecycle_state = load_lifecycle_state_relaxed(workspace_root, &lifecycle_state_path)?;
    validate_approval_continuity(&lifecycle_state, &lifecycle_state_path, &approval)?;
    if lifecycle_state.lifecycle_stage != LifecycleStage::PublicationReady {
        return Err(Error::Validation(format!(
            "refresh-publication requires lifecycle stage `publication_ready` at `{}` (found `{}`)",
            lifecycle_state_path,
            lifecycle_state.lifecycle_stage.as_str()
        )));
    }
    validate_stage_evidence(
        &lifecycle_state,
        LifecycleStage::PublicationReady,
        &lifecycle_state_path,
    )?;
    validate_expected_next_command(&approval, &entry, &lifecycle_state_path, &lifecycle_state)?;

    let publication_packet_path = agent_lifecycle::publication_ready_path_for_entry(&entry);
    let _publication_packet_sha256 = file_sha256(workspace_root, &publication_packet_path)
        .map_err(|err| Error::Validation(err.to_string()))?;
    let publication_packet =
        load_publication_ready_packet_relaxed(workspace_root, &publication_packet_path)?;
    validate_packet_identity(
        &publication_packet_path,
        &publication_packet,
        &approval,
        &entry,
        &lifecycle_state_path,
    )?;

    Ok(RefreshContext {
        approval,
        entry,
        lifecycle_state_path,
        lifecycle_state,
        publication_packet_path,
        publication_packet,
    })
}
