use super::*;

pub(super) fn check(
    view: &View,
    committed: &Source,
    record: &DepthRecord,
) -> Result<LifecycleState, String> {
    let version = &record.policy.selection.enrollment.version;
    if record.policy.selection.enrollment.lifecycle_path != LifecyclePath::Onboarding
        || !record.policy.path_enabled
        || !matches!(record.event, Event::Onboarding { .. })
    {
        return Err("onboarding has no lifecycle-path authority".into());
    }
    if view.records.get(version) != Some(record) {
        return Err("onboarding record differs from coherent candidate bytes".into());
    }
    let registry = crate::agent_registry::REGISTRY_RELATIVE_PATH;
    if view.source.read(registry)? != committed.read(registry)? {
        return Err("onboarding cannot change registry authority".into());
    }
    let path = agent_lifecycle::approval_artifact_path_for_entry(&view.entry);
    let raw = committed.read(&path)?;
    if view.source.read(&path)? != raw {
        return Err("onboarding approval differs from committed authority".into());
    }
    let approval = OnboardingApproval::parse(&raw, &path)?;
    if approval.event != record.event || approval.support_depth_version.as_ref() != Some(version) {
        return Err("onboarding approval is not the exact current Event/version".into());
    }
    if let Some(why) = approval.attribution_mismatch(
        &view.entry.agent_id,
        &view.entry.manifest_root,
        &view.entry.scaffold.onboarding_pack_prefix,
    ) {
        return Err(why);
    }
    let record_path = format!(
        "{}/reports/{version}/depth-record.json",
        view.entry.manifest_root
    );
    let prior = DepthRecord::parse(
        &committed.read(&record_path)?,
        &view.entry.agent_id,
        version,
    )?;
    if prior.event != record.event
        || prior.policy != record.policy
        || prior.obligations != record.obligations
    {
        return Err("onboarding candidate changed its committed Event/P/O".into());
    }
    if !record
        .acceptance_entries
        .starts_with(&prior.acceptance_entries)
    {
        return Err("onboarding candidate changed retained entries".into());
    }
    let state_path = agent_lifecycle::lifecycle_state_path_for_entry(&view.entry);
    let state_raw = view.source.read(&state_path)?;
    super::super::input::json(&state_raw)?;
    let state: LifecycleState = serde_json::from_str(&state_raw).map_err(|e| e.to_string())?;
    state.validate().map_err(|e| e.to_string())?;
    let previous_raw = committed.read(&state_path)?;
    super::super::input::json(&previous_raw)?;
    let previous: LifecycleState =
        serde_json::from_str(&previous_raw).map_err(|e| e.to_string())?;
    previous.validate().map_err(|e| e.to_string())?;
    if state.agent_id != view.entry.agent_id
        || state.onboarding_pack_prefix != view.entry.scaffold.onboarding_pack_prefix
        || state.approval_artifact_path != path
        || state.approval_artifact_sha256 != artifacts::digest(&raw)
        || state.support_depth.as_deref().map_or(true, |object| {
            record
                .request_mismatch(&object.binding(approval.event.clone()))
                .is_some()
        })
        || state.support_depth != previous.support_depth
        || state.written_paths_sha256 != previous.written_paths_sha256
    {
        return Err("onboarding lifecycle is not the exact approval/P/O binding".into());
    }
    if state.side_states.contains(&SideState::Drifted)
        || previous.side_states.contains(&SideState::Drifted)
    {
        return Err("onboarding cannot clear maintenance drift".into());
    }
    Ok(state)
}

pub(super) fn proving(view: &View, state: &LifecycleState, path: &str) -> Result<(), String> {
    use crate::proving_run_closeout as closeout;
    let raw = view.source.read(path)?;
    closeout::check_closeout_form(&raw)?;
    let value = super::super::input::json(&raw)?;
    if value["state"] != "closed" || value["preflight_passed"] != true {
        return Err(
            "onboarding proving requires exact closed closeout with passed preflight".into(),
        );
    }
    if value["approval_ref"] != state.approval_artifact_path
        || value["approval_sha256"] != state.approval_artifact_sha256
    {
        return Err("onboarding proving closeout is not bound to its exact approval".into());
    }
    let maintenance = crate::approval_artifact::parse_approval_maintenance(
        &view.source.read(&state.approval_artifact_path)?,
        &state.approval_artifact_path,
    )
    .map_err(|e| e.to_string())?;
    let settlement = crate::close_proving_run::settlement::settle_maintenance_readiness(
        &view.entry,
        &maintenance,
        std::path::Path::new(path),
    )
    .map_err(|e| e.to_string())?;
    let expected = serde_json::json!({"mode":settlement.mode.as_str(),"approval_section_sha256":settlement.approval_section_sha256,"release_watch_sha256":settlement.release_watch_sha256,"deferral_sha256":settlement.deferral_sha256});
    if value["maintenance_settlement"] != expected {
        return Err("onboarding proving maintenance settlement differs from exact approval".into());
    }
    // Reuse the shared semantic field validator, after the strict raw schema reader above.
    let duration = match (
        value["duration_seconds"].as_u64(),
        value["duration_missing_reason"].as_str(),
    ) {
        (Some(seconds), None) => closeout::DurationTruth::Seconds(seconds),
        (None, Some(reason)) => closeout::DurationTruth::MissingReason(reason.into()),
        _ => return Err("onboarding proving requires exact duration truth".into()),
    };
    let residual = match (
        value["residual_friction"].as_array(),
        value["explicit_none_reason"].as_str(),
    ) {
        (Some(items), None) => closeout::ResidualFrictionTruth::Items(
            items
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or("invalid residual friction".to_owned())
                })
                .collect::<Result<_, _>>()?,
        ),
        (None, Some(reason)) => closeout::ResidualFrictionTruth::ExplicitNone(reason.into()),
        _ => return Err("onboarding proving requires exact residual friction truth".into()),
    };
    let text = |key| super::super::input::text(&value, key).map(str::to_owned);
    let count = |key| value[key].as_u64().ok_or_else(|| format!("invalid {key}"));
    let parsed = closeout::build_closeout(
        closeout::ProvingRunCloseoutState::Closed,
        closeout::ProvingRunCloseoutMachineFields {
            approval_ref: text("approval_ref")?,
            approval_sha256: text("approval_sha256")?,
            approval_source: text("approval_source")?,
            maintenance_settlement: Some(settlement),
            preflight_passed: true,
            recorded_at: text("recorded_at")?,
            commit: text("commit")?,
        },
        closeout::ProvingRunCloseoutHumanFields {
            manual_control_plane_edits: count("manual_control_plane_edits")?,
            partial_write_incidents: count("partial_write_incidents")?,
            ambiguous_ownership_incidents: count("ambiguous_ownership_incidents")?,
            duration,
            residual_friction: residual,
        },
    )
    .map_err(|e| e.to_string())?;
    if closeout::has_unresolved_placeholders(&parsed) {
        return Err("onboarding proving has unresolved placeholders".into());
    }
    Ok(())
}
