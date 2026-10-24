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
