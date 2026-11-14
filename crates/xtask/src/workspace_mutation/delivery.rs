//! Guarded public delivery and the private mechanical entry for checked owning routes.
use super::*;

pub fn apply_mutations(
    workspace_root: &Path,
    mutations: &[PlannedMutation],
) -> Result<ApplySummary, WorkspaceMutationError> {
    xtask::depth_routes::refuse_workspace(workspace_root, "neutral mutation writer")
        .map_err(WorkspaceMutationError::Validation)?;
    for mutation in mutations {
        if !mutation.remove {
            xtask::depth_routes::refuse_output(
                workspace_root,
                mutation.relative_path(),
                &mutation.desired_after,
                "neutral mutation writer",
            )
            .map_err(WorkspaceMutationError::Validation)?;
        }
    }
    apply_admitted_mutations(workspace_root, mutations)
}

#[cfg(test)]
pub(crate) fn apply_admitted_mutations_with_test_failure(
    root: &Path,
    mutations: &[PlannedMutation],
    fail_before: usize,
) -> Result<ApplySummary, WorkspaceMutationError> {
    apply_with_hook(root, mutations, |index| {
        if index == fail_before {
            Err(WorkspaceMutationError::Internal(
                "injected repair delivery failure".into(),
            ))
        } else {
            Ok(())
        }
    })
}

impl PlannedMutation {
    /// Owning proposals may retire a regular file in the same checked batch as their writes.
    pub(crate) fn remove(relative_path: impl Into<PathBuf>, expected_before: Vec<u8>) -> Self {
        let mut mutation = Self::replace(relative_path, expected_before, Vec::new());
        mutation.remove = true;
        mutation
    }

    pub fn create(relative_path: impl Into<PathBuf>, desired_after: Vec<u8>) -> Self {
        Self {
            relative_path: relative_path.into(),
            expected_before: None,
            desired_after,
            remove: false,
            expected_mode: None,
            git_mode: None,
        }
    }

    pub fn replace(
        relative_path: impl Into<PathBuf>,
        expected_before: Vec<u8>,
        desired_after: Vec<u8>,
    ) -> Self {
        Self {
            relative_path: relative_path.into(),
            expected_before: Some(expected_before),
            desired_after,
            remove: false,
            expected_mode: None,
            git_mode: None,
        }
    }

    /// Optional mode precondition for invocation snapshots; bytes alone cannot detect chmod.
    pub(crate) fn with_expected_mode(mut self, mode: u32) -> Self {
        self.expected_mode = Some(mode);
        self
    }

    /// Exact Git regular-file mode for freeze delivery, including newly created files.
    pub(crate) fn with_git_mode(mut self, mode: u32) -> Self {
        self.git_mode = Some(mode);
        self
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }
}
