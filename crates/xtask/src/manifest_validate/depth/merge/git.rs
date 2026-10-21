use std::{collections::BTreeMap, path::Path, process::Command};

use xtask::{
    agent_registry::{AgentRegistry, REGISTRY_RELATIVE_PATH},
    depth_record::{DepthRecord, DEPTH_RECORD_FILE_NAME},
};

/// The caller's baseline is resolved once. All subsequent reads name this immutable commit.
pub(super) struct Tip<'a> {
    workspace: &'a Path,
    commit: String,
}

impl<'a> Tip<'a> {
    pub(super) fn new(workspace: &'a Path, revision: &str) -> Result<Self, String> {
        let top = run(workspace, &["rev-parse", "--show-toplevel"])?;
        let top = Path::new(top.trim())
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if top != workspace {
            return Err("the workspace must be the root of its own Git repository".into());
        }
        let commit = run(
            workspace,
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{revision}^{{commit}}"),
            ],
        )?;
        Ok(Self {
            workspace,
            commit: commit.trim().to_string(),
        })
    }

    pub(super) fn commit(&self) -> &str {
        &self.commit
    }

    pub(super) fn registry(&self) -> Result<AgentRegistry, String> {
        let raw = run(
            self.workspace,
            &["show", &format!("{}:{REGISTRY_RELATIVE_PATH}", self.commit)],
        )?;
        AgentRegistry::parse(&raw).map_err(|e| e.to_string())
    }

    /// Anything at the exact record name is discovered, including a tree or symlink. The caller
    /// cannot turn an unreadable baseline into an empty set by replacing a file's kind.
    pub(super) fn files(&self, root: &str) -> Result<BTreeMap<String, String>, String> {
        let raw = run(
            self.workspace,
            &[
                "ls-tree",
                "-r",
                "-t",
                "-z",
                "--full-tree",
                &self.commit,
                "--",
                &format!("{root}/reports"),
            ],
        )?;
        let prefix = format!("{root}/reports/");
        let mut files = BTreeMap::new();
        for row in raw.split('\0').filter(|r| !r.is_empty()) {
            let (meta, path) = row.split_once('\t').ok_or("malformed Git tree entry")?;
            let Some(relative) = path.strip_prefix(&prefix) else {
                continue;
            };
            let parts: Vec<_> = relative.split('/').collect();
            if parts.len() != 2 || parts[1] != DEPTH_RECORD_FILE_NAME {
                continue;
            }
            let mut meta = meta.split_whitespace();
            let mode = meta.next().ok_or("Git tree entry has no mode")?;
            let kind = meta.next().ok_or("Git tree entry has no kind")?;
            let object = meta.next().ok_or("Git tree entry has no object")?;
            if !matches!(mode, "100644" | "100755") || kind != "blob" {
                return Err(format!("{path}: the tip record is not a regular file"));
            }
            files.insert(parts[0].to_string(), object.to_string());
        }
        Ok(files)
    }

    pub(super) fn record(
        &self,
        object: &str,
        agent: &str,
        version: &str,
    ) -> Result<DepthRecord, String> {
        DepthRecord::parse(
            &run(self.workspace, &["cat-file", "blob", object])?,
            agent,
            version,
        )
    }
}

fn run(workspace: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("--literal-pathspecs")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "git {}: {}",
            args[0],
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("Git output is not UTF-8: {e}"))
}
