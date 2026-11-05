//! CP-P2: a non-Git scratch view borrows its caller's registry and pinned Git inputs.
use super::{CandidateBinding, CandidateRegistry, CapturedRegistry, FileIdentity, CANDIDATE};
use crate::{
    agent_maintenance::closeout_proposal::IsolatedView, agent_registry::REGISTRY_RELATIVE_PATH,
    git_inputs::EvaluationSource, runtime_advertising::source::Source,
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

impl EvaluationSource {
    pub(crate) fn verify_current(
        &self,
        view: &IsolatedView<'_>,
        caller: &Path,
    ) -> Result<(), String> {
        view.verify_current(caller)?;
        if proposed_source(&self.root)?.fingerprint != self.fingerprint {
            return Err("isolated proposal changed after evaluation binding".into());
        }
        Ok(())
    }
}
impl IsolatedView<'_> {
    /// Recheck the original proposal/caller, destination identity and absence of scratch Git.
    pub(crate) fn verify_current(&self, caller: &Path) -> Result<(), String> {
        self.registry_binding(caller).map(drop)
    }

    /// Read proposed regular bytes only after attesting the original caller and scratch identity.
    pub(crate) fn evaluation_source(&self, caller: &Path) -> Result<EvaluationSource, String> {
        let (root, _, _) = self.registry_binding(caller)?;
        let source = proposed_source(root)?;
        source.verify_current(self, caller)?;
        Ok(source)
    }
}
fn proposed_source(root: &Path) -> Result<EvaluationSource, String> {
    let workspace = Source::workspace(root)?;
    let mut paths = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry
            .file_name()
            .into_string()
            .map_err(|_| "non-UTF8 proposed input")?;
        paths.insert(path.clone());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            paths.extend(workspace.paths(&path)?);
        }
    }
    let mut files = std::collections::BTreeMap::new();
    let mut digest = Sha256::new();
    let mut size = 0usize;
    for path in paths {
        let metadata = std::fs::symlink_metadata(root.join(&path)).map_err(|e| e.to_string())?;
        if metadata.is_dir() {
            continue;
        }
        if metadata.len() > (536_870_912 - size) as u64 || files.len() >= 50_000 {
            return Err("proposed inputs exceed observation limit".into());
        }
        let bytes = workspace
            .optional_bytes(&path)?
            .ok_or("proposed input disappeared")?;
        size = size
            .checked_add(bytes.len())
            .ok_or("proposed inputs exceed observation limit")?;
        if size > 536_870_912 || files.len() >= 50_000 {
            return Err("proposed inputs exceed observation limit".into());
        }
        #[cfg(unix)]
        let executable = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o111 != 0
        };
        #[cfg(not(unix))]
        let executable = false;
        digest.update((path.len() as u64).to_le_bytes());
        digest.update(path.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(&bytes);
        digest.update([u8::from(executable)]);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            digest.update((metadata.permissions().mode() & 0o7777).to_le_bytes());
        }
        files.insert(path, (bytes, executable));
    }
    Ok(EvaluationSource {
        source: Source::captured(root, files)?,
        root: root.to_path_buf(),
        fingerprint: hex::encode(digest.finalize()),
    })
}
impl super::Context {
    pub(crate) fn view_inputs(
        &self,
        view: &IsolatedView<'_>,
        host: &super::EvidenceHost,
    ) -> Result<(EvaluationSource, crate::git_inputs::GitInputs), String> {
        self.recheck(&self.root)?;
        self.require_view_host(host)?;
        let source = view.evaluation_source(&self.root)?;
        let inputs =
            crate::git_inputs::GitInputs::borrow_view(&source.root, &self.inputs, &source.source)?;
        self.verify_view(view, &source, host)?;
        Ok((source, inputs))
    }
    pub(crate) fn verify_view(
        &self,
        view: &IsolatedView<'_>,
        source: &EvaluationSource,
        host: &super::EvidenceHost,
    ) -> Result<(), String> {
        self.recheck(&self.root)?;
        self.require_view_host(host)?;
        source.verify_current(view, &self.root)
    }
    fn require_view_host(&self, host: &super::EvidenceHost) -> Result<(), String> {
        let expected = self.evidence_host(&self.root)?;
        if (host.repository, host.id, host.owner, &host.tip)
            != (
                expected.repository,
                expected.id,
                expected.owner,
                &expected.tip,
            )
        {
            return Err("evaluation host differs from the authenticated caller".into());
        }
        Ok(())
    }
}
