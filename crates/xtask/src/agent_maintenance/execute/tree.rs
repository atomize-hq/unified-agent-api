//! Invocation-local file states. Never follows links; this is an observation, not a lock.
use super::Error;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Node {
    File(Vec<u8>, u32),
    Directory(u32),
    Link(PathBuf),
}
pub(super) type Tree = BTreeMap<String, Node>;

pub(super) fn capture(root: &Path, tracked: &BTreeSet<String>) -> Result<Tree, Error> {
    fn walk(root: &Path, path: &Path, out: &mut Tree, size: &mut usize) -> Result<(), Error> {
        let meta = fs::symlink_metadata(path).map_err(io)?;
        #[cfg(unix)]
        if path
            .strip_prefix(root)
            .map_err(|e| Error::Internal(e.to_string()))?
            .components()
            .any(|c| c.as_os_str().to_string_lossy().contains('\\'))
        {
            return Err(Error::Validation(
                "literal backslash relay paths are unsupported".into(),
            ));
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|e| Error::Internal(e.to_string()))?
            .to_str()
            .ok_or_else(|| Error::Validation("non-UTF8 relay path".into()))?
            .replace('\\', "/");
        if relative == ".git"
            || relative == "target"
            || relative.split('/').any(|c| c == "__pycache__")
            || relative.ends_with(".pyc")
        {
            return Ok(());
        }
        let node = if meta.file_type().is_symlink() {
            Node::Link(fs::read_link(path).map_err(io)?)
        } else if meta.is_dir() {
            Node::Directory(mode(&meta))
        } else if meta.is_file() {
            Node::File(read_bounded(path, size)?, mode(&meta))
        } else {
            return Err(Error::Validation(format!(
                "unsupported relay file type: {relative}"
            )));
        };
        let directory = matches!(node, Node::Directory(_));
        if !relative.is_empty() && out.insert(relative, node).is_some() {
            return Err(Error::Validation("relay snapshot path collision".into()));
        }
        if out.len() > 50_000 {
            return Err(Error::Validation(
                "relay snapshot exceeds 50000 paths".into(),
            ));
        }
        if directory {
            for entry in fs::read_dir(path).map_err(io)? {
                walk(root, &entry.map_err(io)?.path(), out, size)?
            }
        }
        Ok(())
    }
    let mut tree = Tree::new();
    let mut size = 0;
    walk(root, root, &mut tree, &mut size)?;
    // Ignored caches never conceal a tracked evidence/dependency input.
    let jail = crate::workspace_mutation::WorkspacePathJail::new(root)?;
    for path in tracked {
        if path.contains('\\') {
            return Err(Error::Validation(
                "literal backslash relay paths are unsupported".into(),
            ));
        }
        if !tree.contains_key(path) {
            let full = jail.resolve(Path::new(path))?;
            let meta = match fs::symlink_metadata(&full) {
                Ok(meta) => meta,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(io(e)),
            };
            if !meta.is_file() {
                return Err(Error::Validation(
                    "tracked relay input is not regular".into(),
                ));
            }
            tree.insert(
                path.clone(),
                Node::File(read_bounded(&full, &mut size)?, mode(&meta)),
            );
            if tree.len() > 50_000 {
                return Err(Error::Validation(
                    "relay snapshot exceeds 50000 paths".into(),
                ));
            }
        }
    }
    Ok(tree)
}
fn read_bounded(path: &Path, size: &mut usize) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(io)?
        .take((536_870_912 - *size + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(io)?;
    *size += bytes.len();
    if *size > 536_870_912 {
        return Err(Error::Validation("relay snapshot exceeds 512 MiB".into()));
    }
    Ok(bytes)
}
pub(super) fn source(
    root: &Path,
    tree: &Tree,
) -> Result<crate::runtime_advertising::source::Source, Error> {
    let files = tree
        .iter()
        .filter_map(|(path, node)| match node {
            Node::File(bytes, mode) => Some((path.clone(), (bytes.clone(), mode & 0o111 != 0))),
            _ => None,
        })
        .collect();
    crate::runtime_advertising::source::Source::captured(root, files).map_err(Error::Validation)
}
pub(super) fn changed(before: &Tree, after: &Tree) -> Vec<String> {
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|p| before.get(*p) != after.get(*p))
        .cloned()
        .collect()
}
pub(super) fn materialize(root: &Path, tree: &Tree) -> Result<(), Error> {
    for (path, node) in tree {
        let full = root.join(path);
        match node {
            Node::Directory(_) => fs::create_dir_all(&full).map_err(io)?,
            Node::File(bytes, mode) => {
                fs::create_dir_all(full.parent().unwrap()).map_err(io)?;
                fs::write(&full, bytes).map_err(io)?;
                set_mode(&full, *mode)?;
            }
            Node::Link(_) => {
                return Err(Error::Validation(format!(
                    "unsupported initial relay symlink: {path}"
                )))
            }
        }
    }
    for (path, node) in tree.iter().rev() {
        if let Node::Directory(mode) = node {
            set_mode(&root.join(path), *mode)?;
        }
    }
    Ok(())
}
pub(super) fn io(e: std::io::Error) -> Error {
    Error::Internal(e.to_string())
}
#[cfg(unix)]
pub(super) fn mode(meta: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o7777
}
#[cfg(not(unix))]
pub(super) fn mode(meta: &fs::Metadata) -> u32 {
    u32::from(meta.permissions().readonly())
}
#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<(), Error> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).map_err(io)
}
#[cfg(not(unix))]
fn set_mode(path: &Path, mode: u32) -> Result<(), Error> {
    let mut p = fs::metadata(path).map_err(io)?.permissions();
    p.set_readonly(mode != 0);
    fs::set_permissions(path, p).map_err(io)
}
