//! Two-tree record invariants. Route admission and acceptance-effect detection (check 9) are
//! later slices; see docs/backlog.json. Both parsed revisions enforce check 11 through check 13.

use std::{collections::BTreeMap, fs, path::Path};

use xtask::{
    agent_lifecycle,
    agent_registry::{AgentRegistry, AgentRegistryEntry},
    approval_artifact,
    depth_record::{self, DepthRecord, Event, DEPTH_RECORD_FILE_NAME},
};

use super::{
    super::{rel_path, workspace_root_for, ValidateCtx, Violation},
    violation, StandingRequest,
};

mod git;

pub(in crate::manifest_validate) fn validate(
    ctx: &ValidateCtx,
    revision: &str,
    violations: &mut Vec<Violation>,
) {
    let Some(workspace) = workspace_root_for(&ctx.root) else {
        emit(
            violations,
            "DEPTH_TIP_UNREADABLE",
            ctx.root.display().to_string(),
            "--tip requires a Git workspace".into(),
        );
        return;
    };
    let root_dir = fs::canonicalize(&ctx.root).unwrap_or_else(|_| ctx.root.clone());
    let root = rel_path(&workspace, &root_dir);
    let tip = match git::Tip::new(&workspace, revision) {
        Ok(tip) => tip,
        Err(error) => {
            emit(violations, "DEPTH_TIP_UNREADABLE", root, error);
            return;
        }
    };
    let registry = AgentRegistry::load(&workspace).ok();
    let entry = registry
        .as_ref()
        .and_then(|r| r.agents.iter().find(|e| e.manifest_root == root));
    // Keep the named root anchored to the tip even if the candidate replaces it with a symlink
    // or moves its registry entry. A convenience symlink outside the registered path still
    // resolves through the canonical root, as ordinary one-tree validation does.
    let earlier_registry = match tip.registry() {
        Ok(registry) => registry,
        Err(error) => {
            emit(violations, "DEPTH_TIP_UNREADABLE", root, error);
            return;
        }
    };
    let named = named_root(&workspace, &ctx.root);
    let earlier_entry = earlier_registry
        .agents
        .iter()
        .find(|e| e.manifest_root == named)
        .or_else(|| {
            earlier_registry
                .agents
                .iter()
                .find(|e| e.manifest_root == root)
        })
        .or_else(|| entry.and_then(|entry| earlier_registry.find(&entry.agent_id)));
    let baseline_root = earlier_entry.map_or(root.as_str(), |e| e.manifest_root.as_str());
    let files = match tip.files(baseline_root) {
        Ok(files) => files,
        Err(error) => {
            emit(
                violations,
                "DEPTH_TIP_UNREADABLE",
                baseline_root.into(),
                error,
            );
            return;
        }
    };
    let candidates = depth_record::record_files(&root_dir);
    if !files.is_empty() && earlier_entry.is_none() {
        emit(
            violations,
            "DEPTH_TIP_UNREADABLE",
            root,
            "tip records have no registry anchor".into(),
        );
        return;
    }
    if (!files.is_empty() || !candidates.is_empty()) && entry.is_none() {
        emit(
            violations,
            "DEPTH_RECORD_UNANCHORED",
            root.clone(),
            "candidate records have no registry anchor".into(),
        );
    }
    if !files.is_empty()
        && earlier_entry.is_some_and(|earlier| {
            earlier.manifest_root != root
                || entry.is_some_and(|entry| entry.agent_id != earlier.agent_id)
        })
    {
        emit(
            violations,
            "DEPTH_RECORD_UNANCHORED",
            root.clone(),
            "the tip's record root was relocated or assigned to another agent".into(),
        );
    }
    let mut previous = BTreeMap::new();
    for (version, object) in files {
        let agent = &earlier_entry.expect("tip records are anchored").agent_id;
        match tip.record(&object, agent, &version) {
            Ok(record) => {
                previous.insert(version, record);
            }
            Err(error) => emit(
                violations,
                "DEPTH_TIP_UNREADABLE",
                format!("{root}/reports/{version}/{DEPTH_RECORD_FILE_NAME}"),
                error,
            ),
        }
    }
    let mut versions: std::collections::BTreeSet<_> = previous.keys().cloned().collect();
    versions.extend(candidates.into_iter().map(|(v, _)| v));
    let Some(anchor) = entry.or(earlier_entry) else {
        return;
    };
    if !versions.is_empty() {
        match xtask::depth_acceptance::compare(&workspace, tip.commit(), &anchor.agent_id) {
            Ok(report) => {
                for error in report.input_issues {
                    emit(
                        violations,
                        "DEPTH_ACCEPTANCE_INPUT_INVALID",
                        root.clone(),
                        error,
                    );
                }
                for requirement in report.missing_entries {
                    let contexts = requirement
                        .effects
                        .iter()
                        .map(|e| {
                            format!(
                                "{}:{} target={}",
                                e.path,
                                e.kind,
                                e.target.as_deref().unwrap_or("covered scope")
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ");
                    emit(
                        violations,
                        "DEPTH_ACCEPTANCE_ENTRY_MISSING",
                        format!("{root}/reports/{}/depth-record.json", requirement.version),
                        format!(
                            "{}/{} requires current record_sha256 {} for {contexts} (check 9)",
                            anchor.agent_id, requirement.version, requirement.digest
                        ),
                    );
                }
            }
            Err(error) => emit(
                violations,
                "DEPTH_ACCEPTANCE_INPUT_INVALID",
                root.clone(),
                error,
            ),
        }
    }
    for version in versions {
        let path = root_dir
            .join("reports")
            .join(&version)
            .join(DEPTH_RECORD_FILE_NAME);
        let relative = rel_path(&workspace, &path);
        let raw = match read_record(&root_dir, &path) {
            Ok(raw) => raw,
            Err(error) => {
                let code = if previous.contains_key(&version)
                    && fs::symlink_metadata(&path)
                        .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                {
                    "DEPTH_RECORD_REMOVED"
                } else {
                    "DEPTH_RECORD_INVALID"
                };
                emit(violations, code, relative, error.to_string());
                continue;
            }
        };
        let record = match DepthRecord::parse(&raw, &anchor.agent_id, &version) {
            Ok(record) => record,
            Err(error) => {
                emit(violations, "DEPTH_RECORD_INVALID", relative, error);
                continue;
            }
        };
        if let Some(old) = previous.get(&version) {
            if !record
                .acceptance_entries
                .starts_with(&old.acceptance_entries)
            {
                emit(
                    violations,
                    "DEPTH_ACCEPTANCE_ENTRIES_CHANGED",
                    relative.clone(),
                    "tip acceptance entries must remain an unchanged prefix (check 2)".into(),
                );
            }
            let lacking = record.lacking_from(old);
            if !lacking.is_empty() {
                emit(
                    violations,
                    "DEPTH_SELECTION_SHRUNK",
                    relative.clone(),
                    format!("selection lacks {} (check 2)", lacking.join(", ")),
                );
            }
        }
        // Listing entries changes no statement. Check 4 also covers first introduction.
        let changed = previous
            .get(&version)
            .map(|old| old.stated_sha256())
            .transpose();
        match (changed, record.stated_sha256()) {
            (Ok(Some(before)), Ok(now)) if before == now => {}
            (Ok(_), Ok(_)) => {
                if let Some(entry) = entry {
                    if !has_freeze(&workspace, entry, &record, violations) {
                        emit(violations, "DEPTH_RECORD_WITHOUT_FREEZE", relative, "a new or changed statement requires its own version and lifecycle path's frozen request or approval (check 4)".into());
                    }
                }
            }
            _ => emit(
                violations,
                "DEPTH_RECORD_INVALID",
                relative,
                "cannot compute stated digest".into(),
            ),
        }
    }
}

fn has_freeze(
    workspace: &Path,
    entry: &AgentRegistryEntry,
    record: &DepthRecord,
    violations: &mut Vec<Violation>,
) -> bool {
    let version = &record.policy.selection.enrollment.version;
    match &record.event {
        Event::Maintenance { .. } => {
            let relative = agent_lifecycle::maintenance_request_path(&entry.agent_id);
            let path = workspace.join(&relative);
            match fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|text| {
                    toml_edit::de::from_str::<StandingRequest>(&text).map_err(|e| e.to_string())
                }) {
                Ok(request) => {
                    request
                        .detected_release
                        .as_ref()
                        .and_then(|d| d.target_version.as_ref())
                        == Some(version)
                        && request
                            .support_depth
                            .as_ref()
                            .and_then(|t| t.policy_identity.as_ref())
                            .is_some_and(|p| !p.is_empty())
                }
                Err(error) => {
                    if fs::symlink_metadata(path).is_ok() {
                        emit(violations, "DEPTH_REQUEST_UNREADABLE", relative, error);
                    }
                    false
                }
            }
        }
        Event::Onboarding { .. } => {
            let relative = agent_lifecycle::approval_artifact_path_for_entry(entry);
            let path = workspace.join(&relative);
            if fs::symlink_metadata(&path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
            {
                return false;
            }
            match approval_artifact::load_approval_artifact(workspace, &relative) {
                Ok(approval) => {
                    approval.descriptor.agent_id == entry.agent_id
                        && approval.descriptor.manifest_root == entry.manifest_root
                        && approval.support_depth_version.as_ref() == Some(version)
                }
                Err(error) => {
                    emit(
                        violations,
                        "DEPTH_APPROVAL_UNREADABLE",
                        relative,
                        error.to_string(),
                    );
                    false
                }
            }
        }
    }
}

fn emit(violations: &mut Vec<Violation>, code: &'static str, path: String, message: String) {
    if !violations.iter().any(|v| v.code == code && v.path == path) {
        violations.push(violation(code, path, message));
    }
}

/// A checkout cannot hold a versioned file through a tracked directory symlink. Following one
/// would hide removal of the canonical Git path while reading the old content from elsewhere.
fn read_record(root: &Path, path: &Path) -> std::io::Result<String> {
    let mut current = root.to_path_buf();
    for component in path
        .strip_prefix(root)
        .expect("record below root")
        .components()
    {
        current.push(component);
        if fs::symlink_metadata(&current)?.file_type().is_symlink() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "a canonical depth record cannot be reached through a symlink below its manifest root"));
        }
    }
    fs::read_to_string(path)
}

/// Normalize dot segments without following a candidate-controlled root symlink.
fn named_root(workspace: &Path, root: &Path) -> String {
    let absolute = if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(root)
    };
    let mut normalized = std::path::PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    rel_path(workspace, &normalized)
}
