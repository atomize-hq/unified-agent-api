//! Invocation-local isolated maintenance binding. Scope and freshness never grant admission.
// The foundation deliberately has no installed profile or production bound-Proposal caller.
#![allow(dead_code)]
mod host;
pub(crate) mod onboarding;
mod scope;
mod view;

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
    // Exactly one compiled subject; no declaration, receipt or flag chooses it.
    subject: &'static scope::Subject,
}
const LIVE_PROFILE: Option<Profile> = None;
impl Profile {
    fn depth_tables(&self) -> DepthTables {
        DepthTables {
            subject: self.subject,
        }
    }
}
/// Registry admission of exactly one `support_depth` table: the installed profile's compiled
/// subject's (roadmap B.8). Only an installed profile mints one, so no flag, environment value,
/// path or registry content can; [`AgentRegistry::parse_isolated`] is its only reader.
pub(crate) struct DepthTables {
    subject: &'static scope::Subject,
}
impl crate::agent_registry::DepthAdmission for DepthTables {
    fn admitted_agent(&self) -> &'static str {
        self.subject.agent_id
    }
}

/// Read-only diagnostic. There is no selector for a repository, profile, trust or write mode.
pub fn check(root: &Path) -> Result<(), String> {
    Context::resolve(root, LIVE_PROFILE, Arc::new(host::Github)).map(drop)
}

/// The isolated binding as a supported public route sees it (route design, "Public routes and
/// delivery"). It wraps a freshly resolved context; it has no public constructor and grants no
/// write by itself: the route's own owning checks still decide.
pub struct Route(Context);
impl Route {
    /// The scoped registry, after rechecking the workspace, inputs and host identities.
    pub fn registry(&self, root: &Path) -> Result<&AgentRegistry, String> {
        self.0.registry(root)
    }
    /// MA4 captures the route once, in the caller workspace, after the same recheck.
    pub(crate) fn capture(self, root: &Path) -> Result<CapturedRegistry, String> {
        let registry = self.0.registry(root)?.clone();
        Ok(CapturedRegistry {
            route: Arc::new(self),
            registry,
        })
    }
    pub(crate) fn context(&self) -> &Context {
        &self.0
    }
    /// Everything a plan built under this route is bound to: the workspace path, its root and Git
    /// directory identities, the candidate HEAD and index, and the host observation (repository
    /// identities, protected tips, approved source and governing registry).
    pub fn binding(&self) -> RouteBinding {
        RouteBinding {
            root: self.0.root.clone(),
            workspace: self.0.workspace.clone(),
            host: self.0.host.clone(),
        }
    }
    pub fn recheck(&self, root: &Path) -> Result<(), String> {
        self.0.recheck(root)
    }
    /// MA5 captures its closeout proposal under this route (CP-P1, G2). The proposal borrows the
    /// context as its owning authority; copying the route's files or binding supplies none.
    pub fn proposal(self, root: &Path) -> Result<Proposal, String> {
        self.recheck(root)?;
        Proposal::capture_isolated(root, Arc::new(self))
    }
}
/// An opaque, comparable copy of a route's binding. It carries no authority of its own: only a
/// freshly resolved route with an equal binding may act on a plan that recorded it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteBinding {
    root: PathBuf,
    workspace: Workspace,
    host: host::Observation,
}

/// `None` only while no isolated profile is installed, so ordinary routes keep their refusals.
/// An installed profile resolves positively or fails; it never falls back to ordinary handling.
pub fn route(root: &Path) -> Result<Option<Route>, String> {
    // Unit tests may install a fixture profile for the calling thread; production cannot.
    #[cfg(test)]
    if let Some((profile, api)) = tests::fixture::installed() {
        return route_in(root, Some(profile), api);
    }
    route_in(root, LIVE_PROFILE, Arc::new(host::Github))
}
/// The registry an MA2 read route uses (acquisition plan, validation, stand-down check, support
/// matrix, audit status), plus capability publication. A resolved isolated route yields its scoped
/// registry, rechecked against the workspace, inputs and host; inside a bound MA4 candidate, the
/// registry its execution captured. Otherwise this is the production load, with every refusal it
/// keeps, `support_depth` included. A failing installed route errors; it never falls back.
pub fn registry(root: &Path) -> Result<AgentRegistry, RegistryError> {
    if let Some(registry) = candidate_registry(root).map_err(RegistryError::Isolated)? {
        return Ok(registry);
    }
    match route(root).map_err(RegistryError::Isolated)? {
        Some(route) => route
            .registry(root)
            .cloned()
            .map_err(RegistryError::Isolated),
        None => AgentRegistry::load(root).map_err(|err| RegistryError::Production(err.to_string())),
    }
}
/// A resolved route and its scoped registry, captured when an MA4 execution started. Only
/// [`Route::capture`] mints one. MA4's independent candidate cannot resolve the route itself, so
/// its in-process checks use this registry for that candidate's exact root while a
/// [`CandidateRegistry`] guard is held; the relay separately refuses any registry change, and
/// [`CapturedRegistry::recheck`] re-authenticates the caller before delivery.
#[derive(Clone)]
pub(crate) struct CapturedRegistry {
    route: Arc<Route>,
    registry: AgentRegistry,
}
impl std::fmt::Debug for CapturedRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CapturedRegistry(..)")
    }
}
impl CapturedRegistry {
    /// MA6's proposal shares the resolved caller route; scratch never resolves another context.
    pub(crate) fn proposal(&self, root: &Path) -> Result<Proposal, String> {
        Proposal::capture_isolated(root, self.route.clone())
    }
    pub(crate) fn registry(&self) -> &AgentRegistry {
        &self.registry
    }
    /// The authenticated evidence host of the captured route, after rechecking the caller. It
    /// stays in the controller: the candidate and its executor never receive it.
    pub(crate) fn evidence_host(&self, caller: &Path) -> Result<EvidenceHost, String> {
        self.route.0.evidence_host(caller)
    }
    /// Recheck the captured route against the caller workspace, inputs and host.
    pub(crate) fn recheck(&self, caller: &Path) -> Result<(), String> {
        self.route.recheck(caller)
    }
    /// Make this registry the routed registry of `candidate` on this thread until the guard drops.
    /// The candidate must be an independent clone of the captured caller: its HEAD descends from
    /// the caller's captured HEAD, and its registry equals the captured registry.
    pub(crate) fn bind_candidate(&self, candidate: &Path) -> Result<CandidateRegistry, String> {
        let root = fs::canonicalize(candidate).map_err(|e| e.to_string())?;
        if root == self.route.0.root {
            return Err("an MA4 candidate must be independent of the caller workspace".into());
        }
        let not_checkout = || "MA4 candidate is not a checkout of the captured caller".to_string();
        let caller_head = &self.route.0.workspace.head;
        git(&root, &["merge-base", "--is-ancestor", caller_head, "HEAD"])
            .map_err(|_| not_checkout())?;
        let raw = fs::read_to_string(root.join(crate::agent_registry::REGISTRY_RELATIVE_PATH))
            .map_err(|_| not_checkout())?;
        let tables = self.route.0.profile.depth_tables();
        if AgentRegistry::parse_isolated(&raw, &tables).map_err(|e| e.to_string())? != self.registry
        {
            return Err(not_checkout());
        }
        CANDIDATE.with(|bound| {
            let mut bound = bound.borrow_mut();
            if bound.is_some() {
                return Err("an MA4 candidate registry is already bound on this thread".to_string());
            }
            *bound = Some(CandidateBinding {
                root,
                registry: self.registry.clone(),
                view: None,
            });
            Ok(CandidateRegistry(std::marker::PhantomData))
        })
    }
}
struct CandidateBinding {
    root: PathBuf,
    registry: AgentRegistry,
    view: Option<view::Binding>,
}
pub(crate) use view::git_inputs as view_git_inputs;
std::thread_local! {
    static CANDIDATE: std::cell::RefCell<Option<CandidateBinding>> =
        const { std::cell::RefCell::new(None) };
}
/// Unbinds the candidate registry on drop. Not `Send`: it must drop on the thread that bound it.
pub(crate) struct CandidateRegistry(std::marker::PhantomData<*const ()>);
impl Drop for CandidateRegistry {
    fn drop(&mut self) {
        CANDIDATE.with(|bound| *bound.borrow_mut() = None);
    }
}
fn candidate_registry(root: &Path) -> Result<Option<AgentRegistry>, String> {
    CANDIDATE.with(|bound| {
        let bound = bound.borrow();
        let Some(bound) = bound.as_ref() else {
            return Ok(None);
        };
        // Recheck before canonicalization so a removed or redirected destination cannot fall
        // through to production loading while a scratch binding is still alive.
        if let Some(view) = &bound.view {
            view.recheck(&bound.root)?;
        }
        Ok(fs::canonicalize(root)
            .ok()
            .filter(|root| *root == bound.root)
            .map(|_| bound.registry.clone()))
    })
}
/// Which load failed. A caller that tolerates a production load failure, because another check
/// already reports it, must still report a failing isolated route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    Production(String),
    Isolated(String),
}
impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Production(error) => f.write_str(error),
            Self::Isolated(error) => write!(f, "isolated maintenance route: {error}"),
        }
    }
}
fn route_in(
    root: &Path,
    profile: Option<Profile>,
    api: Arc<dyn Api>,
) -> Result<Option<Route>, String> {
    match profile {
        None => Ok(None),
        profile => Context::resolve(root, profile, api).map(|c| Some(Route(c))),
    }
}

/// The authenticated isolated repository and protected tip for the synthetic evidence verdict
/// (SA-P3, IR-P1). Only a resolved context for the synthetic subject makes one; no receipt,
/// remote, environment value or flag can. It grants no write or admission authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvidenceHost {
    repository: &'static str,
    // The authenticated GitHub repository and owner ids; a reader rechecks them so a renamed and
    // replaced repository under the same name is never read as this one.
    id: u64,
    owner: u64,
    tip: String,
}
impl EvidenceHost {
    pub(crate) fn repository(&self) -> &'static str {
        self.repository
    }
    /// Whether a host repository read is exactly the authenticated repository.
    pub(crate) fn same_repository(&self, identity: &Value) -> bool {
        identity["id"].as_u64() == Some(self.id)
            && identity["owner"]["id"].as_u64() == Some(self.owner)
            && identity["full_name"].as_str() == Some(self.repository)
    }
    pub(crate) fn tip(&self) -> &str {
        &self.tip
    }
    #[cfg(test)]
    pub(crate) fn for_test(repository: &'static str, tip: &str) -> Self {
        Self {
            repository,
            id: 10,
            owner: 20,
            tip: tip.into(),
        }
    }
}
/// Resolves the installed isolated profile. Uninstalled (`LIVE_PROFILE = None`), it refuses
/// before any host read.
pub(crate) fn evidence_host(root: &Path) -> Result<EvidenceHost, String> {
    // Unit tests may install a fixture profile for the calling thread; production cannot.
    #[cfg(test)]
    if let Some((profile, api)) = tests::fixture::installed() {
        return Context::resolve(root, Some(profile), api)?.evidence_host(root);
    }
    Context::resolve(root, LIVE_PROFILE, Arc::new(host::Github))?.evidence_host(root)
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
            &profile.depth_tables(),
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
        Route(self).proposal(root)
    }
    pub(crate) fn evidence_host(&self, root: &Path) -> Result<EvidenceHost, String> {
        self.recheck(root)?;
        if *self.profile.subject != scope::SYNTHETIC {
            return Err("only the synthetic subject has an isolated evidence host".into());
        }
        Ok(EvidenceHost {
            repository: self.profile.isolated.name,
            id: self.profile.isolated.id,
            owner: self.profile.isolated.owner,
            tip: self.host.isolated_tip.clone(),
        })
    }
    pub(crate) fn registry(&self, root: &Path) -> Result<&AgentRegistry, String> {
        self.recheck(root)?;
        Ok(&self.registry)
    }
    /// The authenticated protected tip T (CP-P1), after rechecking the caller. The owning
    /// predicates bind the local integration ref to it; it is not itself an admission.
    pub(crate) fn protected_tip(&self, root: &Path) -> Result<&str, String> {
        self.recheck(root)?;
        Ok(&self.host.isolated_tip)
    }
    /// Whether `raw` registry bytes parse, under this profile's depth admission, to exactly the
    /// scoped registry, after rechecking the caller.
    pub(crate) fn require_registry(&self, root: &Path, raw: &str) -> Result<(), String> {
        self.recheck(root)?;
        let tables = self.profile.depth_tables();
        if AgentRegistry::parse_isolated(raw, &tables).map_err(|e| e.to_string())? != self.registry
        {
            return Err("owning bindings read a registry other than the isolated scope".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Workspace {
    head: String,
    root_identity: FileIdentity,
    git_identity: FileIdentity,
    index: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
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
    // An entry that vanishes mid-walk (for example Git's detached auto-maintenance lock) means
    // the metadata changed under the observation; refuse as a change, still failing closed.
    let changed = |error: std::io::Error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "isolated caller workspace or Git ownership changed".to_string()
        } else {
            error.to_string()
        }
    };
    let metadata = fs::symlink_metadata(path).map_err(changed)?;
    if metadata.file_type().is_symlink() {
        return Err("shared symlinked isolated Git metadata descendant".into());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(changed)? {
            let entry = entry.map_err(changed)?;
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

#[cfg(all(test, unix))]
pub(crate) use tests::onboarding_producer_workspace;
