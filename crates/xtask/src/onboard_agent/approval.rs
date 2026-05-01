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
