//! CP-P2: a non-Git scratch view borrows only its caller's captured registry.
use super::{CandidateRegistry, CapturedRegistry, CANDIDATE};
use crate::{
    agent_maintenance::closeout_proposal::IsolatedView, agent_registry::REGISTRY_RELATIVE_PATH,
    runtime_advertising::source::Source,
};
use sha2::{Digest, Sha256};

impl CapturedRegistry {
    /// Bind this registry to the attested scratch root on this thread until the guard drops.
    /// The captured route and proposal must still verify against the same caller; the view
    /// checks its destination identity and refuses Git metadata. Its registry must retain the
    /// exact captured bytes. This supplies no evidence host, admission or write authority.
    pub(crate) fn bind_view(&self, view: &IsolatedView<'_>) -> Result<CandidateRegistry, String> {
        self.recheck(&self.route.0.root)?;
        let (root, expected) = view.registry_binding(&self.route.0.root)?;
        // Compare with the route's captured input identity as well as the proposal's bytes;
        // freshness observations are not a lock. Source refuses symlinked scratch inputs.
        if hex::encode(Sha256::digest(&expected))
            != self
                .route
                .0
                .inputs
                .reference(REGISTRY_RELATIVE_PATH)?
                .sha256
            || Source::workspace(root)?
                .optional_bytes(REGISTRY_RELATIVE_PATH)?
                .as_ref()
                != Some(&expected)
        {
            return Err("isolated scratch registry bytes differ from the captured registry".into());
        }
        CANDIDATE.with(|bound| {
            let mut bound = bound.borrow_mut();
            if bound.is_some() {
                return Err(
                    "an isolated candidate registry is already bound on this thread".into(),
                );
            }
            *bound = Some((root.to_path_buf(), self.registry.clone()));
            Ok(CandidateRegistry(std::marker::PhantomData))
        })
    }
}
