//! Invocation-local closeout proposal. Observation and expected-before repair are not a lock.
use crate::{
    runtime_advertising::source::Source,
    workspace_mutation::{
        apply_admitted_mutations as apply_mutations, ApplySummary, PlannedMutation,
        WorkspacePathJail,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

// Closeout validation reads governance, registry, acquisition and implementation inputs. Keep the
// same view through planning and delivery; build caches and agent scratch directories are excluded.
const SCOPES: &[&str] = &["crates", "docs", "cli_manifests", ".github", ".cargo"];
const ROOT_INPUTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain",
    "rust-toolchain.toml",
];
type Files = BTreeMap<String, (Vec<u8>, u32)>;
#[derive(PartialEq, Eq)]
struct Snapshot {
    files: Files,
    git: Option<(Vec<u8>, Vec<u8>)>,
}

pub struct Proposal {
    before: Snapshot,
}
impl Proposal {
    pub fn capture(root: &Path) -> Result<Self, String> {
        Ok(Self {
            before: capture(root)?,
        })
    }

    /// Read-only validation supplies no independently applicable mutations or admission receipt.
    pub fn check(&self, root: &Path, files: &[(PathBuf, Vec<u8>)]) -> Result<(), String> {
        self.plan(root, files.to_vec()).map(drop)
    }

    pub(crate) fn plan(
        &self,
        root: &Path,
        files: Vec<(PathBuf, Vec<u8>)>,
    ) -> Result<Vec<PlannedMutation>, String> {
        self.plan_with(root, files, crate::depth_acceptance::plan_manual_closeout)
    }

    pub(crate) fn plan_with(
        &self,
        root: &Path,
        files: Vec<(PathBuf, Vec<u8>)>,
        admit: impl FnOnce(
            &Path,
            Source,
            Source,
            &BTreeSet<String>,
        ) -> Result<BTreeMap<String, Vec<u8>>, String>,
    ) -> Result<Vec<PlannedMutation>, String> {
        self.plan_changes(
            root,
            files.into_iter().map(|(p, b)| (p, Some(b))).collect(),
            admit,
        )
    }

    pub(crate) fn plan_changes(
        &self,
        root: &Path,
        files: Vec<(PathBuf, Option<Vec<u8>>)>,
        admit: impl FnOnce(
            &Path,
            Source,
            Source,
            &BTreeSet<String>,
        ) -> Result<BTreeMap<String, Vec<u8>>, String>,
    ) -> Result<Vec<PlannedMutation>, String> {
        let mut proposed = BTreeMap::new();
        let mut after = self.before.files.clone();
        for (path, bytes) in files {
            let name = path.to_str().ok_or("non-UTF8 closeout output")?.to_owned();
            if name.contains('\\')
                || !SCOPES
                    .iter()
                    .any(|scope| name.starts_with(&format!("{scope}/")))
            {
                return Err("closeout output outside observed inputs".into());
            }
            WorkspacePathJail::new(root)
                .map_err(|e| e.to_string())?
                .resolve(&path)
                .map_err(|e| e.to_string())?;
            if proposed.insert(name.clone(), bytes.clone()).is_some() {
                return Err("duplicate closeout output".into());
            }
            let mode = self
                .before
                .files
                .get(&name)
                .map(|(_, m)| *m)
                .unwrap_or(0o644);
            if let Some(bytes) = bytes {
                after.insert(name, (bytes, mode));
            } else {
                after.remove(&name);
            }
        }
        let entries = admit(
            root,
            source(root, &self.before.files)?,
            source(root, &after)?,
            &proposed.keys().cloned().collect(),
        )?;
        for (path, bytes) in entries {
            if proposed.insert(path, Some(bytes)).is_some() {
                return Err("acceptance entry overwrites another proposed output".into());
            }
        }
        if capture(root)? != self.before {
            return Err(
                "closeout inputs changed during validation; preserving concurrent edits".into(),
            );
        }
        proposed
            .into_iter()
            .map(|(path, bytes)| {
                Ok(match self.before.files.get(&path) {
                    Some((old, mode)) => match bytes {
                        Some(bytes) => PlannedMutation::replace(path, old.clone(), bytes),
                        None => PlannedMutation::remove(path, old.clone()),
                    }
                    .with_expected_mode(*mode),
                    None => {
                        PlannedMutation::create(path, bytes.ok_or("cannot retire an absent file")?)
                    }
                })
            })
            .collect()
    }

    /// Materialize only observed regular inputs for existing file-based renderers. No caller writes.
    pub(crate) fn materialize(&self, destination: &Path) -> Result<(), String> {
        for (path, (bytes, mode)) in &self.before.files {
            let target = destination.join(path);
            fs::create_dir_all(target.parent().ok_or("missing parent")?)
                .map_err(|e| e.to_string())?;
            fs::write(&target, bytes).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(target, fs::Permissions::from_mode(*mode))
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn apply(
        &self,
        root: &Path,
        files: Vec<(PathBuf, Vec<u8>)>,
    ) -> Result<ApplySummary, String> {
        let mutations = self.plan(root, files)?;
        apply_mutations(root, &mutations).map_err(|e| e.to_string())
    }
}
fn source(root: &Path, files: &Files) -> Result<Source, String> {
    Source::captured(
        root,
        files
            .iter()
            .map(|(p, (b, m))| (p.clone(), (b.clone(), m & 0o111 != 0)))
            .collect(),
    )
}
fn capture(root: &Path) -> Result<Snapshot, String> {
    let source = Source::workspace(root)?;
    let mut paths = Vec::new();
    // Include every tracked dependency, including scripts and producer inputs outside the usual
    // closeout scopes. Keep index/HEAD observation separate from the file bytes it will preserve.
    let git = if fs::symlink_metadata(root.join(".git")).is_ok() {
        let index = git(root, &["ls-files", "--stage", "-z"])?;
        for row in index.split(|b| *b == 0).filter(|r| !r.is_empty()) {
            let row = std::str::from_utf8(row).map_err(|e| e.to_string())?;
            paths.push(
                row.split_once('\t')
                    .ok_or("invalid closeout index row")?
                    .1
                    .to_owned(),
            );
        }
        Some((git(root, &["rev-parse", "--verify", "HEAD"])?, index))
    } else {
        None
    };
    for scope in SCOPES {
        paths.extend(source.paths(scope)?);
    }
    paths.extend(ROOT_INPUTS.iter().map(|s| (*s).to_owned()));
    paths.sort();
    paths.dedup();
    let mut files = Files::new();
    let mut size = 0;
    for path in paths {
        if path.contains('\\') {
            return Err("literal backslash closeout input is unsupported".into());
        }
        let full = WorkspacePathJail::new(root)
            .map_err(|e| e.to_string())?
            .resolve(Path::new(&path))
            .map_err(|e| e.to_string())?;
        let metadata = match fs::symlink_metadata(&full) {
            Ok(m) if m.is_dir() => continue,
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.to_string()),
        };
        if metadata.len() > (536_870_912 - size) as u64 {
            return Err("closeout inputs exceed observation limit".into());
        }
        let bytes = source
            .optional_bytes(&path)?
            .ok_or("closeout input disappeared")?;
        size += bytes.len();
        if size > 536_870_912 || files.len() >= 50_000 {
            return Err("closeout inputs exceed observation limit".into());
        }
        files.insert(path, (bytes, mode(&metadata)));
    }
    Ok(Snapshot { files, git })
}
#[cfg(unix)]
fn mode(m: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    m.permissions().mode() & 0o7777
}
#[cfg(not(unix))]
fn mode(m: &fs::Metadata) -> u32 {
    u32::from(m.permissions().readonly())
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = crate::git_inputs::command(root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "closeout Git observation: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}
