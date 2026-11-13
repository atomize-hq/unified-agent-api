//! Independent candidate state and checked delivery, shared by both execution lanes.
use super::{
    candidate_git::{self, GitState},
    tree::{self, Node, Tree},
    Error,
};
use crate::workspace_mutation::{apply_admitted_mutations as apply_mutations, PlannedMutation};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) struct Candidate {
    holder: PathBuf,
    pub root: PathBuf,
    pub before: Tree,
    pub baseline: String,
    caller_git: Option<GitState>,
    tracked: std::collections::BTreeSet<String>,
    config: Vec<u8>,
    default_mode: u32,
    default_dir_mode: u32,
    root_mode: u32,
    pub run_dir_rel: String,
    protected: Vec<String>,
}
impl Candidate {
    pub fn prepare(
        caller: &Path,
        run_dir_rel: &str,
        protected: Vec<String>,
    ) -> Result<Self, Error> {
        let caller_git = GitState::read(caller)?;
        let tracked = GitState::tracked(caller)?;
        let before = tree::capture(caller, &tracked)?;
        let holder = tempfile::Builder::new()
            .prefix("uaa-relay-")
            .tempdir()
            .map_err(tree::io)?
            .keep();
        let root = holder.join("candidate");
        candidate_git::initialize(caller, &root, caller_git.is_some())?;
        tree::materialize(&root, &before)?;
        let probe = holder.join("mode-probe");
        fs::write(&probe, []).map_err(tree::io)?;
        let default_mode = tree::mode(&fs::metadata(&probe).map_err(tree::io)?);
        fs::remove_file(&probe).map_err(tree::io)?;
        fs::create_dir(&probe).map_err(tree::io)?;
        let default_dir_mode = tree::mode(&fs::metadata(&probe).map_err(tree::io)?);
        fs::remove_dir(probe).map_err(tree::io)?;
        super::delivery_index::import(caller, &root, caller_git.as_ref())?;
        let baseline = candidate_git::baseline(&root)?;
        let config = fs::read(root.join(".git/config")).map_err(tree::io)?;
        Ok(Self {
            holder,
            root: root.clone(),
            before,
            baseline,
            caller_git,
            tracked,
            config,
            default_mode,
            default_dir_mode,
            root_mode: tree::mode(&fs::metadata(&root).map_err(tree::io)?),
            run_dir_rel: run_dir_rel.into(),
            protected,
        })
    }
    /// Execution errors are values: observation runs unconditionally once execution may begin.
    /// Each lane then constructs host diagnostics and finalizes both outcomes.
    pub fn execute_observed<T, E>(
        &self,
        execute: impl FnOnce() -> Result<T, E>,
    ) -> (Result<T, E>, Result<Tree, Error>) {
        let execution = execute();
        let observation = self.observe();
        (execution, observation)
    }
    pub fn observe(&self) -> Result<Tree, Error> {
        if tree::mode(&fs::symlink_metadata(&self.root).map_err(tree::io)?) != self.root_mode {
            return Err(Error::Validation(
                "executor changed candidate root mode".into(),
            ));
        }
        let meta = fs::symlink_metadata(self.root.join(".git")).map_err(tree::io)?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(Error::Validation(
                "executor changed candidate Git metadata type".into(),
            ));
        }
        candidate_git::validate_metadata(&self.root)?;
        if fs::read(self.root.join(".git/config")).map_err(tree::io)? != self.config
            || String::from_utf8(candidate_git::run(&self.root, &["rev-parse", "HEAD"])?)
                .map_err(|e| Error::Internal(e.to_string()))?
                .trim()
                != self.baseline
        {
            return Err(Error::Validation(
                "executor changed candidate HEAD or Git configuration".into(),
            ));
        }
        let mut tracked = GitState::tracked(&self.root)?;
        tracked.extend(self.tracked.iter().cloned());
        tree::capture(&self.root, &tracked)
    }
    pub fn check(&self, after: &Tree) -> Result<(), Error> {
        for path in tree::changed(&self.before, after) {
            if self
                .protected
                .iter()
                .any(|p| path == *p || path.starts_with(&format!("{p}/")))
                || path.split('/').any(|s| s == "depth-acquisition")
                || path.ends_with("maintenance-request.toml")
                || path.split('/').any(|s| s == "automation-stand-down")
                || path == format!("{}/input-contract.json", self.run_dir_rel)
                || path == format!("{}/codex-prompt.md", self.run_dir_rel)
            {
                return Err(Error::Validation(format!(
                    "executor changed protected request/marker/frozen packet: {path}"
                )));
            }
            match (self.before.get(&path), after.get(&path)) {
                (_, Some(Node::Link(_))) | (Some(Node::Link(_)), _) => {
                    return Err(Error::Validation(format!(
                        "executor symlink/type change: {path}"
                    )))
                }
                (Some(Node::File(_, mode)), Some(Node::File(_, new))) if mode != new => {
                    return Err(Error::Validation(format!("executor mode change: {path}")))
                }
                (None, Some(Node::File(_, mode))) if *mode != self.default_mode => {
                    return Err(Error::Validation(format!(
                        "unsupported new file mode: {path}"
                    )))
                }
                (None, Some(Node::Directory(mode))) if *mode == self.default_dir_mode => {}
                (Some(Node::File(..)), Some(Node::File(..))) | (None, Some(Node::File(..))) => {}
                _ => {
                    return Err(Error::Validation(format!(
                        "unsupported deletion/type change: {path}"
                    )))
                }
            }
        }
        Ok(())
    }
    pub fn validate_index<T>(
        &self,
        indexed: &[super::delivery_index::Bound],
        check: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        super::delivery_index::validate(&self.root, self.caller_git.as_ref(), indexed, check)
    }
    pub fn deliver(
        &self,
        caller: &Path,
        after: &Tree,
        passed: bool,
        staged: Option<&std::collections::BTreeMap<String, String>>,
    ) -> Result<Vec<String>, Error> {
        self.deliver_with(caller, after, passed, staged, &[])
    }
    pub fn deliver_with(
        &self,
        caller: &Path,
        after: &Tree,
        passed: bool,
        staged: Option<&std::collections::BTreeMap<String, String>>,
        trusted: &[PlannedMutation],
    ) -> Result<Vec<String>, Error> {
        if passed
            && (GitState::read(caller)? != self.caller_git
                || tree::capture(caller, &GitState::tracked(caller)?)? != self.before)
        {
            return Err(Error::Validation(
                "caller changed during relay; preserving concurrent edits and candidate".into(),
            ));
        }
        if passed {
            // Copy-back delivers exactly the bytes the binding admitted; checked again here.
            super::delivery_index::check_staged_closure(
                &self.before,
                after,
                &self.run_dir_rel,
                staged,
            )?;
        }
        // Diagnostics are host-serialized; still, none is delivered over a path that bound
        // evidence names in the caller or in the tree being delivered.
        let bound = super::delivery_index::evidence_paths(&self.before, after);
        let mut mutations = trusted.to_vec();
        let mut paths = Vec::new();
        for path in tree::changed(&self.before, after) {
            let diagnostic = path.starts_with(&format!("{}/", self.run_dir_rel));
            if !passed && !diagnostic {
                continue;
            }
            if diagnostic && bound.contains(&path) {
                continue;
            }
            if self.protected.contains(&path)
                || path == format!("{}/input-contract.json", self.run_dir_rel)
                || path == format!("{}/codex-prompt.md", self.run_dir_rel)
            {
                continue;
            }
            let Some(Node::File(bytes, _)) = after.get(&path) else {
                continue;
            };
            let mutation = match self.before.get(&path) {
                Some(Node::File(old, mode)) => {
                    PlannedMutation::replace(&path, old.clone(), bytes.clone())
                        .with_expected_mode(*mode)
                }
                None => PlannedMutation::create(&path, bytes.clone()),
                _ => {
                    return Err(Error::Validation(
                        "delivery would replace a non-file".into(),
                    ))
                }
            };
            mutations.push(mutation);
            if !diagnostic {
                paths.push(path)
            }
        }
        // The relay validated exactly the caller index plus the new E-bound evidence. Publish
        // those entries from the validated bytes first, so an index failure writes nothing; if a
        // file write then fails, remove exactly the entries this delivery owns.
        let published = if passed && self.caller_git.is_some() {
            super::delivery_index::publish(
                caller,
                &super::delivery_index::bound_new_evidence(
                    &self.before,
                    after,
                    &self.run_dir_rel,
                    staged,
                ),
            )?
        } else {
            vec![]
        };
        if let Err(e) = apply_mutations(caller, &mutations) {
            return Err(match super::delivery_index::unpublish(caller, &published) {
                Ok(()) => e.into(),
                Err(conflict) => Error::Validation(format!("{e}; {conflict}")),
            });
        }
        Ok(paths)
    }
    /// Explicit, best-effort cleanup after successful delivery only. Early returns and panics
    /// deliberately retain the independent copy for recovery.
    pub fn discard(self) {
        let _ = fs::remove_dir_all(self.holder);
    }
    pub fn recovery(&self) -> PathBuf {
        self.root.clone()
    }
}
