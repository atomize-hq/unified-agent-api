//! CP-P2: a non-Git scratch view borrows its caller's registry and pinned Git inputs.
use super::{CandidateBinding, CandidateRegistry, CapturedRegistry, FileIdentity, CANDIDATE};
use crate::{
    agent_maintenance::closeout_proposal::IsolatedView, agent_registry::REGISTRY_RELATIVE_PATH,
    runtime_advertising::source::Source,
};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// Observations retained for every lookup while the scratch registry guard is alive.
pub(super) struct Binding {
    destination_identity: FileIdentity,
    registry_bytes: Vec<u8>,
    caller: Arc<super::Route>,
}
impl Binding {
    pub(super) fn recheck(&self, root: &Path) -> Result<(), String> {
        if FileIdentity::read(root)? != self.destination_identity
            || super::entry_exists(&root.join(".git"))?
        {
            return Err("isolated scratch cannot acquire Git authority".into());
        }
        if Source::workspace(root)?
            .optional_bytes(REGISTRY_RELATIVE_PATH)?
            .as_ref()
            != Some(&self.registry_bytes)
        {
            return Err("isolated scratch registry bytes differ from the captured registry".into());
        }
        let caller = &self.caller.0;
        if super::Workspace::read(&caller.root, &caller.profile)? != caller.workspace {
            return Err("isolated caller workspace or Git ownership changed".into());
        }
        Ok(())
    }
}

impl CapturedRegistry {
    /// Bind this registry to the attested scratch root on this thread until the guard drops.
    /// The captured route and proposal must still verify against the same caller; the view
    /// checks its destination identity and refuses Git metadata. Its registry must retain the
    /// exact captured bytes. This supplies no evidence host, admission or write authority.
    pub(crate) fn bind_view(&self, view: &IsolatedView<'_>) -> Result<CandidateRegistry, String> {
        self.recheck(&self.route.0.root)?;
        let (root, destination_identity, expected) = view.registry_binding(&self.route.0.root)?;
        // Compare with the route's captured input identity as well as the proposal's bytes;
        // freshness observations are not a lock. Source refuses symlinked scratch inputs.
        if hex::encode(Sha256::digest(&expected))
            != self
                .route
                .0
                .inputs
                .reference(REGISTRY_RELATIVE_PATH)?
                .sha256
        {
            return Err("isolated scratch registry bytes differ from the captured registry".into());
        }
        let view = Binding {
            destination_identity,
            registry_bytes: expected,
            caller: self.route.clone(),
        };
        view.recheck(root)?;
        CANDIDATE.with(|bound| {
            let mut bound = bound.borrow_mut();
            if bound.is_some() {
                return Err(
                    "an isolated candidate registry is already bound on this thread".into(),
                );
            }
            *bound = Some(CandidateBinding {
                root: root.to_path_buf(),
                registry: self.registry.clone(),
                view: Some(view),
            });
            Ok(CandidateRegistry(std::marker::PhantomData))
        })
    }
}

/// Only the bound scratch root may borrow objects. It never resolves a ref in scratch.
pub(crate) fn git_inputs(root: &Path) -> Result<Option<(PathBuf, String)>, String> {
    CANDIDATE.with(|bound| {
        let bound = bound.borrow();
        let Some(bound) = bound.as_ref() else {
            return Ok(None);
        };
        let Some(view) = &bound.view else {
            return Ok(None);
        };
        view.recheck(&bound.root)?;
        Ok(root
            .canonicalize()
            .ok()
            .filter(|root| *root == bound.root)
            .map(|_| {
                let caller = &view.caller.0;
                (caller.root.clone(), caller.workspace.head.clone())
            }))
    })
}
