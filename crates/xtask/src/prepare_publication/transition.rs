use super::*;

pub(super) fn prepare(
    context: &PublicationContext,
) -> Result<(LifecycleState, PublicationReadyPacket), Error> {
    if context.lifecycle_state.lifecycle_stage != LifecycleStage::RuntimeIntegrated {
        return Err(Error::Validation(format!(
            "prepare-publication --write requires lifecycle stage `runtime_integrated` at `{}` (found `{}`)",
            context.lifecycle_state_path,
            context.lifecycle_state.lifecycle_stage.as_str()
        )));
    }
    validate_stage_evidence(
        &context.lifecycle_state,
        LifecycleStage::RuntimeIntegrated,
        &context.lifecycle_state_path,
    )?;
    let implementation_summary = require_explicit_implementation_summary(
        &context.lifecycle_state,
        &context.lifecycle_state_path,
    )?
    .clone();

    let mut next_state = context.lifecycle_state.clone();
    next_state.lifecycle_stage = LifecycleStage::PublicationReady;
    next_state.support_tier = SupportTier::BaselineRuntime;
    next_state.current_owner_command = "prepare-publication --write".to_string();
    next_state.expected_next_command =
        agent_lifecycle::publication_ready_refresh_command(&context.approval.relative_path);
    next_state.last_transition_at =
        now_rfc3339().map_err(|err| Error::Internal(err.to_string()))?;
    next_state.last_transition_by = "xtask prepare-publication --write".to_string();
    next_state.required_evidence =
        required_evidence_for_stage(LifecycleStage::PublicationReady).to_vec();
    next_state.satisfied_evidence =
        required_evidence_for_stage(LifecycleStage::PublicationReady).to_vec();
    next_state
        .side_states
        .retain(|state| !matches!(state, SideState::Blocked | SideState::FailedRetryable));
    next_state.blocking_issues.clear();
    next_state.retryable_failures.clear();
    next_state.active_runtime_evidence_run_id = None;
    next_state.implementation_summary = Some(implementation_summary.clone());
    next_state.publication_packet_path = None;
    next_state.publication_packet_sha256 = None;
    next_state
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;

    let lifecycle_state_sha256 = sha256_json(&next_state)?;
    let packet = build_publication_ready_packet(
        &context.approval,
        &context.entry,
        &context.lifecycle_state_path,
        &lifecycle_state_sha256,
        context.publication_packet_path.clone(),
        implementation_summary,
        context.runtime_evidence.runtime_evidence_paths.clone(),
        next_state.blocking_issues.clone(),
    )?;
    packet
        .validate()
        .map_err(|err| Error::Validation(err.to_string()))?;
    Ok((next_state, packet))
}
