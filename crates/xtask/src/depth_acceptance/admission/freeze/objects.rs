//! Authenticate complete raw trees before deriving any writer inputs.
use super::super::{objects::Objects, Object};
use super::PlanningFailure;
use crate::freeze_transition::{tree::validate_inventory, tree::validate_path, Entry, Tree};
use std::{fs, path::Path};

pub(super) fn read(objects: &mut Objects<'_>, object: &Object) -> Result<Tree, PlanningFailure> {
    objects.require(object)?;
    let mut tree = Tree::new();
    walk(objects, &object.tree, "", &mut tree, &mut 0)?;
    validate_inventory(&tree).map_err(PlanningFailure::planning)?;
    Ok(tree)
}
fn walk(
    objects: &mut Objects<'_>,
    identity: &str,
    prefix: &str,
    tree: &mut Tree,
    size: &mut usize,
) -> Result<(), PlanningFailure> {
    if prefix.split('/').count() > 64 {
        return Err(PlanningFailure::Incomplete(
            "freeze tree depth bound exceeded".into(),
        ));
    }
    let bytes = objects.read("tree", identity)?;
    if !prefix.is_empty() && bytes.is_empty() {
        return Err(PlanningFailure::planning(
            "freeze tree contains an empty directory object",
        ));
    }
    *size = size
        .checked_add(bytes.len())
        .ok_or_else(|| PlanningFailure::Incomplete("freeze tree byte overflow".into()))?;
    if *size > 536_870_912 {
        return Err(PlanningFailure::Incomplete(
            "freeze tree inventory bound exceeded".into(),
        ));
    }
    let mut previous_name = None;
    let mut remaining = bytes.as_slice();
    let mut names = std::collections::BTreeSet::new();
    while !remaining.is_empty() {
        let end = remaining
            .iter()
            .position(|b| *b == 0)
            .ok_or_else(|| PlanningFailure::planning("malformed freeze tree"))?;
        let header = std::str::from_utf8(&remaining[..end]).map_err(PlanningFailure::planning)?;
        let (mode, name) = header
            .split_once(' ')
            .ok_or_else(|| PlanningFailure::planning("malformed freeze tree mode"))?;
        if name.contains('/') || !names.insert(name) {
            return Err(PlanningFailure::planning("ambiguous freeze tree entry"));
        }
        let hash = hex::encode(
            remaining
                .get(end + 1..end + 21)
                .ok_or_else(|| PlanningFailure::planning("malformed freeze tree hash"))?,
        );
        remaining = &remaining[end + 21..];
        let ordering_name = if mode == "40000" {
            format!("{name}/")
        } else {
            format!("{name}\0")
        };
        if previous_name
            .as_ref()
            .is_some_and(|previous| previous >= &ordering_name)
        {
            return Err(PlanningFailure::planning(
                "freeze tree entries are not canonically ordered",
            ));
        }
        previous_name = Some(ordering_name);
        let path = format!("{prefix}{name}");
        validate_path(&path).map_err(PlanningFailure::planning)?;
        if mode == "40000" {
            walk(objects, &hash, &format!("{path}/"), tree, size)?;
            continue;
        }
        let entry = match mode {
            "100644" | "100755" | "120000" => Entry::file(mode, objects.read("blob", &hash)?),
            "160000" => Entry {
                mode: mode.into(),
                kind: "commit".into(),
                bytes: hash.into_bytes(),
            },
            _ => {
                return Err(PlanningFailure::planning(
                    "unsupported freeze tree object kind",
                ))
            }
        };
        *size = size
            .checked_add(entry.bytes.len())
            .ok_or_else(|| PlanningFailure::Incomplete("freeze tree byte overflow".into()))?;
        if *size > 536_870_912 || tree.len() >= 50_000 {
            return Err(PlanningFailure::Incomplete(
                "freeze tree inventory bound exceeded".into(),
            ));
        }
        tree.insert(path, entry);
    }
    Ok(())
}

/// Approved filesystem readers receive only authenticated predecessor regular blobs. No
/// checkout, filters, hooks, candidate writers, local actions or generated executables run.
pub(super) struct DataView(tempfile::TempDir);
impl DataView {
    pub(super) fn new(root: &Path, parent: &Object, tree: &Tree) -> Result<Self, String> {
        #[cfg(test)]
        let directory = match super::DATA_VIEW_TEMP_ROOT.with(|slot| slot.borrow_mut().take()) {
            Some(path) => tempfile::tempdir_in(path),
            None => tempfile::tempdir(),
        };
        #[cfg(not(test))]
        let directory = tempfile::tempdir();
        let directory = directory.map_err(|e| e.to_string())?;
        let destination = directory
            .path()
            .to_str()
            .ok_or("non-UTF8 freeze data view")?;
        git(
            root,
            &[
                "clone",
                "--template=",
                "--shared",
                "--no-checkout",
                "--quiet",
                "--",
                ".",
                destination,
            ],
        )?;
        git(directory.path(), &["read-tree", &parent.commit])?;
        git(directory.path(), &["update-ref", "HEAD", &parent.commit])?;
        for (path, entry) in tree.iter().filter(|(_, e)| e.regular()) {
            validate_path(path)?;
            let full = directory.path().join(path);
            fs::create_dir_all(full.parent().ok_or("invalid freeze data parent")?)
                .map_err(|e| e.to_string())?;
            fs::write(&full, &entry.bytes).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    full,
                    fs::Permissions::from_mode(if entry.mode == "100755" { 0o755 } else { 0o644 }),
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Ok(Self(directory))
    }
    /// Stage only authenticated acquisition differences using literal no-filter object plumbing.
    pub(super) fn overlay(&self, before: &Tree, after: &Tree) -> Result<(), String> {
        for path in before
            .keys()
            .chain(after.keys())
            .collect::<std::collections::BTreeSet<_>>()
        {
            if before.get(path) == after.get(path) {
                continue;
            }
            validate_path(path)?;
            let full = self.root().join(path);
            match after.get(path) {
                Some(entry) if entry.regular() => {
                    fs::create_dir_all(full.parent().ok_or("invalid acquisition parent")?)
                        .map_err(|e| e.to_string())?;
                    fs::write(&full, &entry.bytes).map_err(|e| e.to_string())?;
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        fs::set_permissions(
                            &full,
                            fs::Permissions::from_mode(if entry.mode == "100755" {
                                0o755
                            } else {
                                0o644
                            }),
                        )
                        .map_err(|e| e.to_string())?;
                    }
                    let out = crate::git_inputs::command(self.root())
                        .args(["hash-object", "--no-filters", "-w", "--", path])
                        .output()
                        .map_err(|e| e.to_string())?;
                    if !out.status.success() {
                        return Err("cannot stage authenticated acquisition blob".into());
                    }
                    let id = std::str::from_utf8(&out.stdout)
                        .map_err(|e| e.to_string())?
                        .trim();
                    git(
                        self.root(),
                        &[
                            "update-index",
                            "--add",
                            "--cacheinfo",
                            &entry.mode,
                            id,
                            path,
                        ],
                    )?;
                }
                None => {
                    if full.exists() {
                        fs::remove_file(full).map_err(|e| e.to_string())?;
                    }
                    git(self.root(), &["update-index", "--force-remove", "--", path])?;
                }
                _ => return Err("nonregular authenticated acquisition input".into()),
            }
        }
        Ok(())
    }
    pub(super) fn root(&self) -> &Path {
        self.0.path()
    }
}
fn git(root: &Path, args: &[&str]) -> Result<(), String> {
    let output = crate::git_inputs::command(root)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("cannot construct authenticated freeze data view".into());
    }
    Ok(())
}
