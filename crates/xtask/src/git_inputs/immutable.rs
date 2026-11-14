//! Invocation-local object reads shared by clones of one resolved commit.
use super::{command, git};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Stdio},
    sync::{Arc, Mutex, OnceLock},
};

#[derive(Clone)]
pub(crate) struct Immutable(Arc<State>);

struct State {
    root: PathBuf,
    commit: String,
    index: OnceLock<Option<Index>>,
    blobs: Mutex<BlobReader>,
}

pub(super) struct Index {
    // Keep Git's traversal order (tree ordering differs from lexical full-path ordering).
    rows: Vec<(Vec<u8>, Vec<u8>)>,
    paths: BTreeMap<Vec<u8>, usize>,
}

pub(super) struct BlobReader {
    root: PathBuf,
    batch: Option<Batch>,
}

struct Batch {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Immutable {
    pub(crate) fn new(root: &Path, commit: &str) -> Self {
        // The caller resolved the commit once. Content-addressed tree rows cannot change the
        // returned bytes or modes when HEAD, the index, or working files change. Keep only the
        // tree index: each blob fetch still observes object availability on every read. Tree
        // object availability after indexing is intentionally outside this immutable snapshot.
        Self(Arc::new(State {
            root: root.to_owned(),
            commit: commit.into(),
            index: OnceLock::new(),
            blobs: Mutex::new(BlobReader::new(root)),
        }))
    }

    /// None asks the reader to use its original bounded command, including its exact errors.
    pub(crate) fn rows(&self, path: &str, recursive: bool, trees: bool) -> Option<Vec<u8>> {
        if path.contains('\0') {
            return None;
        }
        let index = self
            .0
            .index
            .get_or_init(|| Index::load(&self.0.root, &self.0.commit).ok())
            .as_ref()?;
        if !recursive {
            return Some(
                index
                    .paths
                    .get(path.as_bytes())
                    .map(|i| index.rows[*i].1.clone())
                    .unwrap_or_default(),
            );
        }
        let prefix = format!("{path}/");
        let mut result = Vec::new();
        for (name, row) in &index.rows {
            let tree = row.starts_with(b"040000 tree ");
            // -t also emits traversed ancestors of a nested pathspec, even if its leaf is absent.
            let ancestor = trees
                && tree
                && path
                    .as_bytes()
                    .strip_prefix(name.as_slice())
                    .is_some_and(|tail| tail.starts_with(b"/"));
            if (name == path.as_bytes() || name.starts_with(prefix.as_bytes()) || ancestor)
                && (trees || !tree)
            {
                result.extend(row);
            }
        }
        Some(result)
    }

    pub(crate) fn blob(&self, object: &str) -> Result<Vec<u8>, String> {
        self.0.blobs.lock().map_err(|e| e.to_string())?.read(object)
    }
}

impl Index {
    pub(super) fn load(root: &Path, commit: &str) -> Result<Self, String> {
        let raw = git(
            root,
            &[
                "--literal-pathspecs",
                "ls-tree",
                "-r",
                "-t",
                "-z",
                "--full-tree",
                commit,
            ],
        )?;
        let mut index = Index {
            rows: Vec::new(),
            paths: BTreeMap::new(),
        };
        let mut previous = BTreeMap::<Vec<u8>, Vec<u8>>::new();
        // Unrelated non-UTF8 paths stay opaque; each reader decodes only selected rows.
        for row in raw.split(|b| *b == 0).filter(|r| !r.is_empty()) {
            let tab = row
                .iter()
                .position(|b| *b == b'\t')
                .ok_or("invalid Git tree row")?;
            let path = row[tab + 1..].to_vec();
            let (parent, name) = path
                .iter()
                .rposition(|b| *b == b'/')
                .map_or((&b""[..], path.as_slice()), |slash| {
                    (&path[..slash], &path[slash + 1..])
                });
            if !parent.is_empty()
                && !index
                    .paths
                    .get(parent)
                    .is_some_and(|i| index.rows[*i].1.starts_with(b"040000 tree "))
            {
                return Err("invalid Git tree ancestor".into());
            }
            let mut order = name.to_vec();
            if row.starts_with(b"040000 tree ") {
                order.push(b'/');
            }
            // Git's bounded pathspec walk can stop early in an unsorted malformed tree.
            // Selecting from its complete listing would otherwise accept an invisible entry.
            if previous
                .insert(parent.to_vec(), order.clone())
                .is_some_and(|old| old >= order)
            {
                return Err("unsorted Git tree".into());
            }
            if index.paths.insert(path.clone(), index.rows.len()).is_some() {
                // Malformed trees can contain duplicate names. A map must never collapse the
                // original multi-row rejection into a successful single-file binding.
                return Err("duplicate Git tree path".into());
            }
            let mut terminated = row.to_vec();
            terminated.push(0);
            index.rows.push((path, terminated));
        }
        Ok(index)
    }

    pub(super) fn into_entries(self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.rows.into_iter().collect()
    }
}

impl BlobReader {
    pub(super) fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
            batch: None,
        }
    }

    pub(super) fn read(&mut self, object: &str) -> Result<Vec<u8>, String> {
        let result = (|| {
            if self.batch.is_none() {
                self.batch = Some(Batch::new(&self.root)?);
            }
            self.batch.as_mut().ok_or("missing Git batch")?.read(object)
        })();
        match result {
            Ok(bytes) => Ok(bytes),
            Err(_) => {
                // Missing/corrupt/non-blob objects retain cat-file blob's exact diagnostics
                // and conversion semantics. Discard a failed stream before any further read.
                self.batch = None;
                git(&self.root, &["cat-file", "blob", object])
            }
        }
    }
}

impl Batch {
    fn new(root: &Path) -> Result<Self, String> {
        let mut child = command(root)
            .args(["--literal-pathspecs", "cat-file", "--batch"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let stdin = child.stdin.take().ok_or("missing Git batch stdin")?;
        let stdout = BufReader::new(child.stdout.take().ok_or("missing Git batch stdout")?);
        Ok(Self {
            child,
            stdin,
            stdout,
        })
    }

    fn read(&mut self, object: &str) -> Result<Vec<u8>, String> {
        writeln!(self.stdin, "{object}").map_err(|e| e.to_string())?;
        self.stdin.flush().map_err(|e| e.to_string())?;
        let mut header = String::new();
        self.stdout
            .read_line(&mut header)
            .map_err(|e| e.to_string())?;
        let fields: Vec<_> = header.trim_end().split(' ').collect();
        if fields.len() != 3 || fields[0] != object || fields[1] != "blob" {
            return Err("invalid Git batch blob".into());
        }
        let size: usize = fields[2].parse().map_err(|_| "invalid Git batch size")?;
        let mut bytes = vec![0; size];
        self.stdout
            .read_exact(&mut bytes)
            .map_err(|e| e.to_string())?;
        let mut end = [0];
        self.stdout
            .read_exact(&mut end)
            .map_err(|e| e.to_string())?;
        if end != [b'\n'] {
            return Err("invalid Git batch terminator".into());
        }
        Ok(bytes)
    }
}

impl Drop for Batch {
    fn drop(&mut self) {
        // Also reap on protocol errors; no batch children survive the last source clone.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dead_batch_child_uses_single_read_diagnostics_and_recovers() {
        let root = tempfile::tempdir().unwrap();
        git(root.path(), &["init", "-q"]).unwrap();
        std::fs::write(root.path().join("bound"), b"bytes").unwrap();
        git(root.path(), &["add", "bound"]).unwrap();
        let object =
            String::from_utf8(git(root.path(), &["rev-parse", ":bound"]).unwrap()).unwrap();
        let object = object.trim();
        let mut reader = BlobReader::new(root.path());
        assert_eq!(reader.read(object).unwrap(), b"bytes");
        let batch = reader.batch.as_mut().unwrap();
        batch.child.kill().unwrap();
        batch.child.wait().unwrap();
        assert_eq!(
            reader.read(object),
            git(root.path(), &["cat-file", "blob", object])
        );
        assert!(reader.batch.is_none());
        assert_eq!(reader.read(object).unwrap(), b"bytes");
        assert!(reader.batch.is_some());
    }
}
