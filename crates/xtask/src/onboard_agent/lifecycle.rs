use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::agent_lifecycle::{
    approval_artifact_path, required_evidence_for_stage, write_lifecycle_state, LifecycleStage,
    LifecycleState, SupportTier, LIFECYCLE_SCHEMA_VERSION,
};

use super::{
    DraftEntry, Error, LifecycleStatePreview, CURRENT_OWNER_COMMAND, LAST_TRANSITION_BY,
    RAW_APPROVAL_SHA256_PLACEHOLDER, RAW_LAST_TRANSITION_AT_PLACEHOLDER,
};

pub(super) fn build_lifecycle_state_preview(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<LifecycleStatePreview, Error> {
    let path = crate::agent_lifecycle::lifecycle_state_path(&draft.onboarding_pack_prefix);
    let state = seeded_lifecycle_state(workspace_root, draft)?;
    let contents = render_lifecycle_state_contents(&path, &state)?;
    Ok(LifecycleStatePreview { path, contents })
}

fn seeded_lifecycle_state(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<LifecycleState, Error> {
    let (approval_path, approval_sha256, last_transition_at) =
        lifecycle_seed_provenance(workspace_root, draft)?;
    let required_evidence = required_evidence_for_stage(LifecycleStage::Enrolled).to_vec();

    Ok(LifecycleState {
        schema_version: LIFECYCLE_SCHEMA_VERSION.to_string(),
        agent_id: draft.agent_id.clone(),
        onboarding_pack_prefix: draft.onboarding_pack_prefix.clone(),
        approval_artifact_path: approval_path,
        approval_artifact_sha256: approval_sha256,
        lifecycle_stage: LifecycleStage::Enrolled,
        support_tier: SupportTier::Bootstrap,
        side_states: Vec::new(),
        current_owner_command: CURRENT_OWNER_COMMAND.to_string(),
        expected_next_command: format!("scaffold-wrapper-crate --agent {} --write", draft.agent_id),
        last_transition_at,
        last_transition_by: LAST_TRANSITION_BY.to_string(),
        required_evidence: required_evidence.clone(),
        satisfied_evidence: required_evidence,
        blocking_issues: Vec::new(),
        retryable_failures: Vec::new(),
        implementation_summary: None,
        publication_packet_path: None,
        publication_packet_sha256: None,
        closeout_baseline_path: None,
    })
}

fn lifecycle_seed_provenance(
    workspace_root: &Path,
    draft: &DraftEntry,
) -> Result<(String, String, String), Error> {
    if let Some(provenance) = draft.approval_provenance.as_ref() {
        return Ok((
            provenance.artifact_path.clone(),
            provenance.artifact_sha256.clone(),
            provenance.approval_recorded_at.clone(),
        ));
    }

    let approval_path = approval_artifact_path(&draft.onboarding_pack_prefix);
    let approval_sha256 = if workspace_root.join(&approval_path).is_file() {
        sha256_hex(&workspace_root.join(&approval_path))?
    } else {
        RAW_APPROVAL_SHA256_PLACEHOLDER.to_string()
    };

    Ok((
        approval_path,
        approval_sha256,
        RAW_LAST_TRANSITION_AT_PLACEHOLDER.to_string(),
    ))
}

fn render_lifecycle_state_contents(
    relative_path: &str,
    state: &LifecycleState,
) -> Result<String, Error> {
    let temp_root = std::env::temp_dir().join(format!(
        "xtask-onboard-agent-lifecycle-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| Error::Internal(format!("system time before unix epoch: {err}")))?
            .as_nanos()
    ));
    fs::create_dir(&temp_root)
        .map_err(|err| Error::Internal(format!("create {}: {err}", temp_root.display())))?;

    let rendered = (|| -> Result<String, Error> {
        write_lifecycle_state(&temp_root, relative_path, state).map_err(map_lifecycle_error)?;
        let rendered_path = temp_root.join(relative_path);
        let bytes = fs::read(&rendered_path)
            .map_err(|err| Error::Internal(format!("read {}: {err}", rendered_path.display())))?;
        String::from_utf8(bytes).map_err(|err| {
            Error::Internal(format!(
                "decode {} as utf-8: {err}",
                rendered_path.display()
            ))
        })
    })();

    let cleanup_result = fs::remove_dir_all(&temp_root)
        .map_err(|err| Error::Internal(format!("remove {}: {err}", temp_root.display())));

    let rendered = rendered?;
    cleanup_result?;
    Ok(rendered)
}

fn map_lifecycle_error(err: crate::agent_lifecycle::LifecycleError) -> Error {
    match err {
        crate::agent_lifecycle::LifecycleError::Validation(message) => Error::Validation(message),
        crate::agent_lifecycle::LifecycleError::Internal(message) => Error::Internal(message),
    }
}

fn sha256_hex(path: &Path) -> Result<String, Error> {
    use sha2::{Digest, Sha256};

    let bytes =
        fs::read(path).map_err(|err| Error::Internal(format!("read {}: {err}", path.display())))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}
