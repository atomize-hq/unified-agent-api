//! Bounded content-addressed reads. No ref walk, fetch, candidate execution or global cache.
use super::{oid, Object, State};
use crate::agent_lifecycle::LifecycleState;
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::Path,
    process::Stdio,
};

const OBJECT_LIMIT: usize = 16_777_216;
#[derive(Clone)]
pub(crate) struct Commit {
    pub object: Object,
    pub parents: Vec<String>,
}
pub(crate) struct Objects<'a> {
    root: &'a Path,
    bytes: BTreeMap<(String, String), Vec<u8>>,
    pub commits: BTreeMap<String, Commit>,
}
impl<'a> Objects<'a> {
    pub fn new(root: &'a Path) -> Self {
        Self {
            root,
            bytes: BTreeMap::new(),
            commits: BTreeMap::new(),
        }
    }
    pub fn read(&mut self, kind: &str, identity: &str) -> Result<Vec<u8>, String> {
        oid(identity)?;
        let key = (kind.to_owned(), identity.to_owned());
        if let Some(bytes) = self.bytes.get(&key) {
            return Ok(bytes.clone());
        }
        let object_type = crate::git_inputs::command(self.root)
            .args(["cat-file", "-t", identity])
            .output()
            .map_err(|e| e.to_string())?;
        if !object_type.status.success()
            || String::from_utf8_lossy(&object_type.stdout).trim() != kind
        {
            return Err("IN1 object type mismatch".into());
        }
        let size = crate::git_inputs::command(self.root)
            .args(["cat-file", "-s", identity])
            .output()
            .map_err(|e| e.to_string())?;
        if !size.status.success()
            || String::from_utf8_lossy(&size.stdout)
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|n| *n <= OBJECT_LIMIT)
                .is_none()
        {
            return Err("IN1 missing object or object bound exceeded".into());
        }
        let mut reader = crate::git_inputs::command(self.root)
            .args(["cat-file", kind, identity])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        let read = reader
            .stdout
            .take()
            .ok_or("missing object output")?
            .take(OBJECT_LIMIT as u64 + 1)
            .read_to_end(&mut bytes);
        if read.is_err() || bytes.len() > OBJECT_LIMIT {
            let _ = reader.kill();
            let _ = reader.wait();
            return Err("IN1 unreadable bounded object".into());
        }
        if !reader.wait().map_err(|e| e.to_string())?.success() {
            return Err("IN1 unreadable bounded object".into());
        }
        let mut child = crate::git_inputs::command(self.root)
            .args(["hash-object", "-t", kind, "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        child
            .stdin
            .take()
            .ok_or("missing object hash input")?
            .write_all(&bytes)
            .map_err(|e| e.to_string())?;
        let hash = child.wait_with_output().map_err(|e| e.to_string())?;
        if !hash.status.success() || String::from_utf8_lossy(&hash.stdout).trim() != identity {
            return Err("IN1 Git object hash mismatch".into());
        }
        self.bytes.insert(key, bytes.clone());
        Ok(bytes)
    }
    pub fn commit(&mut self, identity: &str) -> Result<Commit, String> {
        if let Some(commit) = self.commits.get(identity) {
            return Ok(commit.clone());
        }
        if self.commits.len() >= 1000 {
            return Err("IN1 unique commit traversal bound exceeded".into());
        }
        let bytes = self.read("commit", identity)?;
        let mut tree = None;
        let mut parents = vec![];
        for line in bytes
            .split(|b| *b == b'\n')
            .take_while(|line| !line.is_empty())
        {
            if let Some(value) = line.strip_prefix(b"tree ") {
                if tree.is_some() {
                    return Err("IN1 duplicate commit tree".into());
                }
                tree = Some(String::from_utf8(value.to_vec()).map_err(|e| e.to_string())?);
            }
            if let Some(value) = line.strip_prefix(b"parent ") {
                let parent = std::str::from_utf8(value).map_err(|e| e.to_string())?;
                oid(parent)?;
                if parents.len() >= 1000 || parents.iter().any(|p| p == parent) {
                    return Err("IN1 invalid parent population".into());
                }
                parents.push(parent.into());
            }
        }
        let tree = tree.ok_or("IN1 missing commit tree")?;
        oid(&tree)?;
        self.read("tree", &tree)?;
        let commit = Commit {
            object: Object {
                commit: identity.into(),
                tree,
            },
            parents,
        };
        self.commits.insert(identity.into(), commit.clone());
        Ok(commit)
    }
    pub fn require(&mut self, expected: &Object) -> Result<(), String> {
        if self.commit(&expected.commit)?.object != *expected {
            return Err("IN1 commit/tree identity mismatch".into());
        }
        Ok(())
    }
    fn entry(&mut self, object: &Object, path: &str) -> Result<Option<(String, String)>, String> {
        self.require(object)?;
        let parts: Vec<_> = path.split('/').collect();
        if parts.len() > 64
            || parts
                .iter()
                .any(|p| p.is_empty() || matches!(*p, "." | ".."))
        {
            return Err("IN1 invalid input path".into());
        }
        let mut tree = object.tree.clone();
        for (index, name) in parts.iter().enumerate() {
            let bytes = self.read("tree", &tree)?;
            let mut remaining = bytes.as_slice();
            let mut found = None;
            while !remaining.is_empty() {
                let end = remaining
                    .iter()
                    .position(|b| *b == 0)
                    .ok_or("IN1 malformed tree entry")?;
                let header = std::str::from_utf8(&remaining[..end]).map_err(|e| e.to_string())?;
                let (mode, entry) = header.split_once(' ').ok_or("IN1 malformed tree mode")?;
                let hash = remaining
                    .get(end + 1..end + 21)
                    .ok_or("IN1 malformed tree hash")?;
                if entry == *name {
                    if found.is_some() {
                        return Err("IN1 ambiguous tree entry".into());
                    }
                    found = Some((mode.to_owned(), hex::encode(hash)));
                }
                remaining = &remaining[end + 21..];
            }
            let Some((mode, hash)) = found else {
                return Ok(None);
            };
            if index + 1 == parts.len() {
                return Ok(Some((mode, hash)));
            }
            if mode != "40000" {
                return Err("IN1 control input directory is not a tree".into());
            }
            tree = hash;
        }
        Err("IN1 empty input path".into())
    }
    pub fn identity(&mut self, object: &Object, path: &str) -> Result<Option<String>, String> {
        self.entry(object, path)?
            .map(|(mode, hash)| {
                let kind = match mode.as_str() {
                    "40000" => "tree",
                    "100644" | "100755" => "blob",
                    _ => return Err("IN1 policy input is not a tree or regular blob".into()),
                };
                self.read(kind, &hash)?;
                Ok(hash)
            })
            .transpose()
    }
    pub fn blob(
        &mut self,
        object: &Object,
        path: &str,
    ) -> Result<Option<(String, Vec<u8>)>, String> {
        self.entry(object, path)?
            .map(|(mode, hash)| {
                if !matches!(mode.as_str(), "100644" | "100755") {
                    return Err("IN1 control input is not a regular blob".into());
                }
                Ok((hash.clone(), self.read("blob", &hash)?))
            })
            .transpose()
    }
    pub fn state(&mut self, object: &Object, path: &str) -> Result<Option<State>, String> {
        self.blob(object, path)?
            .map(|(blob, bytes)| {
                let state: LifecycleState =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                Ok(State {
                    path: path.into(),
                    blob,
                    stage: state.lifecycle_stage,
                })
            })
            .transpose()
    }
}
