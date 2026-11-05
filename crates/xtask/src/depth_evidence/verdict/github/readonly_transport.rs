//! CP-P3 restricts the existing host adapter to caller-owned immutable objects during view
//! evaluation. Outside that private scope the original transport and acquisition are unchanged.
pub(in crate::depth_evidence::verdict) use super::transport_impl::capture;
#[cfg(test)]
pub(crate) use super::transport_impl::test_reader;
pub(super) use super::transport_impl::{list, required_string, sha, Api, Isolated, CANONICAL};
use crate::{git_inputs::GitInputs, runtime_advertising::source::Source};
use std::{cell::RefCell, path::Path};

thread_local! {
    static READ_ONLY: RefCell<Option<GitInputs>> = const { RefCell::new(None) };
}
/// A restriction, never authority: inherited profile/adoption helpers cannot acquire objects,
/// use a scratch object store, or fetch a missing commit. The original binding must remain live.
pub(in crate::depth_evidence::verdict) struct ReadOnly;
impl Drop for ReadOnly {
    fn drop(&mut self) {
        READ_ONLY.with(|slot| *slot.borrow_mut() = None);
    }
}
pub(in crate::depth_evidence::verdict) fn read_only(
    inputs: &GitInputs,
) -> Result<ReadOnly, String> {
    if !inputs.is_proposed() {
        return Err("read-only view provenance requires proposed caller inputs".into());
    }
    // Compiled-profile checks also invoke Git directly. Refuse inherited redirects rather
    // than allowing those checks to consult another object store or injected Git config.
    if std::env::vars_os().any(|(key, _)| {
        let key = key.to_string_lossy();
        key.starts_with("GIT_")
            && !matches!(
                key.as_ref(),
                "GIT_PAGER" | "GIT_EDITOR" | "GIT_SEQUENCE_EDITOR" | "GIT_TERMINAL_PROMPT"
            )
    }) {
        return Err("view evaluation refuses inherited Git environment overrides".into());
    }
    inputs.verify()?;
    READ_ONLY.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_some() {
            return Err("view provenance is already bound on this thread".into());
        }
        *slot = Some(inputs.clone());
        Ok(ReadOnly)
    })
}
pub(in crate::depth_evidence::verdict) fn source(
    root: &Path,
    repository: &str,
    commit: &str,
) -> Result<Source, String> {
    sha(commit)?;
    let borrowed = READ_ONLY.with(|slot| {
        slot.borrow().as_ref().map(|inputs| {
            if root != inputs.object_root() {
                return Err("view provenance must use the caller's Git object store".into());
            }
            inputs.provenance(commit)
        })
    });
    match borrowed {
        Some(result) => result,
        None => super::transport_impl::source(root, repository, commit),
    }
}
