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
    blobs: Mutex<Blobs>,
}

struct Index {
    // Keep Git's traversal order (tree ordering differs from lexical full-path ordering).
    rows: Vec<(Vec<u8>, Vec<u8>)>,
    paths: BTreeMap<Vec<u8>, usize>,
}

#[derive(Default)]
struct Blobs {
    bytes: BTreeMap<String, Vec<u8>>,
    batch: Option<Batch>,
}

struct Batch {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Immutable {
    pub(crate) fn new(root: &Path, commit: &str) -> Self {
        // The caller already resolved the commit ID. Trees and blobs are content addressed:
        // successful reads cannot change with HEAD, the index, or working files. This cache
        // is local to this immutable value, never a cache of mutable workspace/binding checks.
        Self(Arc::new(State {
            root: root.to_owned(),
            commit: commit.into(),
            index: OnceLock::new(),
            blobs: Mutex::new(Blobs::default()),
        }))
    }

    /// None asks the reader to use its original bounded command, including its exact errors.
    pub(crate) fn rows(&self, path: &str, recursive: bool, trees: bool) -> Option<Vec<u8>> {
        if path.contains('\0') {
            return None;
        }
        let index = self.0.index.get_or_init(|| self.index().ok()).as_ref()?;
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

    fn index(&self) -> Result<Index, String> {
        let raw = git(
            &self.0.root,
            &[
                "--literal-pathspecs",
                "ls-tree",
                "-r",
                "-t",
                "-z",
                "--full-tree",
                &self.0.commit,
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

    pub(crate) fn blob(&self, object: &str) -> Result<Vec<u8>, String> {
        let mut blobs = self.0.blobs.lock().map_err(|e| e.to_string())?;
        if let Some(bytes) = blobs.bytes.get(object) {
            return Ok(bytes.clone());
        }
        let result = (|| {
            if blobs.batch.is_none() {
                blobs.batch = Some(Batch::new(&self.0.root)?);
            }
            blobs
                .batch
                .as_mut()
                .ok_or("missing Git batch")?
                .read(object)
        })();
        let bytes = match result {
            Ok(bytes) => bytes,
            Err(_) => {
                // Missing/corrupt/non-blob objects retain cat-file blob's exact diagnostics
                // and conversion semantics. Discard a failed stream before any further read.
                blobs.batch = None;
                git(&self.0.root, &["cat-file", "blob", object])?
            }
        };
        blobs.bytes.insert(object.into(), bytes.clone());
        Ok(bytes)
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
