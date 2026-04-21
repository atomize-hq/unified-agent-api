use std::path::Path;

use crate::approval_artifact::{self, ApprovalArtifactError};

use super::{DraftDescriptorInput, Error};

pub(super) fn load_descriptor_input(
    approval_path: &str,
    workspace_root: &Path,
) -> Result<DraftDescriptorInput, Error> {
    let artifact = approval_artifact::load_approval_artifact(workspace_root, approval_path)
        .map_err(map_approval_error)?;
    Ok(artifact.into())
}

fn map_approval_error(err: ApprovalArtifactError) -> Error {
    match err {
        ApprovalArtifactError::Validation(message) => Error::Validation(message),
        ApprovalArtifactError::Internal(message) => Error::Internal(message),
    }
}
