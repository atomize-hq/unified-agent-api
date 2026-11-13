//! Controller-held candidate trees, finalization and expected-before delivery.
//! This is a disposable copy boundary, not an OS sandbox or publication authority.
mod candidate;
#[path = "../agent_maintenance/execute/candidate_git.rs"]
pub(crate) mod candidate_git;
pub(crate) mod delivery_index;
#[path = "../agent_maintenance/execute/index_lock.rs"]
pub(crate) mod index_lock;
pub(crate) mod process;
pub(crate) mod tree;
pub(crate) use candidate::Candidate;

#[derive(Debug)]
pub(crate) enum Error {
    Validation(String),
    Internal(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(s) | Self::Internal(s) => f.write_str(s),
        }
    }
}
impl From<crate::workspace_mutation::WorkspaceMutationError> for Error {
    fn from(e: crate::workspace_mutation::WorkspaceMutationError) -> Self {
        match e {
            crate::workspace_mutation::WorkspaceMutationError::Validation(s) => Self::Validation(s),
            crate::workspace_mutation::WorkspaceMutationError::Internal(s) => Self::Internal(s),
        }
    }
}

/// Failure starts from the controller's before-tree and admits only host diagnostics.
/// The caller never reads failed candidate bytes to construct this tree.
pub(crate) fn finalized_tree(
    before: &tree::Tree,
    observed: Option<&tree::Tree>,
    passed: bool,
    diagnostics: tree::Tree,
) -> tree::Tree {
    let mut delivery = if passed {
        observed.unwrap_or(before).clone()
    } else {
        before.clone()
    };
    delivery.extend(diagnostics);
    delivery
}

const EXECUTION_RUNS_ROOT: &str = "docs/agents/.uaa-temp/agent-maintenance/runs";

/// Hold Git's lock while a trusted route rechecks the caller. Read-only `git status` then
/// cannot refresh index stat-cache bytes and invalidate the invocation's exact index binding.
/// The private index copy is discarded; this never publishes entries.
pub(crate) fn lock_index(root: &std::path::Path) -> Result<impl Drop, Error> {
    index_lock::IndexLock::acquire(root)
}

/// Test-only ownership of a recovery copy; production candidates never clean up on drop.
#[cfg(test)]
#[derive(Debug)]
pub(crate) struct TestHolder(std::path::PathBuf);
#[cfg(test)]
impl TestHolder {
    pub(crate) fn new(candidate: &std::path::Path) -> Self {
        Self(candidate.parent().expect("candidate holder").to_path_buf())
    }
}
#[cfg(test)]
impl Drop for TestHolder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
