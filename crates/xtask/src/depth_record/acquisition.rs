//! The maintainer acquisition statement of decision OF-P4(B) of
//! `docs/agents/lifecycle/support-depth-onboarding-freeze-decisions.md`:
//! `docs/agents/lifecycle/<onboarding_pack_prefix>/governance/depth-acquisition/<version>.toml`.
//!
//! It is modeled on the maintenance stand-down markers (`agent_maintenance/stand_down.rs`), with
//! one difference that is the point of the decision: it takes effect only from the integration
//! branch, so this reader reads it from a named git ref and has no working-tree fallback. A caller
//! fetches the integration ref and names it; a statement carried on the onboarding branch that
//! uses it does not count. Nothing calls this reader yet: no onboarding freeze is installed.
//!
//! As for markers, a directory that cannot be read in full is refused: an entry that is
//! malformed, misnamed, or written for another agent means the statement set is unknown.

use std::{collections::BTreeSet, path::Path, process::Command};

use serde::Deserialize;

use super::{file_ref, FileRef};
use crate::agent_lifecycle::{GOVERNANCE_DIR_NAME, LIFECYCLE_DOCS_ROOT};

pub const ACQUISITION_STATEMENT_DIR_NAME: &str = "depth-acquisition";
pub const ACQUISITION_STATEMENT_SCHEMA_VERSION: u32 = 1;

/// The statement directory of an onboarding pack.
pub fn statement_dir(pack_prefix: &str) -> String {
    format!("{LIFECYCLE_DOCS_ROOT}/{pack_prefix}/{GOVERNANCE_DIR_NAME}/{ACQUISITION_STATEMENT_DIR_NAME}")
}

/// Where the statement for `version` lives.
pub fn statement_path(pack_prefix: &str, version: &str) -> String {
    format!("{}/{version}.toml", statement_dir(pack_prefix))
}

/// A maintainer's statement of which acquired inputs they acquired, and how.
///
/// `deny_unknown_fields`, as for markers: a field written in the belief that it widens or narrows
/// what is stated must be refused at once, not silently ignored.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcquisitionStatement {
    pub schema_version: u32,
    pub agent_id: String,
    /// The exact upstream version. It must equal the file's own name.
    pub version: String,
    pub inputs: Vec<AcquiredInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcquiredInput {
    /// Repository-relative path of the acquired input.
    pub path: String,
    pub sha256: String,
    /// The command that produced it.
    pub command: String,
}

impl AcquisitionStatement {
    /// Reads the statement held at `<version>.toml` of `agent_id`'s statement directory.
    pub fn parse(text: &str, agent_id: &str, file_name: &str) -> Result<Self, String> {
        let at = format!("acquisition statement `{file_name}`");
        let version = file_name
            .strip_suffix(".toml")
            .filter(|version| plain_segment(version))
            .ok_or_else(|| format!("{at} is not named `<version>.toml`"))?;
        let statement: Self =
            toml_edit::de::from_str(text).map_err(|err| format!("{at}: {err}"))?;
        if statement.schema_version != ACQUISITION_STATEMENT_SCHEMA_VERSION {
            return Err(format!(
                "{at} declares schema_version {}; this build reads \
                 {ACQUISITION_STATEMENT_SCHEMA_VERSION} only",
                statement.schema_version
            ));
        }
        if statement.agent_id != agent_id {
            return Err(format!(
                "{at} declares agent_id `{}` but sits in `{agent_id}`'s onboarding pack",
                statement.agent_id
            ));
        }
        if statement.version != version {
            return Err(format!(
                "{at} declares version `{}`, which is not the version its file name names",
                statement.version
            ));
        }
        if statement.inputs.is_empty() {
            return Err(format!("{at} states no acquired input"));
        }
        let mut seen = BTreeSet::new();
        for input in &statement.inputs {
            file_ref(&at, &input.file_ref())?;
            if input.command.trim().is_empty() {
                return Err(format!("{at}: `{}` names no producing command", input.path));
            }
            if !seen.insert(&input.path) {
                return Err(format!("{at} states `{}` twice", input.path));
            }
        }
        Ok(statement)
    }

    /// Why a set of bound acquired inputs is not exactly what this statement states: an input the
    /// statement does not cover, one whose SHA-256 differs, or a statement entry with no input.
    pub fn mismatch(&self, bound: &[FileRef]) -> Option<String> {
        let stated: BTreeSet<FileRef> = self.inputs.iter().map(AcquiredInput::file_ref).collect();
        let bound: BTreeSet<FileRef> = bound.iter().cloned().collect();
        let mut paths: Vec<&str> = stated
            .symmetric_difference(&bound)
            .map(|file| file.path.as_str())
            .collect();
        paths.dedup();
        (!paths.is_empty()).then(|| {
            format!(
                "the acquisition statement does not state exactly the bound acquired inputs: {}",
                paths.join(", ")
            )
        })
    }
}

impl AcquiredInput {
    pub fn file_ref(&self) -> FileRef {
        FileRef {
            path: self.path.clone(),
            sha256: self.sha256.clone(),
        }
    }
}

/// Reads `agent_id`'s statement for `version` from `git_ref`, never from the working tree. Every
/// entry of the statement directory at that ref is read and must be a statement of the agent;
/// `Ok(None)` is a directory at the ref that holds no statement for `version`. Any git failure,
/// an unreadable ref included, is an error, never an absence.
pub fn read_from_ref(
    workspace: &Path,
    git_ref: &str,
    pack_prefix: &str,
    agent_id: &str,
    version: &str,
) -> Result<Option<AcquisitionStatement>, String> {
    if git_ref.trim().is_empty() || git_ref.starts_with('-') {
        return Err(format!(
            "`{git_ref}` does not name an integration ref to read the acquisition statement from"
        ));
    }
    if !plain_segment(version) {
        return Err(format!(
            "`{version}` is not a version a statement can be named for"
        ));
    }
    let dir = statement_dir(pack_prefix);
    let listing = git(workspace, &["ls-tree", "-r", "-z", git_ref, "--", &dir])?;
    let mut found = None;
    for entry in listing.split('\0').filter(|entry| !entry.is_empty()) {
        let (meta, path) = entry.split_once('\t').ok_or_else(|| {
            format!("`git ls-tree` printed an entry it does not describe: {entry}")
        })?;
        let mut meta = meta.split(' ');
        let (mode, kind, object) = (meta.next(), meta.next(), meta.next());
        let file_name = path.strip_prefix(&format!("{dir}/")).unwrap_or(path);
        // A link or a nested entry is not a statement, and could never be one.
        if mode != Some("100644") || kind != Some("blob") || file_name.contains('/') {
            return Err(format!(
                "`{path}` at `{git_ref}` is not a regular statement file of `{dir}`"
            ));
        }
        let object = object.ok_or_else(|| format!("`git ls-tree` names no object for `{path}`"))?;
        let text = git(workspace, &["cat-file", "blob", object])?;
        let statement = AcquisitionStatement::parse(&text, agent_id, file_name)
            .map_err(|err| format!("{err} (at `{git_ref}`)"))?;
        if statement.version == version {
            found = Some(statement);
        }
    }
    Ok(found)
}

/// One path segment that names a file, as a version does.
fn plain_segment(value: &str) -> bool {
    !value.is_empty()
        && !matches!(value, "." | "..")
        && !value.contains(['/', '\\'])
        && !value.chars().any(char::is_whitespace)
}

fn git(workspace: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(workspace)
        .args(args)
        .output()
        .map_err(|err| format!("could not run `git {}`: {err}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed ({}): {}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| format!("`git {}` printed non-UTF-8: {err}", args.join(" ")))
}
