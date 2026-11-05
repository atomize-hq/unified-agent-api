//! A disposable copy of the tracked repository that carries the synthetic `depth_fixture` subject
//! (tests/fixtures/depth-synthetic), plus helpers that drive the real `xtask` binary against it and
//! prove a refused route left every byte, type and mode of the tree as it was.

use std::{
    collections::BTreeMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use sha2::{Digest, Sha256};

pub const REGISTRY: &str = xtask::agent_registry::REGISTRY_RELATIVE_PATH;
pub const MANIFEST_ROOT: &str = "cli_manifests/depth_fixture";
pub const RECORD: &str = "cli_manifests/depth_fixture/reports/0.2.0/depth-record.json";
const SYNTHETIC: &str = "tests/fixtures/depth-synthetic";

/// How the workspace carries depth.
#[derive(Clone, Copy, Debug)]
pub enum Depth {
    /// The registry declares the synthetic `support_depth` table; no record is written yet.
    Declared,
    /// The declaration is gone (the entry stays, without its table) and the reviewed synthetic
    /// record is retained under the subject's manifest root.
    Retained,
    /// The subject's whole registry entry is gone; its manifest root and record remain.
    Orphaned,
}

impl Depth {
    pub const ALL: [Depth; 3] = [Depth::Declared, Depth::Retained, Depth::Orphaned];

    /// The refusal `depth_routes` reports for this depth carrier.
    pub fn guard_reason(self) -> &'static str {
        match self {
            Depth::Declared => "unsupported depth write (declared enrollment)",
            Depth::Retained | Depth::Orphaned => "unsupported depth write (retained record)",
        }
    }
}

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn synthetic(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(SYNTHETIC)
        .join(relative)
}

fn tracked() -> &'static [String] {
    static LIST: OnceLock<Vec<String>> = OnceLock::new();
    LIST.get_or_init(|| {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo())
            .args(["ls-files", "-z"])
            .output()
            .unwrap();
        assert!(out.status.success(), "git ls-files failed");
        String::from_utf8(out.stdout)
            .unwrap()
            .split('\0')
            .filter(|p| !p.is_empty())
            // A tracked path may be deleted in a dirty developer checkout.
            .filter(|p| repo().join(p).is_file())
            .map(str::to_owned)
            .collect()
    })
}

pub fn write(root: &Path, relative: &str, bytes: impl AsRef<[u8]>) {
    let full = root.join(relative);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, bytes).unwrap();
}

/// The canonical registry plus the synthetic entry, with or without its `support_depth` table.
pub fn registry_with_subject(with_depth: bool) -> String {
    let canonical = fs::read_to_string(repo().join(REGISTRY)).unwrap();
    let entry = fs::read_to_string(synthetic("governing/registry-entry.toml"))
        .unwrap()
        .replace("{{ISOLATED_OWNER}}", "example-owner")
        .replace("{{ISOLATED_REPO}}", "depth-isolated");
    let raw = format!("{canonical}\n{entry}");
    let mut doc: toml_edit::DocumentMut = raw.parse().unwrap();
    if !with_depth {
        let agent = doc["agents"]
            .as_array_of_tables_mut()
            .unwrap()
            .iter_mut()
            .find(|a| a["agent_id"].as_str() == Some("depth_fixture"))
            .unwrap();
        assert!(agent.remove("support_depth").is_some());
    }
    let raw = doc.to_string();
    let parsed = xtask::agent_registry::AgentRegistry::parse_unenforced(&raw).unwrap();
    assert_eq!(
        parsed
            .find("depth_fixture")
            .unwrap()
            .support_depth
            .is_some(),
        with_depth
    );
    raw
}

pub fn synthetic_record() -> Vec<u8> {
    fs::read(synthetic("record.json")).unwrap()
}

/// A path, its type, its mode and the digest of its bytes (or link target).
pub type Tree = BTreeMap<PathBuf, (char, u32, String)>;

pub fn snapshot(root: &Path) -> Tree {
    fn walk(root: &Path, dir: &Path, out: &mut Tree) {
        let mut entries: Vec<_> = fs::read_dir(dir).unwrap().map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            let meta = fs::symlink_metadata(&path).unwrap();
            let rel = path.strip_prefix(root).unwrap().to_owned();
            let mode = meta.permissions().mode();
            if meta.file_type().is_symlink() {
                let target = fs::read_link(&path).unwrap();
                out.insert(rel, ('l', mode, target.display().to_string()));
            } else if meta.is_dir() {
                out.insert(rel, ('d', mode, String::new()));
                walk(root, &path, out);
            } else {
                let digest = hex::encode(Sha256::digest(fs::read(&path).unwrap()));
                out.insert(rel, ('f', mode, digest));
            }
        }
    }
    let mut out = Tree::new();
    walk(root, root, &mut out);
    out
}

/// The paths that differ between two snapshots, for a readable failure.
pub fn changed(before: &Tree, after: &Tree) -> Vec<PathBuf> {
    let mut paths: Vec<_> = before
        .iter()
        .filter(|(p, v)| after.get(*p) != Some(v))
        .map(|(p, _)| p.clone())
        .collect();
    paths.extend(after.keys().filter(|p| !before.contains_key(*p)).cloned());
    paths
}

pub struct Workspace {
    dir: tempfile::TempDir,
    scratch: tempfile::TempDir,
    empty_path: tempfile::TempDir,
}

impl Workspace {
    /// A copy of every tracked file, with no depth anywhere (the canonical repository has none).
    pub fn plain() -> Self {
        let dir = tempfile::tempdir().unwrap();
        for path in tracked() {
            write(dir.path(), path, fs::read(repo().join(path)).unwrap());
        }
        Self {
            dir,
            scratch: tempfile::tempdir().unwrap(),
            empty_path: tempfile::tempdir().unwrap(),
        }
    }

    /// A copy that also carries the synthetic subject's governing data and the given depth.
    pub fn with(depth: Depth) -> Self {
        let ws = Self::plain();
        let governing = synthetic("governing/manifest-root");
        for (rel, bytes) in files_under(&governing) {
            write(ws.root(), &format!("{MANIFEST_ROOT}/{rel}"), bytes);
        }
        match depth {
            Depth::Declared => write(ws.root(), REGISTRY, registry_with_subject(true)),
            Depth::Retained => {
                write(ws.root(), REGISTRY, registry_with_subject(false));
                write(ws.root(), RECORD, synthetic_record());
            }
            Depth::Orphaned => write(ws.root(), RECORD, synthetic_record()),
        }
        ws
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    /// Run the real binary from the workspace root with no tools on `PATH`, a private `TMPDIR`
    /// and no runtime binary, so a route that failed to refuse cannot reach anything external.
    pub fn xtask(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .current_dir(self.root())
            .env("PATH", self.empty_path.path())
            .env("TMPDIR", self.scratch.path())
            .env(
                "XTASK_RUNTIME_FOLLOW_ON_CODEX_BINARY",
                self.scratch.path().join("absent-codex"),
            )
            .args(args)
            .output()
            .unwrap()
    }

    /// Assert the invocation fails with `reason` and leaves the workspace and scratch untouched.
    pub fn assert_refuses(&self, args: &[&str], reason: &str) {
        let before = snapshot(self.root());
        let out = self.xtask(args);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{args:?} succeeded: {stderr}");
        assert!(
            stderr.contains(reason),
            "{args:?} did not refuse with `{reason}`: {stderr}"
        );
        self.assert_unchanged(&before, &format!("{args:?}"));
    }

    pub fn assert_unchanged(&self, before: &Tree, what: &str) {
        let after = snapshot(self.root());
        assert!(
            after == *before,
            "{what} changed {:?}",
            changed(before, &after)
        );
        assert!(
            fs::read_dir(self.scratch.path()).unwrap().next().is_none(),
            "{what} wrote scratch files"
        );
    }
}

fn files_under(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_owned()];
    while let Some(next) = stack.pop() {
        for entry in fs::read_dir(next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path.strip_prefix(dir).unwrap().display().to_string();
                out.push((rel, fs::read(&path).unwrap()));
            }
        }
    }
    out
}
