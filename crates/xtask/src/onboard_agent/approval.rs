use std::{fs, path::Path};

use crate::approval_artifact::{self, ApprovalArtifactError};
use toml_edit::DocumentMut;

use super::{DraftDescriptorInput, Error};

pub(super) fn load_descriptor_input(
    approval_path: &str,
    workspace_root: &Path,
    allow_staged_paths: bool,
) -> Result<DraftDescriptorInput, Error> {
    if approval_artifact::is_staged_approval_path(approval_path) && !allow_staged_paths {
        return Err(Error::Validation(
            "staged approval paths under `docs/agents/lifecycle/.staging/**` are only allowed with `--dry-run`".to_string(),
        ));
    }
    let artifact = if allow_staged_paths {
        approval_artifact::load_approval_artifact_for_validation(workspace_root, approval_path)
    } else {
        approval_artifact::load_approval_artifact(workspace_root, approval_path)
    }
    .map_err(map_approval_error)?;
    let approval_recorded_at = load_approval_recorded_at(workspace_root, approval_path)?;
    let mut input: DraftDescriptorInput = artifact.into();
    if let Some(provenance) = input.approval_provenance.as_mut() {
        provenance.approval_recorded_at = approval_recorded_at;
    }
    Ok(input)
}

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}

fn load_approval_recorded_at(workspace_root: &Path, approval_path: &str) -> Result<String, Error> {
    let approval_file = workspace_root.join(approval_path);
    let text = fs::read_to_string(&approval_file)
        .map_err(|err| Error::Internal(format!("read {}: {err}", approval_file.display())))?;
    let document = text
        .parse::<DocumentMut>()
        .map_err(|err| Error::Internal(format!("parse {}: {err}", approval_file.display())))?;
    document["approval_recorded_at"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| {
            Error::Internal(format!(
                "approval artifact `{}` is missing `approval_recorded_at` after validation",
                approval_path
            ))
        })
}

pub(super) fn scaffold_descriptor(
    scaffold: &crate::isolated_maintenance::onboarding::Scaffold,
) -> Result<DraftDescriptorInput, Error> {
    use super::ApprovalProvenance;
    use sha2::{Digest, Sha256};
    let entry = &scaffold.entry;
    let path = &scaffold.approval_path;
    let approval = crate::isolated_maintenance::onboarding::approval::read(
        &scaffold.approval_bytes,
        path,
        entry,
    )
    .map_err(Error::Validation)?;
    let raw = std::str::from_utf8(&scaffold.approval_bytes)
        .map_err(|e| Error::Validation(e.to_string()))?;
    let maintenance =
        approval_artifact::parse_approval_maintenance(raw, path).map_err(map_approval_error)?;
    let crate::depth_record::Event::Onboarding {
        approval_recorded_at,
        ..
    } = approval.event
    else {
        return Err(Error::Validation(
            "bootstrap requires an onboarding approval".into(),
        ));
    };
    Ok(DraftDescriptorInput {
        agent_id: entry.agent_id.clone(),
        display_name: entry.display_name.clone(),
        crate_path: entry.crate_path.clone(),
        backend_module: entry.backend_module.clone(),
        manifest_root: entry.manifest_root.clone(),
        package_name: entry.package_name.clone(),
        canonical_targets: entry.canonical_targets.clone(),
        wrapper_coverage_binding_kind: entry.wrapper_coverage.binding_kind.clone(),
        wrapper_coverage_source_path: entry.wrapper_coverage.source_path.clone(),
        always_on_capabilities: entry.capability_declaration.always_on.clone(),
        target_gated_capabilities: vec![],
        config_gated_capabilities: vec![],
        backend_extensions: entry.capability_declaration.backend_extensions.clone(),
        support_matrix_enabled: entry.publication.support_matrix_enabled,
        capability_matrix_enabled: entry.publication.capability_matrix_enabled,
        capability_matrix_target: entry.publication.capability_matrix_target.clone(),
        docs_release_track: entry.release.docs_release_track.clone(),
        onboarding_pack_prefix: entry.scaffold.onboarding_pack_prefix.clone(),
        approval_provenance: Some(ApprovalProvenance {
            artifact_path: path.clone(),
            artifact_sha256: hex::encode(Sha256::digest(&scaffold.approval_bytes)),
            approval_recorded_at,
        }),
        approval_maintenance: Some(maintenance.into()),
        approval_support_depth_version: approval.support_depth_version,
    })
}
