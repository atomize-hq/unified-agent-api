//! Shared frozen-input and runtime checks for ON5 preparation and independent IN1.
use crate::{
    agent_lifecycle::{self, LifecycleStage, LifecycleState},
    agent_registry::AgentRegistryEntry,
    approval_artifact::ApprovalArtifact,
    isolated_maintenance::CapturedRegistry,
    runtime_advertising::source::Source,
    workspace_mutation::{PlannedMutation, WorkspacePathJail},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(test)]
#[path = "on5_test_hooks.rs"]
pub(crate) mod proposal_hooks;
#[cfg(test)]
#[allow(clippy::duplicate_mod)]
#[path = "../../publication_refresh/routed/test_hooks.rs"]
pub(crate) mod test_hooks;

pub(crate) fn authority(
    root: &Path,
    path: &str,
    captured: &CapturedRegistry,
    writer: &str,
) -> Result<ApprovalArtifact, String> {
    let committed = Source::tip(root, "HEAD")?;
    let raw = committed.read(path)?;
    if Source::workspace(root)?.read(path)? != raw {
        return Err("ON5 approval differs from committed authority".into());
    }
    let approval =
        crate::runtime_follow_on::depth::approval::execution(root, raw.as_bytes(), path, captured)?;
    let state_path =
        agent_lifecycle::lifecycle_state_path(&approval.descriptor.onboarding_pack_prefix);
    let state: LifecycleState =
        serde_json::from_str(&committed.read(&state_path)?).map_err(|e| e.to_string())?;
    if state.lifecycle_stage != LifecycleStage::Published {
        crate::depth_routes::refuse_workspace(root, writer)?;
        return Err("ON5 requires its committed published predecessor".into());
    }
    if state
        .side_states
        .contains(&agent_lifecycle::SideState::Drifted)
    {
        return Err("onboarding cannot clear maintenance drift".into());
    }
    let entry = captured
        .registry()
        .find(&approval.descriptor.agent_id)
        .ok_or("unknown ON5 agent")?;
    runtime(root, entry, &committed, captured)?;
    Ok(approval)
}

pub(crate) fn runtime(
    root: &Path,
    entry: &AgentRegistryEntry,
    committed: &Source,
    captured: &CapturedRegistry,
) -> Result<(), String> {
    let path = agent_lifecycle::approval_artifact_path_for_entry(entry);
    let approval = crate::runtime_follow_on::depth::approval::execution(
        root,
        committed.read(&path)?.as_bytes(),
        &path,
        captured,
    )?;
    crate::prepare_publication::acquired_inputs::check(root, &path)?;
    let packet_path = agent_lifecycle::publication_ready_path_for_entry(entry);
    let packet = serde_json::from_str(&Source::workspace(root)?.read(&packet_path)?)
        .map_err(|e| e.to_string())?;
    crate::prepare_publication::validate_packet_pinned_runtime_evidence_for_approval(
        root, &approval, &packet,
    )
    .map_err(|e| e.to_string())?;
    // A retained prepared artifact stays reporting-only, but still has the same semantic
    // validation as local preparation, including unchanged-lifecycle and human-edit candidates.
    let closeout =
        agent_lifecycle::proving_run_closeout_path(&entry.scaffold.onboarding_pack_prefix);
    if let Some(raw) = Source::workspace(root)?.optional(&closeout)? {
        let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        if value["state"] == "prepared" {
            crate::proving_run_closeout::from_authenticated(
                &raw,
                Path::new(&closeout),
                &approval,
                &[crate::proving_run_closeout::ProvingRunCloseoutState::Prepared],
            )
            .map_err(|e| e.to_string())?;
            let version = approval
                .support_depth_version
                .as_deref()
                .ok_or("missing approved depth version")?;
            let path = format!(
                "{}/reports/{version}/depth-record.json",
                entry.manifest_root
            );
            let record = crate::depth_record::DepthRecord::parse(
                &Source::workspace(root)?.read(&path)?,
                &entry.agent_id,
                version,
            )?;
            prepared_record(&record)?;
        }
    }
    Ok(())
}

pub(crate) fn materialize(root: &Path, files: &[(PathBuf, Vec<u8>)]) -> Result<(), String> {
    let jail = WorkspacePathJail::new(root).map_err(|e| e.to_string())?;
    for (path, bytes) in files {
        let path = jail.resolve(path).map_err(|e| e.to_string())?;
        fs::create_dir_all(path.parent().ok_or("output has no parent")?)
            .map_err(|e| e.to_string())?;
        fs::write(path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn deliver(root: &Path, mutations: &[PlannedMutation]) -> Result<(), String> {
    #[cfg(test)]
    if let Some(hook) = test_hooks::delivery_hook() {
        return crate::workspace_mutation::delivery::apply_admitted_mutations_with_test_hook(
            root,
            mutations,
            |index| hook(root, mutations, index),
        )
        .map(drop)
        .map_err(|e| e.to_string());
    }
    #[cfg(test)]
    if let Some(index) = test_hooks::delivery_failure() {
        return crate::workspace_mutation::delivery::apply_admitted_mutations_with_test_failure(
            root, mutations, index,
        )
        .map(drop)
        .map_err(|e| e.to_string());
    }
    crate::workspace_mutation::apply_admitted_mutations(root, mutations)
        .map(drop)
        .map_err(|e| e.to_string())
}

/// Published reporting may prepare the closeout while preserving every acceptance assertion.
pub(crate) fn prepared_lifecycle(
    before: &Source,
    after: &Source,
    entry: &AgentRegistryEntry,
) -> Result<(), String> {
    let path = agent_lifecycle::lifecycle_state_path_for_entry(entry);
    let previous: LifecycleState =
        serde_json::from_str(&before.read(&path)?).map_err(|e| e.to_string())?;
    let next: LifecycleState =
        serde_json::from_str(&after.read(&path)?).map_err(|e| e.to_string())?;
    if previous == next {
        return Ok(());
    }
    let mut expected = previous.clone();
    expected.current_owner_command = "prepare-proving-run-closeout --write".into();
    expected.expected_next_command = agent_lifecycle::publication_ready_closeout_command(
        &previous.approval_artifact_path,
        &entry.scaffold.onboarding_pack_prefix,
    );
    expected.last_transition_at = next.last_transition_at.clone();
    expected.last_transition_by = "xtask prepare-proving-run-closeout --write".into();
    if previous.lifecycle_stage != LifecycleStage::Published || next != expected {
        return Err("prepared ON5 changed a retained lifecycle assertion".into());
    }
    let path = agent_lifecycle::proving_run_closeout_path(&entry.scaffold.onboarding_pack_prefix);
    let raw = after.read(&path)?;
    crate::proving_run_closeout::check_closeout_form(&raw)?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if value["state"] != "prepared"
        || !value["maintenance_settlement"].is_null()
        || value["approval_ref"] != previous.approval_artifact_path
        || value["approval_sha256"] != previous.approval_artifact_sha256
    {
        return Err("prepared ON5 requires its exact reporting-only closeout".into());
    }
    Ok(())
}

pub(crate) fn prepared_batch(
    before: Source,
    after: Source,
    registry: &crate::agent_registry::AgentRegistry,
) -> Result<(), String> {
    let profile = crate::depth_ci::profile::Profile::SyntheticOnboarding;
    let candidate = super::View::with_registry(after.clone(), registry, profile.agent_id())?;
    let record = candidate
        .records
        .get(profile.version())
        .ok_or("prepared ON5 has no retained record")?;
    let path = format!(
        "{}/reports/{}/depth-record.json",
        candidate.entry.manifest_root,
        profile.version()
    );
    let prior = crate::depth_record::DepthRecord::parse(
        &before.read(&path)?,
        &candidate.entry.agent_id,
        profile.version(),
    )?;
    prepared_record(&prior)?;
    prepared_lifecycle(&before, &after, &candidate.entry)?;
    // The reporting relay separately forbids every accepting effect and new entry. This check
    // retains pointer ownership in both directions and the aggregate destination boundaries.
    super::check_effects(
        &before,
        &after,
        &candidate,
        record,
        super::Kind::Publication,
    )
}

pub(crate) fn read_prepared(
    root: &Path,
    path: &str,
    approval: &ApprovalArtifact,
) -> Result<Option<crate::proving_run_closeout::ProvingRunCloseout>, String> {
    let jail = WorkspacePathJail::new(root).map_err(|e| e.to_string())?;
    let resolved = jail.resolve(Path::new(path)).map_err(|e| e.to_string())?;
    match fs::read_to_string(resolved) {
        Ok(raw) => crate::proving_run_closeout::from_authenticated(
            &raw,
            Path::new(path),
            approval,
            &[crate::proving_run_closeout::ProvingRunCloseoutState::Prepared],
        )
        .map(Some)
        .map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("read {path}: {e}")),
    }
}

fn prepared_record(record: &crate::depth_record::DepthRecord) -> Result<(), String> {
    if record.acceptance_entries.iter().any(|entry| {
        matches!(
            entry,
            crate::depth_record::AcceptanceEntry::ProvingRunCloseout { .. }
        )
    }) {
        return Err("prepared ON5 cannot replace a retained proving closeout".into());
    }
    Ok(())
}

/// Pin the real caller's commit through the same hardened runner as route authentication.
pub(crate) fn head(root: &Path) -> Result<String, String> {
    let output = crate::git_inputs::command(root)
        .args(["rev-parse", "--verify", "HEAD^{commit}"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("ON5 cannot pin its committed caller head".into());
    }
    String::from_utf8(output.stdout)
        .map(|head| head.trim().to_owned())
        .map_err(|e| e.to_string())
}
