use std::path::Path;

use crate::approval_artifact::{self, ApprovalArtifactError};

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
    Ok(artifact.into())
}

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}
