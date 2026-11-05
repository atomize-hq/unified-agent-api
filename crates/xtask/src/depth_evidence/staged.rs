//! MA4 staged evidence (B.4 part 2, design option A).
//!
//! The guarded MA4 host, under its captured isolated route only, authenticates the complete
//! applicable run/attempt population and stages a bounded, read-only, temporary bundle outside the
//! candidate. The executor runs `xtask depth-evidence-assemble` inside the candidate: with no
//! network and no credentials it writes reporting E and the referenced receipt/payload files into
//! the packet's normal reporting destinations. The bundle is reachable only through a channel the
//! live host invocation serves for its own candidate; a copied bundle, a path, an environment value
//! or a restored origin is never an installed route. E stays an untrusted claim: the relay keeps
//! its checks and every consumer re-authenticates the full population.
use crate::{
    depth_record::{DepthRecord, Event, ExecutionPart, DEPTH_RECORD_FILE_NAME},
    workspace_mutation::{apply_admitted_mutations, PlannedMutation, WorkspacePathJail},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
mod channel;
#[cfg(all(test, unix))]
mod tests;

/// Locates the live invocation channel. It carries no authority: only the host serving it does.
pub const CHANNEL_ENV: &str = "XTASK_AGENT_MAINTENANCE_EVIDENCE_CHANNEL";
const SCHEMA: &str = "depth-evidence-bundle-v1";
const BUNDLE_LIMIT: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bundle {
    schema: String,
    invocation: String,
    record: String,
    agent_id: String,
    version: String,
    event: Event,
    policy_identity: String,
    obligations_identity: String,
    execution: ExecutionPart,
    files: Vec<BundleFile>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleFile {
    path: String,
    sha256: String,
    hex: String,
}

/// What the MA4 packet targets: the request's agent and version, and the manifest root the
/// captured registry names for that agent.
pub(crate) struct Target {
    pub agent_id: String,
    pub version: String,
    pub manifest_root: String,
}

/// A staged bundle and the channel serving it. Dropping it stops the channel and removes both.
pub(crate) struct Staged {
    channel: channel::Server,
    _dir: tempfile::TempDir,
    files: BTreeMap<String, String>,
}
impl Staged {
    pub(crate) fn locator(&self) -> &Path {
        self.channel.locator()
    }
    /// The evidence files this host authenticated and staged, by path, with their SHA-256. Only
    /// these exact bytes may gain an index identity on delivery; an executor-made file never does.
    pub(crate) fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }
}

/// Host side. `host` is the captured route's evidence host, or why it has none; an assembly
/// failure is served to the executor as a refusal, never as evidence.
pub(crate) fn stage(
    caller: &Path,
    candidate: &Path,
    host: Result<crate::isolated_maintenance::EvidenceHost, String>,
    target: &Target,
) -> Result<Staged, String> {
    let assembly = host.and_then(|host| super::verdict::isolated_assembly(caller, &host));
    stage_answer(candidate, assembly, target)
}

fn stage_answer(
    candidate: &Path,
    assembly: Result<super::verdict::Assembly, String>,
    target: &Target,
) -> Result<Staged, String> {
    let dir = private_dir()?;
    let invocation = invocation(dir.path());
    let bundle = assembly.and_then(|a| bundle(&a, target, &invocation));
    let files = bundle
        .as_ref()
        .map(|b| {
            b.files
                .iter()
                .map(|f| (f.path.clone(), f.sha256.clone()))
                .collect()
        })
        .unwrap_or_default();
    let answer = bundle.and_then(|bundle| publish(dir.path(), &bundle));
    let mut staged = serve(dir, candidate, answer)?;
    staged.files = files;
    Ok(staged)
}

fn serve(
    dir: tempfile::TempDir,
    candidate: &Path,
    answer: Result<channel::Offer, String>,
) -> Result<Staged, String> {
    let candidate = fs::canonicalize(candidate).map_err(|e| e.to_string())?;
    let channel = channel::Server::start(&dir.path().join("channel"), candidate, answer)?;
    Ok(Staged {
        channel,
        _dir: dir,
        files: BTreeMap::new(),
    })
}

fn private_dir() -> Result<tempfile::TempDir, String> {
    let dir = tempfile::Builder::new()
        .prefix("uaa-ev-")
        .tempdir()
        .map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

fn invocation(dir: &Path) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let seed = format!(
        "{}:{nanos}:{}:{}",
        std::process::id(),
        dir.display(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    );
    hex::encode(Sha256::digest(seed.as_bytes()))
}

fn bundle(
    assembly: &super::verdict::Assembly,
    target: &Target,
    invocation: &str,
) -> Result<Bundle, String> {
    let p = assembly.profile;
    if p.agent_id() != target.agent_id || p.version() != target.version {
        return Err("authenticated evidence names another agent or version than the packet".into());
    }
    let reports = format!("{}/reports/{}", target.manifest_root, target.version);
    let o = assembly
        .original
        .obligations
        .as_ref()
        .ok_or("authenticated generation has no frozen O")?;
    Ok(Bundle {
        schema: SCHEMA.into(),
        invocation: invocation.into(),
        record: format!("{reports}/{DEPTH_RECORD_FILE_NAME}"),
        agent_id: target.agent_id.clone(),
        version: target.version.clone(),
        event: assembly.original.event.clone(),
        policy_identity: assembly.original.policy.identity.clone(),
        obligations_identity: o.identity.clone(),
        execution: assembly.execution.clone(),
        files: assembly
            .files
            .iter()
            .map(|(reference, bytes)| BundleFile {
                path: reference.path.clone(),
                sha256: reference.sha256.clone(),
                hex: hex::encode(bytes),
            })
            .collect(),
    })
}

/// Write the read-only bundle into the private directory and describe it for the channel.
fn publish(dir: &Path, bundle: &Bundle) -> Result<channel::Offer, String> {
    let bytes = serde_json::to_vec(bundle).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > BUNDLE_LIMIT {
        return Err("staged evidence bundle exceeds its bound".into());
    }
    let path = dir.join("bundle.json");
    fs::write(&path, &bytes).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).map_err(|e| e.to_string())?;
    }
    Ok(channel::Offer {
        bundle: path,
        sha256: hex::encode(Sha256::digest(&bytes)),
        invocation: bundle.invocation.clone(),
    })
}

/// `xtask depth-evidence-assemble`, run by the MA4 executor inside its candidate.
pub fn assemble_staged(root: &Path) -> Result<String, String> {
    assemble(root, std::env::var_os(CHANNEL_ENV).map(PathBuf::from))
}

pub(crate) fn assemble(root: &Path, locator: Option<PathBuf>) -> Result<String, String> {
    let locator = locator.ok_or(
        "depth-evidence-assemble runs only inside a live isolated MA4 execution; no invocation \
         channel is present",
    )?;
    let root = fs::canonicalize(root).map_err(|e| e.to_string())?;
    let offer = channel::request(&locator, &root)?;
    let metadata = fs::symlink_metadata(&offer.bundle).map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.len() > BUNDLE_LIMIT {
        return Err("staged evidence bundle is not a bounded regular file".into());
    }
    let raw = fs::read(&offer.bundle).map_err(|e| e.to_string())?;
    if hex::encode(Sha256::digest(&raw)) != offer.sha256 {
        return Err("staged evidence bundle differs from what this invocation staged".into());
    }
    let bundle: Bundle = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    if bundle.schema != SCHEMA || bundle.invocation != offer.invocation {
        return Err("staged evidence bundle belongs to another invocation".into());
    }
    write(&root, &bundle)
}

/// Write E and its files into the candidate's normal reporting destinations, and nothing else.
fn write(root: &Path, bundle: &Bundle) -> Result<String, String> {
    super::validate::path(&bundle.record)?;
    let reports = bundle
        .record
        .strip_suffix(&format!("/{DEPTH_RECORD_FILE_NAME}"))
        .filter(|r| r.ends_with(&format!("/reports/{}", bundle.version)))
        .ok_or("staged record is not a normal reporting destination")?;
    let evidence = format!("{reports}/evidence/");
    let mut files = BTreeMap::new();
    for file in &bundle.files {
        super::validate::path(&file.path)?;
        if !file.path.starts_with(&evidence) {
            return Err(format!(
                "staged evidence outside the reporting destination: {}",
                file.path
            ));
        }
        let bytes = hex::decode(&file.hex).map_err(|e| e.to_string())?;
        if hex::encode(Sha256::digest(&bytes)) != file.sha256 {
            return Err(format!("staged evidence digest mismatch: {}", file.path));
        }
        if files
            .insert(file.path.clone(), (file.sha256.clone(), bytes))
            .is_some()
        {
            return Err("staged evidence repeats a path".into());
        }
    }
    let e = &bundle.execution;
    for reference in e.results.iter().flat_map(|r| &r.evidence) {
        if files.get(&reference.path).map(|(sha, _)| sha) != Some(&reference.sha256) {
            return Err("staged E references evidence the bundle does not carry".into());
        }
    }
    if e.policy_identity != bundle.policy_identity
        || e.obligations_identity != bundle.obligations_identity
        || e.identity != crate::depth_record::part_identity(e)?
    {
        return Err("staged E is not bound to the staged P/O".into());
    }
    let jail = WorkspacePathJail::new(root).map_err(|e| e.to_string())?;
    let path = jail
        .resolve(Path::new(&bundle.record))
        .map_err(|e| e.to_string())?;
    let before = fs::read(&path).map_err(|_| "the candidate holds no packet depth record")?;
    let raw = std::str::from_utf8(&before).map_err(|e| e.to_string())?;
    let mut record = DepthRecord::parse(raw, &bundle.agent_id, &bundle.version)?;
    if record.event != bundle.event {
        return Err("staged evidence is for another generation (Event)".into());
    }
    if record.policy.identity != bundle.policy_identity
        || record.obligations.as_ref().map(|o| &o.identity) != Some(&bundle.obligations_identity)
    {
        return Err("staged evidence is bound to another frozen P/O".into());
    }
    record.execution = Some(e.clone());
    let after = format!(
        "{}\n",
        serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?
    );
    DepthRecord::parse(&after, &bundle.agent_id, &bundle.version)?;
    let mut mutations = vec![];
    for (relative, (_, bytes)) in &files {
        let target = jail
            .resolve(Path::new(relative))
            .map_err(|e| e.to_string())?;
        match fs::symlink_metadata(&target) {
            Ok(m) if m.is_file() && fs::read(&target).map_err(|e| e.to_string())? == *bytes => {}
            Ok(_) => return Err(format!("conflicting evidence path: {relative}")),
            Err(_) => mutations.push(PlannedMutation::create(relative, bytes.clone())),
        }
    }
    if after.as_bytes() != before.as_slice() {
        mutations.push(PlannedMutation::replace(
            &bundle.record,
            before,
            after.into_bytes(),
        ));
    }
    apply_admitted_mutations(root, &mutations).map_err(|e| e.to_string())?;
    Ok(format!(
        "wrote reporting E for {} obligations with {} evidence files; it stays untrusted until the \
         audit re-authenticates the population\n",
        e.results.len(),
        files.len()
    ))
}
