//! Invocation-local isolated maintenance binding. Scope and freshness never grant admission.
// The foundation deliberately has no installed profile or production bound-Proposal caller.
#![allow(dead_code)]
mod host;
mod scope;

use crate::{
    agent_maintenance::closeout_proposal::Proposal, agent_registry::AgentRegistry,
    git_inputs::GitInputs, runtime_advertising::source::Source,
};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

trait Api: Send + Sync {
    fn json(&self, endpoint: &str) -> Result<Value, String>;
}
#[derive(Clone)]
struct Repository {
    id: u64,
    owner: u64,
    name: &'static str,
}
// Source-installed authority only. No Deserialize, environment loader, public constructor or receipt.
struct Profile {
    isolated: Repository,
    canonical: Repository,
    baseline: String,
    // Exact compiled bytes and executable modes for the independently approved producer closure.
    closure: Vec<(&'static str, &'static [u8], bool)>,
    scopes: Vec<&'static str>,
}
const LIVE_PROFILE: Option<Profile> = None;

/// Read-only diagnostic. There is no selector for a repository, profile, trust or write mode.
pub fn check(root: &Path) -> Result<(), String> {
    Context::resolve(root, LIVE_PROFILE, Arc::new(host::Github)).map(drop)
}

pub(crate) struct Context {
    profile: Profile,
    api: Arc<dyn Api>,
    root: PathBuf,
    workspace: Workspace,
    host: host::Observation,
    inputs: GitInputs,
    registry: AgentRegistry,
}
impl Context {
    fn resolve(root: &Path, profile: Option<Profile>, api: Arc<dyn Api>) -> Result<Self, String> {
        // This precedes even Git/filesystem probing and all authenticated host calls.
        let profile = profile.ok_or("isolated maintenance profile is not installed")?;
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let workspace = Workspace::read(&root, &profile)?;
        let host = host::observe(api.as_ref(), &profile, &workspace.head)?;
        let mut inputs = GitInputs::new(&root)?;
        source(&root, &profile, &host, &mut inputs)?;
        let registry = scope::load(
            &root,
            &mut inputs,
            &Source::tip(&root, &host.isolated_tip)?,
            &host.registry,
        )?;
        let result = Self {
            profile,
            api,
            root,
            workspace,
            host,
            inputs,
            registry,
        };
        result.recheck(&result.root)?;
        Ok(result)
    }
    pub(crate) fn recheck(&self, root: &Path) -> Result<(), String> {
        if root.canonicalize().map_err(|e| e.to_string())? != self.root
            || Workspace::read(&self.root, &self.profile)? != self.workspace
        {
            return Err("isolated caller workspace or Git ownership changed".into());
        }
        self.inputs.verify()?;
        if host::observe(self.api.as_ref(), &self.profile, &self.workspace.head)? != self.host {
            return Err("isolated host identity, source or protected tip changed".into());
        }
        // Recheck local reads after the potentially slow host observations as well.
        self.inputs.verify()?;
        if Workspace::read(&self.root, &self.profile)? != self.workspace {
            return Err("isolated workspace moved during host observation".into());
        }
        Ok(())
    }
    fn proposal(self, root: &Path) -> Result<Proposal, String> {
        self.recheck(root)?;
        Proposal::capture_isolated(root, self)
    }
    pub(crate) fn registry(&self, root: &Path) -> Result<&AgentRegistry, String> {
        self.recheck(root)?;
        Ok(&self.registry)
    }
}

#[derive(PartialEq, Eq)]
struct Workspace {
    head: String,
    root_identity: FileIdentity,
    git_identity: FileIdentity,
    index: String,
}
#[derive(PartialEq, Eq)]
pub(crate) struct FileIdentity {
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
}
impl FileIdentity {
    pub(crate) fn read(path: &Path) -> Result<Self, String> {
        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err("isolated root/Git metadata must be owned regular directories".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            Ok(Self {
                device: meta.dev(),
                inode: meta.ino(),
            })
        }
        #[cfg(not(unix))]
        {
            Err("isolated workspace ownership requires Unix filesystem identity".into())
        }
    }
}
impl Workspace {
    fn read(root: &Path, profile: &Profile) -> Result<Self, String> {
        let root_identity = FileIdentity::read(root)?;
        let git_identity = FileIdentity::read(&root.join(".git"))?;
        for path in [
            ".git/objects",
            ".git/objects/info",
            ".git/index",
            ".git/HEAD",
            ".git/config",
            ".git/refs",
            ".git/packed-refs",
        ] {
            match fs::symlink_metadata(root.join(path)) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err("shared symlinked isolated Git metadata".into())
                }
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                    return Err(error.to_string())
                }
                _ => {}
            }
        }
        // IB-DEF-1: a real top-level directory can still delegate objects or references
        // through symlinked fanout, pack, namespace or leaf entries. Revisit descendants on
        // every Workspace read, including both freshness observations in Context::recheck.
        let mut remaining = 100_000;
        for namespace in [".git/objects", ".git/refs"] {
            let path = root.join(namespace);
            if entry_exists(&path)? {
                inspect_git_namespace(&path, 0, &mut remaining)?;
            }
        }
        for (option, expected) in [
            ("--show-toplevel", root.to_owned()),
            ("--absolute-git-dir", root.join(".git")),
            ("--git-common-dir", root.join(".git")),
        ] {
            let path = PathBuf::from(git(root, &["rev-parse", option])?);
            let path = if path.is_absolute() {
                path
            } else {
                root.join(path)
            };
            if path.canonicalize().map_err(|e| e.to_string())? != expected {
                return Err("foreign or shared isolated Git metadata".into());
            }
        }
        // Repository spelling is only a consistency check; host ids and ancestry supply authority.
        let remote = git(root, &["remote", "get-url", "--all", "origin"])?;
        if remote != format!("https://github.com/{}.git", profile.isolated.name)
            && remote != format!("git@github.com:{}.git", profile.isolated.name)
        {
            return Err("isolated origin does not match the fixed host repository".into());
        }
        if entry_exists(&root.join(".git/objects/info/alternates"))?
            || entry_exists(&root.join(".git/info/grafts"))?
            || git(root, &["rev-parse", "--is-shallow-repository"])? != "false"
        {
            return Err("shared, grafted or shallow isolated history is unsupported".into());
        }
        if !git(root, &["status", "--porcelain=v1", "--untracked-files=all"])?.is_empty() {
            return Err("isolated caller has uncommitted or untracked local changes".into());
        }
        let head = git(root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
        host::oid(&head)?;
        host::oid(&profile.baseline)?;
        git(
            root,
            &["merge-base", "--is-ancestor", &profile.baseline, &head],
        )?;
        Ok(Self {
            head,
            root_identity,
            git_identity,
            index: git(root, &["ls-files", "--stage", "-z"])?,
        })
    }
}
/// Bounded observation of the two Git storage namespaces, without following symlinks.
fn inspect_git_namespace(path: &Path, depth: usize, remaining: &mut usize) -> Result<(), String> {
    if depth > 64 || *remaining == 0 {
        return Err("isolated Git metadata inventory exceeds bound".into());
    }
    *remaining -= 1;
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("shared symlinked isolated Git metadata descendant".into());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            inspect_git_namespace(&entry.path(), depth + 1, remaining)?;
        }
    } else if !metadata.is_file() {
        return Err("unsupported isolated Git metadata descendant".into());
    }
    Ok(())
}
fn source(
    root: &Path,
    profile: &Profile,
    host: &host::Observation,
    inputs: &mut GitInputs,
) -> Result<(), String> {
    if profile.closure.is_empty() || profile.scopes.is_empty() {
        return Err("empty approved producer closure".into());
    }
    let mut files = std::collections::BTreeSet::new();
    for prefix in &profile.scopes {
        files.extend(inputs.files_under(prefix)?);
    }
    let expected: std::collections::BTreeSet<_> = profile
        .closure
        .iter()
        .map(|(p, _, _)| p.to_string())
        .collect();
    if files != expected || expected.len() != profile.closure.len() {
        return Err("producer source inventory differs from the compiled approved closure".into());
    }
    let protected = Source::tip(root, &host.isolated_tip)?;
    for (path, compiled, executable) in &profile.closure {
        if inputs.required(path)? != *compiled
            || inputs.executable(path)? != *executable
            || protected.optional_bytes(path)?.as_deref() != Some(*compiled)
            || protected.executable(path)? != *executable
        {
            return Err(format!(
                "executed/local/protected producer source differs: {path}"
            ));
        }
        let oid = git(
            root,
            &["rev-parse", &format!("{}:{path}", host.isolated_tip)],
        )?;
        let mode = if *executable { "100755" } else { "100644" };
        if git_blob(root, compiled)? != oid
            || host.source.get(*path) != Some(&(oid, mode.to_owned()))
        {
            return Err(format!(
                "producer source is not independently canonical-approved: {path}"
            ));
        }
    }
    Ok(())
}
pub(crate) fn entry_exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}
fn git_blob(root: &Path, bytes: &[u8]) -> Result<String, String> {
    use std::{io::Write, process::Stdio};
    let mut child = crate::git_inputs::command(root)
        .args(["hash-object", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let written = child
        .stdin
        .take()
        .ok_or("missing Git hash input")?
        .write_all(bytes);
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    written.map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("cannot hash isolated input bytes".into());
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim_end().to_owned())
        .map_err(|e| e.to_string())
}
fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = crate::git_inputs::command(root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("isolated Git observation failed: {}", args[0]));
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim_end().into())
        .map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests;
