//! Disposable execution and expected-before copy-back. Not a sandbox or integration authority.
use super::{
    candidate_git::{self, GitState},
    tree::{self, Node, Tree},
    types::Context,
    Error, INPUT_CONTRACT_FILE_NAME, PROMPT_FILE_NAME,
};
use crate::workspace_mutation::{apply_admitted_mutations as apply_mutations, PlannedMutation};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) struct Candidate {
    pub root: PathBuf,
    pub context: Context,
    pub before: Tree,
    pub baseline: String,
    caller_git: Option<GitState>,
    config: Vec<u8>,
    default_mode: u32,
    default_dir_mode: u32,
    root_mode: u32,
}
impl Candidate {
    pub fn prepare(caller: &Path, context: &Context) -> Result<Self, Error> {
        let caller_git = GitState::read(caller)?;
        let before = tree::capture(caller, &GitState::tracked(caller)?)?;
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
        let mut ctx = context.clone();
        ctx.run_dir = root.join(&ctx.run_dir_rel);
        if ctx.codex_binary.contains('/') || ctx.codex_binary.contains('\\') {
            let binary = Path::new(&ctx.codex_binary);
            let full = if binary.is_absolute() {
                binary.to_owned()
            } else {
                caller.join(binary)
            };
            ctx.codex_binary = if let Ok(relative) = full.strip_prefix(caller) {
                root.join(relative)
            } else {
                full
            }
            .to_string_lossy()
            .into_owned();
        }
        Ok(Self {
            root: root.clone(),
            context: ctx,
            before,
            baseline,
            caller_git,
            config,
            default_mode,
            default_dir_mode,
            root_mode: tree::mode(&fs::metadata(&root).map_err(tree::io)?),
        })
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
        tree::capture(&self.root, &GitState::tracked(&self.root)?)
    }
    pub fn check(&self, after: &Tree) -> Result<(), Error> {
        for path in tree::changed(&self.before, after) {
            if path.ends_with("maintenance-request.toml")
                || path.split('/').any(|s| s == "automation-stand-down")
                || path == format!("{}/{INPUT_CONTRACT_FILE_NAME}", self.context.run_dir_rel)
                || path == format!("{}/{PROMPT_FILE_NAME}", self.context.run_dir_rel)
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
        let paths: Vec<_> = tree::changed(&self.before, after)
            .into_iter()
            .filter(|p| {
                matches!(after.get(p), Some(Node::File(..)))
                    && !p.starts_with(&format!("{}/", self.context.run_dir_rel))
            })
            .collect();
        let mut checks = Vec::new();
        let mut errors = Vec::new();
        super::validate::validate_written_paths(
            &self.root,
            &self.context,
            &paths,
            "relay",
            &mut checks,
            &mut errors,
        )?;
        if !errors.is_empty() {
            return Err(Error::Validation(errors.join("\n")));
        }
        super::delivery_index::validate(&self.root, self.caller_git.as_ref(), || {
            crate::depth_acceptance::check_captured_relay(
                tree::source(&self.root, &self.before)?,
                tree::source(&self.root, after)?,
                &self.root,
            )
            .map_err(Error::Validation)
        })?;
        // Live publisher validators ran above. Refuse if any executor-visible file changed during them.
        if &self.observe()? != after {
            return Err(Error::Validation(
                "candidate changed during relay validation".into(),
            ));
        }
        Ok(())
    }
    pub fn deliver(&self, caller: &Path, after: &Tree, passed: bool) -> Result<Vec<String>, Error> {
        if passed
            && (GitState::read(caller)? != self.caller_git
                || tree::capture(caller, &GitState::tracked(caller)?)? != self.before)
        {
            return Err(Error::Validation(
                "caller changed during relay; preserving concurrent edits and candidate".into(),
            ));
        }
        let mut mutations = Vec::new();
        let mut paths = Vec::new();
        for path in tree::changed(&self.before, after) {
            let diagnostic = path.starts_with(&format!("{}/", self.context.run_dir_rel));
            if !passed && !diagnostic {
                continue;
            }
            if path == format!("{}/{INPUT_CONTRACT_FILE_NAME}", self.context.run_dir_rel)
                || path == format!("{}/{PROMPT_FILE_NAME}", self.context.run_dir_rel)
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
        apply_mutations(caller, &mutations)?;
        Ok(paths)
    }
    pub fn diagnostic_tree(&self) -> Result<Tree, Error> {
        // A malformed candidate can still supply bounded regular diagnostic files; never follow links.
        let jail = crate::workspace_mutation::WorkspacePathJail::new(&self.root)?;
        let root = jail.resolve(Path::new(&self.context.run_dir_rel))?;
        let files = tree::capture(&root, &Default::default())?;
        Ok(files
            .into_iter()
            .map(|(p, n)| (format!("{}/{p}", self.context.run_dir_rel), n))
            .collect())
    }
    pub fn recovery(&self) -> PathBuf {
        self.root.clone()
    }
}
