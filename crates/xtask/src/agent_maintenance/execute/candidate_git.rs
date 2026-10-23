use super::{tree::io, Error};
use std::{collections::BTreeSet, fs, path::Path, process::Command};
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GitState {
    pub head: Vec<u8>,
    pub index: Vec<u8>,
    pub branch: Vec<u8>,
    pub index_file: Option<Vec<u8>>,
}
/// Ignore inherited Git redirections; candidate Git metadata is never shared with the caller.
pub(super) fn command(root: &Path) -> Command {
    let mut c = Command::new("git");
    for (k, _) in std::env::vars_os() {
        if k.to_string_lossy().starts_with("GIT_") {
            c.env_remove(k);
        }
    }
    c.env(
        "GIT_CONFIG_GLOBAL",
        if cfg!(windows) { "NUL" } else { "/dev/null" },
    )
    .env("GIT_CONFIG_NOSYSTEM", "1");
    c.arg("--no-replace-objects").arg("-C").arg(root).args([
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.fsmonitor=false",
    ]);
    c
}
pub(super) fn run(root: &Path, args: &[&str]) -> Result<Vec<u8>, Error> {
    let o = command(root).args(args).output().map_err(io)?;
    if !o.status.success() {
        return Err(Error::Validation(format!(
            "relay git {} failed",
            args.first().unwrap_or(&"command")
        )));
    }
    Ok(o.stdout)
}
impl GitState {
    pub fn read(root: &Path) -> Result<Option<Self>, Error> {
        match fs::symlink_metadata(root.join(".git")) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io(e)),
            Ok(_) => {}
        }
        let head = command(root)
            .args(["rev-parse", "--verify", "HEAD"])
            .output()
            .map_err(io)?;
        let branch = command(root)
            .args(["symbolic-ref", "-q", "HEAD"])
            .output()
            .map_err(io)?;
        Ok(Some(Self {
            head: if head.status.success() {
                head.stdout
            } else {
                vec![]
            },
            index: run(root, &["ls-files", "--stage", "-z"])?,
            branch: branch.stdout,
            index_file: {
                let path = String::from_utf8(run(root, &["rev-parse", "--git-path", "index"])?)
                    .map_err(|e| Error::Validation(e.to_string()))?;
                match fs::read(root.join(path.trim())) {
                    Ok(bytes) => Some(bytes),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                    Err(e) => return Err(io(e)),
                }
            },
        }))
    }
    pub fn tracked(root: &Path) -> Result<BTreeSet<String>, Error> {
        if Self::read(root)?.is_none() {
            return Ok(BTreeSet::new());
        }
        let raw = run(root, &["ls-files", "-z"])?;
        raw.split(|b| *b == 0)
            .filter(|v| !v.is_empty())
            .map(|v| String::from_utf8(v.to_vec()).map_err(|e| Error::Validation(e.to_string())))
            .collect()
    }
}
pub(super) fn initialize(caller: &Path, root: &Path, had_git: bool) -> Result<(), Error> {
    if had_git {
        let o = command(caller)
            .args([
                "clone",
                "--no-local",
                "--no-hardlinks",
                "--no-checkout",
                "--",
            ])
            .arg(caller)
            .arg(root)
            .output()
            .map_err(io)?;
        if !o.status.success() {
            return Err(Error::Validation(
                "cannot create independent relay Git clone".into(),
            ));
        }
        run(root, &["remote", "remove", "origin"])?;
    } else {
        fs::create_dir_all(root).map_err(io)?;
        run(root, &["init", "-q"])?;
    }
    run(
        root,
        &["config", "user.name", "Maintenance relay candidate"],
    )?;
    run(root, &["config", "user.email", "relay-candidate@invalid"])?;
    Ok(())
}
pub(super) fn baseline(root: &Path) -> Result<String, Error> {
    run(root, &["add", "--force", "--all", "--", "."])?;
    run(
        root,
        &[
            "commit",
            "--allow-empty",
            "-qm",
            "Invocation workspace snapshot",
        ],
    )?;
    String::from_utf8(run(root, &["rev-parse", "HEAD"])?)
        .map(|v| v.trim().into())
        .map_err(|e| Error::Internal(e.to_string()))
}

/// Git observations must not traverse links or borrow an executor-selected object store.
pub(super) fn validate_metadata(root: &Path) -> Result<(), Error> {
    fn walk(path: &Path, count: &mut usize) -> Result<(), Error> {
        *count += 1;
        if *count > 50_000 {
            return Err(Error::Validation(
                "candidate Git metadata exceeds path limit".into(),
            ));
        }
        let metadata = fs::symlink_metadata(path).map_err(io)?;
        if metadata.file_type().is_symlink() || !(metadata.is_dir() || metadata.is_file()) {
            return Err(Error::Validation(
                "unsupported candidate Git metadata type".into(),
            ));
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(path).map_err(io)? {
                walk(&entry.map_err(io)?.path(), count)?;
            }
        }
        Ok(())
    }
    walk(&root.join(".git"), &mut 0)?;
    for path in [
        ".git/commondir",
        ".git/objects/info/alternates",
        ".git/objects/info/http-alternates",
        ".git/info/grafts",
    ] {
        if root.join(path).exists() {
            return Err(Error::Validation(
                "candidate Git metadata redirects its object/commit store".into(),
            ));
        }
    }
    run(root, &["fsck", "--full", "--strict", "--no-reflogs"])?;
    Ok(())
}
